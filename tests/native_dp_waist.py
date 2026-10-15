"""Static torso identity, waist-origin and concrete rotation anchors."""
import runpy
import struct
import subprocess

c=runpy.run_path("tests/native_dp_connections.py")
read=c["read"]
for vptr,_,_ in c["bindings"]:
    method=struct.unpack("<Q",read(vptr+0x58,8))[0]
    if vptr==0x1424818A0:
        assert read(method,6)==bytes.fromhex("b8 01 00 00 00 c3")
    else:
        assert read(method,3)==bytes.fromhex("31 c0 c3")
def assembly(name):
    return subprocess.check_output(["objdump","-d","--no-show-raw-insn","--disassemble="+name,str(c["target"])],text=True)
current=assembly("_ZNK6DpBody15CurrentWaistLocEv")
assert "call   *0x58(%rax)" in current
torso=assembly("_ZN7DpTorso11RotateWaistEf")
for anchor in ["ucomiss","<_ZNK5DhDNA11WaistVertexEv>","<_ZNK11AuMatrix4x47ProjectEddd>",
               "<_ZN11AuMatrix4x47RotateXEd>","$0x108"]:
    assert anchor in torso,anchor
print("Verified seven torso identity slots, first-part waist lookup and concrete torso waist/pivot/rotation/stride anchors. Static checks, not native execution.")
