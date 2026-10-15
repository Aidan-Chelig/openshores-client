"""Static arm CreateSkin request/precision anchors, no target execution."""
import json
import runpy
import struct
import subprocess
from pathlib import Path

state = runpy.run_path("tests/native_dp_connections.py")
read = state["read"]
record = json.loads(Path("docs/evidence/native-dp-arm-create-skin.json").read_text())
assert record["subject"]["digest"]["sha256"] == "285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80"
assembly = subprocess.check_output(["objdump","-d","--no-show-raw-insn","--disassemble=_ZN12DpArmAndHand10CreateSkinER11AuGLEffectsR14AuGLModelCache",str(state["target"])],text=True)
for address,fmt,value in [(0x1422BFBC0,"f",0.84),(0x1422BFBC4,"f",0.0625),(0x1422BF890,"f",1.),(0x1422BF8A0,"f",0.5),(0x1422BFB0C,"f",0.25),(0x1422BFBD8,"d",1.5)]:
    assert read(address,struct.calcsize(fmt)) == struct.pack("<"+fmt,value)
for anchor in ["call   *0x10(%rax)","call   *0xa0(%rax)","movss  0x40b4(%rdx),%xmm10","movss  0x4088(%rdx),%xmm11", "divss", "1422bfbc0", "call   1413994c0", "addsd  0x178(%rsp),%xmm10", "addsd  %xmm11,%xmm10", "movsd  %xmm9,0x1f8(%rsp)"]:
    assert anchor in assembly,anchor
pseudocode = record["normalized_result"]
for anchor in ["0xfffffefbefffffffU",":data/WingRight%2.3dsN",":data/WingLeft%2.3dsN","%1ArmRight%2.3dsN","%1ArmLeft%2.3dsN","%1HandRight%2.3dsN","%1HandLeft%2.3dsN","dStack_200 = (double)(fVar22 + fVar22);"]:
    assert anchor in pseudocode,anchor
print("Verified arm CreateSkin virtual inputs, bone-side mask, all resource patterns, source constants, float SizeFactor division and Hand field68/translation updates. Static/Rust checks, not complete CreateSkin or rendered-avatar parity.")
prefix = subprocess.check_output(["objdump","-d","--no-show-raw-insn","--disassemble=_ZNK6DpBody18GetBodyModelPrefixEv",str(state["target"])],text=True)
assert "movzbl 0x28(%rbx),%r8d" in prefix and "and    $0x7,%r8d" in prefix
assert read(0x1422BDCC5,8) == b":data/%1" and read(0x1422BDCBC,8) == b":data/PA"
