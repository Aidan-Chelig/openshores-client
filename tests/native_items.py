"""Static common-reader anchors + independent wire fixtures; no target execution."""
import hashlib
import json
from pathlib import Path
import struct
import subprocess

target = Path("reference/openshores-full/OpenShores.exe")
data = target.read_bytes()
assert hashlib.sha256(data).hexdigest() == "285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80"
# Establish factory type 1 -> AuItem address point -> ls virtual slot 0x28.
pe = struct.unpack_from("<I", data, 60)[0]
optional = pe + 24
optional_size = struct.unpack_from("<H", data, pe + 20)[0]
base = struct.unpack_from("<Q", data, optional + 24)[0]
sections = []
for index in range(struct.unpack_from("<H", data, pe + 6)[0]):
    header = optional + optional_size + index * 40
    rva, size, offset = struct.unpack_from("<III", data, header + 12)
    sections.append((base + rva, size, offset))


def read(address, length):
    for vma, size, offset in sections:
        if vma <= address and address + length <= vma + size:
            return data[offset + address - vma:offset + address - vma + length]
    raise AssertionError(f"Unbacked native address {address:#x}")


table = 0x1422E1338
entry = table + struct.unpack("<i", read(table + 4, 4))[0]
assert entry == 0x1414F0BE0
factory = subprocess.check_output(["objdump", "-d", f"--start-address={entry:#x}",
    f"--stop-address={entry + 120:#x}", str(target)], text=True)
assert "14247c7b0 <_ZTV6AuItem+0x10>" in factory
assert struct.unpack("<Q", read(0x14247C7B0 + 0x28, 8))[0] == 0x1414D07A0
for type_id, factory_entry, address_point, reader in (
    (6, 0x1414F0A30, 0x1423D1EE0, 0x1414D0A00),
    (7, 0x1414F09B8, 0x1423C4720, 0x1414D0A70),
    (10, 0x1414F08D0, 0x14240EE60, 0x1414D0DE0),
    (17, 0x1414F0F38, 0x142438ED0, 0x1414D0A30),
):
    assert table + struct.unpack("<i", read(table + type_id * 4, 4))[0] == factory_entry
    block = subprocess.check_output(["objdump", "-d", f"--start-address={factory_entry:#x}",
        f"--stop-address={factory_entry + 144:#x}", str(target)], text=True)
    assert f"{address_point:x} <_ZTV" in block
    assert struct.unpack("<Q", read(address_point + 0x28, 8))[0] == reader
subtypes = subprocess.check_output(["objdump", "-d", "--start-address=0x1414d0a00",
    "--stop-address=0x1414d0a75", str(target)], text=True)
for anchor in ("1414d0a0d:", "1414d0a20:", "1414d0a3d:", "1414d0a49:", "1414d0a5d:", "1414d0a70:"):
    assert anchor in subtypes, anchor
location = subprocess.check_output(["objdump", "-d",
    "--disassemble=_ZN15AuItemGalaxyLoclsER11QDataStream", str(target)], text=True)
for anchor in ("1414d07a0 <_ZN6AuItemlsER11QDataStream>",
               "0x4c(%rbx)", "1416353c0 <_ZN4AuIdlsER11QDataStream>",
               "0x50(%rbx)", "141648040 <_ZrsR11QDataStreamR7AuPoint>",
               "0x68(%rbx)", "__imp__ZrsR11QDataStreamR7QString"):
    assert anchor in location, anchor
point = subprocess.check_output(["objdump", "-d",
    "--disassemble=_ZrsR11QDataStreamR7AuPoint", str(target)], text=True)
assert "__imp__ZN11QDataStreamrsERd>" in point
assert point.count("call   *%rsi") == 3
assert "0x8(%rbx),%rdx" in point and "0x10(%rbx),%rdx" in point
for type_id, expected_entry, address_point, reader in (
    (8, 0x1414F0998, 0x1423DE1A0, 0x1414D0A80),
    (9, 0x1414F0960, 0x14241E890, 0x1414D0D50),
    (12, 0x1414F0E60, 0x14242B630, 0x1414D0DB0),
):
    entry = table + struct.unpack("<i", read(table + type_id * 4, 4))[0]
    assert entry == expected_entry
    block = subprocess.check_output(["objdump", "-d", f"--start-address={entry:#x}",
        f"--stop-address={entry + 48:#x}", str(target)], text=True)
    assert "_ZN12AuItemWeaponC1Ev" in block
    if type_id in (9, 12):
        assert f"{address_point:x} <_ZTV" in block
    assert struct.unpack("<Q", read(address_point + 0x28, 8))[0] == reader
weapon_constructor = subprocess.check_output(["objdump", "-d", "--start-address=0x1414df620",
    "--stop-address=0x1414df6ec", str(target)], text=True)
