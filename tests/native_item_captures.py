"""Independent framed native-item replay fixtures; no network or target execution."""
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import tempfile

client = "target/debug/openshores-client"


def frame(payload):
    length = len(payload)
    count = next(count for count in range(1, 5) if length < 1 << (count * 8 - 2))
    header = bytearray(length.to_bytes(count, "big"))
    header[0] |= (count - 1) << 6
    return bytes(header) + payload


def atom(body, opcode=0x16, flags=0, entity=7):
    return struct.pack(">BIqB", opcode, entity, 123, flags) + body


payloads = []
expected = []
for subtype in (1, 6, 7, 8, 9, 10, 12, 17):
    value = struct.pack(">BBhB", subtype, 0, 42, 1)
    if subtype in (6, 17):
        value += b"\x80"
    if subtype == 17:
        value += struct.pack(">I", 0xFFFFFFFF)
    if subtype in (8, 9, 12):
        value += struct.pack(">Bh", 0, -123)
    if subtype == 9:
        value += b"\x00\x80\xff\x01"
    if subtype == 12:
        value += b"\xfe"
    if subtype == 10:
        value += struct.pack(">I3dI", 88, 1.25, -0.0, -99.5, 0xFFFFFFFF)
    body = b"\x81" + value + struct.pack(">III", 99, 0xFFFFFFFF, 0)
    payloads.append(atom(body + b"\xde\xad"))
    expected.append({
        "flags": 0x81, "value": {"type_id": subtype, "commodity": 42, "common_flags": 0},
        "ownership": {"kind": "explicit", "id": 99}, "field_39c": False,
        "consumed_bytes": len(body), "remaining_bytes": 2,
    })
payloads.append(atom(b"\x40\xde\xad"))
expected.append({"flags": 64, "value": None, "ownership": {"kind": "sentinel", "id": 1},
                 "field_39c": False, "consumed_bytes": 1, "remaining_bytes": 2})
payloads += [atom(b"\x01\x04\xde\xad"), atom(b"\x01\x06", flags=16),
             atom(b"\x01\x06", flags=64), atom(b"\x01\x06", opcode=5), b"\xff\x01\x02"]
expected += [None] * 5

with tempfile.TemporaryDirectory(prefix="openshores-native-items-") as directory:
    capture = Path(directory) / "scene.bin"
    raw = b"".join(map(frame, payloads))
    capture.write_bytes(raw)
    result = subprocess.run([client, "inspect-native-item-capture", str(capture)],
                            capture_output=True, text=True, check=True)
    rows = list(map(json.loads, result.stdout.splitlines()))
    assert rows[0] == {"frames": len(payloads), "payload_bytes": sum(map(len, payloads))}
    for ordinal, (payload, expected_item, row) in enumerate(zip(payloads, expected, rows[1:])):
        assert row["frame"] == ordinal and row["opcode"] == payload[0]
        assert row["payload_sha1"] == hashlib.sha1(payload).hexdigest()
        assert row["item"] == expected_item, (ordinal, row)
    assert len(rows) == len(payloads) + 1
    assert capture.read_bytes() == raw
    # Prefix-only inspection deliberately continues to accept opaque item bodies.
    malformed = raw + frame(atom(b"\x01\x06"))
    capture.write_bytes(malformed)
    prefix = subprocess.run([client, "inspect-native-capture", str(capture)],
                            capture_output=True, check=True)
    assert len(prefix.stdout.splitlines()) == len(payloads) + 2
    failed = subprocess.run([client, "inspect-native-item-capture", str(capture)], capture_output=True)
    assert failed.returncode != 0 and failed.stdout == b""
    assert capture.read_bytes() == malformed
    failed_replay = subprocess.run([client, "replay-native-items", str(capture)], capture_output=True)
    assert failed_replay.returncode != 0 and failed_replay.stdout == b""
    # Capture-order observations: replacement, omission, invalidation/recovery.
    sequence = [
        atom(b"\x01\x06\x08\x00\x01" + struct.pack(">I", 2) + b"\x00A\x01\x80", entity=7),
        atom(b"\xc0" + struct.pack(">III", 77, 0xFFFFFFFF, 0), entity=7),
        atom(b"\x40", entity=7),
        atom(b"\x01\x01\x00\x00\x02\x00", entity=7),
        atom(b"\x01\x01\x00\x00\x03\x01", entity=8),
        atom(b"\x01\x04\xde\xad", entity=8),
        atom(b"\x20", entity=8),
        atom(b"\xde\xad", flags=16, entity=9),
        atom(b"\x01\x01\x00\x00\x04\x01", entity=10),
        atom(b"\xde\xad", opcode=5, entity=10),
    ]
    state_raw = b"".join(map(frame, sequence))
    capture.write_bytes(state_raw)
    state_rows = list(map(json.loads, subprocess.check_output(
        [client, "replay-native-items", str(capture)], text=True).splitlines()))
    assert state_rows == [
        {"item_entities": 3, "decoded_updates": 7, "opaque_updates": 2},
        {"id": 7, "item": {"type_id": 1, "commodity": 2, "name_present": False},
         "ownership_id": 1, "field_3e0_known": True, "field_3e8_known": True,
         "field_39c": False, "opaque_updates": 0},
        {"id": 8, "item": None, "ownership_id": None,
         "field_3e0_known": False, "field_3e8_known": False,
         "field_39c": True, "opaque_updates": 1},
        {"id": 9, "item": None, "ownership_id": None,
         "field_3e0_known": False, "field_3e8_known": False,
         "field_39c": None, "opaque_updates": 1},
    ], state_rows
    assert capture.read_bytes() == state_raw
    capture.write_bytes(b"")
    empty = subprocess.check_output([client, "inspect-native-item-capture", str(capture)], text=True)
    assert json.loads(empty) == {"frames": 0, "payload_bytes": 0}
    empty_replay = subprocess.check_output([client, "replay-native-items", str(capture)], text=True)
    assert json.loads(empty_replay) == {"item_entities": 0, "decoded_updates": 0, "opaque_updates": 0}
print("Native item capture: eight subtypes, ownership, opaque boundaries, raw hashes, late-error atomic output and unchanged source verified offline.")
print("Retained item observations: capture-order replacement/omission, ownership retention, opaque invalidation and sparse recovery verified offline.")
