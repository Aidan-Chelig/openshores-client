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


def atom(body, opcode=0x16, flags=0):
    return struct.pack(">BIqB", opcode, 7, 123, flags) + body


payloads = []
expected = []
for subtype in (1, 6, 7, 8, 9, 12, 17):
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
    capture.write_bytes(b"")
    empty = subprocess.check_output([client, "inspect-native-item-capture", str(capture)], text=True)
    assert json.loads(empty) == {"frames": 0, "payload_bytes": 0}
print("Native item capture: seven subtypes, ownership, opaque boundaries, raw hashes, late-error atomic output and unchanged source verified offline.")
