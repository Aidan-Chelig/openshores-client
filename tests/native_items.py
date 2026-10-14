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
for type_id in set(range(256)) - {1, 6, 7, 17}:
    wire = bytes([type_id, 0xde, 0xad])
    result = json.loads(subprocess.check_output([client, "inspect-native-item-value", wire.hex()], text=True))
    assert result == {"type_id": None, "payload_hex": wire.hex()}
print("Native common item: static reader anchors, 256 independent fixtures, and malformed input verified offline.")
print("Native item subtypes: four factory/vtable bindings, 36 independent value fixtures, and 252 opaque types verified.")
