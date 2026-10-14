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
import tempfile

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path("reference/OpenShores-server/src").resolve()))
from openshores.gameplay.natives.village import body_axes, gravity_align_euler
from openshores.protocol.framing import encode_size

TARGET = Path("reference/openshores-full/OpenShores.exe")
CLIENT = Path("target/debug/openshores-client")
assert hashlib.sha256(TARGET.read_bytes()).hexdigest() == (
    "285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80")


def assembly(start, end):
    return subprocess.check_output([
        "objdump", "-d", f"--start-address={start:#x}",
        f"--stop-address={end:#x}", str(TARGET)], text=True)


scene_reader = assembly(0x1405E79A0, 0x1405E7B60)
assert "1405e79c7:" in scene_reader and "142892690 <__imp__ZN11QDataStreamC1ERK10QByteArray>" in scene_reader
assert "1405e79df:" in scene_reader and "movl   $0x7,0xe8(%rsp)" in scene_reader
assert "call   140b27e30 <_ZN6DaAtom10RxUpdateIdER11QDataStream>" in scene_reader
assert "call   140b4e470 <_ZN6DaAtom12RxUpdateBaseER11QDataStream>" in scene_reader
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
ether_position = assembly(0x140B41450, 0x140B415D2)
for anchor in ("140b41472:", "0x210(%r12),%esi", "140b4147a:",
               "140b41487:", "140b414e9:", "140b4150d:",
               "142892820 <__imp__ZN13QElapsedTimer7restartEv>",
               "140b4151c:", "test   %esi,%esi", "140b415cd:",
               "140b40610 <_ZNK6DaAtom8TickTockEv>"):
    assert anchor in ether_position, anchor
assert ether_position.count("QElapsedTimer7restartEv") == 1
orientation_position = assembly(0x140B415E0, 0x140B41792)
assert "140b4169e:" in orientation_position and "1418fa0d0 <_ZNK11AuMatrix4x411GetRotationEv>" in orientation_position
invalidator = assembly(0x140B436E0, 0x140B43769)
for anchor in ("140b436eb:", "cmpb   $0x0,0x1d8(%rcx)", "140b4371b:",
               "1408c6400 <_ZNK7AuEther18ListNormalSubatomsERK4AuIdR10DaAtomList>",
               "140b77920 <_ZNK10DaAtomList22InvalidateEnvTransformEv>"):
    assert anchor in invalidator, anchor
rotation_getter = assembly(0x1418FA0D0, 0x1418FA2B9)
for anchor in ("1418fa102:", "1418fa107:", "1418fa10c:", "1418fa111:",
               "1418fa124:", "1418fa16d:", "1418fa175:", "1418fa1d3:",
               "1418fa246:", "1418fa24e:", "1418fa26e:",
               "14164c1b0 <_ZN7AuPoint17CalculateRotationERKS_S1_>"):
    assert anchor in rotation_getter, anchor
assert rotation_getter.count("141e6ba40 <") == 3
constant = subprocess.check_output(["objdump", "-s", "--start-address=0x141e6ba40",
    "--stop-address=0x141e6ba48", str(TARGET)], text=True)
assert "00000000 0000f03f" in constant  # exact little-endian double 1.0

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
        vectors_result = run(profile, payload, "dump-atom-rotation-vectors")
        assert vectors_result.returncode == 0, vectors_result.stderr
        vectors = json.loads(vectors_result.stdout)
        # Independent stage oracle on an already independently verified affine
        # matrix. Keep native add/subtract translation rounding.
        projected = [[(0.0 + matrix[axis][row] + matrix[3][row]) - matrix[3][row]
                      for row in range(3)] for axis in range(2)]
        assert vectors == projected, (profile, flags, vectors, projected)
        cases += 1

