"""Independent OBJ and orthographic SVG checks for every valid model resource."""
import math
from pathlib import Path
import re
import struct
import subprocess
import tempfile
import xml.etree.ElementTree as ET
from reference_geometry import geometry


def check_previews(binary, entries, payloads):
    tested = 0
    with tempfile.TemporaryDirectory(prefix="openshores-preview-tests-") as directory:
        root = Path(directory)
        for index, (entry, payload) in enumerate(zip(entries, payloads, strict=True)):
            extension = Path(entry[1]).suffix
            if extension not in (".3ds", ".3dsN", ".3dsX"):
                continue
            if struct.unpack_from("<HI", payload) != (0x4D4D, len(payload)):
                continue
            _, _, _, _, _, meshes = geometry(payload, extension, return_meshes=True)
            source = root / ("model" + extension)
            source.write_bytes(payload)
            obj_path = root / f"{index}.obj"
            svg_path = root / f"{index}.svg"
            for command, output in [("export-model", obj_path), ("preview-model", svg_path)]:
                subprocess.run([binary, command, source, output], check=True, capture_output=True)
            vertices = []
            expected_faces = []
            expected_objects = []
            offset = 1
            for mesh_index, mesh in enumerate(meshes):
                points = list(struct.iter_unpack("<fff", mesh["vertices"]))
                vertices.extend(points)
                expected_objects.append(f"mesh_{mesh_index}")
                for face in mesh["faces"]:
                    expected_faces.append(tuple(value + offset for value in struct.unpack_from("<HHH", face)))
                offset += len(points)
            obj_vertices = bytearray()
            actual_faces = []
            objects = []
            for line in obj_path.read_text().splitlines():
                fields = line.split()
                if fields[0] == "#":
                    continue
                if fields[0] == "o":
                    assert len(fields) == 2
                    objects.append(fields[1])
                elif fields[0] == "v":
                    assert len(fields) == 4
                    obj_vertices.extend(struct.pack("<fff", *(float(value) for value in fields[1:])))
                elif fields[0] == "f":
                    assert len(fields) == 4
                    actual_faces.append(tuple(int(value) for value in fields[1:]))
                else:
                    raise AssertionError(f"unexpected OBJ directive: {line}")
            assert bytes(obj_vertices) == b"".join(mesh["vertices"] for mesh in meshes)
            assert actual_faces == expected_faces
            assert objects == expected_objects
            tree = ET.parse(svg_path)
            namespace = "{http://www.w3.org/2000/svg}"
            assert tree.getroot().tag == namespace + "svg"
            assert all(element.tag.removeprefix(namespace) in {"svg", "title", "desc", "rect", "g", "text", "path"} for element in tree.iter())
            paths = list(tree.iter(namespace + "path"))
            assert len(paths) == len(expected_faces) * 3
            if vertices:
                minimum = [min(vertex[axis] for vertex in vertices) for axis in range(3)]
                maximum = [max(vertex[axis] for vertex in vertices) for axis in range(3)]
                for panel, (horizontal, vertical) in enumerate([(0, 1), (0, 2), (1, 2)]):
                    span = max(maximum[horizontal] - minimum[horizontal], maximum[vertical] - minimum[vertical])
                    scale = 340 / span if span else 1
                    center_x = (minimum[horizontal] + maximum[horizontal]) / 2
                    center_y = (minimum[vertical] + maximum[vertical]) / 2
                    for face_index, face in enumerate(expected_faces):
                        path = paths[panel * len(expected_faces) + face_index]
                        match = re.fullmatch(r"M (\S+) (\S+) L (\S+) (\S+) L (\S+) (\S+) Z", path.attrib["d"])
                        assert match
                        actual = tuple(float(value) for value in match.groups())
                        expected = []
                        for vertex_index in face:
                            vertex = vertices[vertex_index - 1]
                            expected.extend([panel * 420 + 210 + (vertex[horizontal] - center_x) * scale,
                                             255 - (vertex[vertical] - center_y) * scale])
                        assert all(math.isfinite(value) for value in actual)
                        assert all(abs(a - e) <= 0.000501 for a, e in zip(actual, expected, strict=True))
            if tested == 0:
                for command, output in [("export-model", obj_path), ("preview-model", svg_path)]:
                    before = output.read_bytes()
                    repeated = subprocess.run([binary, command, source, output], capture_output=True)
                    assert repeated.returncode != 0
                    assert output.read_bytes() == before
                source.write_bytes(b"invalid")
                for command in ("export-model", "preview-model"):
                    output = root / command
                    assert subprocess.run([binary, command, source, output], capture_output=True).returncode != 0
                    assert not output.exists()
            tested += 1
    assert tested == 209
    print("Preview parity: all 209 models export exact OBJ positions/faces and finite three-view SVGs; overwrite and invalid-input checks passed")
