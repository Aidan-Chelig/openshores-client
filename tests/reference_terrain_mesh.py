"""Offline OBJ parity against supplied Python terrain; no server or database.

Run after cargo build: nix develop .#server-test --command python tests/reference_terrain_mesh.py
"""
import math
from pathlib import Path
import struct
import subprocess
import sys
import tempfile
from collections import Counter

ROOT = Path(__file__).resolve().parents[1]
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "reference/OpenShores-server/src"))
from openshores.gameplay.worldgen.terrain import terrain_altitude_msl
from openshores.protocol.framing import encode_size


def main():
    binary = ROOT / "target/debug/openshores-client"
    packet = bytes.fromhex((ROOT / "tests/fixtures/atom-globe.hex").read_text())
    parameters = tuple(struct.unpack(">f", struct.pack(">f", x))[0]
                       for x in (3, 2.26, .65, 123.37, 456.81, 0))
    with tempfile.TemporaryDirectory(prefix="openshores-terrain-") as directory:
        directory = Path(directory)
        capture = directory / "scene.bin"
        capture.write_bytes(encode_size(len(packet)) + packet)
        for bands in (3, 16, 128):
            output = directory / f"terrain-{bands}.obj"
            command = [str(binary), "export-terrain", str(capture), "7", str(bands), str(output)]
            subprocess.run(command, check=True, capture_output=True, timeout=30)
            demo = directory / f"demo-{bands}.obj"
            demo_command = [str(binary), "export-terrain-demo", str(bands), str(demo)]
            subprocess.run(demo_command, check=True, capture_output=True, timeout=30)
            assert demo.read_bytes() == output.read_bytes()
            assert subprocess.run(demo_command, capture_output=True, timeout=30).returncode != 0
            assert demo.read_bytes() == output.read_bytes()
            vertices, faces = [], []
            for line in output.read_text().splitlines():
                fields = line.split()
                if fields and fields[0] == "v":
                    vertices.append(tuple(map(float, fields[1:])))
                if fields and fields[0] == "f":
                    faces.append(tuple(int(x) - 1 for x in fields[1:]))
            sectors = 2 * bands
            assert len(vertices) == 2 + (bands - 1) * sectors
            assert len(faces) == 2 * sectors * (bands - 1)
            expected = []
            altitudes = []
            for row in range(bands + 1):
                lat = -math.pi / 2 + math.pi * row / bands
                columns = 1 if row in (0, bands) else sectors
                for column in range(columns):
                    lon = 0 if columns == 1 else -math.pi + 2 * math.pi * column / sectors
                    altitude = terrain_altitude_msl(parameters, 8, lat, lon)
                    altitudes.append(altitude)
                    radius = 8 * (1000 / 1.609344 * 5) + altitude
                    if row == 0:
                        point = (0, 0, -radius)
                    elif row == bands:
                        point = (0, 0, radius)
                    else:
                        point = (radius * math.cos(lat) * math.cos(lon),
                                 radius * math.cos(lat) * math.sin(lon), radius * math.sin(lat))
                    expected.append(point)
            for actual, point in zip(vertices, expected):
                assert all(abs(a - b) < 1e-7 for a, b in zip(actual, point))
            summary = subprocess.run([str(binary), "inspect-terrain-demo", str(bands)],
                                     check=True, capture_output=True, text=True, timeout=30).stdout
            assert f"{len(vertices)} vertices, {len(faces)} triangles" in summary
            limits = summary.split("sampled altitude MSL: ")[1].split(" (not global extrema)")[0]
            low, high = map(float, limits.split(" .. "))
            assert abs(low - min(altitudes)) < 1e-8
            assert abs(high - max(altitudes)) < 1e-8
            edges = Counter()
            for a, b, c in faces:
                assert len({a, b, c}) == 3
                assert all(0 <= i < len(vertices) for i in (a, b, c))
                for u, v in ((a, b), (b, c), (c, a)):
                    edges[u, v] += 1
            assert all(count == 1 and edges[v, u] == 1 for (u, v), count in edges.items())
            original = output.read_bytes()
            assert subprocess.run(command, capture_output=True, timeout=30).returncode != 0
            assert output.read_bytes() == original
        for bands in (0, 2, 129):
            output = directory / f"invalid-{bands}.obj"
            assert subprocess.run([str(binary), "export-terrain", str(capture), "7",
                                   str(bands), str(output)], capture_output=True, timeout=30).returncode != 0
            assert not output.exists()
            assert subprocess.run([str(binary), "export-terrain-demo", str(bands), str(output)],
                                  capture_output=True, timeout=30).returncode != 0
            assert not output.exists()
    print("Terrain OBJ: Python parity at 3/16/128 bands; closed seams/poles; no-overwrite and bounds verified.")


if __name__ == "__main__":
    main()
