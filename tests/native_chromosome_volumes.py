"""Static volume dispatch bindings; this does not implement morphology or execute the target."""
import hashlib
from pathlib import Path
import struct
import subprocess

target = Path("reference/openshores-full/OpenShores.exe")
data = target.read_bytes()
assert hashlib.sha256(data).hexdigest() == "285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80"
pe = struct.unpack_from("<I", data, 60)[0]
optional = pe + 24
base = struct.unpack_from("<Q", data, optional + 24)[0]
headers = optional + struct.unpack_from("<H", data, pe + 20)[0]


def read(address, length):
    for index in range(struct.unpack_from("<H", data, pe + 6)[0]):
        rva, size, offset = struct.unpack_from("<III", data, headers + 40 * index + 12)
        if base + rva <= address and address + length <= base + rva + size:
            start = offset + address - base - rva
            return data[start:start + length]
    raise AssertionError(f"unbacked chromosome volume slot {address:#x}")


vptrs = [0x14241A510,0x142435960,0x14241A660,0x14241A900,0x14241A7B0,0x1423D0500,
         0x1423FC250,0x1423DAB10,0x1423DA720,0x1423EAA40,0x1423CFFC0,0x1423D0650,
         0x1423D03B0,0x14241B6B0,0x1423DAC60,0x14240A5A0,0x1423DA870,0x1423D0110,
         0x1423D0260,0x1423DA9C0,0x1423EA8F0,0x1423EA7A0,0x1423EA650,0x1423EA500,
         0x142441330,0x142435AB0,0x14244B4F0,0x14240A6F0]
overrides = {8:0x141205B90,9:0x141205D30,10:0x141205E70,11:0x141206110,
             17:0x1412065E0,19:0x1412066D0,20:0x1412067F0,27:0x141206910}
assert len(vptrs) == 28
for address, bits, width in [
    (0x1422B4668, 0x3F6AE06B81AE06B0, 8),
    (0x1422B4660, 0x3FE0000000000000, 8),
    (0x1422B4578, 0x40860A92, 4),
    (0x1422B457C, 0x40490FDB, 4),
    (0x1422B4588, 0x40A00000, 4),
    (0x1422B45B0, 0x3F333333, 4),
]:
    assert int.from_bytes(read(address, width), "little") == bits, hex(address)
for kind, vptr in enumerate(vptrs):
    assert struct.unpack("<Q", read(vptr + 0x128, 8))[0] == overrides.get(kind,0x141900F50), kind
zero = subprocess.check_output(["objdump","-d","--disassemble=_ZNK12AuChromosome6VolumeEv",str(target)],text=True)
assert "pxor   %xmm0,%xmm0" in zero and "141900f54:" in zero
chromatid = subprocess.check_output(["objdump","-d","--disassemble=_ZNK11AuChromatid6VolumeEv",str(target)],text=True)
assert "call   *0x128(%rax)" in chromatid
assert "addss  %xmm0,%xmm6" in chromatid and "movaps %xmm6,%xmm0" in chromatid
print("Verified28 factory-vptr volume slots:20 shared zero methods and8 distinct implementations, six constant patterns, plus ordered chromatid f32 accumulation. Binding/constant checks only; no runtime or morphology validation.")
