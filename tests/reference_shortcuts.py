"""Read default-person shortcut construction from supplied PE; never execute it."""
import hashlib
from pathlib import Path
import re
import struct
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def defaults():
    path = ROOT / "reference/openshores-full/OpenShores.exe"
    data = path.read_bytes()
    assert hashlib.sha256(data).hexdigest() == "285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80", "shortcut addresses belong only to the supplied client build"
    pe = struct.unpack_from("<I", data, 0x3c)[0]
    count, optional_size = struct.unpack_from("<H", data, pe + 6)[0], struct.unpack_from("<H", data, pe + 20)[0]
    image_base = struct.unpack_from("<Q", data, pe + 24 + 24)[0]
    sections = []
    for index in range(count):
        offset = pe + 24 + optional_size + index * 40
        _, rva, size, raw = struct.unpack_from("<IIII", data, offset + 8)
        sections.append((image_base + rva, size, raw))
    def string(address):
        for start, size, raw in sections:
            if start <= address < start + size:
                offset = raw + address - start
                end = data.index(0, offset, min(offset + 1024, raw + size))
                return data[offset:end].decode("utf-8")
        raise ValueError("string outside initialized PE section")
    # The symbol-bearing build's static initializer, established from references
    # to s_PersonDefaults and its RestoreDefaultControls loop (stride 0x78).
    asm = subprocess.run(["objdump", "-d", "--start-address=0x141c67f50", "--stop-address=0x141c6bca3", str(path)],
                         check=True, capture_output=True, text=True, timeout=30).stdout
    rows = []
    label = key = action = modifiers = None
    for line in asm.splitlines():
        match = re.search(r"lea .*%r8\s*# ([0-9a-f]+)", line)
        if match:
            label = string(int(match[1], 16))
        match = re.search(r"mov\s+\$0x([0-9a-f]+),%r9d", line)
        if match:
            key = int(match[1], 16)
        match = re.search(r"mov\s+\$0x([0-9a-f]+),%r8d", line)
        if match:
            action = int(match[1], 16)
        match = re.search(r"movl\s+\$0x([0-9a-f]+),0x20\(%rsp\)", line)
        if match:
            modifiers = int(match[1], 16)
        if "call" in line and "<" in line and "DiControlInputC1ERK7QStringhN2Qt3Key" in line:
            assert None not in (label, key, action, modifiers)
            rows.append((label, action, key, modifiers))
    # Only this constructor overload is covered; other overloads are not guessed.
    assert len(rows) == 67, len(rows)
    by_label = {row[0]: row[1:] for row in rows}
    for label, action, key in [("Move Forward", 0x84, 0x57), ("Move Backward", 0x82, 0x53),
                              ("Move Left", 0x85, 0x41), ("Move Right", 0x86, 0x44),
                              ("Move Up", 0x87, 0x43), ("Move Down", 0x83, 0x5a),
                              ("Use Item", 0x9c, 0x55), ("Drop Item", 0xaf, 0x01ffffff)]:
        assert by_label[label] == (action, key, 0), (label, by_label[label])
    return hashlib.sha256(data).hexdigest(), rows


if __name__ == "__main__":
    digest, rows = defaults()
    print("Source SHA-256:", digest)
    for label, action, key, modifiers in rows:
        print(f"{label}: action={action:#04x} QtKey={key:#010x} modifiers={modifiers:#010x}")
