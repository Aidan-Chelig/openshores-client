//! Native FoldLegs/FoldArms stages; full action admission is separate.
use super::*;
use std::io;

impl NativeDpInitialization {
    pub fn fold_legs(
        &mut self,
        selector: i32,
        field_4068: i32,
        feet: &mut NativeDpFootCache<'_>,
    ) -> io::Result<()> {
        self.spread_legs(f32::from_bits(0x3eb2b8c2), selector, field_4068, feet)?;
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
            if selected {
                part.stand_leg(
                    &mut self.bones,
                    NativeDpLegStandAngles {
                        hip: f32::from_bits(0xbeb2b8c2),
                        knee: f32::from_bits(0xc021f770),
                        ankle: f32::from_bits(0x3fc90fdb),
                    },
                )?;
            }
        }
        Ok(())
    }
    pub fn fold_arms(&mut self, side: i32, override_field_30: Option<f32>) -> io::Result<()> {
        if override_field_30.is_some_and(|v| !v.is_finite()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "nonfinite Fold owner pitch",
            ));
        }
        self.spread_arms(f32::from_bits(0x3e32b8c2), f32::from_bits(0x3e32b8c2), side)?;
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
            // Relax's override uses pitch-shoulder, hence the native 40384e89
            // offset, NOT the normal elbow's one-ULP-lower 40384e88.
            part.relax_arm(
                &mut self.bones,
                NativeDpArmStandAngles {
                    shoulder: f32::from_bits(0xc0384e89),
                    elbow: f32::from_bits(0x40384e88),
                    wrist: 0.,
                },
                pending.take(),
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fold_arm_override_is_first_selected_only_and_preserves_other_bones() {
        let mut dna = NativeDhDna([0; 24]);
        dna.0[1] = 3;
        let mut base = NativeDpInitialization::begin(&dna);
        for part in &mut base.parts {
            part.calculate(&base.dimensions, &mut || Ok(0.5)).unwrap();
        }
        for side in [0, 1, 2] {
            let mut normal = base.clone();
            let mut overridden = base.clone();
            normal.fold_arms(side, None).unwrap();
            overridden.fold_arms(side, Some(0.4)).unwrap();
            let first = base
                .parts
                .iter()
                .find(|part| {
                    super::super::spread_arms::ordinary_arm(part)
                        && super::super::spread_arms::selected(part, side)
                })
                .unwrap()
                .limb_bones()
                .unwrap();
            assert!(
                first
                    .iter()
                    .any(|&i| normal.bones.slots[i] != overridden.bones.slots[i])
            );
            for (i, bone) in normal.bones.slots.iter().enumerate() {
                assert!(bone.matrix_00.iter().all(|v| v.is_finite()));
                if !first.contains(&i) {
                    assert_eq!(bone, &overridden.bones.slots[i]);
                }
            }
        }
        let mut body = base.clone();
        assert!(body.fold_arms(2, Some(f32::NAN)).is_err());
        assert_eq!(body.bones, base.bones);
    }
}
