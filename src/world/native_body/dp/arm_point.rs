//! Native PointArms stage: owner pitch applies to every selected ordinary arm.
use super::*;
use std::io;

impl NativeDpInitialization {
    pub fn point_arms(&mut self, side: i32, pitch_30: f32) -> io::Result<()> {
        if !pitch_30.is_finite() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "nonfinite Point owner pitch",
            ));
        }
        self.spread_arms(f32::from_bits(0xbc8efa35), f32::from_bits(0x3db2b8c2), side)?;
        for part in self
            .parts
            .iter()
            .skip(1)
            .filter(|part| super::spread_arms::ordinary_arm(part))
        {
            if !super::spread_arms::selected(part, side) {
                continue;
            }
            part.relax_arm(
                &mut self.bones,
                NativeDpArmStandAngles {
                    shoulder: pitch_30 - f32::from_bits(0x3db2b8c2),
                    elbow: f32::from_bits(0x3db2b8c2),
                    wrist: 0.,
                },
                None,
            )?;
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn point_pitch_changes_every_selected_arm_and_no_other_bones() {
        let mut dna = NativeDhDna([0; 24]);
        dna.0[1] = 3;
        let mut base = NativeDpInitialization::begin(&dna);
        for part in &mut base.parts {
            part.calculate(&base.dimensions, &mut || Ok(0.5)).unwrap();
        }
        for side in [-1, 0, 1, 2, 3] {
            let mut first = base.clone();
            let mut second = base.clone();
            first.point_arms(side, 0.).unwrap();
            second.point_arms(side, 0.4).unwrap();
            let mut selected = std::collections::BTreeSet::new();
            for part in base.parts.iter().filter(|part| {
                super::super::spread_arms::ordinary_arm(part)
                    && super::super::spread_arms::selected(part, side)
            }) {
                let bones = part.limb_bones().unwrap();
                assert!(
                    bones
                        .iter()
                        .any(|&i| first.bones.slots[i] != second.bones.slots[i])
                );
                selected.extend(bones);
            }
            for (index, bone) in second.bones.slots.iter().enumerate() {
                assert!(bone.matrix_00.iter().all(|v| v.is_finite()));
                if !selected.contains(&index) {
                    assert_eq!(bone, &base.bones.slots[index]);
                }
            }
        }
        let mut body = base.clone();
        assert!(body.point_arms(2, f32::NAN).is_err());
        assert_eq!(body.bones, base.bones);
    }
}
