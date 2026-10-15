"""Static outer SetDNA corroboration only; never executes the supplied target."""
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


outer = assembly("_ZN10DaCreature6SetDNAERK9AuDNABase")
for anchor in ("14097c099:", "0x8(%rax),%esi", "0x2c(%rax),%esi", "0x30(%rax),%eax",
               "14097c25c:", "_ZNK6DpBody9ChromatidEv", "0x3a(%rdx),%eax", "xor    $0x1,%eax",
               "14097c290:", "14097c2bc:", "_ZTI5DhDNA", "14097c320:", "_ZN6DpBodyC1EcbRK5DhDNA",
               "14097c376:", "_ZN6DpBody6SetDNAERK5DhDNAf", "14097c393:", "cmovge %edi,%eax",
               "14097c120:", "_ZN10AuAnimalia12SetHitPointsEs", "14097c139:", "_ZN6DpBodyD1Ev",
               "14097c167:", "14097c1b1:", "_ZN10AuAnimalia9InitPosesERK6AuPosePS0_",
               "14097c1cc:", "_ZN10AuAnimalia4PoseERK4AuIdPK6AuPosebf", "14097c1d8:",
               "14097c3a5:", "_ZN10AuAnimaliaD1Ev", "14097c3c4:"):
    assert anchor in outer, anchor
chromatid = assembly("_ZNK6DpBody9ChromatidEv")
assert "cmpb   $0x1,0x64(%rcx)" in chromatid and "setne  %al" in chromatid
hp = assembly("_ZN10AuAnimalia12SetHitPointsEs")
assert "$0xffffffe2,%eax" in hp and "cmovl  %eax,%edx" in hp
# SetDNA's Dp update loads a float from this original PE constant, not a
# decompiler-inferred decimal. Validate the actual little-endian source bytes.
pe = struct.unpack_from("<I", data, 60)[0]
optional = pe + 24
base = struct.unpack_from("<Q", data, optional + 24)[0]
headers = optional + struct.unpack_from("<H", data, pe + 20)[0]
for index in range(struct.unpack_from("<H", data, pe + 6)[0]):
    rva, size, offset = struct.unpack_from("<III", data, headers + 40 * index + 12)
    address = 0x1421C9C1C
    if base + rva <= address and address + 4 <= base + rva + size:
        start = offset + address - base - rva
        assert data[start:start + 4] == struct.pack("<f", 1.0)
        break
else:
    raise AssertionError("unbacked SetDNA scale constant")
print("Native SetDNA: digest-bound dynamic branches, raw configuration helpers, HP clamps, destruction/pose/timer anchors and float scale verified statically.")
