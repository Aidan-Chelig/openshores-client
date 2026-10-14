"""Boot the actual reference server with an isolated PostgreSQL database.

cargo build
nix develop .#server-test --command python tests/local_server.py

All credentials/world data are synthetic. The server runs from an owned /tmp
directory; the reference source, accounts, and any existing databases are untouched.
"""
from __future__ import annotations

import asyncio
import getpass
import math
import os
from pathlib import Path
import signal
import socket
import struct
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "reference/OpenShores-server"
sys.dont_write_bytecode = True
sys.path.insert(0, str(SOURCE / "src"))
USERNAME = "rust-local-test"
PASSWORD = "synthetic-local-test-password"


def run(*args: str, **kwargs) -> subprocess.CompletedProcess:
    return subprocess.run(args, check=True, stdout=subprocess.PIPE,
                          stderr=subprocess.STDOUT, text=True, timeout=120, **kwargs)


async def seed(dsn: str) -> int:
    import asyncpg
    from openshores.core.accounts import default_store, _wire_password
    from openshores.core.config import Config
    from openshores.database.repositories.bundle import pick_boot_globe
    from openshores.protocol.login_reply import _default_dna
    from openshores.root import first_run

    store = default_store()
    store.create(USERNAME, _wire_password(PASSWORD))
    conn = await asyncpg.connect(dsn)
    try:
        await first_run(Config.load(), conn)
        globe = await pick_boot_globe(conn)
        # This is a synthetic avatar fixture, not the unimplemented creation UI.
        avatar = await conn.fetchval('''
            INSERT INTO "a_Person"
              ("name", "dna", "sex", "idp", "locX", "locY", "locZ",
               "hp", "max_hp", "stamina", "max_stamina", "hunger", "max_hunger",
               "bank", "xp", "isonline", "atRest", "autoEat", "islefty", "inv",
               "timeCreate", "timeModified")
            VALUES ($1,$2,1,$3,0,0,1000,100,100,100,100,100,100,
                    0,0,0,1,1,0,$4,$5,$5) RETURNING "id"
        ''', "Rust Explorer", _default_dna(), int(globe["id"]), bytes(1), int(time.time() * 1000))
        store.add_avatar(USERNAME, int(avatar))
        return int(avatar)
    finally:
        await conn.close()


