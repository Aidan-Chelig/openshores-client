//! Concrete RotateNeck/LeanNeck, not complete TiltHeadTo/PositionHeads.
use super::{NativeDpBoneTransform, NativeDpBoneTransforms, NativeDpPartKind, NativeDpPartState};
use std::io;

impl NativeDpPartState {
    /// RotateX about the current attachment, premultiplied into all bones in
    /// this head's inclusive native range. Caller supplies the actual owner.
    pub fn rotate_neck(&self, owner: &mut NativeDpBoneTransforms, angle: f32) -> io::Result<()> {
        self.transform_neck(owner, angle, false)
    }

    /// Native LeanNeck uses RotateY about the current attachment, NOT RotateZ.
    /// Static comparison proves the remaining instruction sequence is shared.
    pub fn lean_neck(&self, owner: &mut NativeDpBoneTransforms, angle: f32) -> io::Result<()> {
        self.transform_neck(owner, angle, true)
    }

    fn transform_neck(
        &self,
        owner: &mut NativeDpBoneTransforms,
        angle: f32,
        lean: bool,
    ) -> io::Result<()> {
        if self.allocation.kind != NativeDpPartKind::HeadAndNeck {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "native neck transform requires HeadAndNeck",
            ));
        }
        // Actual wrapper skips both +0/-0, but not NaN. In particular it does
        // not read attachment matrices or clear any flags on the zero path.
        if angle == 0. {
            return Ok(());
        }
        let pivot = self.current_connect_loc(owner)?;
        let mut transform = NativeDpBoneTransform::identity();
        transform.translate(pivot);
        if lean {
            transform.rotate_y(f64::from(angle));
        } else {
            transform.rotate_x(f64::from(angle));
        }
        transform.translate(pivot.map(|v| -v));
        let start = 7 + usize::from(self.allocation.index == 1) * 5;
        let raw = &self.allocation.dna.0;
        let count = usize::from((raw[14] & 3) + ((raw[5] >> 1) & 1));
        for bone in &mut owner.slots[start..=start + count] {
            premultiply_rotate_neck(bone, &transform.matrix_00);
        }
        Ok(())
    }
}

// This is the RotateNeck inline multiply's observed instruction grouping, not
// a generic matrix multiplication contract. Rows1/2 add Z to (Y+X), and row3
// adds W after that triple; normalizing it to a left fold changes rounding.
#[allow(clippy::if_same_then_else)] // Preserve the native column-specific operand expressions.
fn premultiply_rotate_neck(bone: &mut NativeDpBoneTransform, r: &[f64; 16]) {
    let m = bone.matrix_00;
    for column in 0..4 {
        let k = column * 4;
        let x = r[0] * m[k];
        let y = r[4] * m[k + 1];
        let z = r[8] * m[k + 2];
        let w = r[12] * m[k + 3];
        bone.matrix_00[k] = if column < 2 {
            ((x + y) + z) + w
        } else {
            (z + (y + x)) + w
        };
        for row in 1..3 {
            let x = r[row] * m[k];
            let y = r[4 + row] * m[k + 1];
            let z = r[8 + row] * m[k + 2];
            let w = r[12 + row] * m[k + 3];
            bone.matrix_00[k + row] = (z + (y + x)) + w;
        }
        let x = r[3] * m[k];
        let y = r[7] * m[k + 1];
        let z = r[11] * m[k + 2];
        let w = r[15] * m[k + 3];
        bone.matrix_00[k + 3] = if column == 3 {
            w + ((x + y) + z)
        } else {
            w + (z + (y + x))
        };
    }
    bone.field_100 = 0;
}

#[cfg(test)]
mod tests;
