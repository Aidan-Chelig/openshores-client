"""Digest-bound machine-code return/operation anchors; no supplied target execution."""
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


code = assembly("_ZNK10DaCreature16MovementSpeedModEv")
for anchor in ("1418f4fb4:", "movaps %xmm6,%xmm0", "1418f4fcc:", "1418f4ff3:",
               "test   %ax,%ax", "1418f5108:", "$0x1,%ebx", "idiv   %ecx", "1418f4ffc:",
               "test   %bx,%bx", "1418f5017:", "0x29(%rax),%esi", "shr    $0x4,%sil",
               "1418f5054:", "cvtsi2ss %eax,%xmm7", "1418f505f:", "1418f5074:",
               "subss  %xmm0,%xmm6", "divss  %xmm7,%xmm6", "1418f5089:",
               "add    %ebx,%ebx", "cmp    %bx,%ax", "1418f50b2:", "1418f50b6:"):
    assert anchor in code, anchor
assert code.count("call   *0x970(%rax)") == 2
assert "_ZNK10AuAnimalia9BurdenMaxEv" in code
assert "1422b2858" in assembly("_ZNK10AuAnimalia9BurdenMaxEv")
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
    raise AssertionError(f"unbacked movement address {address:#x}")


assert read(0x1422B2858, 4) == struct.pack("<f", 200.)
assert read(0x1421C9C1C, 4) == struct.pack("<f", 1.)
assert struct.unpack("<Q", read(0x14248C5B0 + 0x970, 8))[0] == 0x1419466B0
assert "<_ZNK8DaPerson6BurdenEv>:" in assembly("_ZNK8DaPerson6BurdenEv")
print("Native movement: f32 return, growth/short operations, two burden samples, exact200/1 constants and Person burden binding verified statically.")
