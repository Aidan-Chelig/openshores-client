"""Read selected native vtable slots from immutable PE bytes; never execute it."""
import hashlib
from pathlib import Path
import re
import struct
import subprocess

TARGET = Path("reference/openshores-full/OpenShores.exe")
data = TARGET.read_bytes()
assert hashlib.sha256(data).hexdigest() == "285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80"
pe = struct.unpack_from("<I", data, 0x3C)[0]
assert data[pe:pe + 4] == b"PE\0\0"
count = struct.unpack_from("<H", data, pe + 6)[0]
optional_size = struct.unpack_from("<H", data, pe + 20)[0]
optional = pe + 24
assert struct.unpack_from("<H", data, optional)[0] == 0x20B
base = struct.unpack_from("<Q", data, optional + 24)[0]
sections = []
for index in range(count):
    header = optional + optional_size + index * 40
    rva, size, offset = struct.unpack_from("<III", data, header + 12)
    sections.append((base + rva, size, offset))

symbols = {}
reverse = {}
table = subprocess.check_output(["objdump", "-t", str(TARGET)], text=True)
for line in table.splitlines():
    match = re.search(r"\(sec\s+(\d+)\).*\(scl\s+2\).*\(nx\s+\d+\)\s+(0x[0-9a-f]+)\s+(\S+)$", line)
    if not match:
        continue
    section, value, name = match.groups()
    section = int(section)
    if not 1 <= section <= len(sections):
        continue
    address = sections[section - 1][0] + int(value, 16)
    symbols[name] = address
    reverse.setdefault(address, []).append(name)


def pointer(address):
    for vma, size, offset in sections:
        if vma <= address and address + 8 <= vma + size:
            return struct.unpack_from("<Q", data, offset + address - vma)[0]
    raise AssertionError(f"Unbacked vtable address {address:#x}")


assert struct.pack("<Q", pointer(symbols["_ZNK6DaAtom20CanRenderEnvironmentEv"]))[:3] == b"\x31\xc0\xc3"
for predicate in ("_ZNK8DaSector20CanRenderEnvironmentEv", "_ZNK13DaSolarSystem20CanRenderEnvironmentEv"):
    assert struct.pack("<Q", pointer(symbols[predicate]))[:6] == b"\xb8\x01\x00\x00\x00\xc3"

for name in ("_ZTV6DaAtom", "_ZTV6DaItem", "_ZTV8DaPerson", "_ZTV7DaWorld", "_ZTV12DaWorldGlobe", "_ZTV9DaWorldLL", "_ZTV8DaSector", "_ZTV13DaSolarSystem", "_ZTV9DaVehicle"):
    # Primary Itanium ABI address point follows offset-to-top and RTTI pointers.
    vtable = symbols[name]
    assert pointer(vtable) == 0
    for slot in (0x348, 0x4A8, 0x608, 0x7B0):
        address = vtable + 16 + slot
        target = pointer(address)
        names = reverse.get(target, [])
        assert names, f"Unresolved slot {name}+{slot:#x} -> {target:#x}"
        expected = ({"_ZTV13DaSolarSystem": "_ZNK13DaSolarSystem12EnvTransformEv"}
            .get(name, "_ZNK6DaAtom12EnvTransformEv")) if slot == 0x348 else {
            "_ZTV8DaSector": "_ZNK8DaSector20CanRenderEnvironmentEv",
            "_ZTV13DaSolarSystem": "_ZNK13DaSolarSystem20CanRenderEnvironmentEv",
        }.get(name, "_ZNK6DaAtom20CanRenderEnvironmentEv")
        if slot == 0x608:
            expected = ("_ZNK13DaSolarSystem28GetTransformRelativeToParentER11AuMatrix4x4PK6DaAtom"
                if name == "_ZTV13DaSolarSystem" else "_ZNK6DaAtom28GetTransformRelativeToParentER11AuMatrix4x4PKS_")
        if slot == 0x4A8:
            expected = ("_ZN13DaSolarSystem21FindParentPlanetInEnvEv"
                if name == "_ZTV13DaSolarSystem" else
                "_ZN7DaWorld21FindParentPlanetInEnvEv" if name in
                ("_ZTV7DaWorld", "_ZTV12DaWorldGlobe", "_ZTV9DaWorldLL") else
                "_ZN6DaAtom21FindParentPlanetInEnvEv")
            assert expected in names, (name, slot, names)
            print(f"{name} slot={slot:#x} -> {target:#x} {names}")
            continue
        assert expected in names, (name, slot, names)
        print(f"{name} table={vtable:#x} slot={slot:#x} bytes={address:#x} -> {target:#x} {names}")
