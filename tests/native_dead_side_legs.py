"""SHA-bound side-facing death leg stages; no target execution."""
import struct
import subprocess
from native_dp_connections import read, target

for address, bits in {
    0x1422BFD30: 0xBD0EFA35, 0x1422BF8A0: 0x3F000000,
    0x1422BFA54: 0x3FC49809, 0x1422BFDFC: 0x3F9C61AA,
    0x1422BFE00: 0xC03DE44E, 0x1422BFA8C: 0xBFA78D36,
    0x1422BFA1C: 0xBF060A92, 0x1422BFA50: 0xBFC90FDB,
    0x1422BFDE8: 0xC021F770, 0x1422BFDF8: 0xBF5F66F3,
    0x1422BFDF0: 0xC0437A15, 0x1422BFA30: 0xBE32B8C2,
    0x1422BFA20: 0x3F490FDB,
}.items():
    assert struct.unpack("<I", read(address, 4))[0] == bits

def assembly(symbol):
    return subprocess.check_output(
        ["objdump", "-d", "--disassemble=" + symbol, str(target)], text=True)

for facing, length in [("Left", 18), ("Right", 19)]:
    body = assembly(f"_ZN6DpBody{length}DeadLegsFacing{facing}Ei")
    assert "1422bfd30" in body and "SpreadLegs" in body
    assert "sub    $0x8,%rbx" in body and "0x4c(%rcx)" in body
    assert "cmp    $0x2,%edi" in body and "cmp    $0x1,%edi" in body
    part = assembly(f"_ZN12DpLegAndFoot{length - 4}DeadFacing{facing}Ev")
    for token in ["and    $0xc,%eax", "cmp    $0x4,%al", "comiss 0x48(%rcx),%xmm0",
                  "testb  $0xc,0x24(%rcx)", "CGHeightStanding", "CalcStandAngles",
                  "HasAnkleBend", "$0x3f32b8c3", "$0x3fc90fdb", "maxss"]:
        assert token in part
    for address in ["1422bfa54", "1422bfdfc", "1422bfe00", "1422bfa8c",
                    "1422bfa1c", "1422bfa50", "1422bfde8", "1422bfdf8",
                    "1422bfdf0", "1422bfa30"]:
        assert address in part
    assert part.index("RotateHip") < part.index("RotateKnee") < part.index("HasAnkleBend") < part.index("RotateAnkle") < part.index("SplayHip")
    branches = part.split("test   %al,%al")[1:]
    # The generic and special branches reverse IsLeft consistently.
    assert ("jne " if facing == "Left" else "je ") in branches[0].splitlines()[1]
    assert ("je " if facing == "Left" else "jne ") in branches[-1].splitlines()[1]
    assert part.count("comiss %xmm6,%xmm0") == 2
for pose, destination in [(0x12, 0x1409808E7), (0x13, 0x140980978),
                          (0x14, 0x140980988), (0x15, 0x140980950)]:
    table = 0x1421C664C
    assert table + struct.unpack("<i", read(table + (pose - 4) * 4, 4))[0] == destination
    table = 0x1421C5E34
    assert table + struct.unpack("<i", read(table + (pose - 3) * 4, 4))[0] == 0x140978B10
for address, bits in [(0x1421CA1A8, 0x400921FB54442D18),
                      (0x1421C9C48, 0x3FF921FB54442D18),
                      (0x1421CA180, 0xBFF921FB54442D18)]:
    assert struct.unpack("<Q", read(address, 8))[0] == bits
adjust = subprocess.check_output([
    "objdump", "-d", "-C", "--start-address=0x140980830",
    "--stop-address=0x1409809e0", str(target)], text=True)
assert "mov    0x48(%rax),%ebx" in adjust
assert "AuMatrix4x4::RotateY(double)" in adjust
for address in ["1421ca1a8", "1421c9c48", "1421ca180"]:
    assert address in adjust
print("Verified side-death leg constants, selectors, signed splay gate, standing dependencies, side reversal and joint order; separate render-rotation dependency confirmed. Static evidence only.")
