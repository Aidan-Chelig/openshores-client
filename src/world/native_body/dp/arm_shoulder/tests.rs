use super::*;

fn part(kind: NativeDpPartKind, index: u32, arms: u8, wing: u8) -> NativeDpPartState {
    let mut dna = NativeDhDna([0; 24]);
    dna.0[1] = arms;
    dna.0[2] = wing;
    NativeDpPartState {
        allocation: NativeDpAllocatedPart { kind, index, dna },
        connect_30: [0.25, -0.5, 0.75],
        joints: NativeDpPartJoints::None,
    }
}

#[test]
fn all_packed_wing_selectors_sides_and_pair_fallbacks_choose_actual_bones() {
    for kind in [
        NativeDpPartKind::LeftArmAndHand,
        NativeDpPartKind::RightArmAndHand,
    ] {
        for index in [0, 1, 2, 99] {
            for arms in 0..4 {
                for wing in 0..=255 {
                    let part = part(kind, index, arms, wing);
                    let mut bones = NativeDpBoneTransforms::initialized();
                    for bone in &mut bones.slots {
                        bone.field_100 = 99;
                    }
                    let before = bones.clone();
                    let pivot = part.current_connect_loc(&bones).unwrap();
                    let indices = part.limb_bones().unwrap();
                    part.rotate_arm_shoulder(&mut bones, 0.375).unwrap();
                    let all = arms == 0 || matches!(wing & 7, 0 | 1 | 5 | 6);
                    let selected = &indices[..if all { 3 } else { 1 }];
                    let (s, c) = f64::from(0.375_f32).sin_cos();
                    for (i, bone) in bones.slots.iter().enumerate() {
                        if selected.contains(&i) {
                            let m = bone.matrix_00;
                            assert_eq!(bone.field_100, 0);
                            assert_eq!(m[0], 1.);
                            assert!((m[5] - c).abs() < 1e-15);
                            assert!((m[6] - s).abs() < 1e-15);
                            assert!((m[9] + s).abs() < 1e-15);
                            assert!((m[10] - c).abs() < 1e-15);
                            let attached = bone.project(pivot);
                            for axis in 0..3 {
                                assert!((attached[axis] - pivot[axis]).abs() < 1e-15);
                            }
                        } else {
                            assert_eq!(*bone, before.slots[i]);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn dense_primary_inputs_and_repeated_current_attachment_use_native_product() {
    let part = part(NativeDpPartKind::RightArmAndHand, 1, 2, 0);
    let mut bones = NativeDpBoneTransforms::initialized();
    for bone in &mut bones.slots {
        bone.matrix_00 = std::array::from_fn(|i| (i as f64 + 1.) / 17.);
        bone.field_100 = 171;
    }
    for angle in [0.25, -0.125, 0.5] {
        let before = bones.clone();
        let transform = shoulder_pivot(part.current_connect_loc(&bones).unwrap(), angle);
        part.rotate_arm_shoulder(&mut bones, angle).unwrap();
        for index in part.limb_bones().unwrap() {
            assert_eq!(bones.slots[index], transform.multiply(&before.slots[index]));
        }
    }
}

#[test]
fn zero_angles_preserve_all_bits_and_wrong_kinds_are_unsupported() {
    let arm = part(NativeDpPartKind::LeftArmAndHand, 0, 1, 0);
    let mut bones = NativeDpBoneTransforms::initialized();
    bones.slots[25].matrix_00[0] = f64::NAN;
    bones.slots[25].field_100 = 123;
    let before = bones.clone();
    for angle in [0., -0.] {
        arm.rotate_arm_shoulder(&mut bones, angle).unwrap();
        for (a, b) in bones.slots.iter().zip(&before.slots) {
            assert_eq!(a.matrix_00.map(f64::to_bits), b.matrix_00.map(f64::to_bits));
            assert_eq!(a.field_100, b.field_100);
        }
    }
    for kind in [
        NativeDpPartKind::Torso,
        NativeDpPartKind::Tail,
        NativeDpPartKind::HeadAndNeck,
        NativeDpPartKind::LeftLegAndFoot,
        NativeDpPartKind::RightLegAndFoot,
    ] {
        assert_eq!(
            part(kind, 0, 1, 0)
                .rotate_arm_shoulder(&mut bones, 0.5)
                .unwrap_err()
                .kind(),
            io::ErrorKind::Unsupported
        );
    }
    let mut bones = NativeDpBoneTransforms::initialized();
    arm.rotate_arm_shoulder(&mut bones, f32::NAN).unwrap();
    assert!(bones.slots[25].matrix_00.iter().any(|v| v.is_nan()));
    assert_eq!(bones.slots[25].field_100, 0);
}
