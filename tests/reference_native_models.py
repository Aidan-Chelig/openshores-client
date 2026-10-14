"""Pin static model-loader observations to the supplied patched executable.

GNU disassembly/file-byte checks only: not REA Evidence or runtime observation.
No original code is executed. NT has synthetic codec coverage, but no supplied
real NT resource is claimed by this check.
"""

import hashlib
from pathlib import Path
import struct
import subprocess
import tempfile

from reference_geometry import geometry
from model_coordinates import check_coordinates
from model_roles import check_roles


TARGET = Path("reference/openshores-full/OpenShores.exe")
if not TARGET.exists():
    raise SystemExit("Supplied patched executable is required for native model checks")
image = TARGET.read_bytes()
assert hashlib.sha256(image).hexdigest() == "285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80"
pe = struct.unpack_from("<I", image, 0x3C)[0]
assert image[pe:pe + 4] == b"PE\0\0"
count = struct.unpack_from("<H", image, pe + 6)[0]
optional_size = struct.unpack_from("<H", image, pe + 20)[0]
optional = pe + 24
assert struct.unpack_from("<H", image, optional)[0] == 0x20B
base = struct.unpack_from("<Q", image, optional + 24)[0]
sections = optional + optional_size


def raw(address, length):
    rva = address - base
    for index in range(count):
        entry = sections + index * 40
        section_rva, size, offset = struct.unpack_from("<III", image, entry + 12)
        if section_rva <= rva and rva + length <= section_rva + size:
            start = offset + rva - section_rva
            assert start + length <= len(image)
            return image[start:start + length]
    raise AssertionError(f"Unbacked address {address:#x}")


def assembly(start, end):
    return subprocess.check_output(
        ["objdump", "-d", f"--start-address={start:#x}",
         f"--stop-address={end:#x}", str(TARGET)], text=True,
    )


def direct_call(address):
    instruction = raw(address, 5)
    assert instruction[0] == 0xE8
    return address + 5 + struct.unpack("<i", instruction[1:])[0]


assert raw(0x1422CF22D, 6) == b".3dsX\0"
assert raw(0x1422CF233, 7) == b".3dsNT\0"
assert raw(0x1422CF23A, 6) == b".3dsN\0"
# MODEL_TYPE minus one admits N/NT into the common nine-float reader.
assert raw(0x1413A3ECB, 3) == bytes.fromhex("83 ee 01")
assert raw(0x1413A40B4, 5) == bytes.fromhex("83 7c 24 50 01")
assert raw(0x1413A41F8, 8) == bytes.fromhex("83 bc 24 b8 01 00 00 02")
float_symbol = "<_ZN9Au3dsGLSL9LoadFloatER11QDataStream>"
assert assembly(0x1413A4158, 0x1413A41F3).count(float_symbol) == 9
assert assembly(0x1413A43D0, 0x1413A4490).count(float_symbol) == 10
# X matrix handling scales existing vertex coordinates with a binary64 1/12.
assert raw(0x14139C44E, 4) == bytes.fromhex("41 83 f9 03")
assert struct.unpack("<d", raw(0x1422CF2B0, 8))[0] == 1.0 / 12.0
assert struct.unpack("<d", raw(0x1422CF250, 8))[0] == 1.0
# Default forward matrix has identity diagonal and inverse-valid flag is clear.
assert raw(0x141788DA8, 4) == bytes.fromhex("f2 0f 11 33")
for address, offset in ((0x141788DAC, 0x28), (0x141788DB1, 0x50), (0x141788DB6, 0x78)):
    assert raw(address, 5) == bytes.fromhex("f2 0f 11 73") + bytes([offset])
# Distinct local-frame and forward-frame Project calls; not a single raw-matrix
# application. Exact argument/state interpretation is recorded in the ledger.
for address in (0x14139CBBB, 0x14139B7AB, 0x14139E490, 0x14139E4CC, 0x14139E508):
    assert direct_call(address) == 0x1418FD760
assert direct_call(0x1413A7232) == 0x1417A5A80  # VertexArray::ScaleBy(double)
# Helper predicates, custom group framing, and their caller's removal path.
assert raw(0x141398067, 4) == bytes.fromhex("66 83 38 21")
assert raw(0x1413982DB, 4) == bytes.fromhex("66 83 38 40")
assert raw(0x141398381, 6) == bytes.fromhex("41 b8 10 00 00 00")
assert raw(0x1422CF1BC, 9) == b"UpVector\0"
assert raw(0x1422CF0C0, 8) == b"Screen1\0"
assert raw(0x1422CF0C8, 8) == b"Screen2\0"
assert raw(0x14139A1DB, 5) == bytes.fromhex("66 81 fd 13 41")
assert raw(0x14139A1E2, 5) == bytes.fromhex("66 81 fd 14 41")
assert assembly(0x14139A090, 0x14139A0B9).count(float_symbol) == 4
for address, target in ((0x1413A6D2D, 0x141398280), (0x1413A6F12, 0x141398020),
                        (0x1413A797E, 0x141398530), (0x1413A7A6E, 0x14139ABE0)):
    assert direct_call(address) == target
