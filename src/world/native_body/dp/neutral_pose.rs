//! Deterministic offline Stand(0x24) composition, not general animation dispatch.
use super::*;
use crate::world::native_creature::{NativeDpBodyPoseState, NativePoseRhythmTimer};
use std::io;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeDpStandingPose {
    pub pose: NativeDpBodyPoseState,
    /// CGHeight(false), not an invented vertical body translation.
    pub height: f32,
    pub torso_angle: f32,
}

impl NativeDpInitialization {
    /// Offline neutral fixture: Stand in all ten pose words, mode=false,
    /// speed=0, owner pitch30/roll34=0, no first-arm override. The height is
    /// consumed by native posing; this does not add a display/world offset.
    pub fn pose_neutral_standing(
        &mut self,
        field_4068: i32,
        hands: &mut NativeDpHandCache<'_>,
        feet: &mut NativeDpFootCache<'_>,
    ) -> io::Result<NativeDpStandingPose> {
        self.validate_neutral_standing()?;
        let pose = NativeDpBodyPoseState {
            fields_3c: [0x24; 10],
            field_6c: false,
        };
        // Native AnimatePose computes CGHeight BEFORE selective matrix reset.
        let height = feet.pose_height(&self.dna, &self.dimensions, field_4068, &pose, false)?;
        self.bones.reset_for_animation(&self.dna);
        // PositionLegs reads field4c (later pairs), then field3c (first pair).
        self.stand_legs(height, 0, field_4068, feet)?;
        self.stand_legs(height, 1, field_4068, feet)?;
        let mut timer = NoStandingRhythm;
        if self.dna.0[14] & 0x30 != 0 {
            self.position_tail(&pose, height, 0., false, &mut timer)?;
        }
        let torso_angle = self.position_torso(&pose, height, 0., field_4068, hands, &mut timer)?;
        self.position_heads(&pose, 0., 0., &mut timer)?;
        if self.dna.0[1] & 3 != 0 {
            self.stand_arms(height, 0, None, field_4068, hands)?;
            self.stand_arms(height, 1, None, field_4068, hands)?;
        }
        if !height.is_finite()
            || !torso_angle.is_finite()
            || self
                .bones
                .slots
                .iter()
                .any(|b| !b.matrix_00.iter().all(|v| v.is_finite()))
        {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "nonfinite composed standing pose",
            ));
        }
        Ok(NativeDpStandingPose {
            pose,
            height,
            torso_angle,
        })
    }

    fn validate_neutral_standing(&self) -> io::Result<()> {
        if !self.parts_calculated() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "neutral standing requires completed native part calculations",
            ));
        }
        if self.dna.0[12] & 12 == 0 {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "legless neutral standing requires native tail-rhythm composition",
            ));
        }
        if self.dna.0[1] & 3 != 0 && self.dna.0[2] & 6 != 0 {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "winged neutral standing requires native StandWings composition",
            ));
        }
        Ok(())
    }
}

// No wall-clock substitutes: every supported Stand branch is non-rhythmic.
// Fail if a future change starts selecting a timed branch accidentally.
struct NoStandingRhythm;
impl NativePoseRhythmTimer for NoStandingRhythm {
    fn start(&mut self) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "unexpected rhythm in neutral standing pose",
        ))
    }
    fn msecs_since_reference(&mut self) -> io::Result<i64> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "unexpected timed neutral standing pose",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uninitialized_parts_fail_before_bone_writes() {
        for (legs, wings) in [(0, 0), (1, 2)] {
            let mut dna = NativeDhDna([0; 24]);
            dna.0[1] = 1;
            dna.0[12] = legs << 2;
            dna.0[2] = wings;
            let body = NativeDpInitialization::begin(&dna);
            let before = body.bones.clone();
            assert_eq!(
                body.validate_neutral_standing().unwrap_err().kind(),
                io::ErrorKind::InvalidData
            );
            assert_eq!(body.bones, before);
        }
    }

    #[test]
    fn neutral_pose_has_no_clock_fallback() {
        let mut timer = NoStandingRhythm;
        assert_eq!(
            timer.start().unwrap_err().kind(),
            io::ErrorKind::Unsupported
        );
        assert_eq!(
            timer.msecs_since_reference().unwrap_err().kind(),
            io::ErrorKind::Unsupported
        );
    }
}