assert "1423de1a0 <_ZTV12AuItemWeapon+0x10>" in weapon_constructor
for offset in ("0x50", "0x5c", "0x64", "0x6c", "0x78"):
    assert f"movl   $0x0,{offset}(%rcx)" in weapon_constructor
weapon_reader = subprocess.check_output(["objdump", "-d", "--start-address=0x1414d0a80",
    "--stop-address=0x1414d0de0", str(target)], text=True)
for anchor in ("1414d0aa8:", "1414d0ab8:", "test   $0x1c,%al",
               "1414d0c17:", "1414d0c71:", "1414d0d1f:", "1414d0c7a:",
               "1414d0d5e:", "1414d0d74:", "1414d0d80:", "1414d0d8c:",
               "1414d0da3:", "1414d0dbd:", "1414d0dd3:"):
    assert anchor in weapon_reader, anchor
body = subprocess.check_output(["objdump", "-d", "--start-address=0x140b7ad70",
    "--stop-address=0x140b7ae3d", str(target)], text=True)
for anchor in ("140b7adb9:", "1414f05e0 <_ZN6AuItem6CreateEh>",
               "call   *0x28(%rax)", "140b7ade3:", "140b7adf3:",
               "140b7ae12:", "140b7ae28:", "140b7ae34:"):
    assert anchor in body, anchor
assembly = subprocess.check_output([
    "objdump", "-d", "--start-address=0x1414d07a0",
    "--stop-address=0x1414d0960", str(target)], text=True)
for anchor in (
    "1414d07c4:", "1414d07cd:", "1414d0805:", "1414d0815:",
    "1414d0822:", "1414d082a:", "1414d089c:", "1414d08ef:",
    "1414d0907:", "1414d0927:", "1414d0947:",
    "__imp__ZN11QDataStreamrsERs", "__imp__ZN11QDataStreamrsERi",
    "__imp__ZrsR11QDataStreamR7QString", "DbCommodity10IsObsolete",
):
    assert anchor in assembly, anchor
assert assembly.count("test   $0x1,%al") == 1
assert assembly.count("test   $0x2,%al") == 1
assert assembly.count("test   $0x4,%al") == 1
assert assembly.count("testb  $0x8,") == 1
assert assembly.count("test   $0x10,%al") == 1
assert assembly.count("test   $0x20,%al") == 1

client = "target/debug/openshores-client"
for flags in range(256):
    wire = struct.pack(">Bh", flags, -123)
    if flags & 1:
        wire += struct.pack(">i", -2147483648)
    if flags & 2:
        wire += struct.pack(">i", -321)
    if flags & 4:
        wire += b"\xfe"
    if flags & 8:
        name = 'quoted " item 🚀'.encode("utf-16-be")
        wire += struct.pack(">I", len(name)) + name
    wire += b"\x00"
    if flags & 16:
        wire += struct.pack(">i", 2147483647)
    if flags & 32:
        wire += b"\x81"
    result = json.loads(subprocess.check_output([
        client, "inspect-native-item-common", (wire + b"\xde\xad").hex()], text=True))
    assert result == {
        "flags": flags, "commodity": -123,
        "field_0c": -2147483648 if flags & 1 else 1,
        "field_10": -321 if flags & 2 else 0,
        "field_14": 254 if flags & 4 else None,
        "name_present": bool(flags & 8),
        "name_utf8_hex": 'quoted " item 🚀'.encode().hex() if flags & 8 else None,
        "wire_field_20": 0, "field_20": 1,
        "field_24": 2147483647 if flags & 16 else 0,
        "field_28": 129 if flags & 32 else 0,
        "consumed_bytes": len(wire), "tail_hex": "dead",
    }, (flags, result)

for wire in ("", "000001", "080001000000010000", "08000100000002d80001", "00" * 4097):
    result = subprocess.run([client, "inspect-native-item-common", wire], capture_output=True)
    assert result.returncode != 0 and result.stdout == b"", wire[:32]
for type_id in (1, 6, 7, 17):
    for state in (0, 128, 255):
        for message in (None, "", "native message 🚀"):
            wire = bytes([type_id]) + struct.pack(">BhB", 0, 1, 0)
            extension = {}
            if type_id in (6, 17):
                wire += bytes([state])
                extension["field_4c"] = state
            if type_id == 17:
                raw = message.encode("utf-16-be") if message is not None else None
                wire += struct.pack(">I", len(raw) if raw is not None else 0xFFFFFFFF)
                if raw is not None:
                    wire += raw
                extension["field_50_utf8_hex"] = message.encode().hex() if message is not None else None
            result = json.loads(subprocess.check_output([client, "inspect-native-item-value",
                (wire + b"\xde\xad").hex()], text=True))
            assert result["type_id"] == type_id and result["extension"] == extension
            assert result["consumed_bytes"] == len(wire) and result["tail_hex"] == "dead"
            assert result["common"]["commodity"] == 1 and result["common"]["field_20"] == 1
