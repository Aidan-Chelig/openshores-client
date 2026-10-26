"""SHA-bound ProneCreep stages and phase tables; no original execution."""
import struct
import subprocess
from native_dp_connections import read, target

def assembly(symbol):
    return subprocess.check_output(["objdump", "-d", "--disassemble=" + symbol, str(target)], text=True)

for address, bits in {
    0x1422BFCF0: 0x40060A92, 0x1422BFCF4: 0x40860A92,
    0x1422BFCF8: 0x40A78D36, 0x1422BFCFC: 0x40EA9280,
    0x1422BFD00: 0x40C90FDB, 0x1422BFB2C: 0x40490FDB,
    0x1422BFAD0: 0x40A00000, 0x1422BFB10: 0x41200000,
    0x1422BFC90: 0x3C8EFA35, 0x1422BFDA4: 0x40384E89,
    0x1422BFDA0: 0x40384E88, 0x1422BFD30: 0xBD0EFA35,
    0x1422BFDEC: 0xBDB2B8C2, 0x1422BFC80: 0x3E32B8C2,
    0x1422BFA30: 0xBE32B8C2, 0x1422BFDF4: 0x3F32B8C3,
    0x1422BF890: 0x3F800000, 0x1422BFCDC: 0x3FB2B8C2,
}.items():
    assert struct.unpack("<I", read(address, 4))[0] == bits
arms = assembly("_ZN12DpArmAndHand10ProneCreepEfbf")
legs = assembly("_ZN12DpLegAndFoot10ProneCreepEf")
for text in [arms, legs]:
    assert "call   *0xa8(%rax)" in text and "$0x21,0x3c(%rsp)" in text
    assert "DhDNA6Rhythm" in text
    for literal in ["1422bfcf0", "1422bfcf4", "1422bfcf8", "1422bfcfc", "1422bfd00", "1422bfb2c"]:
        assert literal in text
assert "cmpl   $0x1,0x50(%r12)" in arms and "and    $0x3,%eax" in arms
for literal in ["1422bfad0", "1422bfb10", "1422bfc90", "1422bfda4", "1422bfda0", "1422bfd30"]:
    assert literal in arms
assert arms.index("Rhythm") < arms.index("RotateShoulder") < arms.index("RotateWrist") < arms.index("RotateElbow") < arms.index("SplayShoulder") < arms.index("LiftShoulder")
assert "subss  %xmm1,%xmm7" in arms
for anchor in ["and    $0xc,%eax", "cmpl   $0x1,0x4c(%r12)", "WalksOnHands",
               "comiss %xmm3,%xmm2", "mulss  %xmm2,%xmm4", "subss  %xmm4,%xmm2",
               "subss  %xmm2,%xmm7", "subss  %xmm0,%xmm6", "mulss  %xmm3,%xmm6"]:
    assert anchor in legs
assert legs.index("RotateHip") < legs.index("RotateKnee") < legs.index("HasAnkleBend") < legs.index("SplayHip")
body_arms = assembly("_ZN6DpBody14ProneCreepArmsEfibf")
body_legs = assembly("_ZN6DpBody14ProneCreepLegsEif")
assert "SpreadArms" in body_arms and "1422bfcdc" in body_arms and "1422bfc80" in body_arms
assert "add    $0x8,%rbx" in body_arms and "xor    %ebp,%ebp" in body_arms
assert "SpreadLegs" in body_legs and "1422bfdec" in body_legs and "sub    $0x8,%rbx" in body_legs
for table, bias, destination, consumer in [
    (0x1422BF404, 0, 0x1412F18D8, "ProneCreepLegs"),
    (0x1422BF534, 4, 0x1412F1AC0, "ProneCreepLegs"),
    (0x1422BF080, 0, 0x1412EE740, "ProneCreepArms"),
    (0x1422BF1B0, 0, 0x1412EE438, "ProneCreepArms"),
]:
    assert table + struct.unpack("<i", read(table + (0x21 - bias) * 4, 4))[0] == destination
    branch = subprocess.check_output(["objdump", "-d", "-C", f"--start-address={destination:#x}",
                                      f"--stop-address={destination + 0x48:#x}", str(target)], text=True)
    assert consumer in branch
table = 0x1421C664C
assert table + struct.unpack("<i", read(table + (0x21 - 4) * 4, 4))[0] == 0x1409808E7
table = 0x1421C5E34
assert table + struct.unpack("<i", read(table + (0x21 - 3) * 4, 4))[0] == 0x140978BE7
print("Verified crawling rhythm ID, phase literals/DNA gates, arm and leg arithmetic/order, traversal, first-arm override and no render rotation. Static evidence only.")
