"""Static torso segment/skin plan anchors; no target execution."""
import json
import runpy
import struct
import subprocess
from pathlib import Path

state = runpy.run_path("tests/native_dp_arm_skin.py")
read = state["read"]
assert struct.unpack("<54I",read(0x1422BF900,54*4)) == tuple(range(1,55))
for name in ["native-dp-torso-create-skin", "native-dp-torso-segment", "native-dp-torso-segment-part0"]:
    record = json.loads(Path(f"docs/evidence/{name}.json").read_text())
    assert record["subject"]["digest"]["sha256"] == "285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80"

def assembly(symbol):
    return subprocess.check_output(["objdump","-d","--no-show-raw-insn",f"--disassemble={symbol}",str(state["state"]["target"])],text=True)

selector = assembly("_ZNK6DpBody12TorsoSegmentEj.part.0")
for anchor in ["call   1412ad6f0", "call   1412ad1b0", "test   $0xc,%r8b", "testb  $0x3,0x19(%rsi)", "testb  $0x6,0x1a(%rsi)", "lea    (%rdx,%rdx,8),%rdx", "1422bf900", "1422bf9d0", "and    $0xfffffffd,%eax"]:
    assert anchor in selector,anchor
skin = assembly("_ZN7DpTorso10CreateSkinER11AuGLEffectsR14AuGLModelCache")
for anchor in ["call   1413994c0", "movsd  %xmm7,0x58(%rsp)", "movsd  %xmm6,0x60(%rsp)", "movapd %xmm8,%xmm1", "MergeTorsoWithInstance", "CreatePatchedModel", "0x303"]:
    assert anchor in skin,anchor
assert read(0x1422BFE68,8) == struct.pack("<d",0.05)
pseudocode = json.loads(Path("docs/evidence/native-dp-torso-create-skin.json").read_text())["normalized_result"]
for anchor in ["%1%2TC%3%4", "%1TorsoCrest%2%3.3dsN", "uStack_220 = uStack_220 - dVar4;", "local_1d8 = CONCAT44(local_364,iVar5);", "local_1d0 = iVar1;", "dVar22 * 0.05 * (double)(fVar17 / 0.84)"]:
    assert anchor in pseudocode,anchor
print("Verified literal54-entry segment table, selector inputs, torso definition ABI, combined/crest plan and fallback bone/translation/tolerance anchors. Planning only: merge/cache/publication and rendered parity remain unimplemented.")
