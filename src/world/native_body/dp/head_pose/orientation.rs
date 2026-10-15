use super::*;

/// Native tilt correction's ordered/unordered COMISD branches. None means the
/// original pitch proceeds without even subtracting a signed zero correction.
pub(super) fn correction(owner: &NativeDpBoneTransforms, segmented: bool) -> Option<f32> {
    let (mut x, mut y, mut z) = if segmented {
        let m = &owner.slots[0].matrix_00;
        let w = (m[7] + 0.) + m[15];
        let mut up = [m[4] + 0., m[5] + 0., m[6] + 0.];
        if w != 0. && w != 1. {
            for v in &mut up {
                *v /= w;
            }
        }
        (up[0], up[1], up[2])
    } else {
        (0., 1., 0.)
    };
    let length = ((x * x + y * y) + z * z).sqrt();
    let mut x_squared = 1.;
    if length != 0. {
        x /= length;
        y /= length;
        z /= length;
        x_squared = x * x;
    }
    let horizontal = (y * y + x_squared).sqrt();
    let epsilon = 1e-5_f64;
    // COMISD's JB includes unordered; do not replace with plain '<'.
    if less_or_unordered(epsilon, z.abs()) {
        if less_or_unordered(epsilon, horizontal) {
            Some(z.atan2(horizontal) as f32)
        } else if less_or_unordered(z, -epsilon) {
            Some(-f32::from_bits(0x3fc90fdb))
        } else {
            Some(f32::from_bits(0x3fc90fdb))
        }
    } else if horizontal >= -epsilon {
        None
    } else {
        Some(f32::from_bits(0x40490fdb))
    }
}

fn less_or_unordered(a: f64, b: f64) -> bool {
    a.is_nan() || b.is_nan() || a < b
}
