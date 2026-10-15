//! DpBody::PositionHeads: both pose selections precede all bone writes.
use super::*;
use crate::world::native_creature::{
    NativeDpBodyPoseState, NativePoseRhythmTimer, native_dh_pose_rhythm,
};
use std::io;

const HALF_PI: f32 = f32::from_bits(0x3fc9_0fdb);
const SMALL_ARC: f32 = f32::from_bits(0x3cd6_7750);
const DEGREE: f32 = f32::from_bits(0x3c8e_fa35);
const SPREAD: f32 = f32::from_bits(0x3f32_b8c2);

fn pitch(
    dna: &NativeDhDna,
    pose: u32,
    base: f32,
    second: bool,
    timer: &mut impl NativePoseRhythmTimer,
) -> io::Result<f32> {
    Ok(match pose {
        4 | 0x88 => base - HALF_PI,
        0x12..=0x15 | 0x4c..=0x53 => -HALF_PI,
        0x31..=0x34 | 0x6c => {
            let phase = if second {
                f32::from_bits(0x3e86_0a92)
            } else {
                0.
            };
            let wave = native_dh_pose_rhythm(dna, pose, phase, 1., timer)?;
            if second {
                wave * SMALL_ARC
            } else {
                wave * SMALL_ARC - HALF_PI
            }
        }
        0x17 | 0x38 => {
            let phase = if second {
                f32::from_bits(0x4049_0fdb)
            } else {
                0.
            };
            let wave = native_dh_pose_rhythm(dna, pose, phase, 1., timer)?;
            let offset = (wave * 10. - 10.) * DEGREE;
            if second {
                base + (offset - HALF_PI)
            } else {
                offset + base
            }
        }
        _ => base,
    })
}

impl NativeDpInitialization {
    /// Position initialized heads using the actual pose words at40/54 and
    /// body angles at30/34. This is one animation stage, not full Animate.
    pub fn position_heads(
        &mut self,
        pose: &NativeDpBodyPoseState,
        pitch_30: f32,
        roll_34: f32,
        timer: &mut impl NativePoseRhythmTimer,
    ) -> io::Result<()> {
        if !self.parts_calculated() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "native head positioning requires completed part calculations",
            ));
        }
        // Native computes the second pose even when DNA has only one head.
        let first = pitch(&self.dna, pose.fields_3c[1], pitch_30, false, timer)?;
        let second = pitch(&self.dna, pose.fields_3c[6], pitch_30, true, timer)?;
        let heads = self
            .parts
            .iter()
            .filter(|part| part.allocation.kind == NativeDpPartKind::HeadAndNeck);
        if self.dna.0[5] & 2 == 0 {
            if let Some(head) = heads.into_iter().next() {
                head.tilt_head_to(&mut self.bones, &self.dimensions, first, roll_34)?;
            }
            return Ok(());
        }
        for (i, head) in heads.take(2).enumerate() {
            head.tilt_head_to(
                &mut self.bones,
                &self.dimensions,
                if i == 0 { first } else { second },
                roll_34,
            )?;
        }
        let mut lean = -(f32::from((self.dna.0[5] >> 1) & 1) * SPREAD) * 0.5;
        for head in self
            .parts
            .iter()
            .filter(|part| part.allocation.kind == NativeDpPartKind::HeadAndNeck)
            .take(2)
        {
            head.lean_neck(&mut self.bones, lean)?;
            lean += SPREAD;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
