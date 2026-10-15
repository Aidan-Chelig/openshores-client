"""Static leg CreateSkin geometry anchors, no target execution."""
import json
import runpy
import subprocess
from pathlib import Path

state = runpy.run_path("tests/native_dp_arm_skin.py")
record = json.loads(Path("docs/evidence/native-dp-leg-create-skin.json").read_text())
assert record["subject"]["digest"]["sha256"] == "285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80"
assembly = subprocess.check_output(["objdump","-d","--no-show-raw-insn","--disassemble=_ZN12DpLegAndFoot10CreateSkinER11AuGLEffectsR14AuGLModelCache",str(state["state"]["target"])],text=True)
for anchor in ["call   *0x10(%rax)","call   *0xa0(%rax)","movss  0x409c(%rcx),%xmm10","movss  0x4080(%rcx),%xmm11","call   1412c6b80","movsd  0x4120(%rdx),%xmm3","movsd  0xfd9ddc(%rip),%xmm1","1422bfa00","divss","1422bfbc0","call   1413994c0","addsd  0x178(%rsp),%xmm10","addsd  %xmm11,%xmm10","movsd  %xmm6,0x1f8(%rsp)"]:
    assert anchor in assembly,anchor
assert state["read"](0x1422BFA00,8) == bytes.fromhex("000000000000f03f")
constructor = assembly[assembly.index("1412e5c09:"):assembly.index("1412e5d12:")]
assert "movsd  0xfd9ddc(%rip),%xmm1" in constructor
assert "cvtsd2si" not in constructor
for anchor in ["0xfbefbfffffffffffU","%1LegRight%2.3dsN","%1LegLeft%2.3dsN","%1FootRight%2.3dsN","%1FootLeft%2.3dsN","DhDNA::HasAnkleBend","dStack_200 = (double)(fVar22 + fVar22);"]:
    assert anchor in record["normalized_result"],anchor
print("Verified leg/foot resource and side selection, constructor float ABI, native owner dimensions, FootLength call and Foot field68/translation updates. Geometry projection only, not complete CreateSkin/rendering parity.")
