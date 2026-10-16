"""SHA-bound jump leg arithmetic/dispatch, no original runtime execution."""
import struct
import subprocess
from native_dp_connections import read, target

for address, bits in {
    0x1422BFC80: 0x3E32B8C2, 0x1422BFC94: 0x3EB2B8C2,
    0x1422BFD30: 0xBD0EFA35, 0x1422BFCB8: 0x4016CBE4,
    0x1422BF8A0: 0x3F000000, 0x1422BFC7C: 0x3E860A92,
    0x1422BFA40: 0x3DB2B8C2,
}.items():
    assert struct.unpack("<I", read(address, 4))[0] == bits

def assembly(symbol):
    return subprocess.check_output(["objdump", "-d", "--disassemble=" + symbol, str(target)], text=True)

body = assembly("_ZN6DpBody8JumpLegsEfif")
part = assembly("_ZN12DpLegAndFoot4JumpEff")
assert "SpreadLegs" in body and "sub    $0x8,%rbx" in body
assert "cmp    $0x2,%edi" in body and "0x4c(%rcx)" in body
assert "movl   $0x1c,0x4c(%rsp)" in part and "DhDNA6Rhythm" in part
assert "HasAnkleBend" in part and "CalcStandAngles" in part
order = [part.index(name) for name in ("RotateHip", "RotateKnee", "RotateAnkle", "SplayHip")]
assert order == sorted(order)
assert part.count("mulss") == 6 and "subss  %xmm0,%xmm1" in part
assert "cvtss2sd" not in part
print("Verified Jump leg literals, pair dispatch, rhythm ID, f32 interpolation and joint order. Static evidence only.")
arms = assembly("_ZN6DpBody8JumpArmsEfibf")
arm = assembly("_ZN12DpArmAndHand4JumpEfbf")
assert "SpreadArms" in arms and "call   *0xa8(%rax)" in arms
assert "xor    %ebp,%ebp" in arms and "add    $0x8,%rbx" in arms
assert "movl   $0x1c,0x4c(%rsp)" in arm and "DhDNA6Rhythm" in arm
assert "HasWristBend" in arm and "CalcStandAngles" in arm
assert "0x30(%rax),%xmm7" in arm and "subss  %xmm0,%xmm7" in arm
order = [arm.index(name) for name in ("RotateShoulder", "RotateWrist", "RotateElbow", "SplayShoulder", "LiftShoulder")]
assert order == sorted(order)
assert "cvtss2sd" not in arm
print("Verified Jump arm spread, first-only override, rhythm, f32 interpolation and joint order. Static evidence only.")
