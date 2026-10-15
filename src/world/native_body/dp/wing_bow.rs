//! Concrete BowWing: mirrored, sequential elbow and wrist Z-pivot stages.
use super::*;
use std::io;

impl NativeDpPartState {
    pub fn bow_wing(&self, owner: &mut NativeDpBoneTransforms, angle: f32) -> io::Result<()> {
        if !matches!(
            self.allocation.kind,
            NativeDpPartKind::LeftArmAndHand | NativeDpPartKind::RightArmAndHand
        ) {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "native BowWing requires ArmAndHand",
            ));
        }
        if angle == 0. {
            return Ok(());
        }
        let indices = self.limb_bones()?;
        let angle = if self.allocation.kind == NativeDpPartKind::LeftArmAndHand {
            -angle
        } else {
            angle
        };
        let NativeDpPartJoints::Arm {
            field_58, field_70, ..
        } = self.joints
        else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "native BowWing requires initialized arm fields58/70",
            ));
        };
        let pivot = bow_project(&owner.slots[indices[1]], field_58);
        let transform = bow_pivot(pivot, angle, false);
        for &index in &indices[1..] {
            owner.slots[index] = transform.multiply(&owner.slots[index]);
        }
        // This projects THROUGH THE ALREADY UPDATED third bone. Capturing a
        // bind-pose or pre-stage wrist pivot changes the native operation.
        let pivot = owner.slots[indices[2]].project(field_70);
        let transform = bow_pivot(pivot, angle, true);
        owner.slots[indices[2]] = transform.multiply(&owner.slots[indices[2]]);
        Ok(())
    }
}

fn bow_project(bone: &NativeDpBoneTransform, [x, y, z]: [f64; 3]) -> [f64; 3] {
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
            ((ty[3] + tx[3]) + tz[3]) + m[15],
        )
    };
    if w != 0. && w != 1. {
        for value in &mut point {
            *value /= w;
        }
    }
    point
}

// Retain both native operand orders, including nominally commutative sums.
#[allow(clippy::if_same_then_else)]
fn bow_pivot([x, y, z]: [f64; 3], angle: f32, second: bool) -> NativeDpBoneTransform {
    let (xz, yz, zz) = (x * 0., y * 0., z * 0.);
    let mut rotation = NativeDpBoneTransform::identity();
    let xy_zero = if second { xz + yz } else { yz + xz };
    rotation.matrix_00[12] = if second {
        ((yz + x) + zz) + 0.
    } else {
        ((x + yz) + zz) + 0.
    };
    rotation.matrix_00[13] = if second {
        ((xz + y) + zz) + 0.
    } else {
        ((y + xz) + zz) + 0.
    };
    rotation.matrix_00[14] = (z + xy_zero) + 0.;
    rotation.matrix_00[15] = (zz + xy_zero) + 1.;
    rotation.rotate_z(f64::from(angle));
    let m = rotation.matrix_00;
    for row in 0..4 {
        rotation.matrix_00[12 + row] = if second {
            ((-x * m[row] + -y * m[4 + row]) + -z * m[8 + row]) + m[12 + row]
        } else {
            ((-y * m[4 + row] + -x * m[row]) + -z * m[8 + row]) + m[12 + row]
        };
    }
    rotation.field_100 = 0;
    rotation
}

#[cfg(test)]
mod tests;
