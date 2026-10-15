use super::*;

fn part(kind: NativeDpPartKind, index: u32, arms: u8, wing: u8) -> NativeDpPartState {
    let mut dna = NativeDhDna([0; 24]);
    dna.0[1] = arms;
    dna.0[2] = wing;
    NativeDpPartState {
        allocation: NativeDpAllocatedPart { kind, index, dna },
        connect_30: [0.25, -0.5, 0.75],
        joints: NativeDpPartJoints::Arm {
            field_58: [-0.125, 0.375, -0.625],
            field_70: [99.; 3],
            field_88: [199.; 3],
        },
    }
}

#[test]
fn all_wing_bytes_counts_sides_and_pair_fallbacks_use_exact_pivots_and_outputs() {
    for kind in [
        NativeDpPartKind::LeftArmAndHand,
        NativeDpPartKind::RightArmAndHand,
    ] {
        for pair in [0, 1, 2, 99] {
            for count in 0..4 {
                for wing in 0..=255 {
                    let part = part(kind, pair, count, wing);
                    let mut bones = NativeDpBoneTransforms::initialized();
                    for bone in &mut bones.slots {
                        bone.field_100 = 91;
                    }
                    let before = bones.clone();
                    let special = count != 0 && matches!(wing & 7, 2 | 3 | 4 | 7);
                    let pivot = if special {
                        part.connect_30
                    } else {
                        [-0.125, 0.375, -0.625]
                    };
                    let indices = part.limb_bones().unwrap();
                    let selected = &indices[1..if special { 2 } else { 3 }];
                    part.rotate_arm_elbow(&mut bones, 0.375).unwrap();
                    let (s, c) = f64::from(0.375_f32).sin_cos();
                    for (index, bone) in bones.slots.iter().enumerate() {
                        if selected.contains(&index) {
                            assert_eq!(bone.field_100, 0);
                            assert_eq!(bone.matrix_00[0], 1.);
                            assert!((bone.matrix_00[5] - c).abs() < 1e-15);
                            assert!((bone.matrix_00[6] - s).abs() < 1e-15);
                            let attached = bone.project(pivot);
                            for axis in 0..3 {
                                assert!((attached[axis] - pivot[axis]).abs() < 1e-15);
                            }
                        } else {
                            assert_eq!(*bone, before.slots[index]);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn current_elbow_and_attachment_pivots_follow_their_distinct_bones() {
    for wing in [0, 2, 5, 6, 7] {
        let part = part(NativeDpPartKind::RightArmAndHand, 2, 3, wing);
        let indices = part.limb_bones().unwrap();
        let mut bones = NativeDpBoneTransforms::initialized();
        for (i, bone) in bones.slots.iter_mut().enumerate() {
            bone.matrix_00 = std::array::from_fn(|k| (i * 16 + k + 1) as f64 / 107.);
            bone.field_100 = 181;
        }
        for angle in [0.25, -0.125, 0.5] {
            let before = bones.clone();
            let special = matches!(wing, 2 | 7);
            let pivot = if special {
                part.current_connect_loc(&before).unwrap()
            } else {
                before.slots[indices[1]].project([-0.125, 0.375, -0.625])
            };
            let transform = elbow_pivot(pivot, angle, special);
            part.rotate_arm_elbow(&mut bones, angle).unwrap();
            for index in &indices[1..if special { 2 } else { 3 }] {
                assert_eq!(
                    bones.slots[*index],
                    transform.multiply(&before.slots[*index])
                );
            }
            assert_eq!(bones.slots[indices[0]], before.slots[indices[0]]);
        }
    }
}

#[test]
fn missing_joints_error_before_writes_but_wing_and_zero_paths_do_not_require_them() {
    let mut part = part(NativeDpPartKind::LeftArmAndHand, 0, 1, 0);
    part.joints = NativeDpPartJoints::None;
    let mut bones = NativeDpBoneTransforms::initialized();
    bones.slots[24].field_100 = 199;
    let before = bones.clone();
    assert_eq!(
        part.rotate_arm_elbow(&mut bones, 0.5).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(bones, before);
    for angle in [0., -0.] {
        part.rotate_arm_elbow(&mut bones, angle).unwrap();
        assert_eq!(bones, before);
    }
    part.allocation.dna.0[2] = 2;
    part.rotate_arm_elbow(&mut bones, 0.5).unwrap();
    assert_ne!(bones, before);
    for kind in [
        NativeDpPartKind::Torso,
        NativeDpPartKind::Tail,
        NativeDpPartKind::HeadAndNeck,
        NativeDpPartKind::LeftLegAndFoot,
        NativeDpPartKind::RightLegAndFoot,
    ] {
        part.allocation.kind = kind;
        let before = bones.clone();
        assert_eq!(
            part.rotate_arm_elbow(&mut bones, 0.5).unwrap_err().kind(),
            io::ErrorKind::Unsupported
        );
        assert_eq!(bones, before);
    }
}

#[test]
fn nan_angle_enters_rotation_and_zero_preserves_nan_payload_and_flags() {
    let part = part(NativeDpPartKind::LeftArmAndHand, 0, 1, 0);
    let mut bones = NativeDpBoneTransforms::initialized();
    bones.slots[24].matrix_00[0] = f64::from_bits(0x7ff8000000000042);
    bones.slots[24].field_100 = 123;
    let before = bones.clone();
    part.rotate_arm_elbow(&mut bones, -0.).unwrap();
    for (a, b) in bones.slots.iter().zip(&before.slots) {
        assert_eq!(a.matrix_00.map(f64::to_bits), b.matrix_00.map(f64::to_bits));
        assert_eq!(a.field_100, b.field_100);
    }
    let mut bones = NativeDpBoneTransforms::initialized();
    part.rotate_arm_elbow(&mut bones, f32::NAN).unwrap();
    assert!(bones.slots[24].matrix_00.iter().any(|v| v.is_nan()));
    assert_eq!(bones.slots[24].field_100, 0);
    assert_eq!(bones.slots[25], NativeDpBoneTransform::identity());
}
