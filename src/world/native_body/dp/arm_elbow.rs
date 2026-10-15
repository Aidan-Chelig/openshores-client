//! RotateElbow's two concrete pivot and Bones-output branches.
use super::*;
use std::io;

impl NativeDpPartState {
    pub fn rotate_arm_elbow(
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
                "native RotateElbow requires ArmAndHand",
            ));
        }
        if angle == 0. {
            return Ok(());
        }
        let indices = self.limb_bones()?;
        let raw = &self.allocation.dna.0;
        let wing = raw[1] & 3 != 0 && raw[2] & 6 != 0 && (raw[2] & 7).wrapping_sub(5) > 1;
        let pivot = if wing {
            self.current_connect_loc(owner)?
        } else {
            let NativeDpPartJoints::Arm { field_58, .. } = self.joints else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "native RotateElbow requires initialized arm field58",
                ));
            };
            owner.slots[indices[1]].project(field_58)
        };
        let transform = elbow_pivot(pivot, angle, wing);
        // Preserve actual out-reference order, not an anatomical assumption:
        // ordinary branch modifies outputs2/3; wing branch modifies output2.
        for &index in &indices[1..if wing { 2 } else { 3 }] {
            owner.slots[index] = transform.multiply(&owner.slots[index]);
        }
        Ok(())
    }
}

// Preserve the two native operand orders even where Clippy treats floating
// addition as commutative. Do not silently select one SIMD branch's ordering.
#[allow(clippy::if_same_then_else)]
fn elbow_pivot([x, y, z]: [f64; 3], angle: f32, wing: bool) -> NativeDpBoneTransform {
    let (xz, yz, zz) = (x * 0., y * 0., z * 0.);
    let xy_zero = if wing { xz + yz } else { yz + xz };
    let mut rotation = NativeDpBoneTransform::identity();
    rotation.matrix_00[12] = ((yz + x) + zz) + 0.;
    rotation.matrix_00[13] = ((xz + y) + zz) + 0.;
    rotation.matrix_00[14] = (z + xy_zero) + 0.;
    rotation.matrix_00[15] = (zz + xy_zero) + 1.;
    rotation.rotate_x(f64::from(angle));
    let m = rotation.matrix_00;
    for row in 0..4 {
        rotation.matrix_00[12 + row] = if wing {
            ((-y * m[4 + row] + -x * m[row]) + -z * m[8 + row]) + m[12 + row]
        } else {
            ((-x * m[row] + -y * m[4 + row]) + -z * m[8 + row]) + m[12 + row]
        };
    }
    rotation.field_100 = 0;
    rotation
}

#[cfg(test)]
mod tests;
