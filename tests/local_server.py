"""Boot the actual reference server with an isolated PostgreSQL database.

cargo build
nix develop .#server-test --command python tests/local_server.py

All credentials/world data are synthetic. The server runs from an owned /tmp
directory; the reference source, accounts, and any existing databases are untouched.
"""
from __future__ import annotations

import asyncio
import getpass
import os
from pathlib import Path
import signal
import socket
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
                             "--chat-port", str(chat), "--capture", str(runtime / "scene.frames"), "--password-stdin",
                             input=PASSWORD + "\n")
                print(result.stdout, end="", flush=True)
                assert "Scene initialized" in result.stdout
                assert PASSWORD not in result.stdout
                assert "client ACK for" in log_path.read_text(), "reference server did not observe init ACK"
                async def verify_capture():
                    from openshores.protocol.framing import read_framed
                    reader = asyncio.StreamReader()
                    reader.feed_data((runtime / "scene.frames").read_bytes())
                    reader.feed_eof()
                    packets = []
                    while not reader.at_eof():
                        packets.append(await read_framed(reader))
                    assert len(packets) > 100
                    assert any(p == bytes([0x2A]) + avatar.to_bytes(4, "big") for p in packets)
                    assert any(p[0] == 0x38 for p in packets)
                    assert packets[-1][0] == 0x18 and packets[-1][5] == 1
                    count = int.from_bytes(packets[-1][6:8], "big")
                    ids = [int.from_bytes(packets[-1][i:i+4], "big") for i in range(8, len(packets[-1]), 4)]
                    assert len(ids) == count and avatar in ids
                    print(f"Captured {len(packets)} scene packets; final manifest verified", flush=True)
                asyncio.run(verify_capture())
                print("Full local reference-server integration passed", flush=True)
        except BaseException:
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
