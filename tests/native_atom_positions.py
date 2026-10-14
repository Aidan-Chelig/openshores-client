"""Static native reader anchors and independent packed-field fixtures.

Does not execute the original image or contact any service. The Python oracle
checks serialized fields, not native matrix mutation or full frame dispatch.
"""
import hashlib
import json
import math
from pathlib import Path
import struct
import subprocess
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path("reference/OpenShores-server/src").resolve()))
from openshores.gameplay.natives.village import body_axes, gravity_align_euler

TARGET = Path("reference/openshores-full/OpenShores.exe")
CLIENT = Path("target/debug/openshores-client")
assert hashlib.sha256(TARGET.read_bytes()).hexdigest() == (
    "285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80")


def assembly(start, end):
    return subprocess.check_output([
        "objdump", "-d", f"--start-address={start:#x}",
        f"--stop-address={end:#x}", str(TARGET)], text=True)


base = assembly(0x140B4E470, 0x140B4E880)
assert "140b4e49a:" in base and "test   $0x1,%al" in base
assert "call   1408b2c50 <_ZN9AuAtomPos2RxER11QDataStream>" in base
assert "call   140b44d70 <_ZN6DaAtom9ChangePosERK9AuAtomPos>" in base
identity = assembly(0x140B27E30, 0x140B27E60)
assert "call   1416353c0 <_ZN4AuIdlsER11QDataStream>" in identity
assert "jmp    14163a380 <_ZN6AuTimelsER11QDataStream>" in identity
time_reader = assembly(0x14163A380, 0x14163A390)
assert "142892710 <__imp__ZN11QDataStreamrsERx>" in time_reader
reader = assembly(0x1408B2C50, 0x1408B33C0)
assert "1428926e0 <__imp__ZN11QDataStreamrsERa>" in reader
assert "call   1416353c0 <_ZN4AuIdlsER11QDataStream>" in reader
assert reader.count("1428926f8 <__imp__ZN11QDataStreamrsERf>") == 6
assert reader.count("1428926f0 <__imp__ZN11QDataStreamrsERd>") == 3
assert reader.count("call   14175b1f0 <sincos>") == 3
assert "test   $0x10,%al" in reader and "test   $0x20,%al" in reader
assert "test   $0x40,%al" in reader and "js     1408b3200" in reader
item_render = assembly(0x140B85330, 0x140B87140)
assert "140b85840:" in item_render and "call   *0x4a8(%rax)" in item_render
assert "140b861bf:" in item_render and "call   *0x608(%rax)" in item_render

rotation = (0.125, -0.25, 1.5)
position = (12345678.125, -42.25, 3.5)


def run(profile, payload, command="inspect-atom-position"):
    return subprocess.run([str(CLIENT), command, profile,
                           payload.hex()], capture_output=True, text=True)


cases = 0
for profile in ("type-widths", "single", "double"):
    for flags in range(256):
        payload = bytearray([flags])
        expected = dict(flags=flags, parent=None, position=[0.0] * 3,
                        rotation=[0.0] * 3)
        if flags & 1:
            payload += struct.pack(">I", 0xF1234567)
            expected["parent"] = 0xF1234567
        for field, values, bits in (
            ("rotation", rotation, (0x10, 0x20, 0x40)),
            ("position", position, (2, 4, 8)),
        ):
            for index, (value, bit) in enumerate(zip(values, bits)):
                if not flags & bit:
                    continue
                double = field == "position" and bool(flags & 0x80)
                width = "d" if (profile == "double" or
                    (profile == "type-widths" and double)) else "f"
                payload += struct.pack(">" + width, value)
                expected[field][index] = value if double and width == "d" else (
                    struct.unpack(">f", struct.pack(">f", value))[0])
        result = run(profile, payload)
        assert result.returncode == 0, (profile, flags, result.stderr)
        assert json.loads(result.stdout) == expected, (profile, flags)
        matrix_result = run(profile, payload, "dump-atom-matrix")
        assert matrix_result.returncode == 0, matrix_result.stderr
        matrix = json.loads(matrix_result.stdout)
        axes = body_axes(*expected["rotation"])
        for axis, column in zip(axes, matrix[:3]):
            assert column[3] == 0
            assert all(math.isclose(a, b, abs_tol=1e-12) for a, b in zip(axis, column))
        assert matrix[3] == [*expected["position"], 1.0]
        cases += 1

# Real source gravity alignment: local +Z must point outward at both poles
# and off-axis locations, after native float-angle narrowing.
for position in ((0, 0, 100), (0, 0, -100), (100, 0, 0), (20, -30, -40)):
    angles = gravity_align_euler(position, heading_rad=0.7)
    payload = bytes([0xFE]) + struct.pack(">3f3d", *angles, *position)
    result = run("type-widths", payload, "dump-atom-matrix")
    assert result.returncode == 0, result.stderr
    matrix = json.loads(result.stdout)
    radius = math.sqrt(sum(v * v for v in position))
    assert all(abs(a - b / radius) < 1e-6 for a, b in zip(matrix[2], position))

for profile in ("type-widths", "single", "double"):
    assert run(profile, b"\x00\x00").returncode != 0
    assert run(profile, b"\x01").returncode != 0
    assert run(profile, b"\x10").returncode != 0
    assert run(profile, b"\x02").returncode != 0

prefix_cases = 0
for profile in ("type-widths", "single", "double"):
    for flags in range(256):
        payload = bytearray(struct.pack(">IqB", 0xF1234567, -123, flags))
        position = None
        if flags & 1:
            width = "f" if profile == "single" else "d"
            value = struct.unpack(">" + width, struct.pack(">" + width, 12345678.125))[0]
            payload += b"\x83" + struct.pack(">I" + width, 77, value)
            position = dict(flags=0x83, parent=77, position=[value, 0.0, 0.0], rotation=[0.0] * 3)
        if flags & 2:
            payload += struct.pack(">q", 111)
        if flags & 4:
            payload += struct.pack(">q", 222)
        consumed = len(payload)
        payload += b"\xde\xad\xbe\xef"
        expected = dict(id=0xF1234567, timestamp_ms=-123, flags=flags, position=position,
            created_ms=111 if flags & 2 else None, modified_ms=222 if flags & 4 else None,
            in_motion=bool(flags & 0x20), consumed_bytes=consumed,
            tail_kind="effects-and-following" if flags & 0x10 else
                "sync-controllers-and-following" if flags & 0x40 else "type-specific",
            tail_hex="deadbeef")
        result = run(profile, payload, "inspect-native-base")
        assert result.returncode == 0, result.stderr
        assert json.loads(result.stdout) == expected, (profile, flags)
        prefix_cases += 1
    for malformed in (b"", b"\x00" * 12, struct.pack(">IqB", 1, 2, 1)):
        assert run(profile, malformed, "inspect-native-base").returncode != 0
    assert run(profile, bytes(4097), "inspect-native-base").returncode != 0
print(f"Native ID/base prefix: {prefix_cases} independent fixtures preserve unknown tail kinds and bytes.")
print(f"Native packed fields: {cases} independent profile/flag fixtures; "
      "local matrices match source body axes; both-pole gravity alignment, "
      "static reader anchors and malformed records verified offline.")
