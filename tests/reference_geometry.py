"""Independent raw-byte 3DS-family geometry decoder for resource parity checks.

Called by reference_models.py; hashes serialized geometry without float conversion.
"""
import ast
import hashlib
import re
import struct
import subprocess


def material_fingerprint(data, variant):
    """Hash decoded material records using raw payload bytes, no float arithmetic."""
    records = []

    def chunks(start, end, context, record=None, property_id=None):
        while start < end:
            kind, length = struct.unpack_from("<HI", data, start)
            assert length >= 6 and start + length <= end
            body, stop = start + 6, start + length
            payload = data[body:stop]
            if context == "main" and kind == 0x3D3D:
                chunks(body, stop, "editor")
            elif context == "editor" and kind == 0xAFFF:
                material = dict(name=None, textures=[], colors=[], amounts=[], two_sided=False, shading=None)
                records.append(material)
                chunks(body, stop, "material", material)
            elif context in ("material", "texture") and kind in (0xA000, 0xA300):
                assert payload.endswith(b"\0") and b"\0" not in payload[:-1]
                if kind == 0xA000:
                    record["name"] = payload[:-1]
                else:
                    record["textures"].append(payload[:-1])
            elif context == "material" and kind == 0xA200:
                chunks(body, stop, "texture", record)
            elif context == "material" and kind in (0xA010, 0xA020, 0xA030):
                chunks(body, stop, "color", record, kind)
            elif context == "color" and kind in (0x10, 0x11, 0x12):
                assert len(payload) == (12 if kind == 0x10 else 3)
                record["colors"].append((property_id, kind, payload))
            elif context == "material" and kind in (0xA040, 0xA041, 0xA050, 0xA052, 0xA053):
                chunks(body, stop, "amount", record, kind)
            elif context == "amount" and kind == 0x30:
                assert len(payload) in (2, 4)
                assert len(payload) == 2 or variant != ".3dsX"
                record["amounts"].append((property_id, payload))
            elif context == "material" and kind == 0xA081:
                assert not payload
                record["two_sided"] = True
            elif context == "material" and kind == 0xA100:
                assert len(payload) == 2
                record["shading"] = payload
            start = stop

    chunks(6, len(data), "main")
    digest = hashlib.sha1()

    def length(value):
        digest.update(struct.pack("<I", value))

    def name(value):
        length(len(value))
        digest.update(value)

    length(len(records))
    for record in records:
        digest.update(bytes([record["name"] is not None]))
        if record["name"] is not None:
            name(record["name"])
        length(len(record["textures"]))
        for texture in record["textures"]:
            name(texture)
        length(len(record["colors"]))
        for property_id, encoding, value in record["colors"]:
            digest.update(struct.pack("<HH", property_id, encoding))
            digest.update(value)
        length(len(record["amounts"]))
        for property_id, value in record["amounts"]:
            digest.update(struct.pack("<HB", property_id, len(value)))
            digest.update(value)
        digest.update(bytes([record["two_sided"], record["shading"] is not None]))
        if record["shading"] is not None:
            digest.update(record["shading"])
    return digest.hexdigest(), records


