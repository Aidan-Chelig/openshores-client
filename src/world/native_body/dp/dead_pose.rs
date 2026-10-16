//! Native face-down dead limb stages; full action admission is separate.
//! SHA-bound static anchors: tests/native_dead_pose.py.
use super::*;
use std::io;

impl NativeDpInitialization {
    /// Native trailing-leg reverse traversal and pair selector (0 later, 1 first,
    /// 2 all). Caller must reset and compose the rest of AnimatePose.
    pub fn dead_legs_facing_down(
        &mut self,
        selector: i32,
        field_4068: i32,
        feet: &mut NativeDpFootCache<'_>,
    ) -> io::Result<()> {
        self.spread_legs(f32::from_bits(0xbc8efa35), selector, field_4068, feet)?;
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
            let NativeDpPartJoints::Leg { splay_48, .. } = part.joints else {
                return Err(io::Error::other("dead leg missing joint state"));
            };
            part.rotate_leg_hip(&mut self.bones, dead_hip(&part.allocation.dna, splay_48))?;
            part.rotate_leg_knee(&mut self.bones, f32::from_bits(0xbe32b8c2))?;
            if native_dh_has_ankle_bend(&part.allocation.dna) {
                part.rotate_leg_ankle(&mut self.bones, f32::from_bits(0x3f32b8c3))?;
            }
            part.splay_leg_hip(&mut self.bones, splay_48)?;
        }
        Ok(())
    }

    /// Native ordinary-arm forward traversal and left/right side selector.
    /// Deliberately no wrist rotation or LiftShoulder in this consumer.
    pub fn dead_arms_facing_down(&mut self, side: i32) -> io::Result<()> {
        self.spread_arms(f32::from_bits(0x3edf66f3), 0., side)?;
        for part in self
            .parts
            .iter()
            .skip(1)
            .filter(|part| super::spread_arms::ordinary_arm(part))
        {
            if !super::spread_arms::selected(part, side) {
                continue;
            }
            let NativeDpPartJoints::Arm { splay_4c, .. } = part.joints else {
                return Err(io::Error::other("dead arm missing joint state"));
            };
            part.rotate_arm_shoulder(&mut self.bones, f32::from_bits(0xc04b4bc3))?;
            part.rotate_arm_elbow(&mut self.bones, f32::from_bits(0x3d0efa35))?;
            part.splay_arm_shoulder(&mut self.bones, splay_4c)?;
        }
        Ok(())
    }
}

fn dead_hip(dna: &NativeDhDna, splay: f32) -> f32 {
    // Native compares the stored signed splay directly, not abs(splay).
    f32::from_bits(
        // Part +0x24 = DhDNA object +0x14 = packed byte 12, after
        // the native DNA object's eight-byte header.
        if dna.0[12] & 0x0c == 4 && splay < f32::from_bits(0x3f490fdb) {
            0xc0437a15
        } else {
            0xbdb2b8c2
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dead_arm_stage_selects_sides_and_preserves_other_bones() {
        for side in [-1, 0, 1, 2, 3] {
            let mut dna = NativeDhDna([0; 24]);
            dna.0[1] = 3;
            let mut body = NativeDpInitialization::begin(&dna);
            for part in &mut body.parts {
                part.calculate(&body.dimensions, &mut || Ok(0.5)).unwrap();
            }
            let before = body.bones.clone();
            body.dead_arms_facing_down(side).unwrap();
            let mut selected_bones = Vec::new();
            for part in body
                .parts
                .iter()
                .filter(|part| super::super::spread_arms::ordinary_arm(part))
            {
                if super::super::spread_arms::selected(part, side) {
                    selected_bones.extend(part.limb_bones().unwrap());
                }
            }
            for (index, (actual, initial)) in body.bones.slots.iter().zip(&before.slots).enumerate()
            {
                assert!(actual.matrix_00.iter().all(|v| v.is_finite()));
                if selected_bones.contains(&index) {
                    assert_ne!(actual, initial);
                } else {
                    assert_eq!(actual, initial);
                }
            }
            body.bones = before;
            body.dead_arms_facing_down(side).unwrap();
            let repeated = body.bones.clone();
            body.bones.reset_for_animation(&dna);
            body.dead_arms_facing_down(side).unwrap();
            assert_eq!(body.bones, repeated);
        }
    }
    #[test]
    fn dead_hip_uses_exact_dna_mask_and_signed_strict_threshold() {
        let mut dna = NativeDhDna([0; 24]);
        assert_eq!(dead_hip(&dna, 0.).to_bits(), 0xbdb2b8c2);
        dna.0[20] = 4;
        assert_eq!(dead_hip(&dna, 0.).to_bits(), 0xbdb2b8c2);
        dna.0[12] = 4;
        assert_eq!(dead_hip(&dna, -2.).to_bits(), 0xc0437a15);
        assert_eq!(dead_hip(&dna, 0.7).to_bits(), 0xc0437a15);
        assert_eq!(
            dead_hip(&dna, f32::from_bits(0x3f490fdb)).to_bits(),
            0xbdb2b8c2
        );
        assert_eq!(dead_hip(&dna, f32::NAN).to_bits(), 0xbdb2b8c2);
        dna.0[12] = 12;
        assert_eq!(dead_hip(&dna, 0.).to_bits(), 0xbdb2b8c2);
    }
}
