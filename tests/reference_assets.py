"""Check the derived HAR reader against the supplied sound archive offline.

Run after cargo build: python3 tests/reference_assets.py
"""
from __future__ import annotations

import ast
from pathlib import Path
import re
import struct
import subprocess
import sys
import tempfile
import zlib

ROOT = Path(__file__).resolve().parents[1]
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "reference/OpenShores-server/src"))
from openshores.protocol.stream import QDS


def main() -> None:
    binary = ROOT / "target/debug/openshores-client"
    archive = ROOT / "reference/openshores-full/sfx.har"
    data = archive.read_bytes()
    index_offset = struct.unpack_from(">I", data)[0]
    length, raw_length = struct.unpack_from(">II", data, index_offset)
    assert index_offset + 4 + length == len(data)
    directory = zlib.decompress(data[index_offset + 8:])
    assert len(directory) == raw_length
    reader = QDS(directory)
    count = reader.read_u32()
    entries = {}
    for _ in range(count):
        name = reader.read_qstring()
        entries[name] = (reader.read_u32(), reader.read_u32(), reader.read_u32())
    assert reader.pos == len(directory)
    assert count == len(entries) == 112
    inspected = subprocess.check_output([binary, "inspect-har", archive], text=True)
    actual = {}
    for line in inspected.splitlines()[1:]:
        match = re.fullmatch(r'(".*"): (\d+) compressed / (\d+) raw bytes; offset=(\d+)', line)
        assert match, line
        name, compressed, raw, offset = match.groups()
        actual[ast.literal_eval(name)] = (int(compressed), int(offset), int(raw))
    assert actual == entries
    total = 0
    for name, (compressed, offset, raw) in entries.items():
        size, declared = struct.unpack_from(">II", data, offset)
        assert (size, declared) == (compressed, raw)
        payload = zlib.decompress(data[offset + 8:offset + 4 + compressed])
        assert len(payload) == raw
        total += raw
    verified = subprocess.check_output([binary, "verify-har", archive], text=True)
    assert f"Verified 112 entries; {total} decompressed bytes." in verified
    with tempfile.TemporaryDirectory(prefix="openshores-assets-", dir="/tmp") as temporary:
        for index, name in enumerate(["Chat2.wav", "GasLamp.wav", "EatingPlants.wav"]):
            compressed, offset, _ = entries[name]
            expected = zlib.decompress(data[offset + 8:offset + 4 + compressed])
            output = Path(temporary) / f"entry-{index}.wav"
            subprocess.run([binary, "extract-har", archive, name, output], check=True, capture_output=True, text=True)
            assert output.read_bytes() == expected
            assert expected[:4] == b"RIFF" and expected[8:12] == b"WAVE"
            # A repeated extraction must preserve the original output file.
            rejected = subprocess.run([binary, "extract-har", archive, name, output], capture_output=True)
            assert rejected.returncode != 0
            assert output.read_bytes() == expected
    print(f"HAR parity: all {count} directory entries and compressed blocks verified; {total} raw bytes; three byte-exact WAV extractions")


if __name__ == "__main__":
    main()
