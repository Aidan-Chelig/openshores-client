//! Concrete RotateWrist, including its separately inlined projection order.
use super::*;
use std::io;

impl NativeDpPartState {
    pub fn rotate_arm_wrist(
        &self,
        owner: &mut NativeDpBoneTransforms,
        angle: f32,
    ) -> io::Result<()> {
        if !matches!(
            self.allocation.kind,
            NativeDpPartKind::LeftArmAndHand | NativeDpPartKind::RightArmAndHand
        ) {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "native RotateWrist requires ArmAndHand",
            ));
        }
        if angle == 0. {
            return Ok(());
        }
        let index = self.limb_bones()?[2];
        let NativeDpPartJoints::Arm { field_70, .. } = self.joints else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "native RotateWrist requires initialized arm field70",
            ));
        };
        let pivot = wrist_project(&owner.slots[index], field_70);
        let transform = wrist_pivot(pivot, angle);
        owner.slots[index] = transform.multiply(&owner.slots[index]);
        Ok(())
    }
}

pub(super) fn wrist_project(bone: &NativeDpBoneTransform, [x, y, z]: [f64; 3]) -> [f64; 3] {
    let m = &bone.matrix_00;
    let (mut point, w) = if x == 0. && y == 0. && z == 0. {
        ([m[12], m[13], m[14]], ((m[3] + m[7]) + m[11]) + m[15])
    } else {
        let tx = if x == 0. {
            [0.; 4]
        } else {
            [m[0] * x, m[1] * x, m[2] * x, x * m[3]]
        };
        let ty = if y == 0. {
            [0.; 4]
        } else {
            [m[4] * y, m[5] * y, m[6] * y, y * m[7]]
        };
        let tz = if z == 0. {
            [0.; 4]
        } else {
            [m[8] * z, m[9] * z, m[10] * z, z * m[11]]
        };
        (
            [
                ((ty[0] + tx[0]) + tz[0]) + m[12],
                ((tx[1] + ty[1]) + tz[1]) + m[13],
                ((tx[2] + ty[2]) + tz[2]) + m[14],
            ],
            ((tx[3] + ty[3]) + tz[3]) + m[15],
        )
    };
    if w != 0. && w != 1. {
        for value in &mut point {
            *value /= w;
        }
    }
    point
}

pub(super) fn wrist_pivot([x, y, z]: [f64; 3], angle: f32) -> NativeDpBoneTransform {
    let (s, c) = f64::from(angle).sin_cos();
    let (xz, yz, zz) = (x * 0., y * 0., z * 0.);
    let (cz, sz) = (c * 0., s * 0.);
    let a = cz + sz;
    let b = cz - sz;
    let d = c + sz;
    let e = cz - s;
    let f = s + cz;
    let g = c - sz;
    let xy_zero = yz + xz;
    let initial_x = ((yz + x) + zz) + 0.;
    let initial_y = ((y + xz) + zz) + 0.;
    let initial_z = (z + xy_zero) + 0.;
    let initial_w = (zz + xy_zero) + 1.;
    let tx = initial_x + ((a * -y - x) + -z * b);
    // Unlike RotateShoulder, the native wrist adds initial Y AFTER products.
    let ty = ((d * -y - xz) + e * -z) + initial_y;
    let tz = ((f * -y - xz) + -z * g) + initial_z;
    let tw = ((a * -y - xz) + -z * b) + initial_w;
    NativeDpBoneTransform {
        matrix_00: [1., 0., 0., 0., a, d, f, a, b, e, g, b, tx, ty, tz, tw],
        field_100: 0,
    }
}

#[cfg(test)]
mod tests;
