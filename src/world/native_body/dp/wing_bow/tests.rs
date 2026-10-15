use super::*;

fn part(kind: NativeDpPartKind, index: u32) -> NativeDpPartState {
    NativeDpPartState {
        allocation: NativeDpAllocatedPart {
            kind,
            index,
            dna: NativeDhDna([0; 24]),
        },
        connect_30: [199.; 3],
        joints: NativeDpPartJoints::Arm {
            field_58: [0.125, -0.25, 0.5],
            field_70: [-0.375, 0.5, -0.25],
            field_88: [99.; 3],
        },
    }
}

fn rotate(point: [f64; 3], pivot: [f64; 3], angle: f64) -> [f64; 3] {
    let (s, c) = angle.sin_cos();
    let x = point[0] - pivot[0];
    let y = point[1] - pivot[1];
    [pivot[0] + x * c - y * s, pivot[1] + x * s + y * c, point[2]]
}

#[test]
fn both_sides_pairs_and_fallbacks_apply_two_sequential_pivots() {
    for kind in [
        NativeDpPartKind::LeftArmAndHand,
        NativeDpPartKind::RightArmAndHand,
    ] {
        for index in [0, 1, 2, 99] {
            let part = part(kind, index);
            let indices = part.limb_bones().unwrap();
            let mut bones = NativeDpBoneTransforms::initialized();
            for bone in &mut bones.slots {
                bone.field_100 = 177;
            }
            let angle = if kind == NativeDpPartKind::LeftArmAndHand {
                -0.375_f32
            } else {
                0.375
            };
            let elbow = [0.125, -0.25, 0.5];
            let wrist = rotate([-0.375, 0.5, -0.25], elbow, f64::from(angle));
            part.bow_wing(&mut bones, 0.375).unwrap();
            for (i, bone) in bones.slots.iter().enumerate() {
                if i == indices[1] || i == indices[2] {
                    let mut expected = rotate([1., 2., 3.], elbow, f64::from(angle));
                    if i == indices[2] {
                        expected = rotate(expected, wrist, f64::from(angle));
                    }
                    let actual = bone.project([1., 2., 3.]);
                    for axis in 0..3 {
                        assert!((actual[axis] - expected[axis]).abs() < 2e-15);
                    }
                    assert_eq!(bone.field_100, 0);
                } else {
                    assert_eq!(bone.matrix_00, NativeDpBoneTransform::identity().matrix_00);
                    assert_eq!(bone.field_100, 177);
                }
            }
        }
    }
}

#[test]
fn dense_nonaffine_second_projection_occurs_after_first_stage_writes() {
    for kind in [
        NativeDpPartKind::LeftArmAndHand,
        NativeDpPartKind::RightArmAndHand,
    ] {
        let part = part(kind, 2);
        let indices = part.limb_bones().unwrap();
        let mut bones = NativeDpBoneTransforms::initialized();
        for bone in &mut bones.slots {
            bone.matrix_00 = std::array::from_fn(|i| (i as f64 - 3.) / 17.);
        }
        for angle in [0.25_f32, -0.125, 0.375] {
            let signed = if kind == NativeDpPartKind::LeftArmAndHand {
                -angle
            } else {
                angle
            };
            let mut expected = bones.clone();
            let first = bow_pivot(
                bow_project(&expected.slots[indices[1]], [0.125, -0.25, 0.5]),
                signed,
                false,
            );
            for &i in &indices[1..] {
                expected.slots[i] = first.multiply(&expected.slots[i]);
            }
            let second_pivot = expected.slots[indices[2]].project([-0.375, 0.5, -0.25]);
            let second = bow_pivot(second_pivot, signed, true);
            expected.slots[indices[2]] = second.multiply(&expected.slots[indices[2]]);
            part.bow_wing(&mut bones, angle).unwrap();
            assert_eq!(bones, expected);
        }
    }
}

#[test]
fn inlined_projection_handles_origin_sparse_coordinates_and_w_division() {
    let mut bone = NativeDpBoneTransform {
        matrix_00: [f64::NAN; 16],
        field_100: 123,
    };
    bone.matrix_00[4..8].copy_from_slice(&[1., 2., 3., 0.]);
    bone.matrix_00[12..].copy_from_slice(&[10., 20., 30., 1.]);
    assert_eq!(bow_project(&bone, [-0., 2., 0.]), [12., 24., 36.]);
    assert!(bow_project(&bone, [0.; 3]).iter().all(|v| v.is_nan()));
    bone.matrix_00.fill(0.);
    bone.matrix_00[12..].copy_from_slice(&[12., 24., 36., 1.]);
    bone.matrix_00[3] = 2.;
    bone.matrix_00[7] = 3.;
    bone.matrix_00[11] = 6.;
    assert_eq!(bow_project(&bone, [0.; 3]), [1., 2., 3.]);
    bone.matrix_00[15] = -11.;
    assert_eq!(bow_project(&bone, [0.; 3]), [12., 24., 36.]);
    assert_eq!(bone.field_100, 123);
}

#[test]
fn zero_and_missing_joint_paths_do_not_write_and_wrong_kinds_are_unsupported() {
    let mut part = part(NativeDpPartKind::LeftArmAndHand, 0);
    part.joints = NativeDpPartJoints::None;
    let mut bones = NativeDpBoneTransforms::initialized();
    let before = bones.clone();
    for angle in [0., -0.] {
        part.bow_wing(&mut bones, angle).unwrap();
        assert_eq!(bones, before);
    }
    assert_eq!(
        part.bow_wing(&mut bones, 0.25).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(bones, before);
    for kind in [
        NativeDpPartKind::Torso,
        NativeDpPartKind::Tail,
        NativeDpPartKind::HeadAndNeck,
        NativeDpPartKind::LeftLegAndFoot,
        NativeDpPartKind::RightLegAndFoot,
    ] {
        part.allocation.kind = kind;
        assert_eq!(
            part.bow_wing(&mut bones, 0.25).unwrap_err().kind(),
            io::ErrorKind::Unsupported
        );
        assert_eq!(bones, before);
    }
}

#[test]
fn zero_preserves_nonfinite_bits_but_nan_angle_enters_both_stages() {
    let part = part(NativeDpPartKind::LeftArmAndHand, 0);
    let mut bones = NativeDpBoneTransforms::initialized();
    bones.slots[23].matrix_00[0] = f64::from_bits(0x7ff8000000000042);
    bones.slots[23].field_100 = 199;
    let before = bones.clone();
    part.bow_wing(&mut bones, -0.).unwrap();
    for (a, b) in bones.slots.iter().zip(&before.slots) {
        assert_eq!(a.matrix_00.map(f64::to_bits), b.matrix_00.map(f64::to_bits));
        assert_eq!(a.field_100, b.field_100);
    }
    let mut bones = NativeDpBoneTransforms::initialized();
    part.bow_wing(&mut bones, f32::NAN).unwrap();
    for i in [23, 24] {
        assert!(bones.slots[i].matrix_00.iter().any(|v| v.is_nan()));
        assert_eq!(bones.slots[i].field_100, 0);
    }
    assert_eq!(bones.slots[25], NativeDpBoneTransform::identity());
}
