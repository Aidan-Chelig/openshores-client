"""Offline Qt pool parity against the supplied DLL; requires cargo build and objdump.

COFF symbols and section offsets come from objdump rather than the Rust parser.
No library is loaded/executed and no reference assets are modified.
"""
from __future__ import annotations

import ast
from collections import Counter
from pathlib import Path
import re
import struct
import subprocess
import tempfile
import zlib
import sys
sys.dont_write_bytecode = True
from reference_geometry import check_geometry
from reference_preview import check_previews

ROOT = Path(__file__).resolve().parents[1]


def main() -> None:
    binary = ROOT / "target/debug/openshores-client"
    dll = ROOT / "reference/openshores-full/libAuModelRes.dll"
    data = dll.read_bytes()
    sections = {}
    headers = subprocess.check_output(["objdump", "-h", dll], text=True)
    for line in headers.splitlines():
        match = re.match(r"\s*(\d+)\s+\S+\s+([0-9a-f]+)\s+[0-9a-f]+\s+[0-9a-f]+\s+([0-9a-f]+)", line)
        if match:
            number, size, offset = match.groups()
            sections[int(number) + 1] = (int(offset, 16), int(size, 16))
    symbols = subprocess.check_output(["objdump", "-t", dll], text=True)
    arrays = []
    for line in symbols.splitlines():
        match = re.search(r"\(sec\s+(\d+)\).*?\s0x([0-9a-f]+)\s+_ZL\d+qt_resource_(data|name|struct)$", line)
        if match:
            section, relative, kind = match.groups()
            base, size = sections[int(section)]
            arrays.append((kind, base + int(relative, 16), base + size))
    assert len(arrays) == 30
    expected = []
    payloads = []
    for pool in range(10):
        payload, names, tree = arrays[pool * 3:pool * 3 + 3]
        assert [payload[0], names[0], tree[0]] == ["data", "name", "struct"]
        queue = [(0, ":")]
        while queue:
            node, parent = queue.pop(0)
            pos = tree[1] + node * 22
            name_offset, flags = struct.unpack_from(">IH", data, pos)
            if node:
                name_pos = names[1] + name_offset
                length = struct.unpack_from(">H", data, name_pos)[0]
                name = data[name_pos + 6:name_pos + 6 + length * 2].decode("utf-16-be")
                path = parent + "/" + name
            else:
                path = parent
            if flags & 2:
                count, first = struct.unpack_from(">II", data, pos + 6)
                queue.extend((child, path) for child in range(first, first + count))
                continue
            country, language, offset = struct.unpack_from(">HHI", data, pos + 6)
            start = payload[1] + offset
            stored = struct.unpack_from(">I", data, start)[0]
            body = data[start + 4:start + 4 + stored]
            assert len(body) == stored
            assert flags in (0, 1)
            if flags:
                raw = struct.unpack_from(">I", body)[0]
                decoder = zlib.decompressobj()
                output = decoder.decompress(body[4:]) + decoder.flush()
                assert decoder.eof and not decoder.unused_data and not decoder.unconsumed_tail
                assert len(output) == raw
            else:
                output = body
                raw = stored
            expected.append((pool, path, country, language, flags, stored, raw, start + 4))
            payloads.append(output)
    assert len(expected) == 446
    inspected = subprocess.check_output([binary, "inspect-qt", dll, "3"], text=True)
    lines = inspected.splitlines()
    assert lines[0] == "Qt: 10 pools; 446 entries."
    for index, (line, fields) in enumerate(zip(lines[1:], expected, strict=True)):
        match = re.fullmatch(r'(\d+): pool=(\d+) (".*"); locale=(\d+)/(\d+); flags=(\d+); stored=(\d+); raw=(\d+); offset=(\d+)', line)
        assert match, line
        values = match.groups()
        actual = (int(values[1]), ast.literal_eval(values[2]), *(int(value) for value in values[3:]))
        assert int(values[0]) == index and actual == fields, (actual, fields)
    total = sum(len(payload) for payload in payloads)
    verified = subprocess.check_output([binary, "verify-qt", dll, "3"], text=True)
    assert f"Verified 446 entries; {total} raw bytes." in verified
    with tempfile.TemporaryDirectory(prefix="openshores-models-") as temp:
        for extension in (".png", ".bmp", ".3ds", ".3dsN", ".3dsX", ".SoH"):
            index = next(i for i, entry in enumerate(expected) if entry[1].endswith(extension))
            entry = expected[index]
            output = Path(temp) / ("asset" + extension)
            command = [binary, "extract-qt", dll, "3", str(entry[0]), str(index), output]
            subprocess.run(command, check=True, capture_output=True)
            assert output.read_bytes() == payloads[index]
            if extension == ".png":
                assert payloads[index].startswith(b"\x89PNG\r\n\x1a\n")
            elif extension == ".bmp":
                assert payloads[index].startswith(b"BM")
            repeated = subprocess.run(command, capture_output=True)
            assert repeated.returncode != 0
            assert output.read_bytes() == payloads[index]
            wrong_pool = command.copy()
            wrong_pool[4] = str(entry[0] + 1)
            wrong_pool[-1] = Path(temp) / "must-not-exist"
            assert subprocess.run(wrong_pool, capture_output=True).returncode != 0
            assert not wrong_pool[-1].exists()
    formats = Counter(Path(entry[1]).suffix for entry in expected)
    check_geometry(binary, dll, expected, payloads)
    check_previews(binary, expected, payloads)
    print(f"Qt parity: 10 pools, 446 entries, {total} raw bytes; six byte-exact extractions; overwrite and pool mismatch rejected")
    print(f"Asset extensions: {dict(sorted(formats.items()))}")


if __name__ == "__main__":
    main()
