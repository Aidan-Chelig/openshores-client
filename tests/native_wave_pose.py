"""Bounded Wave static anchors, never original runtime execution."""
import struct
import subprocess
from native_dp_connections import read, target

for address, bits in {
    0x1422BFC9C: 0xBE860A92, 0x1422BFA3C: 0xBFBDE44E,
    0x1422BF890: 0x3F800000, 0x1422BFB10: 0x41200000,
    0x1422BFCA4: 0x41700000, 0x1422BFDA8: 0x42AA0000,
    0x1422BFC90: 0x3C8EFA35, 0x1422BFC80: 0x3E32B8C2,
    0x1422BFD50: 0x3DD67750, 0x1422BFD30: 0xBD0EFA35,
    0x1422BFCF4: 0x40860A92, 0x1422BFB2C: 0x40490FDB,
    0x1422BFCF0: 0x40060A92,
}.items():
    assert struct.unpack("<I", read(address, 4))[0] == bits

def assembly(symbol):
    return subprocess.check_output(["objdump", "-d", "--disassemble=" + symbol, str(target)], text=True)

body = assembly("_ZN6DpBody8WaveArmsEi")
part = assembly("_ZN12DpArmAndHand4WaveEf")
assert "SpreadArms" in body and "call   *0xa8(%rax)" in body
assert "0x30(%rsi),%xmm1" in body and "add    $0x8,%rbx" in body
assert "movl   $0x1,0x3c(%rsp)" in part and "DhDNA6Rhythm" in part
assert "0x19(%rcx),%eax" in part and "0x50(%rcx),%eax" in part
order = [part.index(name) for name in ("RotateShoulder", "RotateWrist", "RotateElbow", "SplayShoulder", "LiftShoulder")]
assert order == sorted(order)
assert "HasWristBend" in part and "mulss" in part and "addss" in part
# DhDNA method byte offset 0x13 corresponds to packed byte 11: object
# header is eight bytes, also required by part+0x24 -> packed byte12.
wrist = assembly("_ZNK5DhDNA12HasWristBendEv")
assert "0x13(%rcx),%eax" in wrist and "cmp    $0x80,%al" in wrist
print("Verified native Wave constants, pose-1 rhythm, phase inputs, owner pitch and joint order. Static evidence only.")
