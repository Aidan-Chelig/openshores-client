"""Digest-bound native DP dimension tables/constants, without target execution."""
from pathlib import Path
import re
import runpy
import struct

inventory = runpy.run_path(str(Path(__file__).with_name("native_chromosome_volumes.py")))
read = inventory["read"]
source = Path("src/world/native_body/dp/layout.rs").read_text()
table = source.split("const SIZE_BITS:", 1)[1].split("=", 1)[1].split("];", 1)[0]
bits = [int(value, 16) for value in re.findall(r"0x[0-9a-fA-F_]+", table)]
assert len(bits) == 32
assert struct.unpack("<32I", read(0x1422BFB40, 128)) == tuple(bits)
assert struct.unpack("<4f", read(0x1422BF890, 16)) == tuple(
    struct.unpack("<f", struct.pack("<f", value))[0] for value in [1, .8, .7, .6])
assert struct.unpack("<4f", read(0x1422BF8B0, 16)) == tuple(
    struct.unpack("<f", struct.pack("<f", value))[0] for value in [1, .9, .8, .7])
for address, expected, width in [
    (0x1422BF8A0, .5, 4), (0x1422BF89C, .6, 4),
    (0x1422BFBC4, .0625, 4), (0x1422BFBC8, 1.3, 8),
    (0x1422BFBD0, .75, 4), (0x1422BFB0C, .25, 4),
    (0x1422BF8E8, 3, 4),
]:
    fmt = "<f" if width == 4 else "<d"
    assert read(address, width) == struct.pack(fmt, expected), hex(address)
print("Verified32 literal DP size entries and dimension arithmetic constants against native PE; no model/runtime execution.")
