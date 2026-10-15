use super::*;

fn part(index: u32) -> NativeDpPartState {
    NativeDpPartState {
        allocation: NativeDpAllocatedPart {
            kind: NativeDpPartKind::HeadAndNeck,
            index,
            dna: NativeDhDna([0; 24]),
        },
        connect_30: [999.; 3],
        joints: NativeDpPartJoints::Head {
            field_50: [1., 2., 3.],
        },
    }
}

#[test]
fn head_uses_joint_attachment_bone_range_and_ordered_default_face_parts() {
    let owner = NativeDhDna([0; 24]);
    let d = NativeDpDimensions::from_dna(&owner);
    for index in [0, 1, 2] {
        let plan = part(index)
            .head_skin_plan(&owner, &d, 0, [1., 2., 3., 4.])
            .unwrap();
        assert_eq!(plan.combined_key, ":data/0Head01010101");
        assert_eq!(plan.geometry.fields_40, [1., 2., 3.]);
        assert_eq!(plan.bones, [if index == 1 { 12 } else { 7 }; 3]);
        assert_eq!(
            plan.fallback
                .iter()
                .map(|p| p.resource.as_str())
                .collect::<Vec<_>>(),
            [
                ":data/0Skull.3dsN",
                ":data/0Eye01.3dsN",
                ":data/0Ear01.3dsN",
                ":data/0Nose01.3dsN",
                ":data/0Mouth01.3dsN"
            ]
        );
        assert!(plan.neck.is_none() && plan.eye_reference.is_none());
        assert_eq!(plan.geometry.fields_58[2], f64::from(0.5_f32 / 1.5_f32));
    }
}

#[test]
fn helmet_selector_uses_full_native_field_and_separate_eye_reference() {
    let owner = NativeDhDna([0; 24]);
    let d = NativeDpDimensions::from_dna(&owner);
    for field in [-1, 0, 0x41, 0x48, 0x6a, 0x9a, 0x9c, 0x148, i32::MAX] {
        let plan = part(0).head_skin_plan(&owner, &d, field, [0.; 4]).unwrap();
        let pa = matches!(field, 0x48 | 0x6a | 0x9a | 0x9c);
        assert_eq!(plan.fallback.len(), if pa { 1 } else { 5 });
        assert_eq!(
            plan.fallback[0].resource,
            if pa {
                ":data/PA0Skull.3dsN"
            } else {
                ":data/0Skull.3dsN"
            }
        );
        assert_eq!(
            plan.eye_reference,
            pa.then(|| ":data/0Skull.3dsN".to_owned())
        );
    }
}

#[test]
fn integrated_skull_features_are_omitted_but_horn_and_crest_order_remains() {
    let mut owner = NativeDhDna([0; 24]);
    owner.0[16] = 5;
    let d = NativeDpDimensions::from_dna(&owner);
    let mut p = part(0);
    p.allocation.dna = owner.clone();
    let raw = &mut p.allocation.dna.0;
    raw[8] = 4;
    raw[6] = 4;
    raw[15] = 3;
    raw[13] = 5;
    raw[12] = 2;
    raw[7] = 0x60;
    raw[4] = 0x0c;
    let plan = p.head_skin_plan(&owner, &d, 0, [0.; 4]).unwrap();
    assert_eq!(plan.combined_key, ":data/5Head41413151HS12HC32");
    assert_eq!(
        plan.fallback
            .iter()
            .map(|p| p.resource.as_str())
            .collect::<Vec<_>>(),
        [
            ":data/5Skull.3dsN",
            ":data/5Horn12.3dsN",
            ":data/5HeadCrest32.3dsN"
        ]
    );
}

#[test]
fn neck_condition_resource_bones_and_float_attachment_offset_match_native() {
    let owner = NativeDhDna([0; 24]);
    let d = NativeDpDimensions::from_dna(&owner);
    let mut p = part(0);
    p.allocation.dna.0[14] = 2;
    let neck = p
        .head_skin_plan(&owner, &d, 0, [0.; 4])
        .unwrap()
        .neck
        .unwrap();
    assert_eq!(neck.resource, ":data/0Neck21.3dsN");
    assert_eq!(neck.bones, [8, 9, 7]);
    assert_eq!(
        neck.geometry.fields_40[1],
        2. - f64::from((d.fields_4080_through_40b8()[10] / 2.5_f32) * 2.)
    );
    p.allocation.dna.0[5] = 2;
    assert!(
        p.head_skin_plan(&owner, &d, 0, [0.; 4])
            .unwrap()
            .neck
            .is_none()
    );
    p.allocation.index = 1;
    let neck = p
        .head_skin_plan(&owner, &d, 0, [0.; 4])
        .unwrap()
        .neck
        .unwrap();
    assert_eq!(neck.resource, ":data/0Neck32.3dsN");
    assert_eq!(neck.bones, [8, 10, 12]);
    p.allocation.dna.0[14] = 3;
    let neck = p
        .head_skin_plan(&owner, &d, 0, [0.; 4])
        .unwrap()
        .neck
        .unwrap();
    assert_eq!(neck.resource, ":data/0Neck42.3dsN");
    assert_eq!(neck.bones, [8, 8, 12]); // Native default branch for count4.
}

#[test]
fn wrong_kind_bad_joint_storage_and_incomplete_owner_fail_explicitly() {
    let owner = NativeDhDna([0; 24]);
    let d = NativeDpDimensions::from_dna(&owner);
    let mut p = part(0);
    p.joints = NativeDpPartJoints::None;
    assert_eq!(
        p.head_skin_plan(&owner, &d, 0, [0.; 4]).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    p.allocation.kind = NativeDpPartKind::Torso;
    assert_eq!(
        p.head_skin_plan(&owner, &d, 0, [0.; 4]).unwrap_err().kind(),
        io::ErrorKind::Unsupported
    );
    assert_eq!(
        NativeDpInitialization::begin(&owner)
            .head_skin_geometry_plan(0, 0, [0.; 4])
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidData
    );
}

#[test]
fn skin_recalculation_uses_fresh_attachment_without_changing_live_pose_copy() {
    let owner = NativeDhDna([0; 24]);
    let d = NativeDpDimensions::from_dna(&owner);
    let posed = part(0);
    let mut fresh = posed.clone();
    fresh
        .calculate(&d, &mut || {
            panic!("head OneTimeCalcs must not request HandLength")
        })
        .unwrap();
    let plan = fresh.head_skin_plan(&owner, &d, 0, [0.; 4]).unwrap();
    assert_eq!(plan.geometry.fields_40, [0.; 3]);
    assert_eq!(
        posed.joints,
        NativeDpPartJoints::Head {
            field_50: [1., 2., 3.]
        }
    );
}