assert direct_call(0x1413A6EF6) == 0x141855600
assert direct_call(0x1413A7AAE) == 0x141855600
print("Patched native model anchors: N/NT nine-float + NT ten-float layout; X matrix scale constant 1/12. Static evidence only.")


def chunk(kind, body):
    return struct.pack("<HI", kind, len(body) + 6) + body


# Independent little-endian NT fixture, including child records after its
# 84-byte face. Only the Rust replacement is executed for this codec check.
vertices = struct.pack("<H9f", 3, 0, 0, 0, 1, 0, 0, 0, 1, 0)
face = struct.pack("<4H19f", 0, 1, 2, 7, *range(19))
faces = struct.pack("<H", 1) + face
faces += chunk(0x4150, struct.pack("<I", 42))
faces += chunk(0x4130, b"mat\0" + struct.pack("<HH", 1, 0))
mesh = chunk(0x4110, vertices) + chunk(0x4120, faces)
fixture = chunk(0x4D4D, chunk(0x3D3D, chunk(0x4000, b"triangle\0" + chunk(0x4100, mesh))))
expected = geometry(fixture, ".3dsNT")
assert expected[:3] == (1, 3, 1)
with tempfile.TemporaryDirectory(prefix="openshores-native-models-") as directory:
    path = Path(directory) / "triangle.3dsNT"
    path.write_bytes(fixture)
    output = subprocess.check_output(["target/debug/openshores-client", "inspect-model", str(path)], text=True)
    assert "Model: NT;" in output and f"geometry={expected[3]}" in output
    assert "smoothing=true; material groups=1" in output
    for extension in (".3ds", ".3dsN", ".3dsX"):
        wrong = Path(directory) / ("wrong" + extension)
        wrong.write_bytes(fixture)
        result = subprocess.run(["target/debug/openshores-client", "inspect-model", str(wrong)], capture_output=True)
        assert result.returncode != 0
print("Synthetic NT parity: 84-byte face, both float extensions, digest and following child chunks; wrong variants rejected.")

coordinate_cases = 0
with tempfile.TemporaryDirectory(prefix="openshores-coordinate-cases-") as directory:
    for extension in (".3ds", ".3dsN", ".3dsNT", ".3dsX"):
        source = Path(directory) / ("model" + extension)
        for values in (
            (1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0),
            (0, 1, 0, -1, 0, 0, 0, 0, 1, 12, 24, 36),
            (2, 0, 0, 1, 4, 0, 0, 0, 8, 12, 24, 36),
            (1.25, -0.75, 0.2, 0.15, 0.9, -0.1, 0.5, 0.25, 1.1, -1.25, 2.3, -0.125),
        ):
            parts = [chunk(0x4110, vertices), chunk(0x4160, struct.pack("<12f", *values))]
            for ordered in (parts, parts[::-1]):
                data = chunk(0x4D4D, chunk(0x3D3D, chunk(0x4000, b"mesh\0" + chunk(0x4100, b"".join(ordered)))))
                source.write_bytes(data)
                meshes = geometry(data, extension, return_meshes=True)[-1]
                for scale in (1.0, 0.125, 2.3):
                    check_coordinates("target/debug/openshores-client", source, extension, meshes, scale)
                    coordinate_cases += 1
print(f"Coordinate-stage mathematical parity: {coordinate_cases} synthetic format/matrix/order/scale cases")


def object_fixture(name, groups=(), face=True):
    fields = chunk(0x4110, vertices)
    if face:
        fields += chunk(0x4120, struct.pack("<H4H9f", 1, 0, 1, 2, 0, *range(9)))
    for group_name, positions in groups:
        raw = struct.pack("<H", len(positions)) + b"".join(struct.pack("<4f", *point, 1.0) for point in positions)
        fields += chunk(0x4112, chunk(0x4113, group_name + b"\0") + chunk(0x4114, raw))
    return chunk(0x4000, name + b"\0" + chunk(0x4100, fields))


with tempfile.TemporaryDirectory(prefix="openshores-helper-cases-") as directory:
    points = [(0, 0, 0), (1, 0, 0), (0, 1, 0), (1, 1, 0)]
    objects = [object_fixture(name) for name in (b"!3", b"!x", b"!", b"@2FF", b"@x+f", b"@2\xa0f", b"@2-1", b"@200", b"DataOnly", b"-visible")]
    objects += [object_fixture(b"UpVector", face=False), object_fixture(b"DataOnly", face=False), object_fixture(b"ordinary", face=False)]
    objects += [object_fixture(b"first screen", [(b"Screen1", points)]), object_fixture(b"second screen", [(b"Screen2", points)]), object_fixture(b"later object")]
    objects += [object_fixture(b"UpVector", [(b"Screen1", points[:3])]), object_fixture(b"after invalid screen")]
    data = chunk(0x4D4D, chunk(0x3D3D, b"".join(objects)))
    source = Path(directory) / "helpers.3dsN"
    source.write_bytes(data)
    meshes = geometry(data, ".3dsN", return_meshes=True)[-1]
    check_roles("target/debug/openshores-client", source, ".3dsN", meshes)
print("Synthetic helper/group parity: names, empty-object acceptance, paired-screen state and invalid groups")