large = bytes([0xFE]) + struct.pack(">3f3d", 0, 0, 0, 1e17, 1e17, 1e17)
large_result = run("type-widths", large, "dump-atom-rotation-vectors")
assert large_result.returncode == 0
assert json.loads(large_result.stdout) == [[0.0] * 3, [0.0] * 3]

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
native_kinds = {
    0x02: "Animal", 0x03: "Arena", 0x04: "Asteroid", 0x05: "Person",
    0x06: "Building", 0x12: "Designer", 0x13: "DesignPlane", 0x14: "Effect",
    0x15: "Galaxy", 0x16: "Item", 0x17: "Missile", 0x18: "Npc",
    0x19: "Spacecraft", 0x1A: "Sector", 0x1C: "SolarSystem", 0x1D: "Star",
    0x1E: "Teleporter", 0x20: "Universe", 0x21: "Vehicle", 0x23: "GasGiant",
    0x24: "Globe", 0x27: "Ring", 0x28: "RingSection",
}
def atom(payload):
    return subprocess.run([str(CLIENT), "inspect-native-atom", payload.hex()],
                          capture_output=True, text=True)

for opcode in range(256):
    payload = bytes([opcode]) + struct.pack(">IqBBId", 0xF1234567, -123, 1, 0x83, 77, 12345678.125) + b"\xde\xad"
    result = atom(payload)
    assert result.returncode == 0, result.stderr
    record = json.loads(result.stdout)
    kind = native_kinds.get(opcode)
    if kind is None:
        assert record == dict(kind=None, payload_hex=payload.hex())
    else:
        assert record["kind"] == kind
        assert record["can_render_environment"] == (opcode in (0x03, 0x12, 0x1A, 0x1C))
        assert record["is_instance_universe"] == (opcode in (0x03, 0x12))
        assert record["base"] == dict(id=0xF1234567, timestamp_ms=-123, flags=1,
            position=dict(flags=0x83, parent=77, position=[12345678.125, 0.0, 0.0], rotation=[0.0] * 3),
            created_ms=None, modified_ms=None, in_motion=False, consumed_bytes=26,
            tail_kind="type-specific", tail_hex="dead")
        assert atom(bytes([opcode])).returncode != 0
assert atom(bytes(4097)).returncode != 0
assert atom(b"").returncode != 0

with tempfile.TemporaryDirectory(prefix="openshores-native-capture-") as temporary:
    packets = [b"\xff\x01\x02\x03", bytes([0x16]) +
        struct.pack(">IqBBId", 7, 123, 1, 0x83, 77, 12345678.125) + b"\xde\xad"]
    capture_bytes = b"".join(encode_size(len(packet)) + packet for packet in packets)
    path = Path(temporary) / "native.bin"
    path.write_bytes(capture_bytes)
    result = subprocess.run([str(CLIENT), "inspect-native-capture", str(path)], capture_output=True, text=True)
    assert result.returncode == 0, result.stderr
    rows = [json.loads(line) for line in result.stdout.splitlines()]
    assert rows[0] == dict(frames=2, payload_bytes=sum(map(len, packets)))
    for index, packet in enumerate(packets):
        assert rows[index + 1]["frame"] == index
        assert rows[index + 1]["opcode"] == packet[0]
        assert rows[index + 1]["payload_sha1"] == hashlib.sha1(packet).hexdigest()
    assert rows[1]["kind"] is None and rows[1]["tail_bytes"] == len(packets[0])
    assert rows[2]["kind"] == "Item" and rows[2]["id"] == 7
    assert rows[2]["position"]["position"][0] == 12345678.125
    assert rows[2]["tail_bytes"] == 2
    assert path.read_bytes() == capture_bytes
    path.write_bytes(capture_bytes[:-1])
    result = subprocess.run([str(CLIENT), "inspect-native-capture", str(path)], capture_output=True, text=True)
    assert result.returncode != 0 and not result.stdout
print("Native capture CLI: independent framing, prefix replay, raw hashes, unchanged source and truncated-frame rejection verified.")
print("Native atom dispatch: all 256 opcodes, 23 kinds, legacy double preservation and unknown payload retention verified offline.")
print(f"Native ID/base prefix: {prefix_cases} independent fixtures preserve unknown tail kinds and bytes.")
print(f"Native packed fields: {cases} independent profile/flag fixtures; "
      "local matrices match source body axes; both-pole gravity alignment, "
      "static reader anchors and malformed records verified offline.")
print(f"Native rotation inputs: {cases} projected-input stage fixtures and large-coordinate cancellation verified; final angle extraction remains separate.")
