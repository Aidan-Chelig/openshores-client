"""Static native arm gait anchors; SHA-bound, never execute original code."""
import struct
import subprocess
from native_dp_connections import read, target

def assembly(symbol):
    return subprocess.check_output(["objdump", "-d", "--disassemble=" + symbol, str(target)], text=True)

for address, bits in {
    0x1422BFCBC: 0x3D0EFA35, 0x1422BFC80: 0x3E32B8C2,
    0x1422BFD30: 0xBD0EFA35, 0x1422BFC90: 0x3C8EFA35,
    0x1422BFA40: 0x3DB2B8C2,
}.items():
    assert struct.unpack("<I", read(address, 4))[0] == bits
for address, value in {0x1422BFDD0: 5., 0x1422BFDC8: 10., 0x1422BFDC0: 20., 0x1422BF8F0: 3., 0x1422BF9D8: 0.5}.items():
    assert struct.unpack("<d", read(address, 8))[0] == value
for size, verb, pose in [(4, "Walk", "26"), (3, "Run", "25")]:
    code = assembly(f"_ZN12DpArmAndHand{size}{verb}Efbf")
    assert f"$0x{pose}," in code and "DhDNA6Rhythm" in code
    assert "call   *0xa8(%rax)" in code and "CalcStandAngles" in code
    order = [code.index(name) for name in ["RotateShoulder", "RotateWrist", "RotateElbow", "SplayShoulder", "LiftShoulder"]]
    assert order == sorted(order)
    assert "0x50(%r12)" in code and "0x30(%rax)" in code
    assert "cvtss2sd" in code and "cvtsd2ss" in code
    owner = assembly(f"_ZN6DpBody{size+4}{verb}ArmsEfibf")
    assert "WalksOnHands" in owner and "SpreadArms" in owner
    assert "xor    %ebp,%ebp" in owner  # consume first-arm override once
    assert "cmp    $0x2,%edi" in owner and "call   *0xa8(%rax)" in owner
    assert f"DpArmAndHand{size}{verb}" in owner
assert "WalksOnHands" in assembly("_ZN12DpArmAndHand3RunEfbf")
assert "divsd" in assembly("_ZN12DpArmAndHand3RunEfbf")
print("Verified arm gait constants, rhythm IDs, first-arm override and native shoulder/wrist/elbow/splay/lift order. Static evidence, not rendered parity.")
