"""Verify native job factory bindings against immutable PE bytes; never execute it."""
import hashlib
import json
from pathlib import Path
import re
import struct
import subprocess

target = Path("reference/openshores-full/OpenShores.exe")
data = target.read_bytes()
index = json.loads(Path("docs/evidence/native-job-factory-bindings.json").read_text())
assert hashlib.sha256(data).hexdigest() == index["target_sha256"]
pe = struct.unpack_from("<I", data, 0x3c)[0]
assert data[pe:pe + 4] == b"PE\0\0"
count = struct.unpack_from("<H", data, pe + 6)[0]
optional_size = struct.unpack_from("<H", data, pe + 20)[0]
optional = pe + 24
assert struct.unpack_from("<H", data, optional)[0] == 0x20b
base = struct.unpack_from("<Q", data, optional + 24)[0]
sections = []
for number in range(count):
    header = optional + optional_size + number * 40
    rva, size, offset = struct.unpack_from("<III", data, header + 12)
    sections.append((base + rva, size, offset))


def read(address, size):
    for vma, length, offset in sections:
        if vma <= address and address + size <= vma + length:
            return data[offset + address - vma:offset + address - vma + size]
    raise AssertionError(f"Unbacked static address {address:#x}")


assembly = subprocess.check_output(["objdump", "-d",
    "--disassemble=_ZN5AuJob9CreateJobENS_4TYPEE", str(target)], text=True)
assert "cmp    $0x1a,%ecx" in assembly
assert "ja     14091bed8" in assembly
assert "xor    %eax,%eax" in assembly[assembly.index("   14091bed8:"):]
table = int(index["jump_table"], 16)
assert len(index["rows"]) == 27
for number, row in enumerate(index["rows"]):
    assert row["type"] == number
    entry = table + struct.unpack("<i", read(table + number * 4, 4))[0]
    assert entry == int(row["entry"], 16)
    block = assembly[assembly.index(f"   {entry:x}:"):]
    terminal = re.search(r"\b(?:jmp|ret)\b", block)
    assert terminal
    block = block[:terminal.start()]
    assert f'# {int(row["vptr"], 16):x} <{row["class"]}+0x10>' in block
    vptr = int(row["vptr"], 16)
    assert struct.unpack("<Q", read(vptr + 0x70, 8))[0] == int(row["reader"], 16)
    assert struct.unpack("<Q", read(vptr + 0x50, 8))[0] == int(row["discard"], 16)
    discard = read(int(row["discard"], 16), 6)
    assert discard[:3] == b"\x31\xc0\xc3" if number not in (1, 2, 3) else discard == b"\xb8\x01\0\0\0\xc3"

qt = Path("reference/openshores-full/Qt5Core.dll")
assert hashlib.sha256(qt.read_bytes()).hexdigest() == "3aad8885b1c7b2c0c5e2016095c8c51c3089c17a70269bc332cfe2c483b65ccc"
qt_assembly = subprocess.check_output(["objdump", "-d", "--start-address=0x3955481c0",
    "--stop-address=0x3955483e0", str(qt)], text=True)
for anchor in ("39554822c:", "3955483b0:", "39554831f:", "395548330:",
               "395548345:", "395548349:", "39554834d:", "395548354:"):
    assert anchor in qt_assembly
print("Verified all27 native job factory/vptr/reader/obsolete bindings and bounded Qt7 datetime anchors; static evidence, not runtime parity.")
