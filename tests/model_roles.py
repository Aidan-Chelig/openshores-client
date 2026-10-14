"""Independent raw-group and native-model helper classification checks.

Rules are reconstructed from static instructions, not original runtime probes.
"""
from collections import Counter
import hashlib
import json
import re
import struct
import subprocess


def group_digest(meshes):
    digest = hashlib.sha1()
    def count(value):
        digest.update(struct.pack("<I", value))
    count(len(meshes))
    for mesh in meshes:
        count(len(mesh["vertex_groups"]))
        for group in mesh["vertex_groups"]:
            for field in ("name", "members"):
                payload = group[field]
                digest.update(bytes([payload is not None]))
                if payload is not None:
                    count(len(payload) if field == "name" else len(payload) // 16)
                    digest.update(payload)
    return digest.hexdigest()


def roles(meshes, extension):
    screens = set()
    result = []
    for mesh in meshes:
        name = mesh["name"]
        accepted_empty = extension in (".3dsN", ".3dsNT") and (name.startswith((b"!", b"@")) or name in (b"DataOnly", b"UpVector"))
        if not mesh["vertices"] or (not mesh["faces"] and not accepted_empty):
            result.append("rejected")
            continue
        if len(name) >= 4 and name.startswith(b"@"):
            number = name[2:4].decode("latin-1").strip("\t\n\v\f\r \x85\xa0")
            if re.fullmatch(r"\+?[0-9a-fA-F]+", number) and int(number, 16) != 0:
                result.append("station")
                continue
        if len(name) >= 2 and name.startswith(b"!"):
            result.append("weapon")
            continue
        valid = True
        for group in mesh["vertex_groups"]:
            if group["name"] not in (b"Screen1", b"Screen2"):
                continue
            members = group["members"] or b""
            positions = {struct.unpack_from("<fff", members, offset) for offset in range(0, len(members), 16)}
            if len(positions) <= 3:
                valid = False
                break
            screens.add(group["name"])
        if valid and screens == {b"Screen1", b"Screen2"}:
            result.append("screens")
        elif b"UpVector" in name:
            result.append("up-vector")
        else:
            result.append("geometry")
    return result


def check_roles(binary, source, extension, meshes):
    expected = roles(meshes, extension)
    actual = json.loads(subprocess.check_output([binary, "dump-model-roles", source], text=True))
    assert actual == expected, (source, actual, expected)
    actual_digest = subprocess.check_output([binary, "inspect-model-groups", source], text=True).strip()
    assert actual_digest == "groups=" + group_digest(meshes), source
    return Counter(expected), sum(len(mesh["vertex_groups"]) for mesh in meshes)
