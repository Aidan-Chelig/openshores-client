"""SHA-bound fixed Fold limb literals and dispatch. No original execution."""
import struct
import subprocess
from native_dp_connections import read, target

for address, bits in {
    0x1422BFC94: 0x3EB2B8C2, 0x1422BFD90: 0xBEB2B8C2,
    0x1422BFDE8: 0xC021F770, 0x1422BFA48: 0x3FC90FDB,
    0x1422BFC80: 0x3E32B8C2, 0x1422BFDBC: 0xC0384E89,
    0x1422BFDA0: 0x40384E88, 0x1422BFDA4: 0x40384E89,
    0x1422BFD30: 0xBD0EFA35,
}.items():
    assert struct.unpack("<I", read(address, 4))[0] == bits

def assembly(symbol):
    return subprocess.check_output(["objdump", "-d", "--disassemble=" + symbol, str(target)], text=True)

legs = assembly("_ZN6DpBody8FoldLegsEi")
arms = assembly("_ZN6DpBody8FoldArmsEib")
assert "SpreadLegs" in legs and "sub    $0x8,%rbx" in legs
assert "cmp    $0x2,%edi" in legs and "0x4c(%r12)" in legs
assert "HasAnkleBend" in legs and "RotateAnkle" in legs
assert legs.index("RotateHip") < legs.index("RotateKnee") < legs.index("SplayHip")
assert "SpreadArms" in arms and "call   *0xa8(%rax)" in arms
assert "xor    %ebp,%ebp" in arms and "add    $0x8,%rbx" in arms
assert "addss  0x30(%rax),%xmm6" in arms
assert arms.index("RotateShoulder") < arms.index("RotateElbow") < arms.index("SplayShoulder") < arms.index("LiftShoulder")
assert "Rhythm" not in legs + arms
print("Verified Fold fixed limb literals, conditional ankle, pair/side traversal and first-arm override. Static evidence only.")
