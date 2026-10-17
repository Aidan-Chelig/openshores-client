"""Bounded SHA-bound PointArms static anchors. No target execution."""
import struct
import subprocess
from native_dp_connections import read, target

for address, bits in {0x1422BFA80: 0xBC8EFA35, 0x1422BFA40: 0x3DB2B8C2}.items():
    assert struct.unpack("<I", read(address, 4))[0] == bits
code = subprocess.check_output(["objdump", "-d", "--disassemble=_ZN6DpBody9PointArmsEi", str(target)], text=True)
assert "SpreadArms" in code and "call   *0xa8(%rax)" in code
assert "add    $0x8,%rbx" in code and "0x30(%rsi),%xmm1" in code
assert "subss" in code and "RotateWrist" not in code and "Rhythm" not in code
order = [code.index(name) for name in ("RotateShoulder", "RotateElbow", "SplayShoulder", "LiftShoulder")]
assert order == sorted(order)
assert "xor    %ebp,%ebp" not in code
print("Verified Point pitch, literals, ordinary-arm/side traversal and shoulder/elbow/splay/lift order. Static evidence only.")
