//! Actual virtual arm MoveBy: two operator* calls and one inlined product.
use super::*;
use std::io;
impl NativeDpPartState {
    pub fn move_arm_by(
        &self,
        owner: &mut NativeDpBoneTransforms,
        [x, y, z]: [f64; 3],
    ) -> io::Result<()> {
        if !matches!(
            self.allocation.kind,
            NativeDpPartKind::LeftArmAndHand | NativeDpPartKind::RightArmAndHand
        ) {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "native arm MoveBy requires ArmAndHand",
            ));
        }
        let [first, second, third] = self.limb_bones()?;
        let mut translation = NativeDpBoneTransform::identity();
        let xz = x * 0.;
        let yz = y * 0.;
        let zz = z * 0.;
        translation.matrix_00[12] = ((yz + x) + zz) + 0.;
        translation.matrix_00[13] = ((xz + y) + zz) + 0.;
        translation.matrix_00[14] = (z + (yz + xz)) + 0.;
        translation.matrix_00[15] = (zz + (yz + xz)) + 1.;
        owner.slots[first] = translation.multiply(&owner.slots[first]);
        owner.slots[second] = translation.multiply(&owner.slots[second]);
        root_product(&mut owner.slots[third], &translation);
        Ok(())
    }
}
fn root_product(bone: &mut NativeDpBoneTransform, translation: &NativeDpBoneTransform) {
    let a = &translation.matrix_00;
    let m = bone.matrix_00;
    for col in 0..4 {
        let k = col * 4;
        let [x, y, z, w] = [m[k], m[k + 1], m[k + 2], m[k + 3]];
        bone.matrix_00[k] = match col {
            0 => ((a[12] * w + a[8] * z) + a[0] * x) + a[4] * y,
            3 => ((a[0] * x + a[4] * y) + a[8] * z) + a[12] * w,
            _ => ((a[8] * z + a[4] * y) + a[0] * x) + a[12] * w,
        };
        bone.matrix_00[k + 1] = match col {
            0 => ((a[13] * w + a[9] * z) + a[1] * x) + a[5] * y,
            3 => ((a[1] * x + a[5] * y) + a[9] * z) + a[13] * w,
            _ => ((a[9] * z + a[5] * y) + a[1] * x) + a[13] * w,
        };
        for row in 2..4 {
            bone.matrix_00[k + row] = match col {
                2 => ((y * a[4 + row] + x * a[row]) + z * a[8 + row]) + w * a[12 + row],
                3 => ((x * a[row] + y * a[4 + row]) + a[8 + row] * z) + a[12 + row] * w,
                _ => ((z * a[8 + row] + y * a[4 + row]) + x * a[row]) + w * a[12 + row],
            };
        }
    }
    bone.field_100 = 0;
}

#[cfg(test)]
mod tests;
