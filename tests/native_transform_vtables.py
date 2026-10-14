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

# Constructor dispatch inventory. Do not reuse server opcodes by assumption.
factory_table = 0x1421F7D98
native_classes = {
    0x02: "DaAnimal", 0x03: "DaArena", 0x04: "DaAsteroid", 0x05: "DaPerson",
    0x06: "DaBd", 0x12: "DaDesigner", 0x13: "DaDesignPlane", 0x14: "DaEffect",
    0x15: "DaGalaxy", 0x16: "DaItem", 0x17: "DaMissile", 0x18: "DaNpc",
    0x19: "DaSc", 0x1A: "DaSector", 0x1C: "DaSolarSystem", 0x1D: "DaStar",
    0x1E: "DaTeleporter", 0x20: "DaUniverse", 0x21: "DaVehicle",
    0x23: "DaWorldGasGiant", 0x24: "DaWorldGlobe", 0x27: "DaWorldRing",
    0x28: "DaWorldRingSection",
}
for opcode in range(0x29):
    relative = struct.unpack("<i", struct.pack("<Q", pointer(factory_table + opcode * 4))[:4])[0]
    entry = factory_table + relative
    block = subprocess.check_output(["objdump", "-d", f"--start-address={entry:#x}",
        f"--stop-address={entry + 48:#x}", str(TARGET)], text=True)
    calls = re.findall(r"call\s+[0-9a-f]+ <([^>]+C1[^>]*)>", block)
    cls = native_classes.get(opcode)
    assert calls == ([f"_ZN{len(cls)}{cls}C1Ev"] if cls else []), (opcode, calls)
    if cls is None:
        assert entry == 0x140B2CE68
print(f"Native atom factory: {len(native_classes)} constructor bindings and unsupported cases verified statically.")
for opcode, cls in native_classes.items():
    vtable = symbols[f"_ZTV{len(cls)}{cls}"]
    predicate = pointer(vtable + 16 + 0x7B0)
    code = struct.pack("<Q", pointer(predicate))
    value = False if code[:3] == b"\x31\xc0\xc3" else True if code[:6] == b"\xb8\x01\x00\x00\x00\xc3" else None
    assert value == (opcode in (0x03, 0x12, 0x1A, 0x1C)), (opcode, cls, predicate, value)
print("Native environment boundaries: all 23 predicate bindings and constant return values verified.")
for opcode, cls in native_classes.items():
    vtable = symbols[f"_ZTV{len(cls)}{cls}"]
    predicate = pointer(vtable + 16 + 0x7B8)
    code = struct.pack("<Q", pointer(predicate))
    value = False if code[:3] == b"\x31\xc0\xc3" else True if code[:6] == b"\xb8\x01\x00\x00\x00\xc3" else None
    assert value == (opcode in (0x03, 0x12)), (opcode, cls, predicate, value)
print("Native instance-universe predicate: all 23 bindings verified; distinct from environment boundaries and Universe opcode.")

# TickTock's virtual calls, measured at primary ABI address points. Do not
# mistake DtClient's QObject primary table for an AuGlobal subobject table.
for table_name, bindings in {
    "_ZTV6DaAtom": {
        0x2A8: "_ZNK6DaAtom8IsTickerEv",
        0x2C8: "_ZNK6DaAtom8IsTockerEv",
        0x2F0: "_ZNK6DaAtom8AtomTypeEv",
        0x4D0: "_ZNK6DaAtom21FindParentSolarSystemEv",
    },
    "_ZTV8AuGlobal": {
        0x50: "_ZN8AuGlobal17UpdateOrAddTickerERK4AuIdhS2_",
        0x60: "_ZN8AuGlobal17UpdateOrAddTockerERK4AuIdhS2_j",
    },
}.items():
    assert pointer(symbols[table_name]) == 0
    for slot, name in bindings.items():
        assert pointer(symbols[table_name] + 16 + slot) == symbols[name], (table_name, slot, name)
print("Native TickTock: base atom getter and global registration virtual bindings verified statically.")

assert pointer(0x142304550) == 0x8000000000000000
assert pointer(0x142304558) == 0x8000000000000000
for address, pending_offset in [(0x1415A6C73, 0x430), (0x1415A6E13, 0x448)]:
    assert struct.pack("<Q", pointer(address))[:4] == bytes.fromhex("c6 40 38 01")
    assert struct.pack("<Q", pointer(address + 4))[:4] == bytes.fromhex("0f 11 40 28")
    assert struct.pack("<Q", pointer(address + 8))[:7] == b"\xc6\x83" + struct.pack("<I", pending_offset) + b"\x01"
assert struct.pack("<Q", pointer(0x1415A5C88))[:4] == bytes.fromhex("c6 40 38 00")
assert struct.pack("<Q", pointer(0x1415A5C8C))[:3] == bytes.fromhex("89 78 24")
assert struct.pack("<Q", pointer(0x1415A5C8F))[:3] == bytes.fromhex("89 50 3c")
print("Native registration fields: release markers, invalid timer words, pending flags and existing tocker update writes pinned.")

for address in [0x14178EA97, 0x14178EF0D]:
    instruction = struct.pack("<Q", pointer(address))
    assert instruction[:4] == bytes.fromhex("66 0f 6f 05")
    assert address + 8 + struct.unpack("<i", instruction[4:])[0] == 0x142304550
for address in [0x14160E25A, 0x14160E3FA]:
    instruction = struct.pack("<Q", pointer(address))
    assert instruction[:2] == bytes.fromhex("ff 15")
    assert address + 6 + struct.unpack("<i", instruction[2:6])[0] == 0x142892A40
