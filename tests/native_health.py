"""Native model HP/growth static checks; never executes the supplied client."""
import hashlib
from pathlib import Path
import struct
import subprocess

target = Path("reference/openshores-full/OpenShores.exe")
data = target.read_bytes()
assert hashlib.sha256(data).hexdigest() == "285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80"


def assembly(symbol):
    result = subprocess.check_output(["objdump", "-d", f"--disassemble={symbol}", str(target)], text=True)
    assert f"<{symbol}>:" in result
    return result


dp = assembly("_ZNK6DpBody12MaxHitPointsEv")
for anchor in ("0x68(%rcx),%xmm6", "call   *0x98(%rax)", "addss  %xmm0,%xmm6",
               "movss  %xmm6,0x68(%rbx)", "1422bfb30", "cvttss2si %xmm0,%edx",
               "test   %dx,%dx", "$0x4,0x1d(%rbx)", "$0x3,0x19(%rbx)",
               "$0x6,0x1a(%rbx)", "shr    $1,%ax", "$0x1fffe,%eax", "$0xaaaaaaab,%edx"):
    assert anchor in dp, anchor
au = assembly("_ZNK10AuAnimalia12MaxHitPointsEv")
for anchor in ("_ZNK11AuChromatid6VolumeEv", "1422b2850", "cvttss2si %xmm0,%ebx",
               "test   %bx,%bx", "_ZTI12AuAnimalWing", "_ZNK12AuAnimalWing6CanFlyEv", "shr    $1,%r12w"):
    assert anchor in au, anchor
growth = assembly("_ZNK10AuAnimalia10GrowthTimeEv")
for anchor in ("_ZNK10AuAnimalia12MaxHitPointsEv", "sqrtss %xmm0,%xmm0", "addss  %xmm0,%xmm0", "ceilf", "cvttss2si %xmm0,%eax", "$0xff,%edx", "cmovg  %edx,%eax"):
    assert anchor in growth, anchor
creature = assembly("_ZNK10DaCreature12MaxHitPointsEv")
for anchor in ("idiv   %ecx", "test   %ax,%ax", "cmovne %eax,%r12d", "$0x1,%r12d", "0x494(%rbx)"):
    assert anchor in creature, anchor
assert "or     $0x7,%r12d" in assembly("_ZNK10DaCreature10GrowthTimeEv")
pe = struct.unpack_from("<I", data, 60)[0]
optional = pe + 24
base = struct.unpack_from("<Q", data, optional + 24)[0]
headers = optional + struct.unpack_from("<H", data, pe + 20)[0]
for address, bits in ((0x1422BFB30, 0x41EC86EB), (0x1422B2850, 0x421CF478)):
    for index in range(struct.unpack_from("<H", data, pe + 6)[0]):
        rva, size, offset = struct.unpack_from("<III", data, headers + 40 * index + 12)
        if base + rva <= address and address + 4 <= base + rva + size:
            start = offset + address - base - rva
            assert data[start:start + 4] == struct.pack("<I", bits)
            break
    else:
        raise AssertionError(f"unbacked HP scale {address:#x}")
print("Native HP/growth: cache/volume/wing stages, conversion and short masks, exact float scale words and growth arithmetic verified statically.")
