//! Native WalkArms/RunArms and concrete arm gait; no world/root motion.
use super::*;
use crate::world::native_creature::{NativePoseRhythmTimer, native_dh_pose_rhythm};
use std::io;

#[derive(Clone, Copy, Debug)]
pub struct NativeDpArmLocomotion {
    pub gait: NativeDpGait,
    pub height: f32,
    pub speed: f32,
    pub side: i32,
    pub override_field_30: Option<f32>,
    pub field_4068: i32,
}

impl NativeDpInitialization {
    /// One arm stage. Optional owner pitch override is consumed once in native
    /// forward order, not applied to every arm or guessed from torso rotation.
    pub fn locomote_arms(
        &mut self,
        input: NativeDpArmLocomotion,
        hands: &mut NativeDpHandCache<'_>,
        timer: &mut impl NativePoseRhythmTimer,
    ) -> io::Result<()> {
        if !input.height.is_finite()
            || !input.speed.is_finite()
            || input.override_field_30.is_some_and(|v| !v.is_finite())
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "nonfinite arm locomotion input",
            ));
        }
        let on_hands = native_dh_walks_on_hands(&self.dna);
        let (spread, lift) = match input.gait {
            NativeDpGait::Walk => (0x3e32b8c2, if on_hands { 0x3d0efa35 } else { 0x3e32b8c2 }),
            NativeDpGait::Run => (0x3db2b8c2, if on_hands { 0x3c8efa35 } else { 0x3e32b8c2 }),
        };
        self.spread_arms(f32::from_bits(spread), f32::from_bits(lift), input.side)?;
        let mut override_field_30 = input.override_field_30;
        for part in self
            .parts
            .iter()
            .skip(1)
            .filter(|p| super::spread_arms::ordinary_arm(p))
        {
            if !super::spread_arms::selected(part, input.side) {
                continue;
            }
            let rhythm = native_dh_pose_rhythm(
                &part.allocation.dna,
                match input.gait {
                    NativeDpGait::Walk => 0x26,
                    NativeDpGait::Run => 0x25,
                },
                phase(&part.allocation),
                input.speed,
                timer,
            )?;
            let stand = part.arm_stand_angles(
                &self.dna,
                &self.dimensions,
                &self.bones,
                input.field_4068,
                input.height,
                hands,
            )?;
            let override_value = override_field_30.take();
            let angles = calculate(
                stand,
                rhythm,
                input.gait,
                native_dh_walks_on_hands(&part.allocation.dna),
                override_value.is_some(),
            );
            if ![angles.shoulder, angles.elbow, angles.wrist]
                .iter()
                .all(|v| v.is_finite())
            {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "nonfinite native arm locomotion angles",
                ));
            }
            part.relax_arm(&mut self.bones, angles, override_value)?;
        }
        Ok(())
    }
}

fn phase(part: &NativeDpAllocatedPart) -> f32 {
    let left = part.kind == NativeDpPartKind::LeftArmAndHand;
    match (part.dna.0[1] & 3, part.index, left) {
        (2, 1, true) => f32::from_bits(0x40490fdb),
        (2, 1, false) => f32::from_bits(0x40c90fdb),
        (3, 1, true) => f32::from_bits(0x40060a92),
        (3, 1, false) => f32::from_bits(0x40a78d36),
        (3, 2, true) => f32::from_bits(0x40860a92),
        (3, 2, false) => f32::from_bits(0x40ea9280),
        (_, _, true) => 0.,
        _ => f32::from_bits(0x40490fdb),
    }
}

fn calculate(
    mut stand: NativeDpArmStandAngles,
    rhythm: f32,
    gait: NativeDpGait,
    on_hands: bool,
    overridden: bool,
) -> NativeDpArmStandAngles {
    let (bias, swing) = match gait {
        NativeDpGait::Walk => (5., 10.),
        NativeDpGait::Run => (10., 20.),
    };
    stand.shoulder = (f64::from(stand.shoulder)
        + (bias - f64::from(rhythm) * swing) * f64::from(f32::from_bits(0x3c8efa35)))
        as f32;
    if !overridden {
        let sum = rhythm * stand.elbow + stand.elbow;
        stand.elbow = match gait {
            NativeDpGait::Walk => (f64::from(stand.elbow) - f64::from(sum) * 0.5) as f32,
            NativeDpGait::Run => {
                let mut elbow = stand.elbow - sum;
                if !on_hands {
                    elbow = (f64::from(elbow) + f64::from(f32::from_bits(0x40490fdb) - elbow) / 3.)
                        as f32;
                }
                elbow
            }
        };
    }
    stand
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_arm_extrema_and_override_preserve_distinct_float_stages() {
        let stand = NativeDpArmStandAngles {
            shoulder: 0.2,
            elbow: 1.,
            wrist: -0.3,
        };
        let walk = calculate(stand, 1., NativeDpGait::Walk, false, false);
        assert_eq!(walk.elbow, 0.);
        assert_eq!(walk.wrist, stand.wrist);
        let run = calculate(stand, 1., NativeDpGait::Run, true, false);
        assert_eq!(run.elbow, -1.);
        let upright = calculate(stand, 1., NativeDpGait::Run, false, false);
        assert_eq!(
            upright.elbow,
            (-1_f64 + f64::from(f32::from_bits(0x40490fdb) + 1.) / 3.) as f32
        );
        assert_eq!(
            calculate(stand, 1., NativeDpGait::Run, false, true).elbow,
            stand.elbow
        );
    }
    #[test]
    fn arm_phases_are_opposed_to_legs_and_keep_all_pair_literals() {
        for (count, index, left, expected) in [
            (1, 0, true, 0),
            (1, 0, false, 0x40490fdb),
            (2, 1, true, 0x40490fdb),
            (2, 1, false, 0x40c90fdb),
            (3, 1, true, 0x40060a92),
            (3, 1, false, 0x40a78d36),
            (3, 2, true, 0x40860a92),
            (3, 2, false, 0x40ea9280),
        ] {
            let mut dna = NativeDhDna([0; 24]);
            dna.0[1] = count;
            let part = NativeDpAllocatedPart {
                dna,
                index,
                kind: if left {
                    NativeDpPartKind::LeftArmAndHand
                } else {
                    NativeDpPartKind::RightArmAndHand
                },
            };
            assert_eq!(phase(&part).to_bits(), expected);
        }
    }
}
