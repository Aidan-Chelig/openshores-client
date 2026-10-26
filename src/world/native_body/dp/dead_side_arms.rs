//! Native side-facing death arm stages. Full action admission is separate.
use super::*;
use std::io;

impl NativeDpInitialization {
    pub fn dead_arms_facing_left(&mut self, side: i32) -> io::Result<()> {
        self.dead_arms_facing_side(side, true)
    }

    pub fn dead_arms_facing_right(&mut self, side: i32) -> io::Result<()> {
        self.dead_arms_facing_side(side, false)
    }

    fn dead_arms_facing_side(&mut self, side: i32, facing_left: bool) -> io::Result<()> {
        // Native SpreadArms receives angle=0 and lift=35 degrees.
        self.spread_arms(0., f32::from_bits(0x3f1c61aa), side)?;
        for part in self.parts.iter().skip(1).filter(|part| {
            super::spread_arms::ordinary_arm(part) && super::spread_arms::selected(part, side)
        }) {
            let NativeDpPartJoints::Arm { splay_4c, .. } = part.joints else {
                return Err(io::Error::other("side-facing dead arm missing joint state"));
            };
            let near = (part.allocation.kind == NativeDpPartKind::LeftArmAndHand) == facing_left;
            // Preserve cvtsi2ssl's signed field50 and each scalar SSE rounding.
            let index = part.allocation.index as i32 as f32;
            let degrees = if near {
                index * 8. + 75.
            } else {
                index * 10. + 125.
            };
            part.rotate_arm_shoulder(&mut self.bones, -degrees * f32::from_bits(0x3c8efa35))?;
            let wrist = if native_dh_has_wrist_bend(&part.allocation.dna) {
                if near { 0x3e860a92 } else { 0xbedf66f3 }
            } else {
                0xbd0efa35
            };
            part.rotate_arm_wrist(&mut self.bones, f32::from_bits(wrist))?;
            part.rotate_arm_elbow(
                &mut self.bones,
                f32::from_bits(if near { 0x3f91361e } else { 0x3e32b8c2 }),
            )?;
            if near {
                part.lift_arm_shoulder(&mut self.bones, f32::from_bits(0xbeb2b8c2))?;
            }
            part.splay_arm_shoulder(&mut self.bones, splay_4c)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrist_anatomy_changes_hands_without_changing_upper_arms_or_forearms() {
        let mut dna = NativeDhDna([0; 24]);
        dna.0[1] = 3;
        let mut initial = NativeDpInitialization::begin(&dna);
        for part in &mut initial.parts {
            part.calculate(&initial.dimensions, &mut || Ok(0.5))
                .unwrap();
        }
        for facing_left in [true, false] {
            let mut bent = initial.clone();
            let mut straight = initial.clone();
            for part in &mut straight.parts {
                part.allocation.dna.0[11] = 0x80;
            }
            bent.dead_arms_facing_side(2, facing_left).unwrap();
            straight.dead_arms_facing_side(2, facing_left).unwrap();
            for part in initial
                .parts
                .iter()
                .filter(|part| super::super::spread_arms::ordinary_arm(part))
            {
                let [upper, forearm, hand] = part.limb_bones().unwrap();
                assert_eq!(bent.bones.slots[upper], straight.bones.slots[upper]);
                assert_eq!(bent.bones.slots[forearm], straight.bones.slots[forearm]);
                assert_ne!(bent.bones.slots[hand], straight.bones.slots[hand]);
            }
        }
    }

    #[test]
    fn side_death_arms_preserve_other_bones_repeat_and_distinguish_facing() {
        let mut dna = NativeDhDna([0; 24]);
        dna.0[1] = 3;
        let mut body = NativeDpInitialization::begin(&dna);
        for part in &mut body.parts {
            part.calculate(&body.dimensions, &mut || Ok(0.5)).unwrap();
        }
        let initial = body.bones.clone();
        let mut both = Vec::new();
        for facing_left in [true, false] {
            for side in [-1, 0, 1, 2, 3] {
                body.bones = initial.clone();
                body.dead_arms_facing_side(side, facing_left).unwrap();
                let selected: Vec<_> = body
                    .parts
                    .iter()
                    .filter(|part| {
                        super::super::spread_arms::ordinary_arm(part)
                            && super::super::spread_arms::selected(part, side)
                    })
                    .flat_map(|part| part.limb_bones().unwrap())
                    .collect();
                for (index, (actual, original)) in
                    body.bones.slots.iter().zip(&initial.slots).enumerate()
                {
                    assert!(actual.matrix_00.iter().all(|v| v.is_finite()));
                    if selected.contains(&index) {
                        assert_ne!(actual, original);
                    } else {
                        assert_eq!(actual, original);
                    }
                }
                let first = body.bones.clone();
                body.bones.reset_for_animation(&dna);
                body.dead_arms_facing_side(side, facing_left).unwrap();
                assert_eq!(body.bones, first);
                if side == 2 {
                    both.push(first);
                }
            }
        }
        assert_ne!(both[0], both[1]);
    }
}
