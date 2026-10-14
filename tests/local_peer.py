"""Loopback-only Python observer for the disposable two-avatar integration."""
import asyncio
import hashlib
from pathlib import Path
import struct
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "reference/OpenShores-server/src"))
from openshores.protocol.encryption import au_crypt, net_crypt_key
from openshores.protocol.framing import read_framed, write_framed, encode_size
from openshores.protocol.stream import QDS


async def observe(login, scene, chat, avatar, sender, ready, capture):
    writers = []
    packets = []
    async def connect(port):
        # Distinct loopback source avoids the reference server's per-IP session
        # routing. This is another synthetic player, not a reconnect attempt.
        reader, writer = await asyncio.open_connection(
            "127.0.0.1", port, local_addr=("127.0.0.2", 0))
        writers.append(writer)
        return reader, writer

    def resume(hi, lo):
        out = QDS()
        out.write_u8(0x38)
        out.write_qstring("13.110.60")
        return out.getvalue() + struct.pack(">IIIBHBB", hi, lo, avatar, 0, 0, 0x40, 0)

    try:
        reader, writer = await connect(login)
        out = QDS()
        out.write_u8(3)
        out.write_qstring("13.110.60")
        out.write_bytes(au_crypt(b"rust-local-observer", net_crypt_key(0x101)))
        out.write_bytes(hashlib.sha1(b"synthetic-local-test-password").digest())
        await write_framed(writer, out.getvalue())
        assert (await read_framed(reader))[0] == 3
        writer.close()
        await writer.wait_closed()
        reader, initial = await connect(scene)
        await write_framed(initial, resume(0, 0))
        redirect = QDS(await read_framed(reader))
        assert redirect.read_u8() == 0x22
        assert redirect.read_qstring() in ("127.0.0.1", "localhost", "")
        assert redirect.read_u16() == scene
        redirect.read_u8()
        lo, hi = redirect.read_u32(), redirect.read_u32()
        reader, world = await connect(scene)
        await write_framed(world, resume(hi, lo))
        active = initialized = False
        total = 0
        for _ in range(20000):
            packet = await read_framed(reader)
            total += len(packet)
            assert total <= 128 * 1024 * 1024
            packets.append(packet)
            if packet[0] == 0x2A:
                assert int.from_bytes(packet[1:], "big") == avatar
                active = True
            elif packet[0] == 0x38:
                assert active
                _, channel = await connect(chat)
                await write_framed(channel, bytes([0x0B]) + struct.pack(">III", lo, hi, avatar))
                initialized = True
            elif packet[0] == 0x18 and len(packet) >= 8 and packet[5] == 1 and initialized:
                assert avatar in [int.from_bytes(packet[i:i+4], "big") for i in range(8, len(packet), 4)]
                Path(ready).touch()
            if packet[0] == 0x12 and int.from_bytes(packet[1:5], "big") == sender and packet[13] == 8:
                values = struct.unpack(">6f", packet[14:38])
                if values == (12.5, -3.25, 1001.0, 0.25, -0.5, 0.75):
                    Path(capture).write_bytes(b"".join(encode_size(len(p)) + p for p in packets))
                    print("Peer received exact sender position and rotation", flush=True)
                    return
        raise AssertionError("peer packet budget exceeded")
    finally:
        for writer in writers:
            writer.close()
        await asyncio.gather(*(w.wait_closed() for w in writers), return_exceptions=True)


if __name__ == "__main__":
    asyncio.run(asyncio.wait_for(observe(*map(int, sys.argv[1:6]), *sys.argv[6:8]), 60))
