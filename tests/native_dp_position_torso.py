"""Bind every torso dispatch fixture role to supplied executable bytes."""
import runpy
import struct
from pathlib import Path

c = runpy.run_path("tests/native_dp_torso_base.py")
data, pe, sections = c["data"], c["pe"], c["sections"]

def read(address, length):
    for i in range(c["count"]):
        rva, size, offset = struct.unpack_from("<III", data, sections + i * 40 + 12)
        relative = address - c["image_base"] - rva
        if 0 <= relative <= size - length:
            return data[offset + relative:offset + relative + length]
    raise AssertionError(hex(address))

roles = dict(line.split("\t") for line in Path("tests/fixtures/native_dp_position_torso.tsv").read_text().splitlines() if not line.startswith("#"))
walking = {
    "fixed": 0x31e4, "none": 0x3245, "fallback": 0x3440, "small": 0x3480,
    "negative": 0x3420, "23": 0x3782, "24": 0x378c, "25": 0x3767,
    "27": 0x3540, "28": 0x376e, "29": 0x3490, "37": 0x3778,
    "quarter": 0x3294, "41": 0x3753, "42": 0x35a8, "43": 0x3560,
    "45": 0x375d, "46": 0x3580, "tiny": 0x332d, "50": 0x3550, "halfpi": 0x34b8,
}
nonwalking = dict(walking, fallback=0x3430, small=0x3450, negative=0x33f8,
                 **{"23": 0x3647, "24": 0x36e1, "25": 0x3718, "27": 0x34c8,
                    "28": 0x36ad, "37": 0x367e, "41": 0x35c7, "45": 0x3610, "50": 0x3500})
for address, bindings in [(0x1422be680, walking), (0x1422be7c0, nonwalking)]:
    entries = struct.unpack("<80i", read(address, 320))
    for pose, relative in enumerate(entries, 3):
        assert address + relative == 0x1412e0000 + bindings[roles[str(pose)]], pose
for pose in range(83, 160):
    if pose == 83:
        expected = "none"
    elif pose == 108:
        expected = "tiny"
    elif pose < 108:
        expected = "fixed" if (1 << (pose - 84)) & 0x333333 else "fallback"
    else:
        bit = 1 << (pose - 109)
        expected = "fixed" if bit & 0x7ffffe2733333 else "fallback" if bit & 0x58ccccc else "quarter" if pose == 137 else "none"
    assert roles[str(pose)] == expected, pose
torso = c["torso"]
for anchor in ["$0x7ffffe2733333", "$0x58ccccc", "$0x333333", "$0xcccccc", "$0xffdffffff4ffffff", "0x48(%r12)", "0x5c(%r12)", "<_ZNK5DhDNA6RhythmERK6AuPoseff>", "<_ZN6DpBody11RotateWaistEf.part.0>", "movaps %xmm6,%xmm0", "mulss  %xmm9,%xmm0", "addss  %xmm0,%xmm6"]:
    assert anchor in torso, anchor
assert [i for i in range(54) if not (0xffdffffff4ffffff >> i) & 1] == [24, 25, 27, 53]
constants = {
    0x1422bfe48: 0x3fc25c20, 0x1422bfe54: 0x3fbde44f,
    0x1422bfa30: 0xbe32b8c2, 0x1422bfa20: 0x3f490fdb,
    0x1422bfa90: 0x3cd67750, 0x1422bfe44: 0x3c0efa35,
    0x1422bfd08: 0x3d32b8c2, 0x1422bfc7c: 0x3e860a92,
    0x1422bfd0c: 0x3edf66f3, 0x1422bfcbc: 0x3d0efa35,
    0x1422bfe50: 0x3e0efa35, 0x1422bfc90: 0x3c8efa35,
    0x1422bfa40: 0x3db2b8c2, 0x1422bfe40: 0x3d7a35dd,
    0x1422bfa7c: 0x3d567750, 0x1422bfe4c: 0x3fcb4bc4,
    0x1422bfdb0: 0x3fa78d36, 0x1422bfe3c: 0x3fb2b8c3,
    0x1422bfc80: 0x3e32b8c2, 0x1422bfa54: 0x3fc49809,
    0x1422bfa48: 0x3fc90fdb,
}
for address, bits in constants.items():
    assert struct.unpack("<I", read(address, 4))[0] == bits, hex(address)
print("Verified both complete 80-entry torso jump tables, high-pose masks, general-pose mask, float literals, rhythm/order/return and real waist call. Static checks only.")