assert struct.pack("<Q", pointer(0x1415A5CE8)) == bytes.fromhex("41 c6 84 24 48 04 00 00")
assert struct.pack("<Q", pointer(0x1415A5CF0))[:1] == b"\x00"
print("Native registration insertion/cleanup: fresh invalid timers, both zero-wait mutex calls and tocker pending-flag clear pinned.")

for slot, name in [(0x668, "_ZNK6DaAtom20DoesQueryEnvironmentEv"),
                   (0x470, "_ZNK6DaAtom22FindParentCapsuleInEnvEv")]:
    assert pointer(symbols["_ZTV6DaAtom"] + 16 + slot) == symbols[name]
for address, expected in [
    (0x140B3FDF6, "8b 41 1c"),
    (0x140B3FDFC, "81 fa fa ff ff 7f"),
    (0x140B3FFD2, "83 78 1c 02"),
    (0x140B4000C, "a8 20"),
    (0x140B402C0, "a8 08"),
    (0x140B40032, "ff 90 88 0b 00 00"),
    (0x140B40038, "83 f0 01"),
]:
    raw = bytes.fromhex(expected)
    assert struct.pack("<Q", pointer(address))[:len(raw)] == raw
print("Native MustNotTickTock: raw status, parent exclusion, query flags and capsule virtual-result inversion pinned.")

assert symbols["_ZN7AuEther10AddRunawayERK4AuIdS2_"] == 0x1408C4D10
for address, expected in [
    (0x1408C4D27, "48 89 d7"), # first ID argument retained as key
    (0x1408C4D2A, "4c 89 c5"), # second ID argument retained as value
    (0x1408C4DCA, "89 50 0c"), # insert key
    (0x1408C4DD0, "89 50 10"), # insert value
    (0x1408C4E5F, "39 45 00"), # exact value duplicate check
    (0x1408C4F52, "e9 59 fe ff ff"), # same key still allocates a new node
]:
    raw = bytes.fromhex(expected)
    assert struct.pack("<Q", pointer(address))[:len(raw)] == raw
print("Native AddRunaway: parent/child pair insertion, duplicate comparison and multi-value allocation pinned.")

for opcode, cls in native_classes.items():
    predicate = pointer(symbols[f"_ZTV{len(cls)}{cls}"] + 16 + 0x618)
    code = struct.pack("<Q", pointer(predicate))
    value = False if code[:3] == b"\x31\xc0\xc3" else True if code[:6] == b"\xb8\x01\x00\x00\x00\xc3" else None
    assert value == (opcode in (0x15, 0x1A, 0x1C, 0x20)), (opcode, cls, predicate, value)
print("Native AddAtom routing: all 23 InStarField bindings and constant returns verified statically.")

# Independent GNU byte observations, not a REA result or runtime oracle. These
# pin the normal retirement-selection branch while full cleanup is reviewed.
assert symbols["_ZN7AuEther15DeleteTrashAtomEv"] == 0x1408CD9C0
assert symbols["_ZN8AuGlobal15InTickTockQueueERK4AuId"] == 0x1415A69D0
for address, expected in [
    (0x1408CDA8B, "49 29 c6"),  # reference-millisecond subtraction
    (0x1408CDA8E, "49 81 fe 1f 4e 00 00"),  # compare with 19999ms
    (0x1408CDA95, "7e 2c"),  # signed <= skips the liveness query
    (0x1408CDAA9, "e8 22 8f cd 00"),  # InTickTockQueue
    (0x1408CDAB0, "0f 84 92 00 00 00"),  # false branches to selection
    (0x1408CDABF, "0f 11 4d 18"),  # queue-present resets stored epoch
    (0x1408CDB54, "e8 e7 17 f3 00"),  # erase selected retention entry
    (0x1408CDB9D, "ff 50 20"),  # normal non-instrumented virtual destruction
]:
    raw = bytes.fromhex(expected)
    assert struct.pack("<Q", pointer(address))[:len(raw)] == raw
print("Native retirement selection: signed 20000ms gate, queue-protected timer restart and normal erase/destruction anchors pinned.")

for address, expected in [
    (0x1415A69FA, "48 8b be 38 04 00 00"),  # ticker table
    (0x1415A6A53, "48 39 df"),  # found iterator vs end, not record fields
    (0x1415A6A56, "74 78"),  # ticker absence proceeds to tocker table
    (0x1415A6ADD, "48 8b be 50 04 00 00"),  # tocker table
    (0x1415A6B2E, "41 0f 95 c6"),  # key-presence boolean
]:
    raw = bytes.fromhex(expected)
    assert struct.pack("<Q", pointer(address))[:len(raw)] == raw
print("Native InTickTockQueue: ticker-first/tocker-second key-presence return anchors pinned; release flags are not read in the reviewed body.")

assert symbols["_ZN6DaAtomC2Ev"] == 0x14184A920
assert pointer(0x14224BE08) == 5 << 32  # qword store at 0x18 sets raw 0x1c=5
assert pointer(0x14224BCC0) == struct.unpack("<Q", struct.pack("<d", 1.0))[0]
assert pointer(0x14224BD60) == pointer(0x14224BD68) == 1 << 63
for address, expected in [
    (0x14184A955, "41 c7 44 24 14 00 00 00"),  # initial null wire ID
    (0x14184AADB, "41 c7 84 24 10 02 00 00"),  # null position parent
    (0x14184ABCA, "49 89 44 24 18"),  # raw status constant store
    (0x14184AC03, "49 c7 84 24 50 03 00 00"),  # null owner pointer
    (0x14184AB60, "41 0f 11 bc 24 20 03 00"),  # invalid movement timer words
]:
    raw = bytes.fromhex(expected)
    assert struct.pack("<Q", pointer(address))[:len(raw)] == raw
print("Native base constructor: raw status 5, null identity/parent/owner, identity scalar and invalid timer constants pinned; derived defaults remain separate.")
