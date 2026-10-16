//! Native JumpLegs/LegAndFoot::Jump stage, without world motion.
use super::*;
use crate::world::native_creature::{NativePoseRhythmTimer, native_dh_pose_rhythm};
use std::io;

impl NativeDpInitialization {
    pub fn jump_legs(
        &mut self,
        height: f32,
        speed: f32,
        selector: i32,
        field_4068: i32,
        feet: &mut NativeDpFootCache<'_>,
        timer: &mut impl NativePoseRhythmTimer,
    ) -> io::Result<()> {
        if !height.is_finite() || !speed.is_finite() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "nonfinite jump leg input",
            ));
        }
        self.spread_legs(f32::from_bits(0x3e32b8c2), selector, field_4068, feet)?;
        for part in self.parts.iter().skip(1).rev().take_while(|part| {
            matches!(
                part.allocation.kind,
                NativeDpPartKind::LeftLegAndFoot | NativeDpPartKind::RightLegAndFoot
            )
        }) {
            let selected = match selector {
                0 => part.allocation.index != 0,
                1 => part.allocation.index == 0,
                2 => true,
                _ => false,
            };
            if !selected {
                continue;
            }
            let rhythm = native_dh_pose_rhythm(&part.allocation.dna, 0x1c, 0., speed, timer)?;
            part.limb_bones()?;
            let stand = feet.leg_stand_angles(
                &self.dna,
                &part.allocation.dna,
                &self.dimensions,
                field_4068,
                height,
            )?;
            let angles = calculate(
                stand,
                rhythm,
                native_dh_has_ankle_bend(&part.allocation.dna),
            );
            let NativeDpPartJoints::Leg { splay_48, .. } = part.joints else {
                return Err(io::Error::other("jump leg missing joints"));
            };
            part.rotate_leg_hip(&mut self.bones, angles.hip)?;
            part.rotate_leg_knee(&mut self.bones, angles.knee)?;
            // Unlike Stand/Walk/Run, native Jump applies this even without bend.
            part.rotate_leg_ankle(&mut self.bones, angles.ankle)?;
            part.splay_leg_hip(&mut self.bones, splay_48)?;
        }
        Ok(())
    }
}
fn calculate(
    stand: NativeDpLegStandAngles,
    rhythm: f32,
    ankle_bend: bool,
) -> NativeDpLegStandAngles {
    fn interpolate(stand: f32, crouch: f32, rhythm: f32) -> f32 {
        let delta = stand - crouch;
        (rhythm * delta + delta) * 0.5_f32 + crouch
    }
    NativeDpLegStandAngles {
        hip: interpolate(stand.hip, -f32::from_bits(0x4016cbe4), rhythm),
        knee: interpolate(stand.knee, -f32::from_bits(0x3e860a92), rhythm),
        ankle: interpolate(
            stand.ankle,
            f32::from_bits(if ankle_bend { 0x3eb2b8c2 } else { 0xbd0efa35 }),
            rhythm,
        ),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn jump_extrema_and_midpoint_keep_native_crouch_and_ankle_branches() {
        let stand = NativeDpLegStandAngles {
            hip: 0.2,
            knee: 0.8,
            ankle: -0.4,
        };
        for bend in [false, true] {
            let crouch = calculate(stand, -1., bend);
            assert_eq!(crouch.hip.to_bits(), 0xc016cbe4);
            assert_eq!(crouch.knee.to_bits(), 0xbe860a92);
            assert_eq!(
                crouch.ankle.to_bits(),
                if bend { 0x3eb2b8c2 } else { 0xbd0efa35 }
            );
            let middle = calculate(stand, 0., bend);
            assert_eq!(middle.hip, (stand.hip - crouch.hip) * 0.5 + crouch.hip);
            let extended = calculate(stand, 1., bend);
            assert!((extended.hip - stand.hip).abs() < 1e-6);
            assert!((extended.knee - stand.knee).abs() < 1e-6);
            assert!((extended.ankle - stand.ankle).abs() < 1e-6);
        }
    }
}
