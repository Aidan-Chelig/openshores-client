use super::*;

fn part(dna: &NativeDhDna, kind: NativeDpPartKind, index: u32) -> NativeDpPartState {
    NativeDpPartState::allocated(NativeDpAllocatedPart {
        dna: dna.clone(),
        kind,
        index,
    })
}

#[test]
fn minimal_body_calculates_without_requesting_a_hand() {
    let dna = NativeDhDna([0; 24]);
    let mut body = NativeDpInitialization::begin(&dna);
    assert!(!body.parts_calculated());
    for part in &mut body.parts {
        part.calculate(&body.dimensions, &mut || panic!("no arms allocated"))
            .unwrap();
    }
    assert_eq!(body.parts.len(), 2);
    assert_eq!(body.parts[0].connect_30, [0.; 3]);
    assert!(matches!(body.parts[0].joints, NativeDpPartJoints::None));
    assert!(matches!(
        body.parts[1].joints,
        NativeDpPartJoints::Head { .. }
    ));
}

#[test]
fn arm_resource_failure_preserves_pre_call_writes_and_retry_finishes_hand() {
    let mut dna = NativeDhDna([0; 24]);
    dna.0[1] = 1;
    let dimensions = NativeDpDimensions::from_dna(&dna);
    let mut arm = part(&dna, NativeDpPartKind::LeftArmAndHand, 0);
    let error = arm
        .calculate(&dimensions, &mut || {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "missing mounted hand",
            ))
        })
        .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::NotFound);
    let NativeDpPartJoints::Arm {
        field_58,
        field_70,
        field_88,
    } = arm.joints
    else {
        panic!()
    };
    assert!(arm.connect_30[0] < 0.);
    assert!(field_58[1] > arm.connect_30[1]);
    assert!(field_70[1] > field_58[1]);
    assert_eq!(field_88, [0.; 3]);
    arm.calculate(&dimensions, &mut || Ok(8.)).unwrap();
    let NativeDpPartJoints::Arm {
        field_58: elbow,
        field_70: wrist,
        field_88: hand,
    } = arm.joints
    else {
        panic!()
    };
    assert_eq!(elbow, field_58);
    assert_eq!(wrist, field_70);
    assert_eq!(hand, [wrist[0], wrist[1] + 4., wrist[2]]);
}

#[test]
fn paired_wing_and_leg_parts_mirror_only_x() {
    let mut dna = NativeDhDna([0; 24]);
    dna.0[1] = 1;
    dna.0[2] = 2;
    for segmented in [0, 0x60] {
        dna.0[20] = segmented;
        let dimensions = NativeDpDimensions::from_dna(&dna);
        for (left, right) in [
            (
                NativeDpPartKind::LeftArmAndHand,
                NativeDpPartKind::RightArmAndHand,
            ),
            (
                NativeDpPartKind::LeftLegAndFoot,
                NativeDpPartKind::RightLegAndFoot,
            ),
        ] {
            for index in 0..3 {
                let mut l = part(&dna, left, index);
                let mut r = part(&dna, right, index);
                l.calculate(&dimensions, &mut || Ok(2.)).unwrap();
                r.calculate(&dimensions, &mut || Ok(2.)).unwrap();
                assert_eq!(l.connect_30[0], -r.connect_30[0]);
                assert_eq!(l.connect_30[1..], r.connect_30[1..]);
            }
        }
    }
}

#[test]
fn tail_retains_negative_zero_and_heads_ignore_duplicate_index() {
    let mut dna = NativeDhDna([0; 24]);
    dna.0[14] = 3;
    let dimensions = NativeDpDimensions::from_dna(&dna);
    let mut tail = part(&dna, NativeDpPartKind::Tail, 0);
    tail.calculate(&dimensions, &mut || panic!()).unwrap();
    assert_eq!(tail.connect_30[1].to_bits(), (-0.0_f64).to_bits());
    let mut first = part(&dna, NativeDpPartKind::HeadAndNeck, 0);
    let mut second = part(&dna, NativeDpPartKind::HeadAndNeck, 1);
    first.calculate(&dimensions, &mut || panic!()).unwrap();
    second.calculate(&dimensions, &mut || panic!()).unwrap();
    assert_eq!(first.connect_30, second.connect_30);
    assert_eq!(first.joints, second.joints);
}
