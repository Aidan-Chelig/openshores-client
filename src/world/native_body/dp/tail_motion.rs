//! Actual body Swim/Wave/Lash/Wag/RelaxTail composition, not PositionTail.
use super::*;
use crate::world::native_creature::{
    NativeDpBodyPoseState, NativePoseRhythmTimer, native_dh_pose_rhythm,
};
use std::io;

#[derive(Clone, Copy)]
enum Motion {
    Swim,
    Wave,
    Lash,
    Wag,
}

fn angles(
    part: &NativeDpPartState,
    motion: Motion,
    general_pose: u32,
    speed: f32,
    timer: &mut impl NativePoseRhythmTimer,
) -> io::Result<Option<[f32; 2]>> {
    let count = (part.allocation.dna.0[14] >> 6) * 2;
    if count == 0 {
        return Ok(None);
    }
    let (pose, arc, speed) = match motion {
        Motion::Swim => (general_pose, f32::from_bits(0x3eb2b8c2), 1.),
        Motion::Wave => (general_pose, f32::from_bits(0x3e567750), speed),
        Motion::Lash => (30, f32::from_bits(0x3f060a92), speed),
        Motion::Wag => (47, f32::from_bits(0x3e860a92), 1.),
    };
    let wave = native_dh_pose_rhythm(
        &part.allocation.dna,
        pose,
        f32::from_bits(0x40490fdb),
        speed,
        timer,
    )?;
    let amplitude = (arc / f32::from(count)) * wave;
    if amplitude == 0. {
        return Ok(None);
    }
    if !part.tail_can_bend_laterally()? {
        return Ok(Some([amplitude, 0.]));
    }
    let pitch = if matches!(motion, Motion::Wag) && part.tail_can_bend_vertically()? {
        -f32::from_bits(0x3e860a92)
    } else {
        0.
    };
    Ok(Some([pitch, amplitude]))
}

impl NativeDpInitialization {
    pub fn swim_tail(
        &mut self,
        pose: &NativeDpBodyPoseState,
        timer: &mut impl NativePoseRhythmTimer,
    ) -> io::Result<()> {
        self.rhythmic_tail(Motion::Swim, pose.fields_3c[3], 1., -1., timer)
    }
    pub fn wave_tail(
        &mut self,
        pose: &NativeDpBodyPoseState,
        clearance: f32,
        speed: f32,
        timer: &mut impl NativePoseRhythmTimer,
    ) -> io::Result<()> {
        self.rhythmic_tail(Motion::Wave, pose.fields_3c[3], speed, clearance, timer)
    }
    pub fn lash_tail(
        &mut self,
        clearance: f32,
        speed: f32,
        timer: &mut impl NativePoseRhythmTimer,
    ) -> io::Result<()> {
        self.rhythmic_tail(Motion::Lash, 0, speed, clearance, timer)
    }
    pub fn wag_tail(
        &mut self,
        clearance: f32,
        timer: &mut impl NativePoseRhythmTimer,
    ) -> io::Result<()> {
        self.rhythmic_tail(Motion::Wag, 0, 1., clearance, timer)
    }
    fn rhythmic_tail(
        &mut self,
        motion: Motion,
        general: u32,
        speed: f32,
        clearance: f32,
        timer: &mut impl NativePoseRhythmTimer,
    ) -> io::Result<()> {
        self.require_calculated_tail_body()?;
        if let Some(tail) = self
            .parts
            .iter()
            .find(|p| p.allocation.kind == NativeDpPartKind::Tail)
            && let Some([pitch, yaw]) = angles(tail, motion, general, speed, timer)?
        {
            tail.bend_tail(&mut self.bones, &self.dimensions, pitch, yaw, clearance)?;
        }
        Ok(())
    }
    pub fn relax_tail(&mut self, clearance: f32) -> io::Result<()> {
        self.require_calculated_tail_body()?;
        if let Some(tail) = self
            .parts
            .iter()
            .find(|p| p.allocation.kind == NativeDpPartKind::Tail)
        {
            if tail.tail_can_bend_vertically()? && clearance > 0. {
                tail.bend_tail(
                    &mut self.bones,
                    &self.dimensions,
                    f32::from_bits(0x3e860a92),
                    0.,
                    clearance,
                )?;
            } else if tail.tail_can_bend_laterally()? {
                tail.slither_tail(
                    &mut self.bones,
                    &self.dimensions,
                    0.,
                    f32::from_bits(0x3e32b8c2),
                )?;
            }
        }
        Ok(())
    }
    fn require_calculated_tail_body(&self) -> io::Result<()> {
        if self.parts_calculated() {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "native body tail motion requires completed part calculations",
            ))
        }
    }
}

#[cfg(test)]
mod tests;
