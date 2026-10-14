"""Read-only user-installation parity and first-launch cache verification.

Uses a disposable cache, never launches any supplied EXE/DLL.
"""
from pathlib import Path
import ast
import hashlib
import struct
import subprocess
import tempfile
import zlib
from model_coordinates import check_coordinates
from reference_geometry import geometry

ROOT = Path(__file__).resolve().parents[1]
INSTALL = ROOT / "reference/hazeron"
BINARY = ROOT / "target/debug/openshores-client"


def run(*args):
    return subprocess.check_output([str(BINARY), *map(str, args)], text=True)


def main():
    if not INSTALL.is_dir():
        print("User-installation mount test skipped: reference/hazeron not supplied")
        return
    before = {path.relative_to(INSTALL): hashlib.sha256(path.read_bytes()).digest()
              for path in INSTALL.rglob("*") if path.is_file() and not path.is_symlink()}
    archive = (INSTALL / "sfx.har").read_bytes()
    count = struct.unpack_from(">I", archive)[0]
    offset = 4
    sounds = {}
    for _ in range(count):
        length = struct.unpack_from(">I", archive, offset)[0]
        offset += 4
        name = archive[offset:offset + length].decode("utf-16-be")
        offset += length
        length = struct.unpack_from(">I", archive, offset)[0]
        offset += 4
        payload = archive[offset:offset + length]
        offset += length
        expected = struct.unpack_from(">I", payload)[0]
        raw = zlib.decompress(payload[4:])
        assert len(raw) == expected and name not in sounds
        sounds[name] = raw
    assert offset == len(archive)
    inspected = run("inspect-mount", INSTALL).splitlines()
    for resource in (":data/mFlag.3dsX", ":data/mHay.3dsN", ":data/mTorch.3dsN"):
        decoded = run("inspect-mounted-model", INSTALL, resource)
        assert resource in decoded and " meshes, " in decoded and " faces" in decoded
    assets = dict(line.rsplit("\t", 1) for line in inspected[1:])
    assert len(assets) >= 4000
    for name, raw in sounds.items():
        assert int(assets[f"har/sfx.har/{name}"]) == len(raw)
    with tempfile.TemporaryDirectory(prefix="openshores-mount-") as temporary:
        cache = Path(temporary) / "cache"
        first = run("prepare-assets", INSTALL, cache)
        assert "Prepared" in first
        manifest = next(cache.glob("*.manifest-v1.txt")).read_text()
        for line in manifest.splitlines():
            if line.startswith("Opaque model"):
                print(line)
        for raw in sounds.values():
            assert (cache / (hashlib.sha1(raw).hexdigest() + ".raw")).read_bytes() == raw
        texture = (INSTALL / "tex/Exo.png").read_bytes()
        assert (cache / (hashlib.sha1(texture).hexdigest() + ".raw")).read_bytes() == texture
        assert list(cache.glob("*.geometry-v1.obj"))
        for resource in ("data/mFlag.3dsX", "data/mHay.3dsN", "data/mTorch.3dsN"):
            matches = {digest for line in manifest.splitlines() if "\t" in line
                       for digest, name in [line.split("\t", 1)]
                       if ast.literal_eval(name).split("/", 5)[-1] == resource}
            assert len(matches) == 1
            raw = (cache / (matches.pop() + ".raw")).read_bytes()
            source = Path(temporary) / Path(resource).name
            source.write_bytes(raw)
            extension = source.suffix
            meshes = geometry(raw, extension, return_meshes=True)[-1]
            check_coordinates(str(BINARY), source, extension, meshes)
        print("Mounted Flag/Hay/Torch coordinate stages match independent mathematics")
        objects = {p.name: (p.stat().st_mtime_ns, hashlib.sha256(p.read_bytes()).digest())
                   for p in cache.iterdir()}
        assert run("prepare-assets", INSTALL, cache) == first
        assert objects == {p.name: (p.stat().st_mtime_ns, hashlib.sha256(p.read_bytes()).digest())
                           for p in cache.iterdir()}
        # Cache output cannot be written underneath the installation.
        forbidden = INSTALL / "openshores-forbidden-cache-test"
        assert not forbidden.exists()
        rejected = subprocess.run([str(BINARY), "prepare-assets", str(INSTALL), str(forbidden)],
                                  capture_output=True, text=True)
        assert rejected.returncode != 0 and not forbidden.exists()
        print(first.strip())
    after = {path.relative_to(INSTALL): hashlib.sha256(path.read_bytes()).digest()
             for path in INSTALL.rglob("*") if path.is_file() and not path.is_symlink()}
    assert before == after, "installation changed during asset preparation"
    print(f"Mount/cache parity passed: {len(assets)} assets, {count} sequential HAR sounds; source untouched; cache reused")


if __name__ == "__main__":
    main()
