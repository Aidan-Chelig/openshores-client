//! Native JumpArms/ArmAndHand::Jump stage. No world/root motion.
use super::*;
use crate::world::native_creature::{NativePoseRhythmTimer, native_dh_pose_rhythm};
use std::io;

#[derive(Clone, Copy, Debug)]
pub struct NativeDpArmJump {
    pub height: f32,
    pub speed: f32,
    pub side: i32,
    pub override_field_30: Option<f32>,
    pub field_4068: i32,
}
impl NativeDpInitialization {
    pub fn jump_arms(
        &mut self,
        input: NativeDpArmJump,
        hands: &mut NativeDpHandCache<'_>,
        timer: &mut impl NativePoseRhythmTimer,
    ) -> io::Result<()> {
        if !input.height.is_finite()
            || !input.speed.is_finite()
            || input.override_field_30.is_some_and(|v| !v.is_finite())
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "nonfinite jump arm input",
            ));
        }
        self.spread_arms(
            f32::from_bits(0xbd0efa35),
            f32::from_bits(0x3db2b8c2),
            input.side,
        )?;
        let mut pending = input.override_field_30;
        for part in self
            .parts
            .iter()
            .skip(1)
            .filter(|part| super::spread_arms::ordinary_arm(part))
        {
            if !super::spread_arms::selected(part, input.side) {
                continue;
            }
            let wave = native_dh_pose_rhythm(&part.allocation.dna, 0x1c, 0., input.speed, timer)?;
            let stand = part.arm_stand_angles(
                &self.dna,
                &self.dimensions,
                &self.bones,
                input.field_4068,
                input.height,
                hands,
            )?;
            let angles = calculate(stand, wave, native_dh_has_wrist_bend(&part.allocation.dna));
            if ![angles.shoulder, angles.wrist, angles.elbow]
                .iter()
                .all(|v| v.is_finite())
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "nonfinite jump arm angles",
                ));
            }
            // Relax consumes the override against ANIMATED shoulder, matching
            // native Jump, and applies shoulder/wrist/elbow/splay/lift in order.
            part.relax_arm(&mut self.bones, angles, pending.take())?;
        }
        Ok(())
    }
}
fn calculate(stand: NativeDpArmStandAngles, wave: f32, wrist_bend: bool) -> NativeDpArmStandAngles {
    fn interpolate(stand: f32, crouch: f32, wave: f32) -> f32 {
        let delta = stand - crouch;
        (wave * delta + delta) * 0.5_f32 + crouch
    }
    NativeDpArmStandAngles {
        shoulder: interpolate(stand.shoulder, -f32::from_bits(0x3db2b8c2), wave),
        elbow: interpolate(stand.elbow, f32::from_bits(0x3e32b8c2), wave),
        wrist: interpolate(
            stand.wrist,
            f32::from_bits(if wrist_bend { 0x3e32b8c2 } else { 0xbd0efa35 }),
            wave,
        ),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn jump_arm_interpolation_preserves_native_extrema_and_wrist_branch() {
        let stand = NativeDpArmStandAngles {
            shoulder: 0.3,
            elbow: 0.8,
            wrist: -0.2,
        };
        for bend in [false, true] {
            let crouch = calculate(stand, -1., bend);
            assert_eq!(crouch.shoulder.to_bits(), 0xbdb2b8c2);
            assert_eq!(crouch.elbow.to_bits(), 0x3e32b8c2);
            assert_eq!(
                crouch.wrist.to_bits(),
                if bend { 0x3e32b8c2 } else { 0xbd0efa35 }
            );
            let extended = calculate(stand, 1., bend);
            assert!((extended.shoulder - stand.shoulder).abs() < 1e-6);
            assert!((extended.elbow - stand.elbow).abs() < 1e-6);
            assert!((extended.wrist - stand.wrist).abs() < 1e-6);
        }
    }
}
