"""Owned-test-only sparse appearance/rotation and pose fixtures.

Runs the supplied server's real login, bootstrap and position-report receiver.
Its player transform writer currently emits zero rotation and no DNA updates;
this explicit fixture emits two source-profile updates after actual reports.
Pose mode instead emits requested sparse pose updates and observes the unchanged
source delta-tick parser to detect client reports overriding received actions.
It does not edit the reference source or claim unmodified-server echo parity.
"""
import asyncio
import contextlib
import functools
import inspect
import os
from pathlib import Path
import runpy
import struct
import time

from openshores.network.scene import connection
from openshores.protocol.framing import write_framed
from openshores.protocol.login_reply import _default_dna

original = connection.handle_scene
pose_phase = ("bootstrap", 0.)

if os.environ.get("OPENSHORES_TEST_AVATAR_POSE_CONTROL") or os.environ.get("OPENSHORES_TEST_AVATAR_LOCAL_GESTURES") or os.environ.get("OPENSHORES_TEST_AVATAR_LOCAL_CROUCH"):
    original_parse_delta = connection._parse_delta_tick

    @functools.wraps(original_parse_delta)
    def observe_pose_reports(payload):
        result = original_parse_delta(payload)
        if isinstance(result, dict) and result.get("poses"):
            if os.environ.get("OPENSHORES_TEST_AVATAR_LOCAL_GESTURES"):
                poses = bytes(int(v) & 255 for v in result["poses"])
                print(f"AVATAR GESTURE received poses={poses.hex()} aux={result.get('flag08_i16')},{result.get('flag08_i32')}", flush=True)
            if os.environ.get("OPENSHORES_TEST_AVATAR_LOCAL_CROUCH"):
                poses = bytes(int(v) & 255 for v in result["poses"])
                print(f"AVATAR POSTURE received poses={poses.hex()} aux={result.get('flag08_i16')},{result.get('flag08_i32')}", flush=True)
            phase, sent = pose_phase
            print(f"AVATAR POSE parsed client report during {phase}", flush=True)
            # Allow transport already queued before the sparse server update.
            if (phase in ("mixed", "wave", "point", "jump", "dying", "dead", "dead-left", "dead-right", "unsupported", "crouch", "crouch-walk") or phase.startswith("fold-")) and time.monotonic() - sent > 0.3:
                print(f"AVATAR POSE unexpected report during {phase}", flush=True)
        return result

    connection._parse_delta_tick = observe_pose_reports

if os.environ.get("OPENSHORES_TEST_AVATAR_SOUTH"):
    from openshores.gameplay.natives.village import gravity_align_euler
    original_base = connection._bpt2_tc

    @functools.wraps(original_base)
    def southern_base(parent, created, tx=0., ty=0., tz=0., rx=0., ry=0., rz=0.):
        # Selected-player bootstrap hard-codes zero rotation; supply an explicit
        # correct test-server orientation rather than correcting it in Rust.
        if tz < 0 and (rx, ry, rz) == (0., 0., 0.):
            rx, ry, rz = gravity_align_euler((tx, ty, tz))
            print(f"AVATAR SOUTH bootstrap orientation: position={(tx,ty,tz)}, rotation={(rx,ry,rz)}", flush=True)
        return original_base(parent, created, tx, ty, tz, rx, ry, rz)

    connection._bpt2_tc = southern_base


async def observe_reports(live, writer):
    if os.environ.get("OPENSHORES_TEST_AVATAR_LOCAL_GESTURES") or os.environ.get("OPENSHORES_TEST_AVATAR_LOCAL_CROUCH"):
        return  # Decoder observation only; no appearance/pose echo injection.
    if os.environ.get("OPENSHORES_TEST_AVATAR_POSE_CONTROL"):
        await observe_pose_requests(live, writer)
        return
    previous = None
    count = 0
    lifecycle = bool(os.environ.get("OPENSHORES_TEST_AVATAR_LIFECYCLE"))
    while not writer.is_closing() and count < (3 if lifecycle else 2):
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
            if count == 3:
                full = writer._scene_manifest_builder()
                assert full[0] == 0x18 and full[5] == 1
                ids = [value[0] for value in struct.iter_unpack(">I", full[8:])]
                assert auid in ids
                remaining = [value for value in ids if value != auid]
                removed = full[:6] + struct.pack(">h", len(remaining)) + b"".join(struct.pack(">I", value) for value in remaining)
                await write_framed(writer, removed)
                print(f"AVATAR LIFECYCLE manifest removed player={auid}", flush=True)
                await asyncio.sleep(1)
                await write_framed(writer, writer._scene_manifest_builder())
                print(f"AVATAR LIFECYCLE manifest restored player={auid}", flush=True)
                continue
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


async def observe_pose_requests(live, writer):
    """Explicit test writer: real framed sparse packets, never client injection."""
    global pose_phase
    control = Path(os.environ["OPENSHORES_TEST_AVATAR_POSE_CONTROL"])
    previous = None
    channels = {
        "crouch": bytes([0x10] * 10),
        "crouch-walk": bytes([0x11] * 10),
        "point": bytes([0x24, 0x24, 0x1f, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24]),
        **{f"fold-{code:02x}": bytes([code] * 10) for code in (0x18,0x19,0x1b,0x35)},
        "jump": bytes([0x1c] * 10),
        "wave": bytes([0x24, 0x24, 0x30, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24]),
        "mixed": bytes([0x26, 0x24, 0x25, 0x24, 0x25, 0x24, 0x24, 0x24, 0x24, 0x24]),
        "dying": bytes([0x12] * 10),
        "dead": bytes([0x15] * 10),
        "dead-left": bytes([0x13] * 10),
        "dead-right": bytes([0x14] * 10),
        "unsupported": bytes([0x17] * 10),
        "stand": bytes([0x24] * 10),
        "tilt-up": bytes([0x24] * 10),
        "tilt-down": bytes([0x24] * 10),
        "tilt-reset": bytes([0x24] * 10),
    }
    while not writer.is_closing():
        await asyncio.sleep(0.05)
        if not control.exists():
            continue
        request = control.read_text().strip()
        if request == previous or request not in channels:
            continue
        for auid, entry in list(live.items()):
            if entry.get("writer") is not writer:
                continue
            # Source received head tilt is a float, independent of the client's
            # outgoing i16 delta-tick field. Down/reset omit pose channels to
            # exercise sparse merge and the stationary animation fast path.
            tilt = {"tilt-up": 0.45, "tilt-down": -0.45, "tilt-reset": 0.}.get(request)
            with_pose = request not in ("tilt-down", "tilt-reset")
            flags = (2 if tilt is not None else 0) | (8 if with_pose else 0)
            body = b"\x00" + bytes([flags])
            if tilt is not None:
                body += struct.pack(">f", tilt)
            if with_pose:
                body += channels[request] + struct.pack(">I", 0)
            roots = os.environ.get("OPENSHORES_TEST_AVATAR_CROUCH_ROOTS")
            prefix = b""
            if roots and request in ("crouch", "crouch-walk", "stand"):
                standing, crouched = map(float, roots.split(","))
                prefix = struct.pack(">6f", 0., 0., standing if request == "stand" else crouched, 0., 0., 0.)
            packet = (b"\x12" + struct.pack(">IqB", auid, int(time.time()*1000), 8 if prefix else 0)
                      + prefix + body + b"\x00\x00\x00")
            await write_framed(writer, packet)
            pose_phase = (request, time.monotonic())
            previous = request
            print(f"AVATAR POSE sent {request}: player={auid}", flush=True)
            break


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