def main() -> None:
    binary = str(ROOT / "target/debug/openshores-client")
    if not Path(binary).is_file():
        raise RuntimeError("run cargo build first")
    # Hold distinct ephemeral ports until the server is ready to start.
    reservations = [socket.socket() for _ in range(5)]
    for sock in reservations:
        sock.bind(("127.0.0.1", 0))
    login, chat, scene, mail, control = [sock.getsockname()[1] for sock in reservations]
    with tempfile.TemporaryDirectory(prefix="openshores-local-", dir="/tmp") as temporary:
        runtime = Path(temporary)
        socket_dir = runtime / "socket"
        socket_dir.mkdir()
        data = runtime / "postgres"
        server = None
        postgres_started = False
        previous_directory = Path.cwd()
        previous_env = os.environ.copy()
        try:
            print("Initializing isolated local PostgreSQL", flush=True)
            user = getpass.getuser()
            run("initdb", "-D", str(data), "-U", user, "--no-locale", "--encoding=UTF8", "--auth-local=trust", "--auth-host=reject")
            run("pg_ctl", "-D", str(data), "-l", str(runtime / "postgres.log"),
                "-o", f"-k {socket_dir} -p 5432 -c listen_addresses=''", "-w", "start")
            postgres_started = True
            run("createdb", "-h", str(socket_dir), "-p", "5432", "-U", user, "openshores")
            dsn = f"postgresql:///openshores?host={socket_dir}&port=5432&user={user}"
            for key in list(os.environ):
                if key.startswith("OPENSHORES_"):
                    del os.environ[key]
            os.environ.update({
                "PYTHONPATH": str(SOURCE / "src"),
                "PYTHONDONTWRITEBYTECODE": "1",
                "OPENSHORES_DATABASE_URL": dsn,
                "OPENSHORES_BIND_HOST": "127.0.0.1",
                "OPENSHORES_PUBLIC_HOST": "127.0.0.1",
                "OPENSHORES_GD_PATH": str(SOURCE / "GD"),
                "OPENSHORES_LOGIN_PORT": str(login),
                "OPENSHORES_CHAT_PORT": str(chat),
                "OPENSHORES_SCENE_PORT": str(scene),
                "OPENSHORES_MAIL_PORT": str(mail),
                "OPENSHORES_CONTROL_PORT": str(control),
                "OPENSHORES_LOG_LEVEL": "INFO",
            })
            os.chdir(runtime)
            print("Generating the reference world and synthetic avatar", flush=True)
            avatar = asyncio.run(seed(dsn))
            for sock in reservations:
                sock.close()
            print(f"Starting actual reference server on loopback; avatar={avatar}", flush=True)
            log_path = runtime / "server.log"
            with log_path.open("w") as log:
                server = subprocess.Popen([sys.executable, "-m", "openshores"], stdout=log, stderr=log)
                deadline = time.monotonic() + 120
                while "[boot] login  listening" not in log_path.read_text():
                    if server.poll() is not None or time.monotonic() > deadline:
                        raise RuntimeError("reference server failed to start:\n" + log_path.read_text()[-8000:])
                    time.sleep(0.2)
                print("Checking authenticated login, redirect, resume, chat ACK, and manifest", flush=True)
                result = run(binary, "login", USERNAME, f"127.0.0.1:{login}",
                             "--avatar", str(avatar), "--scene-port", str(scene),
                             "--chat-port", str(chat), "--watch-seconds", "3",
                             "--look-at", "0x70000000", "--move-gear", "5", "0", "1",
                             "--capture", str(runtime / "watched.frames"), "--password-stdin",
                             input=PASSWORD + "\n")
                print(result.stdout, end="", flush=True)
                assert "Scene initialized" in result.stdout
                assert PASSWORD not in result.stdout
                assert "client ACK for" in log_path.read_text(), "reference server did not observe init ACK"
                async def verify_capture():
                    from openshores.protocol.framing import read_framed, encode_size
                    from openshores.protocol.stream import QDS
                    from openshores.gameplay.worldgen.terrain import terrain_altitude_msl
                    reader = asyncio.StreamReader()
                    reader.feed_data((runtime / "watched.frames").read_bytes())
                    reader.feed_eof()
                    packets = []
                    while not reader.at_eof():
                        packets.append(await read_framed(reader))
                    assert len(packets) > 100
                    assert any(p == bytes([0x2A]) + avatar.to_bytes(4, "big") for p in packets)
                    assert any(p[0] == 0x38 for p in packets)
                    active = initialized = False
                    for index, packet in enumerate(packets):
                        active |= packet[0] == 0x2A
                        initialized |= packet[0] == 0x38
                        if active and initialized and packet[0] == 0x18 and packet[5] == 1:
                            packets = packets[:index + 1]
                            break
                    else:
                        raise AssertionError("no initialized bootstrap manifest")
                    (runtime / "scene.frames").write_bytes(b"".join(
                        encode_size(len(packet)) + packet for packet in packets))
                    assert packets[-1][0] == 0x18 and packets[-1][5] == 1
                    count = int.from_bytes(packets[-1][6:8], "big")
                    ids = [int.from_bytes(packets[-1][i:i+4], "big") for i in range(8, len(packets[-1]), 4)]
                    assert len(ids) == count and avatar in ids
                    print(f"Captured {len(packets)} scene packets; final manifest verified", flush=True)
                    surfaces = {}
                    for packet in packets:
                        if packet[0] != 0x1F:
                            continue
                        stream = QDS(packet)
                        stream.read_u8()
                        globe_id = stream.read_u32()
                        stream._take(8)
                        base = stream.read_u8()
                        if base & 1: stream._take(4)
                        if base & 2: stream._take(8)
                        if base & 4: stream._take(8)
                        if base & 8: stream._take(24)
                        flags = stream.read_u8()
                        if flags & 1: stream._take(9)
                        stream.read_qstring()
                        globe_flags = stream.read_u8()
                        if globe_flags & 1:
                            size, atm_type, density, water = stream._take(4)
                            parameters = struct.unpack(">6f", stream._take(24))
                            surfaces[globe_id] = (size, parameters)
                    samples = 0
                    for globe_id, (size, parameters) in surfaces.items():
                        for lat_deg, lon_deg in [(0, 0), (37.5, -122), (89, 179), (-90, -180)]:
                            expected = terrain_altitude_msl(parameters, size, math.radians(lat_deg), math.radians(lon_deg))
                            output = run(binary, "sample-terrain", str(runtime / "scene.frames"), str(globe_id), str(lat_deg), str(lon_deg)).stdout
                            actual = float(next(line.split(": ", 1)[1] for line in output.splitlines() if line.startswith("Base altitude MSL:")))
                            assert abs(actual - expected) < 1e-8, (globe_id, lat_deg, lon_deg, actual, expected)
                            samples += 1
                    assert len(surfaces) == 14
                    print(f"Base terrain matches Python for {samples} coordinates across {len(surfaces)} captured globes", flush=True)
                asyncio.run(verify_capture())
                replay = run(binary, "inspect-scene", str(runtime / "scene.frames"))
                assert "Decoded 49 entities; 49 visible in manifest." in replay.stdout, replay.stdout
                assert "Missing manifest IDs: []" in replay.stdout, replay.stdout
                assert "Unresolved parents: {}" in replay.stdout, replay.stdout
                assert f"Player ancestry: Person({avatar:#010x}) -> Globe" in replay.stdout, replay.stdout
                assert "-> Universe(" in replay.stdout, replay.stdout
                player_line = next(line for line in replay.stdout.splitlines()
                                   if line.startswith(f"{avatar:#010x} Person "))
                for field in ["person=Some(PersonState", "dna: Some(Some(", "hp: Some(",
                              "stamina: Some(", "poses: Some(", "citizen: Some(CitizenPrefix",
                              "home_system: Some(", "gear: Some(Gear", "declared_slots: 7",
                              "statistics: Some(PlayerStatistics", "bank: 0.0", "jobs: Some(JobList",
                              "declared_count: 1"]:
                    assert field in player_line, (field, player_line)
                print("Local player creature state decoded from actual server bootstrap", flush=True)
                print("Offline world replay: all 49 manifest entities and player ancestry resolved", flush=True)
                watched = result
                assert "Local scene watch ended: DurationElapsed" in watched.stdout, watched.stdout
                watch_line = next(line for line in watched.stdout.splitlines()
                                  if line.startswith("Local scene watch ended:"))
                updates = int(watch_line.split("; ")[1].split(" ")[0])
                assert updates > 0, watched.stdout
                watched_replay = run(binary, "inspect-scene", str(runtime / "watched.frames"))
                # Ongoing traffic can introduce currently unsupported atom kinds.
                # Preserve/report unresolved visibility, and verify replay agrees
                # with live state rather than pretending every kind is decoded.
                live_missing = next(line for line in watched.stdout.splitlines()
                                    if line.startswith("Missing manifest IDs:"))
                assert live_missing in watched_replay.stdout, watched_replay.stdout
                assert f"Player ancestry: Person({avatar:#010x}) -> Globe" in watched_replay.stdout
                item_line = next(line for line in watched_replay.stdout.splitlines()
                                 if line.startswith("0x70000000 Item "))
                assert "commodity_id: 130" in item_line, item_line
                assert "quality: Some(61)" in item_line, item_line
                assert "Item: 1" in watched_replay.stdout, watched_replay.stdout
                print("Dynamic floor item decoded and retained in ongoing scene state", flush=True)
                watched_player = next(line for line in watched_replay.stdout.splitlines()
                                      if line.startswith(f"{avatar:#010x} Person "))
                moved = "slot_type: 1, subindex: 0, item: Some(InventoryItem { type_id: 1, common: AuItemPrefix { flags: 12, commodity_id: 1,"
                original = "slot_type: 5, subindex: 0, item: Some(InventoryItem { type_id: 1, common: AuItemPrefix { flags: 12, commodity_id: 1,"
                assert original in player_line, player_line
                assert moved in watched_player and original not in watched_player, watched_player
                assert "Observed player gear (latest received batch):" in watched.stdout
                print("Explicit gear move confirmed by server-originated inventory refresh", flush=True)
                print(f"Local ongoing session: {updates} post-bootstrap packets applied and replayed", flush=True)
                print("Full local reference-server integration passed", flush=True)
        except BaseException as error:
            if isinstance(error, subprocess.CalledProcessError) and error.stdout:
                print(error.stdout, file=sys.stderr)
            log_path = runtime / "server.log"
            if log_path.exists():
                print(log_path.read_text()[-8000:], file=sys.stderr)
            raise
        finally:
            for sock in reservations:
                sock.close()
            if server is not None and server.poll() is None:
                server.send_signal(signal.SIGINT)
                try:
                    server.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    server.kill()
                    server.wait()
            if postgres_started:
                run("pg_ctl", "-D", str(data), "-m", "fast", "-w", "stop")
            os.chdir(previous_directory)
            os.environ.clear()
            os.environ.update(previous_env)


if __name__ == "__main__":
    main()
