//! Actual mounted arm joints composed with waist, shoulder and elbow stages.
use openshores_client::{
    asset_mount::AssetMount,
    world::{
        native_body::{
            NativeDpHandCache, NativeDpInitialization, NativeDpPartJoints, NativeDpPartKind,
        },
        native_dna::NativeDhDna,
    },
};

#[test]
#[ignore = "requires OPENSHORES_HAZERON_INSTALLATION pointing to a user-provided copy"]
fn mounted_elbows_use_initialized_joints_after_waist_and_shoulder_motion() {
    let mount = AssetMount::open(
        std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").expect("installation path"),
    )
    .unwrap();
    let mut hands = NativeDpHandCache::new(&mount);
    let mut branches = [false; 2];
    for form in [0, 2, 5, 6, 7] {
        let mut dna = NativeDhDna([0; 24]);
        dna.0[1] = 3;
        dna.0[2] = form;
        dna.0[5] = 2;
        dna.0[12] = 12;
        dna.0[14] = 0x31;
        dna.0[20] = 0x60;
        let mut body = NativeDpInitialization::begin(&dna);
        body.finish_part_calculations(-7, &mut hands).unwrap();
        assert_eq!(body.parts().len(), 16);
        body.rotate_waist(0.25).unwrap();
        for part in body.parts().iter().filter(|p| {
            matches!(
                p.allocation.kind,
                NativeDpPartKind::LeftArmAndHand | NativeDpPartKind::RightArmAndHand
            )
        }) {
            let mut bones = body.bones().clone();
            let indices = part.limb_bones().unwrap();
            let raw = &part.allocation.dna.0;
            let special = raw[1] & 3 != 0 && matches!(raw[2] & 7, 2 | 3 | 4 | 7);
            branches[usize::from(special)] = true;
            let NativeDpPartJoints::Arm {
                field_58, field_70, ..
            } = part.joints
            else {
                panic!("initialized arm joints");
            };
            let selected = &indices[1..if special { 2 } else { 3 }];
            for angle in [0.375_f32, -0.125, 0.25] {
                part.rotate_arm_shoulder(&mut bones, -0.125).unwrap();
                let before = bones.clone();
                let pivot = if special {
                    part.current_connect_loc(&bones).unwrap()
                } else {
                    bones.slots[indices[1]].project(field_58)
                };
                let (s, c) = f64::from(angle).sin_cos();
                part.rotate_arm_elbow(&mut bones, angle).unwrap();
                let after_pivot = if special {
                    part.current_connect_loc(&bones).unwrap()
                } else {
                    bones.slots[indices[1]].project(field_58)
                };
                for axis in 0..3 {
                    assert!((after_pivot[axis] - pivot[axis]).abs() < 1e-12);
                }
                for (index, bone) in bones.slots.iter().enumerate() {
                    if !selected.contains(&index) {
                        assert_eq!(*bone, before.slots[index]);
                        continue;
                    }
                    assert_eq!(bone.field_100, 0);
                    let old = before.slots[index].project([0.125, -0.25, 0.5]);
                    let local = [old[0] - pivot[0], old[1] - pivot[1], old[2] - pivot[2]];
                    let expected = [
                        old[0],
                        pivot[1] + local[1] * c - local[2] * s,
                        pivot[2] + local[1] * s + local[2] * c,
                    ];
                    let actual = bone.project([0.125, -0.25, 0.5]);
                    for axis in 0..3 {
                        assert!((actual[axis] - expected[axis]).abs() < 1e-12);
                    }
                }
                // Mounted bones are affine here, so ordinary Project is an
                // independent numeric oracle for the inlined wrist pivot.
                let before = bones.clone();
                let pivot = bones.slots[indices[2]].project(field_70);
                part.rotate_arm_wrist(&mut bones, angle).unwrap();
                for (index, bone) in bones.slots.iter().enumerate() {
                    if index != indices[2] {
                        assert_eq!(*bone, before.slots[index]);
                        continue;
                    }
                    assert_eq!(bone.field_100, 0);
                    let attached = bone.project(field_70);
                    for axis in 0..3 {
                        assert!((attached[axis] - pivot[axis]).abs() < 1e-12);
                    }
                    let old = before.slots[index].project([0.125, -0.25, 0.5]);
                    let local = [old[0] - pivot[0], old[1] - pivot[1], old[2] - pivot[2]];
                    let expected = [
                        old[0],
                        pivot[1] + local[1] * c - local[2] * s,
                        pivot[2] + local[1] * s + local[2] * c,
                    ];
                    let actual = bone.project([0.125, -0.25, 0.5]);
                    for axis in 0..3 {
                        assert!((actual[axis] - expected[axis]).abs() < 1e-12);
                    }
                }
                let before = bones.clone();
                let signed = if part.allocation.kind == NativeDpPartKind::LeftArmAndHand {
                    -angle
                } else {
                    angle
                };
                let rotate = |point: [f64; 3], pivot: [f64; 3]| {
                    let (s, c) = f64::from(signed).sin_cos();
                    let x = point[0] - pivot[0];
                    let y = point[1] - pivot[1];
                    [pivot[0] + x * c - y * s, pivot[1] + x * s + y * c, point[2]]
                };
                let elbow = before.slots[indices[1]].project(field_58);
                let wrist = rotate(before.slots[indices[2]].project(field_70), elbow);
                part.bow_wing(&mut bones, angle).unwrap();
                for (index, bone) in bones.slots.iter().enumerate() {
                    if !indices[1..].contains(&index) {
                        assert_eq!(*bone, before.slots[index]);
                        continue;
                    }
                    let mut expected =
                        rotate(before.slots[index].project([0.125, -0.25, 0.5]), elbow);
                    if index == indices[2] {
                        expected = rotate(expected, wrist);
                    }
                    let actual = bone.project([0.125, -0.25, 0.5]);
                    for axis in 0..3 {
                        assert!((actual[axis] - expected[axis]).abs() < 1e-12);
                    }
                    assert_eq!(bone.field_100, 0);
                }
            }
        }
    }
    assert_eq!(branches, [true; 2]);
}
