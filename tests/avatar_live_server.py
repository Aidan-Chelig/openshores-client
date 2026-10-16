"""Owned-test-only sparse appearance/rotation echo fixture.

Runs the supplied server's real login, bootstrap and position-report receiver.
Its player transform writer currently emits zero rotation and no DNA updates;
this explicit fixture emits two source-profile updates after actual reports.
It does not edit the reference source or claim unmodified-server echo parity.
"""
import asyncio
import contextlib
import functools
import inspect
import runpy
import struct
import time

from openshores.network.scene import connection
from openshores.protocol.framing import write_framed
from openshores.protocol.login_reply import _default_dna

original = connection.handle_scene


async def observe_reports(live, writer):
    previous = None
    count = 0
    while not writer.is_closing() and count < 2:
        await asyncio.sleep(0.05)
        for auid, entry in list(live.items()):
            if entry.get("writer") is not writer:
                continue
            position, rotation = entry.get("live_pos"), entry.get("live_rot")
            if position is None or rotation is None:
                continue
            report = (tuple(position), tuple(rotation))
            if report == previous:
                continue
            previous = report
            count += 1
            packet = (b"\x12" + struct.pack(">IqB", auid, int(time.time()*1000), 8)
                      + struct.pack(">6f", *position, *rotation) + b"\x00")
            if count == 2:
                dna = bytearray(_default_dna())
                dna[13] = (6 << 3) | (dna[13] & 7)
                packet += b"\x01\x24" + struct.pack(">I", 24) + dna
            else:
                packet += b"\x00"
            packet += b"\x00\x00\x00"  # stamina, growth, omitted person fields
            await write_framed(writer, packet)
            print(f"AVATAR LIVE report {count}: player={auid}, position={position}, rotation={rotation}, DNA update={count == 2}", flush=True)


@functools.wraps(original)
async def handle_scene(conn, reader, writer, **kwargs):
    task = asyncio.create_task(observe_reports(kwargs["_live_avatars"], writer))
    try:
        await original(conn, reader, writer, **kwargs)
    finally:
        task.cancel()
        with contextlib.suppress(asyncio.CancelledError):
            await task


assert inspect.signature(handle_scene) == inspect.signature(original), "preserve source dependency-injection signature"
connection.handle_scene = handle_scene
print("TEST FIXTURE: source-profile position/rotation echo and second-report DNA update", flush=True)
runpy.run_module("openshores", run_name="__main__")
