"""Static native rhythm bindings, constants and SSE stages; never runs the target."""
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
    raise AssertionError(f"unbacked rhythm address {address:#x}")


expected = [0x3F59999A, 0x3F000000, 0x3E99999A, 0x40200000, 0x40400000,
            0x40000000, 0x3DCCCCCD, 0x3EB33333, 0x3F400000, 0x3F800000]
assert list(struct.unpack("<10I", read(0x142305388, 40))) == expected
assert struct.unpack("<d", read(0x1423053B0, 8))[0] == 60000.0
assert struct.unpack("<II", read(0x1423053B8, 8)) == (0x42700000, 0x40490FDB)
assert struct.unpack("<II", read(0x1422BDA90, 8)) == (0x3F800000, 0x40800000)
# Actual signed switch offsets for poses25..52, not inferred from names.
targets = [0x1415B7B30,0x1415B7B50,0x1415B7967,0x1415B7AF0,0x1415B7AF0,
           0x1415B7AF0,0x1415B7967,0x1415B7967,0x1415B7B40,0x1415B7AF0,
           0x1415B7967,0x1415B7967,0x1415B7B00,0x1415B7AF0,0x1415B7AF0,
           0x1415B7967,0x1415B7AF0,0x1415B7B10,0x1415B7B20,0x1415B7AF0,
           0x1415B7A39,0x1415B7967,0x1415B7B00,0x1415B7AF0,0x1415B7990,
           0x1415B7990,0x1415B7990,0x1415B7990]
assert [0x142304720 + offset for offset in struct.unpack("<28i", read(0x142304720, 112))] == targets
au = subprocess.check_output(["objdump", "-d", "--disassemble=_ZNK6AuPose6RhythmEffRK14AuElapsedTimer", str(target)], text=True)
dh = subprocess.check_output(["objdump", "-d", "--disassemble=_ZNK5DhDNA6RhythmERK6AuPoseff", str(target)], text=True)
for instruction in ["$0x3dffe9,%eax", "$0x800006,%eax", "$0x420010,%eax",
                    "$0x7fffffffbffff6,%rdx", "$0x400008,%eax",
                    "cvtsi2sd %rax,%xmm0", "cvtsd2ss %xmm3,%xmm3",
                    "mulss  %xmm9,%xmm1", "mulss  %xmm7,%xmm1",
                    "addss  %xmm0,%xmm0", "addss  %xmm6,%xmm0",
                    "QElapsedTimer19msecsSinceReferenceEv", "<sinf>"]:
    assert instruction in au, instruction
for instruction in ["movzbl 0x19(%rcx),%eax", "$0xf,%eax", "divss  %xmm1,%xmm0",
                    "divss  %xmm0,%xmm3", "QElapsedTimer5startEv", "<\u005fZNK6AuPose6RhythmEffRK14AuElapsedTimer>"]:
    assert instruction in dh, instruction
print("Verified native rhythm switch targets, masks, constants, timer calls and SSE widths. Static binding checks, not native runtime parity.")
