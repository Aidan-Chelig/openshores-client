"""Exercise the Rust executable against the supplied Python protocol codecs.

Run after `cargo build`: python3 tests/reference_interop.py
Uses temporary loopback sockets and synthetic credentials; no database required.
"""

from __future__ import annotations

import asyncio
import hashlib
import os
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "reference/OpenShores-server/src"))

from openshores.protocol.encryption import VERSION_STRING
from openshores.protocol.framing import read_framed, encode_size
from openshores.protocol.login import (
    parse_login_request, build_login_fail_reply, build_redirect_reply,
    build_version_mismatch_reply,
)
from openshores.protocol.login_reply import build_login_ok_reply_for_avatars
from openshores.protocol.scene_init import (
    build_scene_world_redirect, build_scene_init_succeeded,
)
from openshores.protocol.scene_parse import _parse_scene_0x38
from openshores.protocol.stream import QDS
from openshores.protocol.atoms.base_update import _bpt2
from openshores.protocol.atoms.galaxy_rename import _build_sector_atom_rename_pkt
import struct


def check_fixtures() -> None:
    fixtures = ROOT / "tests/fixtures"
    assert bytes.fromhex((fixtures / "login-ok.hex").read_text()) == (
        build_login_ok_reply_for_avatars(
            42, [(42, "Explorer 🚀", bytes(range(24)), 1, True)]
        )
    )
    assert bytes.fromhex((fixtures / "scene-redirect.hex").read_text()) == (
        build_scene_world_redirect(
            server_name="127.0.0.1", account_id_lo=0x89ABCDEF, account_id_hi=-2
        )
    )
    assert bytes.fromhex((fixtures / "scene-init.hex").read_text()) == (
        build_scene_init_succeeded("Welcome 🚀", 0x123456789ABCDEF0)
    )
    resume = bytes.fromhex((fixtures / "scene-resume.hex").read_text())
    stream = QDS(resume)
    assert stream.read_u8() == 0x38
    assert _parse_scene_0x38(stream) == {
        "sceneVersion": VERSION_STRING, "session_hi": -2, "session_lo": 1,
        "charId": 42, "osKind": 0, "winVersion": 0, "flag40": 0x40, "extra": 0,
    }
    assert stream.pos == len(resume)
    sector = QDS()
    sector.write_qstring("Sector 🚀")
    assert bytes.fromhex((fixtures / "atom-sector.hex").read_text()) == (
        bytes([0x14]) + struct.pack(">Iq", 42, 1000)
        + _bpt2(struct.pack(">I", 117), 1.25, -2.5, 3, 0.1, 0.2, 0.3)
        + sector.getvalue()
    )
    assert bytes.fromhex((fixtures / "atom-sector-rename.hex").read_text()) == (
        _build_sector_atom_rename_pkt(42, "Renamed", time_ms=2000)
    )


async def check_login(binary: str, reply: bytes, code: int, output: str) -> None:
    password = "  test café 🚀  "
    handled = asyncio.get_running_loop().create_future()

    async def handle(reader, writer):
        try:
            request = parse_login_request(await asyncio.wait_for(read_framed(reader), 5))
            assert request.version == VERSION_STRING
            assert request.username == "Explorér"
            assert request.password_hash == hashlib.sha1(password.encode()).digest()
            # Exercise real framing plus separately delivered prefix/payload bytes.
            for byte in encode_size(len(reply)) + reply:
                writer.write(bytes([byte]))
                await writer.drain()
                await asyncio.sleep(0)
            handled.set_result(None)
        except BaseException as error:
            handled.set_exception(error)
        finally:
            writer.close()
            await writer.wait_closed()

    server = await asyncio.start_server(handle, "127.0.0.1", 0)
    async with server:
        port = server.sockets[0].getsockname()[1]
        process = await asyncio.create_subprocess_exec(
            binary, "login", "Explorér", f"127.0.0.1:{port}", "--password-stdin",
            stdin=asyncio.subprocess.PIPE, stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
        )
        try:
            stdout, stderr = await asyncio.wait_for(
                process.communicate((password + "\r\n").encode()), 10
            )
            await asyncio.wait_for(handled, 5)
            assert process.returncode == code, (process.returncode, stdout, stderr)
            combined = (stdout + stderr).decode()
            assert output in combined, combined
            assert password not in combined
            assert hashlib.sha1(password.encode()).hexdigest() not in combined
        finally:
            if process.returncode is None:
                process.kill()
                await process.wait()


async def main() -> None:
    binary = os.environ.get("OPENSHORES_TEST_BINARY", str(ROOT / "target/debug/openshores-client"))
    check_fixtures()
    await check_login(binary, build_login_ok_reply_for_avatars(
        42, [(42, "Explorer 🚀", bytes(range(24)), 1, True)]
    ), 0, "Slot 1:")
    await check_login(binary, build_login_ok_reply_for_avatars(1, []), 0, "No avatars")
    await check_login(binary, build_login_fail_reply(1), 1, "account not found")
    await check_login(binary, build_login_fail_reply(2), 1, "incorrect password")
    await check_login(binary, build_redirect_reply("127.0.0.1", 16759), 1, "redirect")
    await check_login(binary, build_version_mismatch_reply(), 1, "protocol version")
    await check_login(binary, b"\xff", 1, "unrecognized login reply")
    print("Python/Rust interoperability: six fixtures and seven login exchanges passed")


if __name__ == "__main__":
    asyncio.run(main())
