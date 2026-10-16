//! Complete normal Walk/Run stage composition in native AnimatePose order.
use super::*;
use crate::world::native_creature::{NativeDpBodyPoseState, NativePoseRhythmTimer};
use std::io;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeDpLocomotionSample {
    pub pose: NativeDpBodyPoseState,
    pub height: f32,
    pub torso_angle: f32,
}

impl NativeDpInitialization {
    /// Uniform walking/running pose, normal unguided mode, zero owner pitch/roll.
    /// Not arbitrary ten-channel action dispatch or a source-wire pose mapping.
    /// Uses caller-supplied reference time. Repeated samples reset selected bones.
    pub fn pose_locomotion(
        &mut self,
        gait: NativeDpGait,
        speed: f32,
        field_4068: i32,
        hands: &mut NativeDpHandCache<'_>,
        feet: &mut NativeDpFootCache<'_>,
        timer: &mut impl NativePoseRhythmTimer,
    ) -> io::Result<NativeDpLocomotionSample> {
        let code = match gait {
            NativeDpGait::Walk => 0x26,
            NativeDpGait::Run => 0x25,
        };
        let pose = NativeDpBodyPoseState {
            fields_3c: [code; 10],
            field_6c: false,
        };
        self.pose_locomotion_channels(&pose, speed, field_4068, hands, feet, timer)
    }

    /// Explicit admitted ten-channel subset: Stand/Walk/Run, normal mode.
    /// Preserve mixed limb/head/torso channels; reject unsupported actions
    /// before changing the skeleton instead of replacing them with Stand.
    pub fn pose_locomotion_channels(
        &mut self,
        pose: &NativeDpBodyPoseState,
        speed: f32,
        field_4068: i32,
        hands: &mut NativeDpHandCache<'_>,
        feet: &mut NativeDpFootCache<'_>,
        timer: &mut impl NativePoseRhythmTimer,
    ) -> io::Result<NativeDpLocomotionSample> {
        self.validate_neutral_standing()?;
        if !speed.is_finite()
            || pose.field_6c
            || pose
                .fields_3c
                .iter()
                .any(|code| !matches!(code, 0x24..=0x26))
        {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "unsupported locomotion channels/mode or nonfinite speed",
            ));
        }
        let height = feet.pose_height(&self.dna, &self.dimensions, field_4068, pose, false)?;
        self.bones.reset_for_animation(&self.dna);
        for selector in [0, 1] {
            let code = pose.fields_3c[if selector == 0 { 4 } else { 0 }];
            if code == 0x24 {
                self.stand_legs(height, selector, field_4068, feet)?;
                continue;
            }
            let gait = if code == 0x25 {
                NativeDpGait::Run
            } else {
                NativeDpGait::Walk
            };
            self.locomote_legs(
                NativeDpLegLocomotion {
                    gait,
                    height,
                    speed,
                    selector,
                    field_4068,
                },
                feet,
                timer,
            )?;
        }
        if self.dna.0[14] & 0x30 != 0 {
            self.position_tail(pose, height, speed, false, timer)?;
        }
        let torso_angle = self.position_torso(pose, height, speed, field_4068, hands, timer)?;
        self.position_heads(pose, 0., 0., timer)?;
        if self.dna.0[1] & 3 != 0 {
            for side in [0, 1] {
                let code = pose.fields_3c[if side == 0 { 2 } else { 5 }];
                if code == 0x24 {
                    self.stand_arms(height, side, None, field_4068, hands)?;
                    continue;
                }
                let gait = if code == 0x25 {
                    NativeDpGait::Run
                } else {
                    NativeDpGait::Walk
                };
                self.locomote_arms(
                    NativeDpArmLocomotion {
                        gait,
                        height,
                        speed,
                        side,
                        override_field_30: None,
                        field_4068,
                    },
                    hands,
                    timer,
                )?;
            }
        }
        if !height.is_finite()
            || !torso_angle.is_finite()
            || self
                .bones
                .slots
                .iter()
                .any(|b| b.matrix_00.iter().any(|v| !v.is_finite()))
        {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "nonfinite composed locomotion pose",
            ));
        }
        Ok(NativeDpLocomotionSample {
            pose: pose.clone(),
            height,
            torso_angle,
        })
    }
}
