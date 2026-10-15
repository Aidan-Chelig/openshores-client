"""Static concrete part vtable bindings for CurrentConnectLoc/Bones, no execution."""
import hashlib
from pathlib import Path
import struct
import subprocess

target = Path("reference/openshores-full/OpenShores.exe")
data = target.read_bytes()
assert hashlib.sha256(data).hexdigest() == "285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80"
pe = struct.unpack_from("<I",data,60)[0]
optional = pe + 24
base = struct.unpack_from("<Q",data,optional+24)[0]
headers = optional + struct.unpack_from("<H",data,pe+20)[0]


def read(address,length):
    for index in range(struct.unpack_from("<H",data,pe+6)[0]):
        rva,size,offset = struct.unpack_from("<III",data,headers+40*index+12)
        if base+rva <= address and address+length <= base+rva+size:
            start = offset+address-base-rva
            return data[start:start+length]
    raise AssertionError(f"unbacked part binding {address:#x}")


bindings = [(0x1424818A0,0x1412B6860,None), (0x1423F97A0,0x1412B7FD0,None),
            (0x14247F2B0,0x1412B7870,None), (0x142425850,0x1412B8690,0x1412B5CC0),
            (0x142434A00,0x1412B8690,0x1412B5D30), (0x142425910,0x1412B8330,0x1412B5DB0),
            (0x142434AC0,0x1412B8330,0x1412B5E10)]
for vptr,current,bones in bindings:
    assert struct.unpack("<Q",read(vptr+0x30,8))[0] == current
    if bones is not None:
        assert struct.unpack("<Q",read(vptr+0xA0,8))[0] == bones
for name in ["_ZNK12DpArmAndHand17CurrentConnectLocEv","_ZNK12DpLegAndFoot17CurrentConnectLocEv"]:
    assembly = subprocess.check_output(["objdump","-d","--disassemble="+name,str(target)],text=True)
    assert "call   *0xa0(%rax)" in assembly
    assert "mov    0x24(%rsp),%ecx" in assembly
    assert "lea    0x24(%rsp),%rdx" in assembly
    assert "addpd" in assembly and "divpd" in assembly
print("Verified seven concrete CurrentConnectLoc vtable bindings, four Bones bindings, and first-reference selection in both limb readers. Static checks, not runtime parity.")
