//! CurrentWaistLoc and concrete torso RotateWaist, not the body-wide wrapper.
use super::*;
use std::io;

impl NativeDpInitialization {
    /// Native examines ONLY the first part, returning zero when it is absent
    /// or not a torso. It does not search later parts for a replacement.
    pub fn current_waist_loc(&self) -> io::Result<[f64; 3]> {
        match self.parts.first() {
            Some(part) if part.allocation.kind == NativeDpPartKind::Torso => {
                part.current_connect_loc(&self.bones)
            }
            _ => Ok([0.; 3]),
        }
    }
}

impl NativeDpPartState {
    /// Rotates only bones0..WaistVertex-1 around the torso's current attachment.
    /// The body's separate head/arm follow-up stage is NOT implied here.
    pub fn rotate_torso_waist(
        &self,
        owner: &mut NativeDpBoneTransforms,
        angle: f32,
    ) -> io::Result<()> {
        if self.allocation.kind != NativeDpPartKind::Torso {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "native torso RotateWaist requires Torso",
            ));
        }
        if angle == 0. {
            return Ok(());
        }
        let count = native_dh_waist_vertex(&self.allocation.dna);
        if count == 0 {
            return Ok(());
        }
        let pivot = self.current_connect_loc(owner)?;
        let transform = pivot_rotation(pivot, angle);
        for bone in &mut owner.slots[..count as usize] {
            apply(bone, &transform);
        }
        Ok(())
    }
}

fn pivot_rotation([x, y, z]: [f64; 3], angle: f32) -> NativeDpBoneTransform {
    let xz = x * 0.;
    let yz = y * 0.;
    let zz = z * 0.;
    let mut transform = NativeDpBoneTransform::identity();
    transform.matrix_00[12] = ((yz + x) + zz) + 0.;
    transform.matrix_00[13] = ((xz + y) + zz) + 0.;
    transform.matrix_00[14] = (z + (yz + xz)) + 0.;
    transform.matrix_00[15] = (zz + (yz + xz)) + 1.;
    transform.rotate_x(f64::from(angle));
    let m = transform.matrix_00;
    for row in 0..3 {
        transform.matrix_00[12 + row] =
            ((m[row] * -x + m[4 + row] * -y) + m[8 + row] * -z) + m[12 + row];
    }
    // Fourth row has native Y/X/Z order, unlike Translate's usual X/Y/Z.
    transform.matrix_00[15] = ((-y * m[7] + -x * m[3]) + -z * m[11]) + m[15];
    transform
}

fn apply(bone: &mut NativeDpBoneTransform, transform: &NativeDpBoneTransform) {
    let r = &transform.matrix_00;
    let m = bone.matrix_00;
    for col in 0..4 {
        let k = col * 4;
        let [x, y, z, w] = [m[k], m[k + 1], m[k + 2], m[k + 3]];
        bone.matrix_00[k] = if col < 2 {
            ((x * r[0] + y * r[4]) + r[8] * z) + w * r[12]
        } else {
            ((r[8] * z + r[4] * y) + r[0] * x) + w * r[12]
        };
        bone.matrix_00[k + 1] = ((r[9] * z + r[5] * y) + r[1] * x) + r[13] * w;
        bone.matrix_00[k + 2] = ((r[10] * z + r[6] * y) + r[2] * x) + r[14] * w;
        bone.matrix_00[k + 3] = if col == 3 {
            ((w * r[15] + x * r[3]) + y * r[7]) + z * r[11]
        } else {
            ((w * r[15] + z * r[11]) + y * r[7]) + x * r[3]
        };
    }
    bone.field_100 = 0;
}

#[cfg(test)]
mod tests;
