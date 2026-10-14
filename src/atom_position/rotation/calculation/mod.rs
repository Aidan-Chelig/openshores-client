//! Native two-vector CalculateRotation, reconstructed from static evidence.
mod basis;
mod trig;
use std::{f64::consts::PI, io};

const ANGLE_ZERO: f64 = 1e-5;

/// Normalize the supplied vectors, use the native empty-secondary defaults,
/// derive X/Y from their cross product, and derive Z after inverse-XY rotation.
/// This is not Gram-Schmidt or generic quaternion/Euler decomposition.
/// Nonfinite inputs and norm overflow/underflow are explicit Rust safety errors.
pub fn native_calculate_rotation(vectors: &[[f64; 3]; 2]) -> io::Result<[f64; 3]> {
    let (right, [up_x, up_y, up_z]) = basis::prepare(vectors)?;
    let has_x = up_y != 0.0;
    let down = up_y == 0.0 && up_x == 0.0 && up_z < 0.0;
    let mut x = 0.0;
    let mut y = 0.0;
    if has_x {
        x = -up_y.atan2(up_z);
        if up_x != 0.0 {
            y = up_x.atan2((up_z * up_z + up_y * up_y).sqrt());
        }
    } else if up_x != 0.0 {
        y = up_x.atan2(up_z);
    } else if down {
        y = PI;
    }
    let z = trig::z_angle(right, x, y, has_x, down);
    // Native derives Z BEFORE snapping the previously computed X/Y values.
    Ok([x, y, z].map(|angle| {
        if angle.abs() <= ANGLE_ZERO {
            0.0
        } else {
            angle
        }
    }))
}

#[cfg(test)]
mod tests;
