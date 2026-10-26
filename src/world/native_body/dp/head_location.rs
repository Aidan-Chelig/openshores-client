//! DpBody::HeadLocation, distinct from authored EyePosition/CameraLocation.
use super::*;
use std::io;

impl NativeDpInitialization {
    /// Nominal adult nonvehicle clearance for an already posed uniform
    /// Prone/ProneCreep body. This is not a mesh-bound offset or collision solver.
    pub fn adult_prone_ground_clearance(
        &self,
        pose: &crate::world::native_creature::NativeDpBodyPoseState,
        field_4068: i32,
        feet: &mut NativeDpFootCache<'_>,
    ) -> io::Result<f64> {
        if pose.field_6c
            || ![0x20, 0x21]
                .iter()
                .any(|&code| pose.fields_3c == [code; 10])
        {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "clearance requires uniform normal prone/crawling pose",
            ));
        }
        let head = self.head_location(pose.fields_3c[3], false)?;
        let height = feet.pose_height(&self.dna, &self.dimensions, field_4068, pose, true)?;
        // CGOffset is negative HeadLocation; preserve the native f64 subtraction.
        let offset_z = -head[2];
        let result = f64::from(height) - offset_z;
        if !result.is_finite() {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "nonfinite prone ground clearance",
            ));
        }
        Ok(result)
    }

    pub fn head_location(&self, general: u32, field_3a: bool) -> io::Result<[f64; 3]> {
        if general == 0x37 {
            return Ok([0.; 3]);
        }
        if !self.parts_calculated() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "head location requires calculated parts",
            ));
        }
        locate(&self.parts, &self.dna, &self.bones, field_3a)
    }
}

fn locate(
    parts: &[NativeDpPartState],
    dna: &NativeDhDna,
    bones: &NativeDpBoneTransforms,
    field_3a: bool,
) -> io::Result<[f64; 3]> {
    let selected = usize::from(dna.0[5] & 2 != 0 && !field_3a);
    let Some(head) = parts
        .iter()
        .skip(1)
        .filter(|part| part.allocation.kind == NativeDpPartKind::HeadAndNeck)
        .nth(selected)
    else {
        return Ok([0.; 3]);
    };
    let NativeDpPartJoints::Head { field_50 } = head.joints else {
        return Err(io::Error::other("head location missing head joint"));
    };
    let bone = if head.allocation.index == 1 { 12 } else { 7 };
    let point = bones.slots[bone].project(field_50);
    if point.iter().any(|v| !v.is_finite()) {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "nonfinite native head location",
        ));
    }
    Ok(point)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn head_location_selects_native_head_joint_and_current_bone_without_mutation() {
        let mut dna = NativeDhDna([0; 24]);
        dna.0[5] = 2;
        let mut body = NativeDpInitialization::begin(&dna);
        for part in &mut body.parts {
            if part.allocation.kind == NativeDpPartKind::HeadAndNeck {
                part.joints = NativeDpPartJoints::Head {
                    field_50: [1., 2., 3.],
                };
            }
        }
        body.bones.slots[7].translate([4., 5., 6.]);
        body.bones.slots[12].translate([-1., -2., -3.]);
        let before = body.bones.clone();
        assert_eq!(
            locate(&body.parts, &dna, &body.bones, true).unwrap(),
            [5., 7., 9.]
        );
        assert_eq!(
            locate(&body.parts, &dna, &body.bones, false).unwrap(),
            [0.; 3]
        );
        dna.0[5] = 0;
        assert_eq!(
            locate(&body.parts, &dna, &body.bones, false).unwrap(),
            [5., 7., 9.]
        );
        assert_eq!(body.bones, before);
        assert!(body.head_location(0x20, false).is_err());
        assert_eq!(body.head_location(0x37, false).unwrap(), [0.; 3]);
        body.parts
            .retain(|part| part.allocation.kind != NativeDpPartKind::HeadAndNeck);
        assert_eq!(
            locate(&body.parts, &dna, &body.bones, false).unwrap(),
            [0.; 3]
        );
    }
}