for message in (None, "", "location 🚀"):
    bits = (0x8000000000000000, 0x7FF8000000001234, 0x3FF4000000000000)
    raw = message.encode("utf-16-be") if message is not None else None
    wire = bytes([10]) + struct.pack(">BhBI3Q", 0, 3, 1, 0xFFFFFFFF, *bits)
    wire += struct.pack(">I", len(raw) if raw is not None else 0xFFFFFFFF)
    if raw is not None:
        wire += raw
    result = json.loads(subprocess.check_output([client, "inspect-native-item-value",
        (wire + b"\xde\xad").hex()], text=True))
    assert result["extension"] == {
        "id_4c": 0xFFFFFFFF,
        "point_50_bits_hex": [f"{value:016x}" for value in bits],
        "string_68_utf8_hex": message.encode().hex() if message is not None else None,
    }
    assert result["type_id"] == 10
    assert result["consumed_bytes"] == len(wire) and result["tail_hex"] == "dead"
    for end in range(len(wire)):
        truncated = subprocess.run([client, "inspect-native-item-value", wire[:end].hex()],
            capture_output=True)
        assert truncated.returncode != 0 and truncated.stdout == b"", end
for type_id in set(range(256)) - {1, 6, 7, 8, 9, 10, 12, 17}:
    wire = bytes([type_id, 0xde, 0xad])
    result = json.loads(subprocess.check_output([client, "inspect-native-item-value", wire.hex()], text=True))
    assert result == {"type_id": None, "payload_hex": wire.hex()}
print("Native common item: static reader anchors, 256 independent fixtures, and malformed input verified offline.")
print("Native simple/location items: five factory/vtable bindings, 39 independent value fixtures, location truncations, and 248 opaque types verified.")
for flags in range(256):
    wire = struct.pack(">Bh", flags, -123)
    groups = []
    # Independent expected values and explicit native-storage destination model.
    storage = bytearray(0x80)
    struct.pack_into("<h", storage, 0x4c, -123)
    for index in range(4):
        if index == 2 and flags & 0x1c:
            wire += struct.pack(">h", -321)
            struct.pack_into("<h", storage, 0x66, -321)
        if not flags & (1 << index):
            groups.append(None)
            continue
        group = [128 + index, -100 - index, 0xf100 + index if flags & 32 else 224 + index,
                 -200 - index, 144 + index, -300 - index]
        groups.append(group)
        wire += struct.pack(">Bh", *group[:2])
        wire += struct.pack(">H" if flags & 32 else ">B", group[2])
        wire += struct.pack(">hBh", *group[3:])
        destinations = ((0x4e, 0x50, 0x52, 0x54, 0x56, 0x58),
                        (0x5a, 0x5c, 0x5e, 0x60, 0x62, 0x64),
                        (0x68, 0x6a, 0x6c, 0x6e, 0x70, 0x72),
                        (0x74, 0x76, 0x6c, 0x7a, 0x7c, 0x7e))[index]
        for offset, value, kind in zip(destinations, group, ("B", "h", "H", "h", "B", "h")):
            struct.pack_into("<" + kind, storage, offset, value)
    for type_id, suffix in ((8, b""), (9, b"\x00\x80\xff\x01"), (12, b"\xfe")):
        value_wire = bytes([type_id]) + struct.pack(">BhB", 0, 1, 0) + wire + suffix
        result = json.loads(subprocess.check_output([client, "inspect-native-item-value",
            (value_wire + b"\xde\xad").hex()], text=True))
        extension = result["extension"]
        weapon = extension if type_id == 8 else extension["weapon"]
        assert weapon == {
            "flags": flags, "field_4c": -123,
            "wire_field_66": -321 if flags & 0x1c else None,
            "groups": groups, "consumed_bytes": len(wire),
            **{f"fresh_field_{offset:02x}": struct.unpack_from("<I", storage, offset)[0]
               for offset in (0x50, 0x5c, 0x6c, 0x78)},
        }, (type_id, flags, weapon)
        if type_id == 9:
            assert extension["fields_80_hex"] == suffix.hex()
        if type_id == 12:
            assert extension["field_80"] == 254
        assert result["type_id"] == type_id
        assert result["consumed_bytes"] == len(value_wire) and result["tail_hex"] == "dead"
print("Native weapons: three factory/vtable bindings, 768 independent fixtures, width switches and overlapping destination writes verified offline.")
for wire in (bytes([8, 0, 0, 1, 0, 0]), bytes([8, 0, 0, 1, 0, 16, 0, 1]),
             bytes([9, 0, 0, 1, 0, 0, 0, 1]), bytes([12, 0, 0, 1, 0, 0, 0, 1])):
    result = subprocess.run([client, "inspect-native-item-value", wire.hex()], capture_output=True)
    assert result.returncode != 0 and result.stdout == b"", wire.hex()
