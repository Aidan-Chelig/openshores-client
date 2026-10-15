"""Digest-bound static constructor checks; never executes the supplied client."""
import hashlib
from pathlib import Path
import subprocess
import struct

target = Path("reference/openshores-full/OpenShores.exe")
data = target.read_bytes()
assert hashlib.sha256(data).hexdigest() == "285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80"


def read(address, length):
    pe = struct.unpack_from("<I", data, 60)[0]
    optional = pe + 24
    base = struct.unpack_from("<Q", data, optional + 24)[0]
    headers = optional + struct.unpack_from("<H", data, pe + 20)[0]
    for index in range(struct.unpack_from("<H", data, pe + 6)[0]):
        rva, size, offset = struct.unpack_from("<III", data, headers + 40 * index + 12)
        if base + rva <= address and address + length <= base + rva + size:
            start = offset + address - base - rva
            return data[start:start + length]
    raise AssertionError(f"Unbacked constructor constant {address:#x}")


def constructor(symbol):
    # GNU selects C1 for these C1/C2 aliases; C2 alone yields an empty listing.
    result = subprocess.check_output(["objdump", "-d", f"--disassemble={symbol}",
                                      str(target)], text=True)
    assert f"<{symbol}>:" in result
    return result


person = constructor("_ZN8DaPersonC1Ev")
creature = constructor("_ZN10DaCreatureC1Ev")
assert "140b209e0 <_ZN10DaSentientC1Ev>" in person
assert "140f02510 <_ZN6DaUnitC1Ev>" in creature
assert "xor    %r8d,%r8d" in creature
assert "mov    %r8w,0x660(%r12)" in creature
for field in (0x10ec, 0x10f8, 0x1110, 0xf80, 0xf88, 0xfb0, 0x10e8,
              0x1130, 0x13a8):
    assert f"movl   $0x0,{field:#x}(%r12)" in person, hex(field)
for field in (0x1108, 0x10cc, 0x1408):
    assert f"movq   $0x0,{field:#x}(%r12)" in person, hex(field)
assert "movups %xmm0,0xf60(%r12)" in person
assert "pxor   %xmm0,%xmm0" in person
assert "movb   $0x0,0xf84(%r12)" in person
assert "movb   $0xf,0x83b(%r12)" in person
assert "mov    $0xf0f,%ecx" in person
assert "mov    %cx,0x83c(%r12)" in person
for register, destination in (("dx", "838"), ("r10w", "fa8"), ("r9w", "f58")):
    assert f"%{register},0x{destination}(%r12)" in person
assert "xor    %edx,%edx" in person
assert "xor    %r10d,%r10d" in person
assert "xor    %r9d,%r9d" in person
assert "__imp__ZN10QArrayData11shared_nullE>" in person
for field in (0x10f0, 0x1100, 0x1118):
    assert f"mov    %rdi,{field:#x}(%r12)" in person
assert "__imp__ZN9QListData11shared_nullE>" in person
assert "mov    %rsi,0xfa0(%r12)" in person
assert "mov    %rsi,0x13b8(%r12)" in person
assert "__imp__ZN9QHashData11shared_nullE>" in person
assert "mov    %rax,0x1018(%r12)" in person
assert "lea    0x1390(%r12),%r13" in person
assert "1414c6440 <_ZN6AuGearC1Ev>" in person
gear = constructor("_ZN6AuGearC1Ev")
assert "__imp__ZN9QListData11shared_nullE>" in gear
assert "mov    %rax,(%rcx)" in gear
for field in (0x1468, 0x1470, 0x1480, 0x1488):
    assert f"movq   $0x0,{field:#x}(%r12)" in person
unit = constructor("_ZN6DaUnitC1Ev")
assert "14184a920 <_ZN6DaAtomC2Ev>" in unit
assert "__imp__ZN10QArrayData11shared_nullE>" in unit
assert "mov    %rax,0x3b0(%r12)" in unit
assert "movl   $0x0,0x3a8(%r12)" in unit
assert "xor    %eax,%eax" in unit and "mov    %ax,0x39c(%r12)" in unit
assert "__imp__ZN9QListData11shared_nullE>" in unit
assert "mov    %rax,0x408(%r12)" in unit
assert "1412dc540 <_ZN6DpBodyC1Ev>" in creature
assert "mov    %r13,0x5b0(%r12)" in creature
assert "1421c9b28" in creature and "mov    %rax,0x5b8(%r12)" in creature
assert read(0x1421C9B28, 8) == struct.pack("<II", 4, 4)
assert "xor    %r9d,%r9d" in creature and "mov    %r9w,0x494(%r12)" in creature
assert "movb   $0x0,0x496(%r12)" in creature
sentient = constructor("_ZN10DaSentientC1Ev")
assert "140973ab0 <_ZN10DaCreatureC1Ev>" in sentient
assert "movl   $0x1,0x6f0(%r12)" in sentient
assert "movq   $0x0,0x708(%r12)" in sentient
assert "__imp__ZN9QListData11shared_nullE>" in sentient
assert "mov    %rdi,0x6e8(%r12)" in sentient
assert "14136d030 <_ZN10AuSkillSetC1Ev>" in sentient
skills = constructor("_ZN10AuSkillSetC1Ev")
assert "__imp__ZN9QHashData11shared_nullE>" in skills
assert "movups %xmm0,(%rcx)" in skills
print("Person inherited constructor subsets: base call chain, initial DpBody, masked priors, scalar/null/empty collection and nibble defaults verified statically; no runtime construction claim.")
