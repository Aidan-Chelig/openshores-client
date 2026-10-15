"""Static head/neck geometry-plan anchors, no target execution."""
import json
import runpy
import subprocess
from pathlib import Path

state = runpy.run_path("tests/native_dp_arm_skin.py")
for name in ["head-create-skin","helmet-model-prefix"]:
    record = json.loads(Path(f"docs/evidence/native-dp-{name}.json").read_text())
    assert record["subject"]["digest"]["sha256"] == "285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80"
for address,expected in [(0x1422BEBAA,b"%1%2"),(0x1422BEBB5,b"%1Head%2%3%4%5"),(0x1422BEBC4,b"HS%1%2"),(0x1422BEBCB,b"HC%1%2")]:
    assert state["read"](address,len(expected)+1) == expected+b"\0"
assembly = subprocess.check_output(["objdump","-d","--no-show-raw-insn","--disassemble=_ZN13DpHeadAndNeck10CreateSkinER11AuGLEffectsR14AuGLModelCache",str(state["state"]["target"])],text=True)
for anchor in ["cmpl   $0x1,0x48(%rbx)","movabs $0x5000000000001,%rax","call   1413994c0","movsd  %xmm4,0x68(%rsp)","movsd  %xmm6,0x70(%rsp)","movl   $0x8,0x270(%rsp)","mov    %r15d,0x274(%rsp)","MergeHeadWithInstance","GetHelmetModelPrefix"]:
    assert anchor in assembly,anchor
pseudocode = json.loads(Path("docs/evidence/native-dp-head-create-skin.json").read_text())["normalized_result"]
for anchor in ["%1Skull.3dsN","%1Eye%2%3.3dsN","%1Ear%2%3.3dsN","%1Nose%2%3.3dsN","%1Mouth%2%3.3dsN","%1Horn%2%3.3dsN","%1HeadCrest%2%3.3dsN","%1Neck%2%3.3dsN","(double)(fVar15 / 1.5)","dVar18 * 0.001 * (double)(fVar15 / 0.84)","(double)(fVar15 / 0.84) * dVar18 * 0.001"]:
    assert anchor in pseudocode,anchor
print("Verified head/neck default resource formats, full helmet selector, geometry/bone/neck anchors and distinct merge operand orders. Plan only: no localized resource translation, merge/cache/eye publication or rendered parity claim.")
