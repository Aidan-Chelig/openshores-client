"""Independent numerical oracle for reconstructed native coordinate stages.

Uses NumPy's matrix inversion, not the Rust/native in-place inversion routine.
This verifies the reconstruction against a separate mathematical implementation,
not runtime parity with the original executable. Raw-byte parity is separate.
"""
import json
import struct
import subprocess

import numpy as np


def loaded(mesh, extension, scale):
    vertices = np.asarray(list(struct.iter_unpack("<fff", mesh["vertices"])), dtype=np.float64).reshape(-1, 3)
    matrix = np.eye(4, dtype=np.float64)
    points = np.empty((0, 3), dtype=np.float64)
    inverse = None
    tolerance = 1e-6

    def project(points, matrix):
        return (matrix[:3, :3] @ points.T).T + matrix[:3, 3]

    def local(points):
        nonlocal tolerance
        points = project(points, inverse)
        # Independent inverse computations can fall on opposite sides of a
        # float rounding boundary. Bound projected storage error by matrix norm.
        if points.size:
            tolerance = max(tolerance, float(np.max(np.abs(points))) * float(np.linalg.norm(matrix[:3, :3], ord=np.inf)) * scale * 4 * np.finfo(np.float32).eps)
        return points.astype(np.float32).astype(np.float64)

    for chunk in mesh["geometry_order"]:
        if chunk == "vertices":
            points = vertices.copy()
            if inverse is not None:
                points = local(points)
        else:
            factor = 1 / 12 if extension == ".3dsX" else 1
            points = (points * factor).astype(np.float32).astype(np.float64)
            values = np.asarray(struct.unpack("<12f", mesh["matrix"]), dtype=np.float64)
            matrix[:3, :] = values.reshape(4, 3).T
            matrix[:3, 3] *= factor
            if not np.array_equal(matrix, np.eye(4)):
                inverse = np.linalg.inv(matrix)
                points = local(points)
    if scale != 1:
        matrix[:3, 3] *= scale
        points = (points * scale).astype(np.float32).astype(np.float64)
    return project(points, matrix), tolerance


def check_coordinates(binary, source, extension, meshes, scale=1.0):
    expected = [loaded(mesh, extension, scale) for mesh in meshes]
    output = subprocess.check_output([binary, "dump-model-coordinates", source, str(scale)], text=True)
    actual = json.loads(output)
    assert len(actual) == len(expected)
    for index, ((points, tolerance), received) in enumerate(zip(expected, actual, strict=True)):
        received = np.asarray(received, dtype=np.float64).reshape(-1, 3)
        assert received.shape == points.shape
        assert np.isfinite(received).all()
        np.testing.assert_allclose(received, points, rtol=0, atol=tolerance, err_msg=f"{source}: mesh {index}")
    return sum(len(points) for points, _ in expected)
