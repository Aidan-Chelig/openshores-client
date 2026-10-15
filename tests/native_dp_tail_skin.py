"""Static tail CreateSkin geometry anchors, no target execution."""
import json
import runpy
import struct
import subprocess
from pathlib import Path

state = runpy.run_path("tests/native_dp_arm_skin.py")
record = json.loads(Path("docs/evidence/native-dp-tail-create-skin.json").read_text())
assert record["subject"]["digest"]["sha256"] == "285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80"
assembly = subprocess.check_output(["objdump","-d","--no-show-raw-insn","--disassemble=_ZN6DpTail10CreateSkinER11AuGLEffectsR14AuGLModelCache",str(state["state"]["target"])],text=True)
for address,value in [(0x1422BF8E8,3.),(0x1422BF8F8,1.5),(0x1422BFE5C,0.875)]:
    assert state["read"](address,4) == struct.pack("<f",value)
for anchor in ["test   $0x30,%al","je     1412e4652","shr    $0x6,%sil","movzbl 0x28(%rbx),%r8d","movsd  0xfdb692(%rip),%xmm1","1422bfa00","divss","1422bfe5c","1422bf8f8","cvtss2sd %xmm4,%xmm4","cvtss2sd %xmm5,%xmm5","movl   $0x11,0x48(%rsp)","movl   $0x11,0x40(%rsp)","movl   $0x11,0x38(%rsp)","call   1413994c0"]:
    assert anchor in assembly,anchor
assert "GetBodyModelPrefix" not in assembly
for anchor in ["\"Tip\"","\"Tail%1\"",":data/%1%2%3.3dsN","(bVar8 - 1) + (uint)bVar8","(double)(fVar13 / 1.5)","(double)((fVar9 + fVar11) / 0.875)"]:
    assert anchor in record["normalized_result"],anchor
print("Verified tail disable gate, part-DNA resource/count rules, fixed bone17 outputs, native float division stages and constructor ABI. Geometry requests only, not complete skin generation/rendering parity.")
