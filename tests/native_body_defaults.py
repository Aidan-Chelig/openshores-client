"""Static native geometry constructor subset, no target execution."""
import json
import runpy
import struct
import subprocess
from pathlib import Path

state = runpy.run_path("tests/native_dp_connections.py")
assert struct.unpack("<2d",state["read"](0x1422CF250,16)) == (1.,1.)
record = json.loads(Path("docs/evidence/native-body-object-constructor.json").read_text())
assert record["subject"]["digest"]["sha256"] == "285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80"
assembly = subprocess.check_output(["objdump","-d","--no-show-raw-insn","--disassemble=_ZN15Au3dsGLSLObjectC1Ev",str(state["target"])],text=True)
for anchor in ["pxor   %xmm4,%xmm4","pxor   %xmm5,%xmm5","1422cf250",
               "movsd  %xmm6,0x140(%rbx)","movsd  %xmm6,0x190(%rbx)",
               "movups %xmm7,0x170(%rbx)","movups %xmm7,0x180(%rbx)"]:
    assert anchor in assembly,anchor
# Each XYZ XY pair lies in a16-byte zero write; Z is a separate scalar write.
for offset in [0x1b8,0x1ec,0x220,0x254,0x288,0x2bc,0x2f0]:
    assert f"movups %xmm4,0x{offset:x}(%rbx)" in assembly
for offset in [0x1c8,0x1fc,0x230,0x264,0x298,0x2cc,0x300]:
    assert f"movl   $0x0,0x{offset:x}(%rbx)" in assembly
copy = subprocess.check_output(["objdump","-d","--no-show-raw-insn","--disassemble=_ZN15Au3dsGLSLVertexaSERKS_.isra.0",str(state["target"])],text=True)
for anchor in ["movups 0x20(%rdx),%xmm3","movups %xmm3,0x20(%rcx)","movss  0x30(%rdx),%xmm0","movss  %xmm0,0x30(%rcx)"]:
    assert anchor in copy,anchor
assert "call " not in copy
print("Verified all six deformation scale defaults, seven XYZ zero fields, two-double unity constant and literal vertex assignment. Constructor geometry subset only; no complete native object or rendering claim.")
