//! GetRotation's projected input stage; final CalculateRotation stays separate.
use std::io;

fn invalid() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "non-finite native rotation input",
    )
}

fn normalize(point: [f64; 3], w: f64) -> io::Result<[f64; 3]> {
    if !w.is_finite() || point.iter().any(|value| !value.is_finite()) {
        return Err(invalid());
    }
    // Native exact zero/one branches: zero is not division-by-zero or epsilon.
    let result = if w == 0.0 || w == 1.0 {
        point
    } else {
        point.map(|value| value / w)
    };
    if result.iter().any(|value| !value.is_finite()) {
        return Err(invalid());
    }
    Ok(result)
}

/// Reconstruct two vectors passed by AuMatrix4x4::GetRotation to CalculateRotation.
/// Column-major matrix. Preserves native projection and translation cancellation;
/// this is NOT scalar angle extraction, normalization of axes, or a quaternion.
/// Nonfinite/overflow rejection is an explicit Rust safety boundary.
pub fn native_rotation_vectors(matrix: &[[f64; 4]; 4]) -> io::Result<[[f64; 3]; 2]> {
    if matrix.iter().flatten().any(|value| !value.is_finite()) {
        return Err(invalid());
    }
    let translation = [matrix[3][0], matrix[3][1], matrix[3][2]];
    // Observed origin branch uses the SUM of all four homogeneous row entries,
    // not just translation.w; do not replace this with a generic project helper.
    let origin_w = ((matrix[0][3] + matrix[1][3]) + matrix[2][3]) + matrix[3][3];
    let origin = normalize(translation, origin_w)?;
    let mut vectors = [[0.0; 3]; 2];
    for axis in 0..2 {
        let numerator = std::array::from_fn(|row| (0.0 + matrix[axis][row]) + translation[row]);
        let projected = normalize(numerator, matrix[axis][3] + matrix[3][3])?;
        vectors[axis] = std::array::from_fn(|row| projected[row] - origin[row]);
    }
    if vectors.iter().flatten().any(|value| !value.is_finite()) {
        return Err(invalid());
    }
    Ok(vectors)
}

#[cfg(test)]
mod tests;
