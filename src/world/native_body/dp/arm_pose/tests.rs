use super::*;

fn arm(left: bool, pair: u32) -> NativeDpPartState {
    NativeDpPartState {
        allocation: NativeDpAllocatedPart {
            kind: if left {
                NativeDpPartKind::LeftArmAndHand
            } else {
                NativeDpPartKind::RightArmAndHand
            },
            index: pair,
            dna: NativeDhDna([0; 24]),
        },
        connect_30: [1., 2., 3.],
        joints: NativeDpPartJoints::Arm {
            lift_48: 0.2,
            splay_4c: 0.3,
            field_58: [1., 4., 3.],
            field_70: [1., 6., 3.],
            field_88: [1., 7., 3.],
        },
    }
}
fn near(a: [f64; 3], b: [f64; 3]) {
    for i in 0..3 {
        assert!((a[i] - b[i]).abs() < 1e-12, "{a:?} != {b:?}");
    }
}

#[test]
fn shoulder_axes_side_signs_and_all_three_output_bindings_match_native() {
    for left in [false, true] {
        for pair in 0..3 {
            for lift in [false, true] {
                let part = arm(left, pair);
                let indices = part.limb_bones().unwrap();
                let mut owner = NativeDpBoneTransforms::initialized();
                for bone in &mut owner.slots {
                    bone.translate([10., 20., 30.]);
                }
                let before = owner.clone();
                let pivot = part.current_connect_loc(&owner).unwrap();
                if lift {
                    part.lift_arm_shoulder(&mut owner, 0.5)
                } else {
                    part.splay_arm_shoulder(&mut owner, 0.5)
                }
                .unwrap();
                let angle = f64::from(if left == lift { 0.5_f32 } else { -0.5 });
                let (s, c) = angle.sin_cos();
                for i in 0..61 {
                    if indices.contains(&i) {
                        near(owner.slots[i].project(part.connect_30), pivot);
                        near(
                            owner.slots[i].project([2., 2., 3.]),
                            if lift {
                                [pivot[0] + c, pivot[1], pivot[2] - s]
                            } else {
                                [pivot[0] + c, pivot[1] + s, pivot[2]]
                            },
                        );
                        assert_ne!(owner.slots[i], before.slots[i]);
                    } else {
                        assert_eq!(owner.slots[i], before.slots[i]);
                    }
                }
            }
        }
    }
}

#[test]
fn lift_zero_is_not_a_noop_and_splay_zero_skips_bone_resolution() {
    let mut part = arm(true, 0);
    let indices = part.limb_bones().unwrap();
    let mut owner = NativeDpBoneTransforms::initialized();
    for &i in &indices {
        owner.slots[i].field_100 = 0xa5;
    }
    let before = owner.clone();
    part.splay_arm_shoulder(&mut owner, -0.).unwrap();
    assert_eq!(owner, before);
    part.lift_arm_shoulder(&mut owner, 0.).unwrap();
    for &i in &indices {
        assert_eq!(owner.slots[i].field_100, 0);
    }
    part.allocation.kind = NativeDpPartKind::Torso;
    assert_eq!(
        part.lift_arm_shoulder(&mut owner, 0.).unwrap_err().kind(),
        io::ErrorKind::Unsupported
    );
}

#[test]
fn relax_matches_explicit_native_order_and_override_changes_only_wrist_and_elbow() {
    let part = arm(true, 1);
    let angles = NativeDpArmStandAngles {
        shoulder: -0.5,
        elbow: 1.2,
        wrist: 0.25,
    };
    for field in [None, Some(0.3)] {
        let mut owner = NativeDpBoneTransforms::initialized();
        let mut expected = owner.clone();
        part.rotate_arm_shoulder(&mut expected, angles.shoulder)
            .unwrap();
        part.rotate_arm_wrist(
            &mut expected,
            if field.is_some() {
                f32::from_bits(0xbd0efa35)
            } else {
                angles.wrist
            },
        )
        .unwrap();
        part.rotate_arm_elbow(
            &mut expected,
            field.map_or(angles.elbow, |v| v - angles.shoulder),
        )
        .unwrap();
        part.splay_arm_shoulder(&mut expected, 0.3).unwrap();
        part.lift_arm_shoulder(&mut expected, 0.2).unwrap();
        part.relax_arm(&mut owner, angles, field).unwrap();
        assert_eq!(owner, expected);
    }
}

#[test]
fn inline_products_preserve_nonaffine_fourth_rows_and_do_not_drop_zero_products() {
    let a = lift_pivot([1., 2., 3.], 0.4);
    let mut b = NativeDpBoneTransform::identity();
    b.matrix_00 = std::array::from_fn(|i| (i as f64 - 3.) * 0.25);
    let actual = lift_last_product(&a, &b);
    let expected = a.multiply(&b);
    for i in 0..16 {
        assert!((actual.matrix_00[i] - expected.matrix_00[i]).abs() < 1e-12);
    }
    let a = splay_pivot([1., 2., 3.], 0.4);
    let actual = splay_first_product(&a, &b);
    let expected = a.multiply(&b);
    for i in 0..16 {
        assert!((actual.matrix_00[i] - expected.matrix_00[i]).abs() < 1e-12);
    }
    b.matrix_00[2] = f64::INFINITY;
    let out = splay_first_product(&a, &b);
    assert!(out.matrix_00[0].is_nan()); // 0 * infinity is NOT skipped.
}
