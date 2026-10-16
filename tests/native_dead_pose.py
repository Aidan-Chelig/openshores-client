"""SHA-bound face-down dead limb stages. No original target execution."""
import struct
import subprocess
from native_dp_connections import read, target

for address, bits in {
    0x1422BFD0C: 0x3EDF66F3, 0x1422BFCC4: 0xC04B4BC3,
    0x1422BFCBC: 0x3D0EFA35, 0x1422BFA80: 0xBC8EFA35,
    0x1422BFA30: 0xBE32B8C2, 0x1422BFDEC: 0xBDB2B8C2,
    0x1422BFA20: 0x3F490FDB, 0x1422BFDF0: 0xC0437A15,
    0x1422BFDF4: 0x3F32B8C3,
}.items():
    assert struct.unpack("<I", read(address, 4))[0] == bits

def assembly(symbol):
    return subprocess.check_output(["objdump", "-d", "--disassemble=" + symbol, str(target)], text=True)

arms = assembly("_ZN6DpBody18DeadArmsFacingDownEi")
legs = assembly("_ZN6DpBody18DeadLegsFacingDownEi")
assert "SpreadArms" in arms and "call   *0xa8(%rax)" in arms
assert "add    $0x8,%rbx" in arms
assert "RotateWrist" not in arms and "LiftShoulder" not in arms
assert [arms.index(name) for name in ["RotateShoulder", "RotateElbow", "SplayShoulder"]] == sorted(arms.index(name) for name in ["RotateShoulder", "RotateElbow", "SplayShoulder"])
assert "SpreadLegs" in legs and "sub    $0x8,%rbx" in legs
assert "0x4c(%r12)" in legs and "cmp    $0x2,%edi" in legs
assert "and    $0xc,%eax" in legs and "cmp    $0x4,%al" in legs
assert "comiss 0x48(%r12),%xmm0" in legs
assert "HasAnkleBend" in legs and "RotateAnkle" in legs
assert legs.index("RotateHip") < legs.index("RotateKnee") < legs.index("SplayHip")
print("Verified face-down dead limb constants, selectors, traversal, DNA/splay branch and joint order. Static evidence only.")
