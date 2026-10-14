"""Independent native-angle mathematical fixtures and static anchors; no execution of the target."""
import hashlib
import itertools
import json
import math
from pathlib import Path
import random
import struct
import subprocess

target = Path("reference/openshores-full/OpenShores.exe")
assert hashlib.sha256(target.read_bytes()).hexdigest() == "285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80"
assembly = subprocess.check_output(["objdump", "-d", "--start-address=0x14164c1b0",
    "--stop-address=0x14164c958", str(target)], text=True)
for anchor in ("14164c285:", "14164c317:", "14164c5e1:", "14164c692:",
               "14164c80e:", "14164c45b:", "14164c460:", "14164c478:",
               "14164c4ea:", "14164c5c2:", "14164c8dd:"):
    assert anchor in assembly, anchor
epsilon = subprocess.check_output(["objdump", "-s", "--start-address=0x14232d158",
    "--stop-address=0x14232d160", str(target)], text=True)
assert "f168e388 b5f8e43e" in epsilon
assert struct.unpack("<d", bytes.fromhex("f168e388b5f8e43e"))[0] == 1e-5


def oracle(a, b):
    def unit(v):
        length = math.sqrt((v[0] ** 2 + v[1] ** 2) + v[2] ** 2)
        return [x / length for x in v]
    a = unit(a) if any(a) else [a[0], 0.0, 0.0]
    if not any(b):
        b = [-1., 0., 0.] if a == [0., 1., 0.] else [1., 0., 0.] if a == [0., -1., 0.] else [0., 1., 0.]
    else:
        b = unit(b)
    up = [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
    x = -math.atan2(up[1], up[2]) if up[1] else 0.0
    if up[0]:
        y = math.atan2(up[0], math.sqrt(up[1] ** 2 + up[2] ** 2) if up[1] else up[2])
    else:
        y = math.pi if not up[1] and up[2] < 0 else 0.0
    sx, cx, sy, cy = math.sin(x), math.cos(x), math.sin(y), math.cos(y)
    if abs(cx) == 1: sx = 0.0
    elif abs(sx) == 1: cx = 0.0
    if abs(cy) == 1: sy = 0.0
    elif abs(sy) == 1: cy = 0.0
    # Independent reduced geometric formula; vendor signed-zero factoring is
    # checked with explicit canonical cases, not claimed by this reduced oracle.
    z = math.atan2(cx * a[1] + sx * a[2], (cy * a[0] + sy * sx * a[1]) - sy * cx * a[2])
    return [v if abs(v) > 1e-5 else 0.0 for v in (x, y, z)]


def run(vectors):
    wire = struct.pack(">6d", *vectors[0], *vectors[1])
    return subprocess.run(["target/debug/openshores-client", "calculate-native-rotation", wire.hex()], capture_output=True, text=True)


axes = [(0., 0., 0.), (1., 0., 0.), (-1., 0., 0.), (0., 1., 0.),
        (0., -1., 0.), (0., 0., 1.), (0., 0., -1.)]
fixtures = list(itertools.product(axes, repeat=2))
generator = random.Random(0x53484F524553)
for _ in range(256):
    fixtures.append(tuple(tuple(generator.uniform(-10, 10) for _ in range(3)) for _ in range(2)))
for angle in (1e-6, 1e-5, 1e-4, -1e-6, -1e-5, -1e-4):
    fixtures.append(((math.cos(angle), math.sin(angle), 0.), (-math.sin(angle), math.cos(angle), 0.)))
for vectors in fixtures:
    result = run(vectors)
    assert result.returncode == 0, (vectors, result.stderr)
    actual = json.loads(result.stdout)
    expected = oracle(*vectors)
    for observed, predicted in zip(actual, expected):
        # Canonical +/-pi aliases are geometrically equivalent; pin important
        # native scalar branch choices separately below.
        assert math.isclose(math.sin(observed), math.sin(predicted), abs_tol=1e-12)
        assert math.isclose(math.cos(observed), math.cos(predicted), abs_tol=1e-12)
        if predicted == 0: assert observed == 0
assert json.loads(run(((1., 0., 0.), (0., -1., 0.))).stdout) == [0., math.pi, math.pi]
assert json.loads(run(((-1., -0., 0.), (0., -1., -0.))).stdout) == [0., 0., math.pi]
for vectors in (((math.nan, 0., 0.), (0., 1., 0.)), ((1., 0., 0.), (math.inf, 0., 0.)),
                ((1e308, 0., 0.), (0., 1., 0.)), ((1e-300, 0., 0.), (0., 1., 0.))):
    result = run(vectors)
    assert result.returncode != 0 and result.stdout == ""
for wire in ("", "00" * 47, "00" * 49, "gg"):
    result = subprocess.run(["target/debug/openshores-client", "calculate-native-rotation", wire], capture_output=True)
    assert result.returncode != 0 and result.stdout == b""
print(f"Native angle calculation: {len(fixtures)} independent mathematical cases, canonical scalar choices, norm safety and bounded diagnostics verified offline; not a runtime oracle.")