def geometry(data, variant, return_meshes=False):
    assert struct.unpack_from("<HI", data) == (0x4D4D, len(data))
    meshes = []

    def word(offset):
        return struct.unpack_from("<H", data, offset)[0]

    def string(start, end):
        stop = data.index(0, start, end)
        return data[start:stop], stop + 1

    def chunks(start, end, context, mesh=None):
        while start < end:
            assert end - start >= 6
            kind, length = struct.unpack_from("<HI", data, start)
            assert length >= 6 and start + length <= end
            body, stop = start + 6, start + length
            if context == "main" and kind == 0x3D3D:
                chunks(body, stop, "editor")
            elif context == "editor" and kind == 0x4000:
                name, pos = string(body, stop)
                current = dict(name=name, vertices=b"", faces=[], uvs=None, matrix=None, smoothing=None, materials=[], geometry_order=[], vertex_groups=[])
                meshes.append(current)
                chunks(pos, stop, "object", current)
            elif context == "object" and kind == 0x4100:
                chunks(body, stop, "mesh", mesh)
            elif context == "mesh" and kind == 0x4110:
                count = word(body)
                assert body + 2 + count * 12 == stop
                mesh["vertices"] = data[body + 2:stop]
                mesh["geometry_order"].append("vertices")
            elif context == "mesh" and kind == 0x4120:
                count = word(body)
                size = {".3dsN": 44, ".3dsNT": 84}.get(variant, 8)
                pos = body + 2
                assert pos + count * size <= stop
                mesh["faces"] = [data[pos + i * size:pos + (i + 1) * size] for i in range(count)]
                chunks(pos + count * size, stop, "faces", mesh)
            elif context == "mesh" and kind == 0x4140:
                count = word(body)
                assert body + 2 + count * 8 == stop
                mesh["uvs"] = data[body + 2:stop]
            elif context == "mesh" and kind == 0x4160:
                assert stop - body == 48
                mesh["matrix"] = data[body:stop]
                mesh["geometry_order"].append("matrix")
            elif context == "mesh" and kind == 0x4112:
                group = dict(name=None, members=None)
                mesh["vertex_groups"].append(group)
                chunks(body, stop, "group", group)
            elif context == "group" and kind == 0x4113:
                assert mesh["name"] is None
                mesh["name"], position = string(body, stop)
                assert position == stop
            elif context == "group" and kind == 0x4114:
                assert mesh["members"] is None
                count = word(body)
                assert body + 2 + count * 16 == stop
                mesh["members"] = data[body + 2:stop]
            elif context == "faces" and kind == 0x4150:
                assert stop - body == len(mesh["faces"]) * 4
                mesh["smoothing"] = data[body:stop]
            elif context == "faces" and kind == 0x4130:
                name, pos = string(body, stop)
                count = word(pos)
                assert pos + 2 + count * 2 == stop
                mesh["materials"].append((name, data[pos + 2:stop]))
            start = stop

    chunks(6, len(data), "main")
    digest = hashlib.sha1()

    def length(value):
        digest.update(struct.pack("<I", value))

    def name(value):
        length(len(value))
        digest.update(value)

    length(len(meshes))
    vertices = faces = mismatched_uvs = 0
    for mesh in meshes:
        name(mesh["name"])
        count = len(mesh["vertices"]) // 12
        vertices += count
        length(count)
        digest.update(mesh["vertices"])
        faces += len(mesh["faces"])
        length(len(mesh["faces"]))
        for face in mesh["faces"]:
            assert max(struct.unpack_from("<HHH", face)) < count
            digest.update(face[:8])
            digest.update(bytes([variant in (".3dsN", ".3dsNT")]))
            digest.update(face[8:44])
            if variant == ".3dsNT":
                digest.update(b"NT")
                digest.update(face[44:])
        uv = mesh["uvs"]
        digest.update(bytes([uv is not None]))
        if uv is not None:
            length(len(uv) // 8)
            digest.update(uv)
            mismatched_uvs += len(uv) // 8 != count
        matrix = mesh["matrix"]
        digest.update(bytes([matrix is not None]))
        if matrix is not None:
            digest.update(matrix)
        smoothing = mesh["smoothing"]
        digest.update(bytes([smoothing is not None]))
        if smoothing is not None:
            length(len(smoothing) // 4)
            digest.update(smoothing)
        length(len(mesh["materials"]))
        for material_name, indices in mesh["materials"]:
            assert all(index[0] < len(mesh["faces"]) for index in struct.iter_unpack("<H", indices))
            name(material_name)
            length(len(indices) // 2)
            digest.update(indices)
    result = (len(meshes), vertices, faces, digest.hexdigest(), mismatched_uvs)
    return (*result, meshes) if return_meshes else result


def check_geometry(binary, dll, entries, payloads):
    result = subprocess.run([binary, "verify-models", dll, "3"], text=True, capture_output=True)
    expected = {}
    rejected = []
    total_meshes = total_vertices = total_faces = mismatched_uvs = 0
    material_count = color_count = word_count = float_count = 0
    for index, (entry, payload) in enumerate(zip(entries, payloads, strict=True)):
        extension = entry[1][entry[1].rfind("."):]
        if extension not in (".3ds", ".3dsN", ".3dsX"):
            continue
        root = struct.unpack_from("<HI", payload)
        if root != (0x4D4D, len(payload)):
            rejected.append(entry[1])
            continue
        meshes, vertices, faces, digest, uv_count = geometry(payload, extension)
        material_digest, materials = material_fingerprint(payload, extension)
        expected[index] = (entry[1], meshes, vertices, faces, digest, material_digest)
        material_count += len(materials)
        for material in materials:
            color_count += len(material["colors"])
            word_count += sum(len(value) == 2 for _, value in material["amounts"])
            float_count += sum(len(value) == 4 for _, value in material["amounts"])
        total_meshes += meshes
        total_vertices += vertices
        total_faces += faces
        mismatched_uvs += uv_count
    actual = {}
    for line in result.stdout.splitlines()[:-1]:
        match = re.fullmatch(r'(\d+): (".*"); meshes=(\d+); vertices=(\d+); faces=(\d+); geometry=([0-9a-f]{40}); materials=([0-9a-f]{40})', line)
        assert match, line
        index, path, meshes, vertices, faces, digest, material_digest = match.groups()
        actual[int(index)] = (ast.literal_eval(path), int(meshes), int(vertices), int(faces), digest, material_digest)
    assert expected == actual
    assert len(expected) == 209
    assert set(rejected) == {":/data/mRocketShip.3ds", ":/data/mHitSeed.3ds", ":/data/mNeptune.3ds", ":/data/mHitPierce.3ds", ":/data/mHitSlash.3ds"}
    assert result.returncode != 0
    for path in rejected:
        assert f'Rejected "{path}": expected exact 3DS root chunk' in result.stderr
    assert "5 model resources rejected; verification is incomplete" in result.stderr
    assert f"Verified 209 models; {total_meshes} meshes; {total_vertices} vertices; {total_faces} faces." in result.stdout
    print(f"Geometry parity: 209 models, {total_meshes} meshes, {total_vertices} vertices, {total_faces} faces; all geometry digests match")
    print(f"Material parity: {material_count} materials, {color_count} color records, {word_count} integer and {float_count} float amount records; all material digests match")
    print(f"Five invalid root lengths rejected; {mismatched_uvs} unequal UV/vertex counts preserved without guessed repair")
