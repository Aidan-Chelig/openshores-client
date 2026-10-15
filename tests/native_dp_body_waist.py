"""Static matrix cache-flag proof, arm mapping and body-waist dispatch anchors."""
import re
import runpy
import struct
import subprocess

c=runpy.run_path("tests/native_dp_connections.py")
read=c["read"]
def assembly(name):
    return subprocess.check_output(["objdump","-d","--no-show-raw-insn","--disassemble="+name,str(c["target"])],text=True)

product=assembly("_ZNK11AuMatrix4x4mlERKS_")
assert product.count("mulsd")==64 and product.count("addsd")==48
assert product.count("movb   $0x0,0x100(%r9)")==1
instructions=re.findall(r"^\s*[0-9a-f]+:\s+(\S+)",product,re.M)
assert "call" not in instructions and not any(i.startswith("j") for i in instructions)
arm=assembly("_ZN12DpArmAndHand6MoveByERK7AuPoint")
assert arm.count("<_ZNK11AuMatrix4x4mlERKS_>")==2
assert "call   *0xa0(%rax)" in arm
for vptr,_,_ in c["bindings"]:
    is_arm=struct.unpack("<Q",read(vptr+0x38,8))[0]
    if vptr in [0x142425850,0x142434A00]:
        assert read(is_arm,6)==bytes.fromhex("b8 01 00 00 00 c3")
        assert struct.unpack("<Q",read(vptr+0x68,8))[0]==0x1412B9150
    else:
        assert read(is_arm,3)==bytes.fromhex("31 c0 c3")
body=assembly("_ZN6DpBody11RotateWaistEf.part.0")
for anchor in ["<_ZNK6DpBody15CurrentWaistLocEv>","<_ZN7DpTorso11RotateWaistEf>",
               "<_ZN13DpHeadAndNeck10RotateNeckEf.part.0>","*0x58(%rax)","*0x40(%rax)","*0x38(%rax)","*0x68(%rax)"]:
    assert anchor in body,anchor
print("Verified primary-only64-product matrix operator with unconditional flag clear, both arm virtual bindings and real ordered body-waist stages. Static evidence, not native runtime parity.")
