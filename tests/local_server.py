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
    timeout = kwargs.pop("timeout", 120)
    return subprocess.run(args, check=True, stdout=subprocess.PIPE,
                          stderr=subprocess.STDOUT, text=True, timeout=timeout, **kwargs)


async def seed(dsn: str, username: str = USERNAME) -> int:
    import asyncpg
    from openshores.core.accounts import default_store, _wire_password
    from openshores.core.config import Config
    from openshores.database.repositories.bundle import pick_boot_globe
    from openshores.protocol.login_reply import _default_dna
    from openshores.root import first_run

    store = default_store()
    store.create(username, _wire_password(PASSWORD))
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
        store.add_avatar(username, int(avatar))
        return int(avatar)
    finally:
        await conn.close()


def main() -> None:
    action = sys.argv[1] if len(sys.argv) == 2 else "move"
    if action not in ("move", "drop", "pickup", "position", "peer", "pose", "use", "use-noop", "gui", "play", "quick-dev", "quick-smoke", "avatar", "avatar-live", "conversation") or len(sys.argv) > 2:
        raise ValueError("usage: local_server.py [move|drop|pickup|position|peer|pose|use|use-noop|gui|play|quick-dev|quick-smoke|avatar|avatar-live|conversation]")
    graphical = action in ("gui", "play", "quick-dev", "quick-smoke", "avatar", "avatar-live", "conversation")
    binary = str(ROOT / "target/debug" / ("openshores-client-slice" if graphical else "openshores-client"))
    if not Path(binary).is_file():
        raise RuntimeError("run cargo build first")
    anchor = None
    installation = Path(os.environ.get("HAZERON_INSTALL", ROOT / "reference/hazeron")).resolve()
    if action in ("avatar", "avatar-live", "quick-dev", "quick-smoke", "play") and installation.is_dir():
        from local_avatar_preparation import anchor_metrics
        anchor = anchor_metrics(str(installation))
    if action in ("avatar", "avatar-live") and anchor is None:
        raise RuntimeError("connected avatar placement test requires a mounted installation")
    fixture = {}
    lifecycle = action == "avatar-live" and bool(os.environ.get("OPENSHORES_TEST_AVATAR_LIFECYCLE"))
    south = action in ("avatar", "avatar-live") and bool(os.environ.get("OPENSHORES_TEST_AVATAR_SOUTH"))
    first_person_mode = os.environ.get("OPENSHORES_TEST_AVATAR_FIRST_PERSON")
    first_person = action == "avatar" and bool(first_person_mode)
    animation = action == "avatar" and bool(os.environ.get("OPENSHORES_TEST_AVATAR_ANIMATION"))
    received_animation = animation and bool(os.environ.get("OPENSHORES_TEST_AVATAR_RECEIVED_POSES"))
    animation_first_person = animation and bool(os.environ.get("OPENSHORES_TEST_AVATAR_ANIMATION_FIRST_PERSON"))
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
        observer = None
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
                "OPENSHORES_LOG_LEVEL": "DEBUG" if animation and not received_animation else "INFO",
            })
            if lifecycle:
                os.environ["OPENSHORES_TEST_AVATAR_LIFECYCLE"] = "1"
            if south:
                os.environ["OPENSHORES_TEST_AVATAR_SOUTH"] = "1"
            if first_person:
                os.environ["OPENSHORES_TEST_AVATAR_FIRST_PERSON"] = "1"
            if animation:
                os.environ["OPENSHORES_TEST_AVATAR_ANIMATION"] = "1"
            if animation_first_person:
                os.environ["OPENSHORES_TEST_AVATAR_ANIMATION_FIRST_PERSON"] = "1"
            if received_animation:
                os.environ["OPENSHORES_TEST_AVATAR_POSE_CONTROL"] = str(runtime / "pose-control")
            if graphical:
                if installation.is_dir():
                    os.environ["HAZERON_INSTALL"] = str(installation)
                os.environ["OPENSHORES_ASSET_CACHE"] = str(runtime / "assets")
            if action == "conversation":
                os.environ["OPENSHORES_TEST_CONVERSATION"] = "1"
            os.chdir(runtime)
            print("Generating the reference world and synthetic avatar", flush=True)
            avatar = asyncio.run(seed(dsn))
            if action == "use" or graphical:
                async def hungry_avatar():
                    import asyncpg
                    conn = await asyncpg.connect(dsn)
                    try:
                        await conn.execute('UPDATE "a_Person" SET "hunger"=40, "autoEat"=0 WHERE "id"=$1', avatar)
                        if graphical:
                            from openshores.database.repositories.bundle import pick_boot_globe
                            from openshores.database.repositories.world_loader import _terrain_floats
                            from openshores.gameplay.worldgen.real import create_terrain_data
                            from openshores.gameplay.worldgen.terrain import terrain_altitude_msl
                            from openshores.protocol.rng import AuDice
                            globe = await pick_boot_globe(conn)
                            radius = float(globe["radius"] or 4)
                            size = (int(radius) if radius <= 255 else next(
                                (code for limit, code in [(5000, 1), (8000, 2), (12000, 3), (16000, 4),
                                                         (22000, 5), (30000, 6), (45000, 8), (70000, 10)]
                                 if radius * 2 < limit), 12))
                            water = (bytes(globe["water"])[0] if globe["water"] else 50) or 50
                            terrain = _terrain_floats(globe["terrain"]) or create_terrain_data(
                                AuDice(seed=int(globe["id"])), size, water, zone_count=3 if size > 2 else 1)
                            # Owned player uses native nominal CG clearance when
                            # mounted; other diagnostics retain the old +5.
                            latitude = -math.pi / 2 if south else math.pi / 2
                            floor = size * 1000 / 1.609344 * 5 + terrain_altitude_msl(terrain, size, latitude, 0)
                            z = (floor + (anchor[0] if anchor else 5)) * (-1 if south else 1)
                            fixture.update(floor=floor, z=z)
                            await conn.execute('UPDATE "a_Person" SET "locZ"=$1 WHERE "id"=$2', z, avatar)
                    finally:
                        await conn.close()
                asyncio.run(hungry_avatar())
            observer_avatar = asyncio.run(seed(dsn, "rust-local-observer")) if action == "peer" else None
            for sock in reservations:
                sock.close()
            print(f"Starting actual reference server on loopback; avatar={avatar}", flush=True)
            log_path = runtime / "server.log"
            with log_path.open("w") as log:
                server_command = ([sys.executable, str(ROOT / "tests/conversation_server.py")]
                                  if action == "conversation" else [sys.executable, "-m", "openshores"])
                if action == "avatar-live" or south or received_animation:
                    server_command = [sys.executable, str(ROOT / "tests/avatar_live_server.py")]
                server = subprocess.Popen(server_command, stdout=log, stderr=log)
                deadline = time.monotonic() + 120
                while "[boot] login  listening" not in log_path.read_text():
                    if server.poll() is not None or time.monotonic() > deadline:
                        raise RuntimeError("reference server failed to start:\n" + log_path.read_text()[-8000:])
                    time.sleep(0.2)
                print("Checking authenticated login, redirect, resume, chat ACK, and manifest", flush=True)
                if action in ("play", "quick-dev", "quick-smoke", "avatar", "avatar-live"):
                    command = [binary, f"127.0.0.1:{login}", "--scene-port", str(scene), "--chat-port", str(chat)]
                    if action == "play":
                        print(f"Owned disposable demo credentials: {USERNAME} / {PASSWORD}", flush=True)
                    else:
                        command += ["--auto-login", USERNAME, str(avatar)]
                    print("Close the client window to clean up the disposable local world.", flush=True)
                    if action in ("avatar", "avatar-live"):
                        directory = Path(tempfile.mkdtemp(prefix="connected-avatar-", dir=ROOT / "target"))
                        from openshores.protocol.login_reply import _default_dna
                        original_dna = _default_dna()
                        smaller_dna = bytearray(original_dna)
                        # Change only the verified size selector, retaining anatomy.
                        smaller_dna[13] = (6 << 3) | (smaller_dna[13] & 7)
                        smaller_dna = bytes(smaller_dna)
                        from local_avatar_preparation import anchor_metrics
                        smaller_anchor = anchor_metrics(str(installation), smaller_dna)
                        assert smaller_anchor[2] - smaller_anchor[1] < anchor[2] - anchor[1]
                        cases = [("landed", 0., original_dna, anchor),
                                 ("off-ground", 5., original_dna, anchor),
                                 ("smaller-received", 0., smaller_dna, smaller_anchor)]
                        if south:
                            cases = [("south-landed", 0., original_dna, anchor),
                                     ("south-off-ground", 5., original_dna, anchor)]
                        if first_person:
                            cases = [("first-person", 0., original_dna, anchor),
                                     ("first-person-down", 0., original_dna, anchor),
                                     ("elevated-look-down", 0., original_dna, anchor),
                                     ("inspection-restored", 0., original_dna, anchor)]
                            if first_person_mode == "elevated":
                                cases = [("elevated-look-down", 0., original_dna, anchor)]
                        if action == "avatar-live":
                            cases = [("live-update", 0., original_dna, smaller_anchor)]
                            if south:
                                cases = [("south-live-update", 0., original_dna, smaller_anchor)]
                            os.environ["OPENSHORES_TEST_AVATAR_LIVE"] = "1"
                            if lifecycle:
                                cases = [("live-lifecycle", 0., original_dna, anchor)]
                        if animation:
                            cases = [("animation-stopped", 0., original_dna, anchor)]
                        for label, elevation, dna, metrics in cases:
                            if first_person:
                                os.environ["OPENSHORES_TEST_AVATAR_FIRST_PERSON"] = {
                                    "first-person": "1", "first-person-down": "down",
                                    "elevated-look-down": "elevated",
                                    "inspection-restored": "restore",
                                }[label]
                            z = fixture["floor"] + (anchor[0] if action == "avatar-live" else metrics[0]) + elevation
                            if south:
                                z = -z
                            async def place_avatar():
                                import asyncpg
                                conn = await asyncpg.connect(dsn)
                                try:
                                    await conn.execute('UPDATE "a_Person" SET "locX"=0, "locY"=0, "locZ"=$1, "dna"=$2 WHERE "id"=$3', z, dna, avatar)
                                finally:
                                    await conn.close()
                            asyncio.run(place_avatar())
                            screenshot = directory / f"{label}.png"
                            os.environ["OPENSHORES_TEST_AVATAR_CAPTURE"] = str(screenshot)
                            os.environ["OPENSHORES_TEST_AVATAR_POSITION"] = f"0,0,{z}"
                            if action == "avatar-live" or animation:
                                os.environ.pop("OPENSHORES_TEST_AVATAR_POSITION", None)
                            expected_dna = smaller_dna if action == "avatar-live" and not lifecycle else dna
                            os.environ["OPENSHORES_TEST_AVATAR_DNA"] = expected_dna.hex()
                            os.environ["OPENSHORES_TEST_AVATAR_HEIGHT"] = str(metrics[2] - metrics[1])
                            result = run(*command, input=PASSWORD + "\n", timeout=180)
                            print(result.stdout, end="", flush=True)
                            assert "Avatar body prepared:" in result.stdout
                            assert "Connected avatar capture saved:" in result.stdout
                            if action == "avatar-live":
                                assert "server-received motion/turn echoed; root and meshes reused" in result.stdout
                                assert "sparse received DNA update replaced root and released old meshes" in result.stdout
                                assert "AVATAR LIVE report 2:" in log_path.read_text()
                                if lifecycle:
                                    assert "received manifest removal released root, children, meshes, material and atlas" in result.stdout
                                    assert "visibility restore reused completed CPU body" in result.stdout
                                    assert "disconnect cleanup remained empty for 30 frames" in result.stdout
                                    assert "same-player authenticated reconnect advanced generation" in result.stdout
                                    assert "AVATAR LIFECYCLE manifest removed" in log_path.read_text()
                                    assert "AVATAR LIFECYCLE manifest restored" in log_path.read_text()
                            elif animation:
                                for gait in (() if received_animation else ("walk", "run")):
                                    assert f"{gait} keyboard input moved player and changed native mesh phases" in result.stdout
                                    for phase in ("a", "b"):
                                        assert (directory / f"{gait}-{phase}.png").is_file()
                                if received_animation:
                                    assert "received mixed phases and unsupported/recovery verified" in result.stdout
                                    if animation_first_person:
                                        assert "received actions followed animated eye and mouse-look with hidden skull" in result.stdout
                                    report_log = log_path.read_text()
                                    assert "AVATAR POSE parsed client report during bootstrap" in report_log
                                    assert "AVATAR POSE parsed client report during stand" in report_log
                                    assert "AVATAR POSE unexpected report during" not in report_log, "local prediction overwrote a received action"
                                    print("Connected avatar animation: source decoder observed neutral reports and no action-overwriting reports after transport grace", flush=True)
                                    for request in ("mixed", "dying", "dead", "wave", "jump", "fold-18", "fold-19", "fold-1b", "fold-35", "point", "tilt-up", "tilt-down", "tilt-reset", "unsupported", "stand"):
                                        assert f"AVATAR POSE sent {request}:" in log_path.read_text()
                                    for tilt in ("tilt-up", "tilt-down", "tilt-reset"):
                                        assert (directory / f"{tilt}.png").is_file()
                                    for phase in ("a", "b"):
                                        assert (directory / f"received-{phase}.png").is_file()
                                        assert (directory / f"wave-{phase}.png").is_file()
                                        assert (directory / f"jump-{phase}.png").is_file()
                                        assert (directory / f"point-{phase}.png").is_file()
                                        for code in ("18", "19", "1b", "35"):
                                            assert (directory / f"fold-{code}-{phase}.png").is_file()
                                    for pose in ("dying", "dead"):
                                        assert (directory / f"received-{pose}.png").is_file()
                                    assert "Connected avatar received/render appearance verified" in result.stdout
                                    assert screenshot.is_file() and screenshot.stat().st_size > 1000
                                    assert "Shader compilation error" not in result.stdout
                                    assert PASSWORD not in result.stdout
                                    continue
                                assert "released input restored exact Stand/eye" in result.stdout
                                pose_log = log_path.read_text()
                                for before, after in (("24", "26"), ("26", "25"), ("25", "24")):
                                    assert f"pose 0x{before} -> 0x{after}" in pose_log, "actual server did not receive locomotion pose transition"
                                print("Connected avatar animation: actual server tracked Walk/Run/Stand pose reports", flush=True)
                                if animation_first_person:
                                    assert "moving first-person followed animated eye with hidden skull" in result.stdout
                                async def verify_animation_reports():
                                    import asyncpg
                                    conn = await asyncpg.connect(dsn)
                                    try:
                                        row = await conn.fetchrow('SELECT "locX", "locY" FROM "a_Person" WHERE "id"=$1', avatar)
                                        assert abs(row["locX"]) + abs(row["locY"]) > 1., "server did not persist actual keyboard movement reports"
                                    finally:
                                        await conn.close()
                                asyncio.run(verify_animation_reports())
                                print("Connected avatar animation: local server persisted keyboard movement reports", flush=True)
                            else:
                                assert "Connected avatar received/render origin verified" in result.stdout
                            assert "Connected avatar received/render appearance verified" in result.stdout
                            if south:
                                assert "southern received/render orientation verified" in result.stdout
                                assert "AVATAR SOUTH bootstrap orientation" in log_path.read_text()
                            assert PASSWORD not in result.stdout
                            assert screenshot.is_file() and screenshot.stat().st_size > 1000
                            assert "Shader compilation error" not in result.stdout
                            assert "warning[B0004]" not in result.stdout
                            print(f"Connected avatar {label}: {screenshot}; initial DNA={dna.hex()}, rendered DNA={expected_dna.hex()}, nominal ground clearance={metrics[0]}, sole residual={metrics[0]+metrics[1]}, elevation={elevation}", flush=True)
                    elif action == "quick-smoke":
                        result = run(*command, "--smoke-frames", "30", input=PASSWORD + "\n")
                        print(result.stdout, end="", flush=True)
                        assert "Quick-dev connected to avatar" in result.stdout
                        assert "Mounted " in result.stdout and "geometry conversions" in result.stdout
                        assert PASSWORD not in result.stdout
                        assert "UseGearItem" not in log_path.read_text()
                        assert "DropGearItem" not in log_path.read_text()
                    else:
                        subprocess.run(command, check=True, text=True,
                                       input=PASSWORD + "\n" if action == "quick-dev" else None)
                    return
                if action in ("gui", "conversation"):
                    screenshot = Path(os.environ.get("GUI_SCREENSHOT", str(runtime / "gui.png")))
                    result = run(str(ROOT / "target/debug/openshores-client-slice"), f"127.0.0.1:{login}",
                                 "--scene-port", str(scene), "--chat-port", str(chat),
                                 "--local-smoke", USERNAME, str(avatar), "--screenshot", str(screenshot), input=PASSWORD + "\n")
                    print(result.stdout, end="", flush=True)
                    assert PASSWORD not in result.stdout
                    assert "GUI slice: clean disconnect passed" in result.stdout
                    assert "GUI slice: authenticated Unicode chat round-trip passed" in result.stdout
                    assert "GUI slice: selected Universe chat range round-trip passed" in result.stdout
                    if os.environ.get("HAZERON_INSTALL"):
                        assert "GUI slice: catalog-bound dropped Flag model rendered from user installation" in result.stdout
                    if action == "conversation":
                        assert "GUI slice: actual native hail/choice/continuation passed" in result.stdout
                    assert screenshot.is_file() and screenshot.stat().st_size > 1000
                    assert "UseGearItem (ate Beans)" in log_path.read_text()
                    async def verify_gui_state():
                        import asyncpg
                        conn = await asyncpg.connect(dsn)
                        try:
                            deadline = time.monotonic() + 10
                            while True:
                                row = await conn.fetchrow('SELECT "locX", "hunger", "isonline" FROM "a_Person" WHERE "id"=$1', avatar)
                                if row["locX"] == 2.0 and row["hunger"] > 40 and row["isonline"] == 0:
                                    return
                                assert time.monotonic() < deadline, tuple(row)
                                await asyncio.sleep(0.1)
                        finally:
                            await conn.close()
                    asyncio.run(verify_gui_state())
                    print("Full graphical local slice and persisted position/hunger/offline state passed", flush=True)
                    return
                if observer_avatar is not None:
                    observer = subprocess.Popen([sys.executable, str(ROOT / "tests/local_peer.py"),
                        str(login), str(scene), str(chat), str(observer_avatar), str(avatar),
                        str(runtime / "peer.ready"), str(runtime / "peer.frames")])
                    deadline = time.monotonic() + 40
                    while not (runtime / "peer.ready").exists():
                        assert observer.poll() is None and time.monotonic() < deadline, "observer bootstrap failed"
                        time.sleep(0.1)
                action_args = (["--use-gear", "5", "3"] if action == "use" else
                               ["--use-gear", "5", "0"] if action == "use-noop" else
                               ["--pose-report", "25252525252525252525"] if action == "pose" else
                               ["--position-report", "12.5", "-3.25", "1001", "0.25", "-0.5", "0.75"] if action in ("position", "peer") else
                               ["--pickup-target", "0x70000000"] if action == "pickup" else
                               ["--drop-gear", "5", "0"] if action == "drop" else
                               ["--move-gear", "5", "0", "1"])
                result = run(binary, "login", USERNAME, f"127.0.0.1:{login}",
                             "--avatar", str(avatar), "--scene-port", str(scene),
                             "--chat-port", str(chat), "--watch-seconds", "3",
                             "--look-at", "0x70000000", *action_args,
                             "--capture", str(runtime / "watched.frames"), "--password-stdin",
                             input=PASSWORD + "\n")
                print(result.stdout, end="", flush=True)
                assert "Scene initialized" in result.stdout
                assert PASSWORD not in result.stdout
                assert "client ACK for" in log_path.read_text(), "reference server did not observe init ACK"
                if observer is not None:
                    assert observer.wait(timeout=20) == 0
                    peer_replay = run(binary, "inspect-scene", str(runtime / "peer.frames")).stdout
                    sender_line = next(line for line in peer_replay.splitlines() if line.startswith(f"{avatar:#010x} Person "))
                    assert "position: [12.5, -3.25, 1001.0]" in sender_line and "rotation: [0.25, -0.5, 0.75]" in sender_line, sender_line
                    print("Rust replay decoded the peer-observed transform", flush=True)
                    # Observer lifecycle changes manifests/entity counts; the
                    # single-avatar baseline assertions below do not apply.
                    return
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
                    manifests = [p for p in packets if p[0] == 0x18 and len(p) >= 8 and p[5] == 1]
                    if action == "pickup":
                        final_ids = [int.from_bytes(manifests[-1][i:i+4], "big")
                                     for i in range(8, len(manifests[-1]), 4)]
                        assert 0x70000000 not in final_ids
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
                assert "Commodity catalog: 615 server-provided rows." in replay.stdout, replay.stdout
                assert '(5, 0, 1, Some("Flag"))' in replay.stdout, replay.stdout
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
                assert "Commodity catalog: 615 server-provided rows." in watched_replay.stdout
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
                assert f"Item: {2 if action == 'drop' else 1}" in watched_replay.stdout, watched_replay.stdout
                print("Dynamic floor item decoded and retained in ongoing scene state", flush=True)
                watched_player = next(line for line in watched_replay.stdout.splitlines()
                                      if line.startswith(f"{avatar:#010x} Person "))
                moved = "slot_type: 1, subindex: 0, item: Some(InventoryItem { type_id: 1, common: AuItemPrefix { flags: 12, commodity_id: 1,"
                original = "slot_type: 5, subindex: 0, item: Some(InventoryItem { type_id: 1, common: AuItemPrefix { flags: 12, commodity_id: 1,"
                assert original in player_line, player_line
                if action in ("move", "drop"):
                    assert original not in watched_player, watched_player
                if action == "move":
                    assert moved in watched_player, watched_player
                elif action == "drop":
                    assert "declared_slots: 6" in watched_player, watched_player
                    dropped = [line for line in watched_replay.stdout.splitlines()
                               if " Item " in line and "commodity_id: 1," in line]
                    assert len(dropped) == 1, watched_replay.stdout
                    assert "parent=Some(" in dropped[0] and "transform=Some(" in dropped[0], dropped[0]
                elif action == "pickup":
                    assert original in watched_player, watched_player
                    assert "declared_slots: 8" in watched_player and "commodity_id: 130" in watched_player, watched_player
                    assert "PICKUP target=0x70000000" in log_path.read_text(), log_path.read_text()[-8000:]
                elif action == "use":
                    eaten = "slot_type: 5, subindex: 3, item: Some(InventoryItem { type_id: 1, common: AuItemPrefix { flags: 12, commodity_id: 15,"
                    assert eaten in player_line and eaten not in watched_player
                    assert original in watched_player and "declared_slots: 6" in watched_player
                    remaining_beans = "slot_type: 4, subindex: 0, item: Some(InventoryItem { type_id: 8, common: AuItemPrefix { flags: 12, commodity_id: 15,"
                    assert remaining_beans in watched_player
                    initial_hunger = int(player_line.split("hunger_raw: Some(", 1)[1].split(")", 1)[0]) & 0x7fff
                    final_hunger = int(watched_player.split("hunger_raw: Some(", 1)[1].split(")", 1)[0]) & 0x7fff
                    assert final_hunger > initial_hunger
                    assert "UseGearItem (ate Beans)" in log_path.read_text()
                    async def persisted_hunger():
                        import asyncpg
                        conn = await asyncpg.connect(dsn)
                        try:
                            deadline = time.monotonic() + 10
                            while True:
                                hunger = await conn.fetchval('SELECT "hunger" FROM "a_Person" WHERE "id"=$1', avatar)
                                if hunger > 40:
                                    return
                                assert time.monotonic() < deadline, hunger
                                await asyncio.sleep(0.1)
                        finally:
                            await conn.close()
                    asyncio.run(persisted_hunger())
                elif action == "use-noop":
                    assert original in watched_player and "declared_slots: 7" in watched_player
                    assert "UseGearItem [no-op]" in log_path.read_text()
                if action in ("position", "pose"):
                    assert original in watched_player and "declared_slots: 7" in watched_player
                    async def verify_position():
                        import asyncpg
                        conn = await asyncpg.connect(dsn)
                        try:
                            deadline = time.monotonic() + 10
                            while True:
                                row = await conn.fetchrow('SELECT "locX", "locY", "locZ", "pose", "stamina" FROM "a_Person" WHERE "id"=$1', avatar)
                                if ((action == "position" and tuple(row)[:3] == (12.5, -3.25, 1001.0)) or
                                    (action == "pose" and row["pose"] == 0x25 and row["stamina"] < 100)):
                                    return
                                assert time.monotonic() < deadline, tuple(row)
                                await asyncio.sleep(0.1)
                        finally:
                            await conn.close()
                    asyncio.run(verify_position())
                    if action == "pose":
                        # The server sends stamina deltas, not an optimistic pose echo.
                        stamina = int(watched_player.split("stamina: Some(", 1)[1].split(")", 1)[0])
                        assert 0 < stamina < 100, watched_player
                    print(f"Rust {action} report persisted by reference server; inventory unchanged", flush=True)
                else:
                    assert "Observed player gear (latest received batch):" in watched.stdout
                    print(f"Explicit gear {action} confirmed by server-originated inventory/world refresh", flush=True)
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
            if observer is not None and observer.poll() is None:
                observer.terminate()
                observer.wait(timeout=10)
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
