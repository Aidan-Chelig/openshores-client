//! Complete PositionTorso selection and its real body-wide RotateWaist call.
use super::*;
use crate::world::native_creature::{
    NativeDpBodyPoseState, NativePoseRhythmTimer, native_dh_pose_rhythm,
};
use std::io;

const FIXED: f32 = f32::from_bits(0x3fc2_5c20);
const FALLBACK: f32 = f32::from_bits(0x3fbd_e44f);
const NEGATIVE: f32 = f32::from_bits(0xbe32_b8c2);
const QUARTER_PI: f32 = f32::from_bits(0x3f49_0fdb);
const SMALL: f32 = f32::from_bits(0x3cd6_7750);
const TINY: f32 = f32::from_bits(0x3c0e_fa35);
const MEDIUM: f32 = f32::from_bits(0x3d32_b8c2);
const FIFTEEN: f32 = f32::from_bits(0x3e86_0a92);
const TWENTY_FIVE: f32 = f32::from_bits(0x3edf_66f3);

fn or_zero(base: f32, replacement: f32) -> f32 {
    if base == 0. { replacement } else { base }
}

fn angle(
    dna: &NativeDhDna,
    pose: &NativeDpBodyPoseState,
    mut base: f32,
    mut speed: f32,
    timer: &mut impl NativePoseRhythmTimer,
) -> io::Result<f32> {
    let selected = pose.fields_3c[8];
    let amplitude = match selected {
        3 | 5..=12 | 14 | 15 | 26 | 35 | 57..=73 => return Ok(FIXED),
        13 | 16 | 36 | 54 => return Ok(or_zero(base, FALLBACK)),
        17 | 22 | 38 => {
            base = or_zero(base, FALLBACK);
            SMALL
        }
        19 | 20 | 77 | 78 | 81 | 82 => return Ok(or_zero(base, NEGATIVE)),
        23 => {
            base = or_zero(base, FIFTEEN);
            speed = 1.;
            f32::from_bits(0x3d0e_fa35)
        }
        24 => {
            base = if base == 0. {
                f32::from_bits(0x3e0e_fa35)
            } else {
                FIFTEEN
            };
            speed = 1.;
            MEDIUM
        }
        25 => {
            base = if base == 0. {
                f32::from_bits(0x3c8e_fa35)
            } else {
                f32::from_bits(0x3db2_b8c2)
            };
            speed = 1.;
            f32::from_bits(0x3d7a_35dd)
        }
        27 | 53 => {
            base = if base == 0. { FIFTEEN } else { TWENTY_FIVE };
            speed = 1.;
            SMALL
        }
        28 => {
            base = if base == 0. { QUARTER_PI } else { base * 0.5 };
            f32::from_bits(0x3d56_7750)
        }
        29 | 39 => {
            base = f32::from_bits(0x3fcb_4bc4);
            speed = 1.;
            SMALL
        }
        37 => {
            base = or_zero(base, f32::from_bits(0x3fa7_8d36));
            MEDIUM
        }
        40 | 137 => return Ok(f32::from_bits(0x3fb2_b8c3)),
        41 => {
            let general = pose.fields_3c[3];
            base = if general < 54 && (0xffdf_ffff_f4ff_ffff_u64 >> general) & 1 == 0 {
                base - f32::from_bits(0x3e32_b8c2)
            } else if matches!(general, 42 | 43 | 45) {
                0.
            } else {
                QUARTER_PI
            };
            speed = 1.;
            TWENTY_FIVE
        }
        42 | 43 => {
            base = 0.;
            speed = 1.;
            if selected == 42 { SMALL } else { MEDIUM }
        }
        45 => {
            base = if base == 0. { FIXED } else { 0. };
            speed = 1.;
            TINY
        }
        46 => {
            base = f32::from_bits(0x3fc4_9809);
            speed = 1.;
            f32::from_bits(0x3c8e_fa35)
        }
        49 | 52 | 108 => {
            base = 0.;
            speed = 1.;
            TINY
        }
        50 | 51 => {
            base = or_zero(base, NEGATIVE);
            speed = 1.;
            TINY
        }
        74 | 75 => return Ok(f32::from_bits(0x3fc9_0fdb)),
        84..=107 => {
            let bit = 1_u64 << (selected - 84);
            return Ok(if bit & 0x333333 != 0 {
                FIXED
            } else {
                or_zero(base, FALLBACK)
            });
        }
        109..=159 => {
            let bit = 1_u64 << (selected - 109);
            return Ok(if bit & 0x7ffffe2733333 != 0 {
                FIXED
            } else if bit & 0x58ccccc != 0 {
                or_zero(base, FALLBACK)
            } else {
                0.
            });
        }
        _ => return Ok(0.), // Genuine native full-width default.
    };
    let wave = native_dh_pose_rhythm(dna, selected, 0., speed, timer)?;
    Ok(base + wave * amplitude)
}

impl NativeDpInitialization {
    /// Native pose words5c/48, actual caller height/speed and mounted hand
    /// dependency. Returns the angle actually applied, not a proposed angle.
    pub fn position_torso(
        &mut self,
        pose: &NativeDpBodyPoseState,
        height: f32,
        speed: f32,
        field_4068: i32,
        hands: &mut NativeDpHandCache<'_>,
        timer: &mut impl NativePoseRhythmTimer,
    ) -> io::Result<f32> {
        // The native geometric prelude precedes even a no-op pose selection.
        let base = hands.torso_base_angle(&self.dna, &self.dimensions, field_4068, height)?;
        let selected = angle(&self.dna, pose, base, speed, timer)?;
        if self.rotate_waist(selected)? {
            Ok(selected)
        } else {
            Ok(0.)
        }
    }
}

#[cfg(test)]
mod tests;
