//! DpHeadAndNeck::BendNeck: descending joints and inclusive bone prefixes.
use super::{
    NativeDpBoneTransform, NativeDpBoneTransforms, NativeDpDimensions, NativeDpPartKind,
    NativeDpPartState,
};
use std::io;

struct Bend {
    a: f64,
    b: f64,
    cos_plus: f64,
    cos_minus: f64,
    sin_plus: f64,
    negative_sin: f64,
}

impl NativeDpPartState {
    /// Dimensions must be from the actual owner body. Joint spacing is rounded
    /// to f32 before widening, not recomputed from the copied part DNA.
    pub fn bend_neck(
        &self,
        owner: &mut NativeDpBoneTransforms,
        dimensions: &NativeDpDimensions,
        angle: f32,
    ) -> io::Result<()> {
        if self.allocation.kind != NativeDpPartKind::HeadAndNeck {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "native BendNeck requires HeadAndNeck",
            ));
        }
        if angle == 0. {
            return Ok(());
        }
        let start = 7 + usize::from(self.allocation.index == 1) * 5;
        let raw = &self.allocation.dna.0;
        let end = start + usize::from((raw[14] & 3) + ((raw[5] >> 1) & 1));
        let step = f64::from(dimensions.fields_4080_through_40b8()[10] * 0.5_f32);
        let mut point = self.connect_30;
        let bend = Bend::new(angle);
        for joint in (start..=end).rev() {
            // Prior stages have already changed these bones. Do not snapshot
            // all joint pivots before the loop or reverse the traversal.
            let pivot = owner.slots[joint].project(point);
            let translation = bend.about(pivot);
            for bone in &mut owner.slots[start..=joint] {
                bend.apply(bone, translation);
            }
            let next_joint_y = step + point[1];
            point[1] = next_joint_y;
        }
        Ok(())
    }
}

impl Bend {
    fn new(angle: f32) -> Self {
        let (s, c) = f64::from(angle).sin_cos();
        let cz = c * 0.;
        let sz = s * 0.;
        Self {
            a: cz + sz,
            b: cz - sz,
            cos_plus: c + sz,
            cos_minus: c - sz,
            sin_plus: s + cz,
            negative_sin: cz - s,
        }
    }

    // Expanded native pivot construction, including explicit zero products and
    // addition grouping. It is not replaced by an affine pivot shortcut.
    fn about(&self, [x, y, z]: [f64; 3]) -> [f64; 4] {
        let xz = x * 0.;
        let yz = y * 0.;
        let zz = z * 0.;
        let sum = yz + xz;
        let negative_y = -y;
        let negative_z = -z;
        [
            (((yz + x) + zz) + 0.) + ((self.a * negative_y - x) + self.b * negative_z),
            (self.negative_sin * negative_z + (self.cos_plus * negative_y - xz))
                + (((y + xz) + zz) + 0.),
            (self.cos_minus * negative_z + (self.sin_plus * negative_y - xz)) + ((z + sum) + 0.),
            ((self.a * negative_y - xz) + self.b * negative_z) + ((zz + sum) + 1.),
        ]
    }

    fn apply(&self, bone: &mut NativeDpBoneTransform, t: [f64; 4]) {
        let m = bone.matrix_00;
        for column in 0..4 {
            let k = column * 4;
            let [x, y, z, w] = [m[k], m[k + 1], m[k + 2], m[k + 3]];
            let xz = x * 0.;
            bone.matrix_00[k] = (((y * self.a) + x) + z * self.b) + w * t[0];
            bone.matrix_00[k + 1] = (self.negative_sin * z + (self.cos_plus * y + xz)) + w * t[1];
            bone.matrix_00[k + 2] = (z * self.cos_minus + (y * self.sin_plus + xz)) + w * t[2];
            bone.matrix_00[k + 3] = ((y * self.a + xz) + z * self.b) + w * t[3];
        }
        bone.field_100 = 0;
    }
}

#[cfg(test)]
mod tests;
