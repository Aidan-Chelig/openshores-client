//! Native ProneCreep limb stages; full action admission is separate.
use super::*;
use crate::world::native_creature::{NativePoseRhythmTimer, native_dh_pose_rhythm};
use std::io;

fn arm_angles(wave: f32) -> NativeDpArmStandAngles {
    NativeDpArmStandAngles {
        shoulder: (5. - wave * 10.) * f32::from_bits(0x3c8efa35) - f32::from_bits(0x40384e89),
        elbow: f32::from_bits(0x40384e88),
        wrist: 0.,
    }
}

impl NativeDpInitialization {
    pub fn creep_legs(
        &mut self,
        selector: i32,
        speed: f32,
        field_4068: i32,
        feet: &mut NativeDpFootCache<'_>,
        timer: &mut impl NativePoseRhythmTimer,
    ) -> io::Result<()> {
        if !speed.is_finite() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "nonfinite creeping leg speed",
            ));
        }
        self.spread_legs(f32::from_bits(0xbdb2b8c2), selector, field_4068, feet)?;
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
            // ProneCreep's leg phase table equals Run's, including the single
            // left leg on hand-walking bodies. Only the phase table is shared.
            let wave = native_dh_pose_rhythm(
                &part.allocation.dna,
                0x21,
                super::leg_locomotion::phase(&part.allocation, NativeDpGait::Run),
                speed,
                timer,
            )?;
            let NativeDpPartJoints::Leg { splay_48, .. } = part.joints else {
                return Err(io::Error::other("creeping leg missing joint state"));
            };
            let (hip, knee, ankle, splay) = leg_angles(&part.allocation.dna, splay_48, wave);
            part.rotate_leg_hip(&mut self.bones, hip)?;
            part.rotate_leg_knee(&mut self.bones, knee)?;
            if native_dh_has_ankle_bend(&part.allocation.dna) {
                part.rotate_leg_ankle(&mut self.bones, ankle)?;
            }
            part.splay_leg_hip(&mut self.bones, splay)?;
        }
        Ok(())
    }

    /// Native height argument is unused by the concrete arm consumer. Pitch
    /// override applies once, to the first selected ordinary arm only.
    pub fn creep_arms(
        &mut self,
        side: i32,
        speed: f32,
        override_field_30: Option<f32>,
        timer: &mut impl NativePoseRhythmTimer,
    ) -> io::Result<()> {
        if !speed.is_finite() || override_field_30.is_some_and(|v| !v.is_finite()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "nonfinite creeping arm input",
            ));
        }
        self.spread_arms(f32::from_bits(0x3e32b8c2), f32::from_bits(0x3fb2b8c2), side)?;
        let mut pending = override_field_30;
        for part in self
            .parts
            .iter()
            .skip(1)
            .filter(|part| super::spread_arms::ordinary_arm(part))
        {
            if !super::spread_arms::selected(part, side) {
                continue;
            }
            // The concrete ProneCreep phase table matches WalkArms exactly.
            let wave = native_dh_pose_rhythm(
                &part.allocation.dna,
                0x21,
                super::arm_locomotion::phase(&part.allocation),
                speed,
                timer,
            )?;
            part.relax_arm(&mut self.bones, arm_angles(wave), pending.take())?;
        }
        Ok(())
    }
}

