"""SHA-bound tail bending constants and loop anchors. Static, no execution."""
import runpy
import struct
import subprocess

c = runpy.run_path("tests/native_dp_connections.py")
read = c["read"]
assert struct.unpack("<f", read(0x1422BF8E8, 4))[0] == 3.
assert struct.unpack("<d", read(0x1422BFC50, 8))[0] == 0.9996573249564867
assert struct.unpack("<d", read(0x1422BFC38, 8))[0] == 0.026176949036146674
assembly = subprocess.check_output([
    "objdump", "-d", "--no-show-raw-insn", "--disassemble=_ZN6DpTail4BendEfff", str(c["target"])
], text=True)
for anchor in ["movss  0x40a8(%r12),%xmm0", "addss  %xmm0,%xmm0",
               "shr    $0x6,%al", "lea    0x10(%rax),%r13d",
               "cmp    $0x10,%r9d", "<sincos>",
               "comiss", "jb", "cmp    $0x11,%r13d"]:
    assert anchor in assembly, anchor
print("Verified tail spacing, segment selection, descending joint bounds, ground gate and correction coefficients. Static anchors only, not runtime parity.")
