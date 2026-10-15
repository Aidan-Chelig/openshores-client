"""Decode native CGHeight dispatch into an offline fixture; never execute target."""
import hashlib
from pathlib import Path
import struct
import subprocess
import sys

target = Path("reference/openshores-full/OpenShores.exe")
data = target.read_bytes()
digest = "285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80"
assert hashlib.sha256(data).hexdigest() == digest
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
    raise AssertionError(f"unbacked CGHeight address {address:#x}")


# Tags identify verified instruction blocks, not inferred pose names.
tags = {0x1412C8550:0, 0x1412C8530:1, 0x1412C8502:2, 0x1412C85A0:3,
        0x1412C8578:4, 0x1412C8680:5, 0x1412C85F0:6, 0x1412C8790:7,
        0x1412C8710:8, 0x1412C8590:9}
kinds = [0,0,0] + [tags[0x1422BDD8C + offset] for offset in struct.unpack("<81i",read(0x1422BDD8C,324))]
constant_mask = struct.unpack("<Q", read(0x1412C84E3,8))[0]
standing_mask = struct.unpack("<Q", read(0x1412C84F5,8))[0]
assert constant_mask == 0x4E66666333333
assert standing_mask == 0xB199998CCCCCC
for pose in range(84,138):
    bit = 1 << (pose - 84)
    kinds.append(1 if bit & constant_mask else 2 if bit & standing_mask else 6 if pose == 137 else 0)
for address,bits in [(0x1422BFBF0,0x3FD55555),(0x1422BFBF4,0x3F2AAAAB),
                     (0x1422BFAD0,0x40A00000),(0x1422BF8F8,0x3FC00000),
                     (0x1422BF89C,0x3F19999A)]:
    assert int.from_bytes(read(address,4),"little") == bits
assert struct.unpack("<4I",read(0x1422BF8B0,16)) == (0x3F800000,0x3F666666,0x3F4CCCCD,0x3F333333)
fixture = "# Static CGHeight dispatch; target SHA-256 " + digest + "\n"
fixture += "# pose kind: 0 default; 1 fixed5/3; 2 standing; 3 relative; 4 half-width; 5 half-standing; 6 neck; 7 zero; 8 fixed1.5; 9 fixed5\n"
fixture += "".join(f"{pose}\t{kind}\n" for pose,kind in enumerate(kinds))
if "--print-fixture" in sys.argv:
    print(fixture,end="")
else:
    assert Path("tests/fixtures/native_dp_height_kinds.tsv").read_text() == fixture
    assembly = subprocess.check_output(["objdump","-d","--start-address=0x1412c8490","--stop-address=0x1412c8845",str(target)],text=True)
    for instruction in ["mov    0x48(%rcx),%ecx", "$0x4e66666333333,%r8", "$0xb199998cccccc,%r8",
                        "maxss  %xmm0,%xmm1", "minss  %xmm1,%xmm0", "CGHeightStandingEb.part.0>"]:
        assert instruction in assembly, instruction
    print("Verified138 native CGHeight dispatch fixture entries, scalar constants, full-width pose load and SSE comparisons. Static evidence, not runtime parity.")
