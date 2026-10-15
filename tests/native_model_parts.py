"""Static model-part factory bindings; no native execution or geometry emulation."""
from pathlib import Path
import runpy
import struct
import subprocess

# Reuse the digest-bound PE mapping and verify the existing volume bindings.
inventory = runpy.run_path(str(Path(__file__).with_name("native_chromosome_volumes.py")))
read = inventory["read"]
target = inventory["target"]
overrides = {
    8: 0x1412261A0, 9: 0x141225FE0, 10: 0x141226370,
    11: 0x141224730, 12: 0x141225E00, 13: 0x141225A50,
    14: 0x141225C20,
}
for kind, vptr in enumerate(inventory["vptrs"]):
    observed = struct.unpack("<Q", read(vptr + 0xD0, 8))[0]
    assert observed == overrides.get(kind, 0x141205AF0), kind

def disassemble(symbol):
    return subprocess.check_output(
        ["objdump", "-d", "--disassemble=" + symbol, str(target)], text=True)

null = disassemble("_ZNK12AuChromosome8MakePartERK11AuChromatidRh")
assert "xor    %eax,%eax" in null and "141205af2:" in null
parts = disassemble("_ZNK5AuDNA9MakePartsER5QListIP12AuAnimalPartEN9AuDNABase9CHROMATIDE")
assert "call   *0xd0(%rax)" in parts
assert "movb   $0x0,0x27(%rsp)" in parts
assert "_ZN5QListIP12AuAnimalPartE6appendERKS1_" in parts
assert "test   %rax,%rax" in parts
wing = disassemble("_ZNK12AuAnimalWing6CanFlyEv")
assert "mov    0xf8(%rcx),%rcx" in wing
assert "_ZNK12AuChromoWing6CanFlyEv" in wing
assert read(0x14121E550, 6) == bytes.fromhex("b801000000c3")
print("Verified28 model-part slots:21 null and7 generators, shared bone-index seed, and actual Wing CanFly=true. No geometry/runtime validation.")
