//! Native strike, writhe and receptive body stages.
use super::*;
use crate::world::native_creature::{
    NativeDpBodyPoseState, NativePoseRhythmTimer, native_dh_pose_rhythm,
};
use std::io;

fn strike_angles(
    tail: &NativeDpPartState,
    general: u32,
    timer: &mut impl NativePoseRhythmTimer,
) -> io::Result<Option<[f32; 2]>> {
    let count = (tail.allocation.dna.0[14] >> 6) * 2;
    if count == 0 {
        return Ok(None);
    }
    let wave = native_dh_pose_rhythm(
        &tail.allocation.dna,
        41,
        f32::from_bits(0x40490fdb),
        1.,
        timer,
    )?;
    let angle = ((wave * 70. + 60.) * f32::from_bits(0x3c8efa35)) / f32::from(count);
    if angle == 0. {
        return Ok(None);
    }
    let special = matches!(general, 24 | 25 | 27 | 42 | 43 | 45 | 53);
    Ok(Some(if special {
        if tail.tail_can_bend_vertically()? {
            [0., angle]
        } else {
            [angle, 0.]
        }
    } else if tail.tail_can_bend_laterally()? {
        [0., angle]
    } else {
        [-angle, 0.]
    }))
}

impl NativeDpInitialization {
    pub fn strike_tail(
        &mut self,
        pose: &NativeDpBodyPoseState,
        clearance: f32,
        timer: &mut impl NativePoseRhythmTimer,
    ) -> io::Result<()> {
        self.require_special_tail_body()?;
        if let Some(tail) = self
            .parts
            .iter()
            .find(|p| p.allocation.kind == NativeDpPartKind::Tail)
            && let Some([pitch, yaw]) = strike_angles(tail, pose.fields_3c[3], timer)?
        {
            tail.bend_tail(&mut self.bones, &self.dimensions, pitch, yaw, clearance)?;
        }
        Ok(())
    }
    pub fn writhe_tail_facing_down(
        &mut self,
        timer: &mut impl NativePoseRhythmTimer,
    ) -> io::Result<()> {
        self.writhe_tail(49, false, timer)
    }
    pub fn writhe_tail_facing_left(
        &mut self,
        timer: &mut impl NativePoseRhythmTimer,
    ) -> io::Result<()> {
        self.writhe_tail(50, true, timer)
    }
    pub fn writhe_tail_facing_right(
        &mut self,
        timer: &mut impl NativePoseRhythmTimer,
    ) -> io::Result<()> {
        self.writhe_tail(51, true, timer)
    }
    pub fn writhe_tail_facing_up(
        &mut self,
        timer: &mut impl NativePoseRhythmTimer,
    ) -> io::Result<()> {
        self.writhe_tail(52, false, timer)
    }
    pub fn receptive_tail_facing_up(
        &mut self,
        timer: &mut impl NativePoseRhythmTimer,
    ) -> io::Result<()> {
        self.writhe_tail(136, false, timer)
    }
    fn writhe_tail(
        &mut self,
        pose: u32,
        vertical: bool,
        timer: &mut impl NativePoseRhythmTimer,
    ) -> io::Result<()> {
        self.require_special_tail_body()?;
        if let Some(tail) = self
            .parts
            .iter()
            .find(|p| p.allocation.kind == NativeDpPartKind::Tail)
        {
            let capable = if vertical {
                tail.tail_can_bend_vertically()?
            } else {
                tail.tail_can_bend_laterally()?
            };
            if capable {
                let wave = native_dh_pose_rhythm(&tail.allocation.dna, pose, 0., 1., timer)?;
                let angle = (wave * 1.5 + 15.) * f32::from_bits(0x3c8efa35);
                tail.slither_tail(
                    &mut self.bones,
                    &self.dimensions,
                    if vertical { angle } else { 0. },
                    if vertical { 0. } else { angle },
                )?;
            }
        }
        Ok(())
    }
    fn require_special_tail_body(&self) -> io::Result<()> {
        if self.parts_calculated() {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "native special tail motion requires completed part calculations",
            ))
        }
    }
}

#[cfg(test)]
mod tests;
