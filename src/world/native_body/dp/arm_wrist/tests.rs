use super::*;

fn part(kind: NativeDpPartKind, index: u32, point: [f64; 3]) -> NativeDpPartState {
    NativeDpPartState {
        allocation: NativeDpAllocatedPart {
            kind,
            index,
            dna: NativeDhDna([0; 24]),
        },
        connect_30: [199.; 3],
        joints: NativeDpPartJoints::Arm {
            field_58: [99.; 3],
            field_70: point,
            field_88: [299.; 3],
        },
    }
}

#[test]
fn every_arm_side_pair_and_fallback_rotates_only_output_three_about_field70() {
    for kind in [
        NativeDpPartKind::LeftArmAndHand,
        NativeDpPartKind::RightArmAndHand,
    ] {
        for index in [0, 1, 2, 99] {
            let pivot = [0.125, -0.25, 0.5];
            let part = part(kind, index, pivot);
            let selected = part.limb_bones().unwrap()[2];
            let mut bones = NativeDpBoneTransforms::initialized();
            for bone in &mut bones.slots {
                bone.field_100 = 199;
            }
            let before = bones.clone();
            part.rotate_arm_wrist(&mut bones, 0.375).unwrap();
            let (s, c) = f64::from(0.375_f32).sin_cos();
            for (i, bone) in bones.slots.iter().enumerate() {
                if i == selected {
                    assert_eq!(bone.field_100, 0);
                    let actual = bone.project([1., 2., 3.]);
                    let expected = [
                        1.,
                        pivot[1] + (2. - pivot[1]) * c - (3. - pivot[2]) * s,
                        pivot[2] + (2. - pivot[1]) * s + (3. - pivot[2]) * c,
                    ];
                    for axis in 0..3 {
                        assert!((actual[axis] - expected[axis]).abs() < 1e-15);
                    }
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

#[test]
fn inlined_projection_preserves_origin_sparse_nonaffine_and_cancellation_paths() {
    let mut bone = NativeDpBoneTransform {
        matrix_00: [0.; 16],
        field_100: 177,
    };
    bone.matrix_00[12..].copy_from_slice(&[12., 24., 36., 1.]);
    bone.matrix_00[3] = 2.;
    bone.matrix_00[7] = 3.;
    bone.matrix_00[11] = 6.;
    assert_eq!(wrist_project(&bone, [-0.; 3]), [1., 2., 3.]);
    bone.matrix_00[15] = -11.;
    assert_eq!(wrist_project(&bone, [0.; 3]), [12., 24., 36.]);
    bone.matrix_00.fill(f64::NAN);
    bone.matrix_00[4..8].copy_from_slice(&[1., 2., 3., 0.]);
    bone.matrix_00[12..].copy_from_slice(&[10., 20., 30., 1.]);
    assert_eq!(wrist_project(&bone, [0., 2., -0.]), [12., 24., 36.]);
    assert!(wrist_project(&bone, [0.; 3]).iter().all(|v| v.is_nan()));
    bone.matrix_00.fill(0.);
    bone.matrix_00[0] = 1e16;
    bone.matrix_00[4] = -1e16;
    bone.matrix_00[8] = 1.;
    bone.matrix_00[12] = 1.;
    bone.matrix_00[15] = 1.;
    assert_eq!(wrist_project(&bone, [1.; 3])[0], 2.);
    assert_eq!(bone.field_100, 177);
    for subset in 1..8 {
        let point: [f64; 3] = std::array::from_fn(|axis| {
            if subset & (1 << axis) == 0 {
                0.
            } else {
                [0.25, -2., 3.][axis]
            }
        });
        for w in [0., 1., 2., -3.] {
            bone.matrix_00 = std::array::from_fn(|i| (i as f64 - 3.) * 0.25);
            for i in [3, 7, 11] {
                bone.matrix_00[i] = 0.;
            }
            bone.matrix_00[15] = w;
            let actual = wrist_project(&bone, point);
            for (row, value) in actual.iter().enumerate() {
                let terms: [f64; 3] = std::array::from_fn(|axis| {
                    if point[axis] == 0. {
                        0.
                    } else {
                        bone.matrix_00[axis * 4 + row] * point[axis]
                    }
                });
                let mut expected = ((terms[0] + terms[1]) + terms[2]) + bone.matrix_00[12 + row];
                if w != 0. && w != 1. {
                    expected /= w;
                }
                assert_eq!(value.to_bits(), expected.to_bits());
            }
        }
    }
}

#[test]
fn required_joint_errors_and_unsupported_kinds_do_not_mutate_bones() {
    let mut part = part(NativeDpPartKind::LeftArmAndHand, 0, [0.; 3]);
    part.joints = NativeDpPartJoints::None;
    let mut bones = NativeDpBoneTransforms::initialized();
    let before = bones.clone();
    assert_eq!(
        part.rotate_arm_wrist(&mut bones, 0.5).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(bones, before);
    for angle in [0., -0.] {
        part.rotate_arm_wrist(&mut bones, angle).unwrap();
        assert_eq!(bones, before);
    }
    for kind in [
        NativeDpPartKind::Torso,
        NativeDpPartKind::Tail,
        NativeDpPartKind::HeadAndNeck,
        NativeDpPartKind::LeftLegAndFoot,
        NativeDpPartKind::RightLegAndFoot,
    ] {
        part.allocation.kind = kind;
        assert_eq!(
            part.rotate_arm_wrist(&mut bones, 0.5).unwrap_err().kind(),
            io::ErrorKind::Unsupported
        );
        assert_eq!(bones, before);
    }
}

#[test]
fn repeated_nonaffine_pivots_are_recomputed_and_nan_angles_enter_rotation() {
    let part = part(NativeDpPartKind::RightArmAndHand, 2, [0.25, -0.5, 0.75]);
    let index = part.limb_bones().unwrap()[2];
    let mut bones = NativeDpBoneTransforms::initialized();
    bones.slots[index].matrix_00 = std::array::from_fn(|i| (i + 1) as f64 / 17.);
    for angle in [0.25, -0.125, 0.375] {
        let before = bones.slots[index];
        let pivot = wrist_project(&before, [0.25, -0.5, 0.75]);
        part.rotate_arm_wrist(&mut bones, angle).unwrap();
        assert_eq!(
            bones.slots[index],
            wrist_pivot(pivot, angle).multiply(&before)
        );
    }
    part.rotate_arm_wrist(&mut bones, f32::NAN).unwrap();
    assert!(bones.slots[index].matrix_00.iter().any(|v| v.is_nan()));
    assert_eq!(bones.slots[index].field_100, 0);
}

#[test]
fn signed_zero_preserves_nonfinite_payloads_and_every_flag_bit() {
    let part = part(NativeDpPartKind::LeftArmAndHand, 0, [f64::NAN; 3]);
    let mut bones = NativeDpBoneTransforms::initialized();
    for (i, bone) in bones.slots.iter_mut().enumerate() {
        bone.matrix_00[0] = f64::from_bits(0x7ff8000000000042 + i as u64);
        bone.matrix_00[1] = -0.;
        bone.field_100 = (i + 1) as u8;
    }
    let before = bones.clone();
    for angle in [0., -0.] {
        part.rotate_arm_wrist(&mut bones, angle).unwrap();
        for (a, b) in bones.slots.iter().zip(&before.slots) {
            assert_eq!(a.matrix_00.map(f64::to_bits), b.matrix_00.map(f64::to_bits));
            assert_eq!(a.field_100, b.field_100);
        }
    }
}
