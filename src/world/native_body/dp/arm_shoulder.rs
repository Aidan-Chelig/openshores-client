//! Concrete RotateShoulder: current attachment, native expanded pivot and
//! actual Bones out-reference order. This is not complete wing animation.
use super::*;
use std::io;

impl NativeDpPartState {
    pub fn rotate_arm_shoulder(
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
                "native RotateShoulder requires ArmAndHand",
            ));
        }
        if angle == 0. {
            return Ok(());
        }
        let indices = self.limb_bones()?;
        let transform = shoulder_pivot(self.current_connect_loc(owner)?, angle);
        let raw = &self.allocation.dna.0;
        let all = raw[1] & 3 == 0 || raw[2] & 6 == 0 || (raw[2] & 7).wrapping_sub(5) < 2;
        for &index in &indices[..if all { 3 } else { 1 }] {
            // Every native path uses the primary-only matrix operator*. Its
            // unconditional zero flag makes the secondary-copy path dead.
            owner.slots[index] = transform.multiply(&owner.slots[index]);
        }
        Ok(())
    }
}

pub(super) fn shoulder_pivot([x, y, z]: [f64; 3], angle: f32) -> NativeDpBoneTransform {
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
    let ty = initial_y + ((d * -y - xz) + e * -z);
    let tz = ((f * -y - xz) + -z * g) + initial_z;
    let tw = ((a * -y - xz) + -z * b) + initial_w;
    NativeDpBoneTransform {
        matrix_00: [1., 0., 0., 0., a, d, f, a, b, e, g, b, tx, ty, tz, tw],
        field_100: 0,
    }
}

#[cfg(test)]
mod tests;
