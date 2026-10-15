use super::*;

impl NativeDpPartState {
    /// Native head MoveBy premultiplies an expanded translation into the
    /// inclusive head/neck range. Zero displacement is an actual no-op.
    pub fn move_head_by(
        &self,
        owner: &mut NativeDpBoneTransforms,
        [x, y, z]: [f64; 3],
    ) -> io::Result<()> {
        if self.allocation.kind != NativeDpPartKind::HeadAndNeck {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "native head MoveBy requires HeadAndNeck",
            ));
        }
        if x == 0. && y == 0. && z == 0. {
            return Ok(());
        }
        let yz = y * 0.;
        let zz = z * 0.;
        let xz = x * 0.;
        let t = [
            ((x + yz) + zz) + 0.,
            ((y + xz) + zz) + 0.,
            (z + (xz + yz)) + 0.,
            (zz + (xz + yz)) + 1.,
        ];
        let start = 7 + usize::from(self.allocation.index == 1) * 5;
        let raw = &self.allocation.dna.0;
        let count = usize::from((raw[14] & 3) + ((raw[5] >> 1) & 1));
        for bone in &mut owner.slots[start..=start + count] {
            let m = bone.matrix_00;
            for column in 0..4 {
                let k = column * 4;
                let xz = m[k] * 0.;
                let yz = m[k + 1] * 0.;
                let zz = m[k + 2] * 0.;
                let w = m[k + 3];
                // Preserve the operand order of the native scalar additions.
                #[allow(clippy::if_same_then_else)]
                let first = if column == 1 || column == 2 {
                    yz + m[k]
                } else {
                    m[k] + yz
                };
                bone.matrix_00[k] = (first + zz) + w * t[0];
                bone.matrix_00[k + 1] = ((m[k + 1] + xz) + zz) + w * t[1];
                bone.matrix_00[k + 2] = if column == 3 {
                    (t[2] * w + m[k + 2]) + (yz + xz)
                } else {
                    (m[k + 2] + (xz + yz)) + w * t[2]
                };
                bone.matrix_00[k + 3] = if column == 0 {
                    (w * t[3] + (xz + yz)) + zz
                } else {
                    ((xz + yz) + zz) + w * t[3]
                };
            }
            bone.field_100 = 0;
        }
        Ok(())
    }
}
