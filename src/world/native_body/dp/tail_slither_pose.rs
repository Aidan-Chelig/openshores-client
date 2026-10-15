//! DpTail::Slither(AuPose const&,float): native gates precede concrete mutation.
use super::*;
use crate::world::native_creature::{
    NativeDpBodyPoseState, NativePoseRhythmTimer, native_dh_pose_rhythm,
};
use std::io;

fn rhythmic(pose: u32) -> bool {
    matches!(
        pose,
        17 | 22 | 24 | 25 | 26 | 28 | 33 | 37 | 38 | 42 | 43 | 75
    )
}

impl NativeDpPartState {
    /// `body_pose` supplies the actual owner's word48 (index3), not the passed
    /// tail-pose word. Rhythm uses this part's copied DNA, as in the native call.
    pub fn slither_tail_for_pose(
        &self,
        owner: &mut NativeDpBoneTransforms,
        dimensions: &NativeDpDimensions,
        tail_pose: u32,
        body_pose: &NativeDpBodyPoseState,
        speed: f32,
        timer: &mut impl NativePoseRhythmTimer,
    ) -> io::Result<()> {
        if self.allocation.kind != NativeDpPartKind::Tail {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "native pose Slither requires Tail",
            ));
        }
        if self.allocation.dna.0[14] & 0xc0 == 0 {
            return Ok(());
        }
        let amplitude = if rhythmic(tail_pose) {
            let wave = native_dh_pose_rhythm(
                &self.allocation.dna,
                tail_pose,
                f32::from_bits(0x40490fdb),
                speed,
                timer,
            )?;
            let value = wave * f32::from_bits(0x3f060a92);
            if value == 0. {
                return Ok(());
            }
            value
        } else {
            if !self.tail_can_bend_laterally()? {
                return Ok(());
            }
            f32::from_bits(0x3f1c61aa)
        };
        let general = body_pose.fields_3c[3];
        let (pitch, yaw) = if matches!(general, 24 | 25 | 27 | 42 | 43 | 45 | 53) {
            if self.tail_can_bend_vertically()? {
                (0., amplitude)
            } else {
                (amplitude, 0.)
            }
        } else {
            if !self.tail_can_bend_laterally()? {
                return Ok(());
            }
            (0., amplitude)
        };
        self.slither_tail(owner, dimensions, pitch, yaw)
    }
}

impl NativeDpInitialization {
    /// Body SlitherTail stage: selects the first actual tail and uses the
    /// supplied pose (which PositionTail passes from word58), speed and owner.
    pub fn slither_tail_pose(
        &mut self,
        body_pose: &NativeDpBodyPoseState,
        tail_pose: u32,
        speed: f32,
        timer: &mut impl NativePoseRhythmTimer,
    ) -> io::Result<()> {
        if !self.parts_calculated() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "native body SlitherTail requires completed part calculations",
            ));
        }
        if let Some(tail) = self
            .parts
            .iter()
            .find(|p| p.allocation.kind == NativeDpPartKind::Tail)
        {
            tail.slither_tail_for_pose(
                &mut self.bones,
                &self.dimensions,
                tail_pose,
                body_pose,
                speed,
                timer,
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
