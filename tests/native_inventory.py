"""Read-only native inventory factory/filter/duplicate-path anchors; no execution."""
import hashlib
from pathlib import Path
import re
import struct
import subprocess

target = Path("reference/openshores-full/OpenShores.exe")
assert hashlib.sha256(target.read_bytes()).hexdigest() == "285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80"


def assembly(*args):
    return subprocess.check_output(["objdump", "-d", *args, str(target)], text=True)


factory = assembly("--disassemble=_ZN6AuItem6CreateEh")
assert "cmp    $0x18,%cl" in factory
assert "ja     1414f0610" in factory
assert "xor    %r12d,%r12d" in factory
raw = subprocess.check_output(["objdump", "-s", "--start-address=0x1422e1338",
    "--stop-address=0x1422e139c", str(target)], text=True)
hex_bytes = ""
for line in raw.splitlines():
    fields = line.split()
    if fields and re.fullmatch(r"1422e[0-9a-f]+", fields[0]):
        hex_bytes += "".join(word for word in fields[1:5] if re.fullmatch(r"[0-9a-f]{8}", word))
table = bytes.fromhex(hex_bytes)
assert len(table) == 100
null_types = []
for kind, (relative,) in enumerate(struct.iter_unpack("<i", table)):
    entry = 0x1422e1338 + relative
    if entry == 0x1414f0610:
        null_types.append(kind)
    else:
        block = factory[factory.index(f"   {entry:x}:"):]
        terminal = re.search(r"\b(?:jmp|ret)\b", block)
        assert terminal
        assert "<_Znwy>" in block[:terminal.start()], (kind, hex(entry))
assert null_types == [0, 2, 3, 11, 15, 21]

obsolete = assembly("--disassemble=_ZN11DbCommodity10IsObsoleteE11COMMODITYID")
for anchor in ("$0x27b", "$0xed", "$0x67", "$0x35", "$0x2000000030001", "$0x36", "$0x119", "$0x117", "$0x27e"):
    # lea -27b is printed as its two's-complement displacement.
    assert anchor in obsolete or anchor == "$0x27b" and "-0x27b" in obsolete
emplace = assembly("--start-address=0x1414cd800", "--stop-address=0x1414cd900")
for anchor in ("1414cd8c7:", "1414cd8cc:", "1414cd8cf:", "1414cd8d6:", "1414cd8da:", "1414cd8dd:", "1414cd8f5:"):
    assert anchor in emplace
assert "addq   $0x1,0x18(%rsi)" in emplace
print("Verified25 factory table entries, null cases, obsolete filter and matching-key duplicate insertion anchors; static only.")
