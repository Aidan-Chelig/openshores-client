use super::super::invalid;
use std::io;

fn unit(vector: [f64; 3]) -> io::Result<[f64; 3]> {
    let squared = (vector[0] * vector[0] + vector[1] * vector[1]) + vector[2] * vector[2];
    let length = squared.sqrt();
    // Rust safety boundary for overflow and nonzero vectors whose norm underflows.
    if !length.is_finite() || length == 0.0 {
        return Err(invalid());
    }
    Ok(vector.map(|value| value / length))
}

pub(super) fn prepare(vectors: &[[f64; 3]; 2]) -> io::Result<([f64; 3], [f64; 3])> {
    if vectors.iter().flatten().any(|value| !value.is_finite()) {
        return Err(invalid());
    }
    // Native all-zero primary branch clears Y/Z but retains the loaded X bits.
    let right = if vectors[0] == [0.0; 3] {
        [vectors[0][0], 0.0, 0.0]
    } else {
        unit(vectors[0])?
    };
    let forward = if vectors[1] == [0.0; 3] {
        if right == [0.0, 1.0, 0.0] {
            [-1.0, 0.0, 0.0]
        } else if right == [0.0, -1.0, 0.0] {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        }
    } else {
        unit(vectors[1])?
    };
    let up = [
        forward[2] * right[1] - forward[1] * right[2],
        forward[0] * right[2] - forward[2] * right[0],
        forward[1] * right[0] - forward[0] * right[1],
    ];
    Ok((right, up))
}
