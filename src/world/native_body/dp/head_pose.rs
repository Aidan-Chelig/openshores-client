//! Concrete TiltHeadTo composition; body-level PositionHeads remains separate.
use super::{
    NativeDpBoneTransform, NativeDpBoneTransforms, NativeDpDimensions, NativeDpPartJoints,
    NativeDpPartKind, NativeDpPartState,
};
use std::io;
mod movement;
mod orientation;

impl NativeDpPartState {
    /// Actual local head pose sequence. No pose-word dispatch, world placement
    /// or renderer mesh publication is implied by completing this operation.
    pub fn tilt_head_to(
        &self,
        owner: &mut NativeDpBoneTransforms,
        dimensions: &NativeDpDimensions,
        pitch: f32,
        roll: f32,
    ) -> io::Result<()> {
        if self.allocation.kind != NativeDpPartKind::HeadAndNeck {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "native TiltHeadTo requires HeadAndNeck",
            ));
        }
        let NativeDpPartJoints::Head {
            field_50: head_point,
        } = self.joints
        else {
            // Actual native constructors always supply these fields. Reject a
            // malformed Rust state before reset, instead of inventing a point.
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "missing native head joint state",
            ));
        };
        let start = 7 + usize::from(self.allocation.index == 1) * 5;
        let raw = &self.allocation.dna.0;
        let count = (raw[14] & 3) + ((raw[5] >> 1) & 1);
        let before = self.current_connect_loc(owner)?;
        owner.slots[start..=start + usize::from(count)].fill(NativeDpBoneTransform::identity());
        let after = self.current_connect_loc(owner)?;
        self.move_head_by(owner, std::array::from_fn(|i| before[i] - after[i]))?;

        let remaining = if let Some(angle) = orientation::correction(owner, raw[20] & 0xe0 != 0) {
            self.rotate_neck(owner, angle)?;
            pitch - angle
        } else {
            pitch
        };
        let bend = (remaining + f32::from_bits(0x3fc90fdb)) / f32::from(count + 1);
        self.bend_neck(owner, dimensions, bend)?;

        let current = owner.slots[start].project(head_point);
        let mut head = NativeDpBoneTransform::identity();
        head.translate(current);
        head.rotate_x(f64::from(pitch));
        head.rotate_z(f64::from(roll));
        head.translate(head_point.map(|v| -v));
        owner.slots[start] = head;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
