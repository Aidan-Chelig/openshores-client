//! Full DpBody::PositionTail pose dispatch; every branch calls real transforms.
use super::*;
use crate::world::native_creature::{NativeDpBodyPoseState, NativePoseRhythmTimer};
use std::io;

impl NativeDpInitialization {
    /// Uses owner pose words48/58, original owner leg DNA, and the actual
    /// caller's clearance/speed/byte3a. This is not complete body Animate.
    pub fn position_tail(
        &mut self,
        pose: &NativeDpBodyPoseState,
        clearance: f32,
        speed: f32,
        field_3a: bool,
        timer: &mut impl NativePoseRhythmTimer,
    ) -> io::Result<()> {
        if !self.parts_calculated() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "native PositionTail requires completed part calculations",
            ));
        }
        let tail_pose = pose.fields_3c[7];
        let legs = self.dna.0[12] & 0xc != 0;
        let override_clearance = if matches!(pose.fields_3c[3], 42 | 43 | 45) {
            -1.
        } else {
            clearance
        };
        match tail_pose {
            3 | 5..=16 | 32 | 36 | 54 | 57..=74 | 84..=107 | 109..=135 | 138..=159 => {
                if legs {
                    self.relax_tail(clearance)
                } else {
                    self.slither_tail_pose(pose, tail_pose, 1., timer)
                }
            }
            4 | 136 => self.receptive_tail_facing_up(timer),
            17 | 22 | 28 | 33 | 38 | 75 => {
                if legs {
                    self.wave_tail(pose, clearance, speed, timer)
                } else {
                    self.slither_tail_pose(pose, tail_pose, speed, timer)
                }
            }
            18 | 21 | 76 | 79 | 80 | 83 => self.dead_tail_facing_down(),
            19 | 20 | 77 | 78 | 81 | 82 => self.dead_tail_facing_left(),
            23 | 47 => self.wag_tail(override_clearance, timer),
            24 | 25 | 27 | 53 => {
                if legs {
                    self.wave_tail(pose, clearance, speed, timer)
                } else {
                    self.slither_tail_pose(pose, tail_pose, 1., timer)
                }
            }
            26 | 35 | 40 | 137 => self.fold_tail(field_3a),
            29 | 30 => self.lash_tail(override_clearance, 1., timer),
            37 => {
                if legs {
                    self.lash_tail(clearance, speed, timer)
                } else {
                    self.slither_tail_pose(pose, tail_pose, speed, timer)
                }
            }
            41 => self.strike_tail(pose, override_clearance, timer),
            42 | 43 | 45 => self.swim_tail(pose, timer),
            49 => self.writhe_tail_facing_down(timer),
            50 => self.writhe_tail_facing_left(timer),
            51 => self.writhe_tail_facing_right(timer),
            52 | 108 => self.writhe_tail_facing_up(timer),
            _ => Ok(()), // Actual native default, including full-width unknowns.
        }
    }
}