fn leg_angles(dna: &NativeDhDna, splay: f32, wave: f32) -> (f32, f32, f32, f32) {
    let swing = f32::from_bits(0x3e32b8c2);
    (
        super::dead_pose::dead_hip(dna, splay),
        f32::from_bits(0xbe32b8c2) - (swing - wave * swing),
        f32::from_bits(0x3f32b8c3),
        (1. - wave) * splay,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn creep_shared_phase_tables_match_all_pair_counts_sides_and_hand_walking() {
        for count in 1..=3 {
            for hand_class in [0, 0x60] {
                let mut dna = NativeDhDna([0; 24]);
                dna.0[1] = count;
                dna.0[12] = count << 2;
                dna.0[11] = hand_class;
                for index in 0..count {
                    for left in [true, false] {
                        let mut part = NativeDpAllocatedPart {
                            dna: dna.clone(),
                            index: u32::from(index),
                            kind: if left {
                                NativeDpPartKind::LeftArmAndHand
                            } else {
                                NativeDpPartKind::RightArmAndHand
                            },
                        };
                        let arm = match (count, index, left) {
                            (2, 1, true) => 0x40490fdb,
                            (2, 1, false) => 0x40c90fdb,
                            (3, 1, true) => 0x40060a92,
                            (3, 1, false) => 0x40a78d36,
                            (3, 2, true) => 0x40860a92,
                            (3, 2, false) => 0x40ea9280,
                            (_, _, true) => 0,
                            _ => 0x40490fdb,
                        };
                        assert_eq!(super::super::arm_locomotion::phase(&part).to_bits(), arm);
                        part.kind = if left {
                            NativeDpPartKind::LeftLegAndFoot
                        } else {
                            NativeDpPartKind::RightLegAndFoot
                        };
                        let leg = match (count, index, left) {
                            (1, _, true) if hand_class == 0 => 0,
                            (2, 1, _) => 0x40490fdb,
                            (2, _, _) => 0,
                            (3, 1, true) => 0x40a78d36,
                            (3, 1, false) => 0x40060a92,
                            (3, 2, true) => 0x40ea9280,
                            (3, 2, false) => 0x40860a92,
                            (_, _, true) => 0x40490fdb,
                            _ => 0,
                        };
                        assert_eq!(
                            super::super::leg_locomotion::phase(&part, NativeDpGait::Run).to_bits(),
                            leg
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn creep_leg_extrema_keep_signed_splay_gate_and_separate_float_operations() {
        let mut dna = NativeDhDna([0; 24]);
        dna.0[12] = 4;
        for wave in [-1., 0., 1.] {
            let (hip, knee, ankle, splay) = leg_angles(&dna, -0.5, wave);
            assert_eq!(hip.to_bits(), 0xc0437a15);
            let ten = f32::from_bits(0x3e32b8c2);
            assert_eq!(knee.to_bits(), (-ten - (ten - wave * ten)).to_bits());
            assert_eq!(ankle.to_bits(), 0x3f32b8c3);
            assert_eq!(splay, (1. - wave) * -0.5);
        }
        assert_eq!(
            leg_angles(&dna, f32::from_bits(0x3f490fdb), 0.).0.to_bits(),
            0xbdb2b8c2
        );
        dna.0[12] = 12;
        assert_eq!(leg_angles(&dna, -0.5, 0.).0.to_bits(), 0xbdb2b8c2);
    }
    struct Timer {
        millis: i64,
        samples: usize,
    }
    impl NativePoseRhythmTimer for Timer {
        fn start(&mut self) -> io::Result<()> {
            Ok(())
        }
        fn msecs_since_reference(&mut self) -> io::Result<i64> {
            self.samples += 1;
            Ok(self.millis)
        }
    }
    #[test]
    fn creep_arm_samples_select_sides_repeat_and_override_only_the_first() {
        let mut dna = NativeDhDna([0; 24]);
        dna.0[1] = 3;
        let mut base = NativeDpInitialization::begin(&dna);
        for part in &mut base.parts {
            part.calculate(&base.dimensions, &mut || Ok(0.5)).unwrap();
        }
        for side in [-1, 0, 1, 2, 3] {
            let selected: Vec<_> = base
                .parts
                .iter()
                .filter(|part| {
                    super::super::spread_arms::ordinary_arm(part)
                        && super::super::spread_arms::selected(part, side)
                })
                .collect();
            let first = selected.first().map(|part| part.limb_bones().unwrap());
            let mut samples = Vec::new();
            for millis in [0, 125, 0] {
                let mut normal = base.clone();
                let mut overridden = base.clone();
                let mut timer = Timer { millis, samples: 0 };
                normal.creep_arms(side, 4., None, &mut timer).unwrap();
                assert_eq!(timer.samples, selected.len());
                overridden
                    .creep_arms(side, 4., Some(0.4), &mut Timer { millis, samples: 0 })
                    .unwrap();
                for (index, bone) in normal.bones.slots.iter().enumerate() {
                    assert!(bone.matrix_00.iter().all(|v| v.is_finite()));
                    if !selected
                        .iter()
                        .any(|part| part.limb_bones().unwrap().contains(&index))
                    {
                        assert_eq!(bone, &base.bones.slots[index]);
                    }
                    if first.is_none_or(|indices| !indices.contains(&index)) {
                        assert_eq!(bone, &overridden.bones.slots[index]);
                    }
                }
                if let Some(first) = first {
                    assert!(
                        first
                            .iter()
                            .any(|&i| normal.bones.slots[i] != overridden.bones.slots[i])
                    );
                }
                samples.push(normal.bones);
            }
            assert_eq!(samples[0], samples[2]);
            if !selected.is_empty() {
                assert_ne!(samples[0], samples[1]);
            }
        }
        let mut body = base.clone();
        assert!(
            body.creep_arms(
                2,
                f32::NAN,
                None,
                &mut Timer {
                    millis: 0,
                    samples: 0
                }
            )
            .is_err()
        );
        assert_eq!(body.bones, base.bones);
    }
}
