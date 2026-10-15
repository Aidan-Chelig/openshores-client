use super::*;
use crate::world::{native_body::NativeDpInitialization, native_dna::NativeDhDna};

fn head(index: u32, count: u8, second: u8) -> NativeDpPartState {
    let mut dna = NativeDhDna([0; 24]);
    dna.0[14] = count;
    dna.0[5] = second << 1;
    NativeDpPartState {
        allocation: super::super::NativeDpAllocatedPart {
            kind: NativeDpPartKind::HeadAndNeck,
            index,
            dna,
        },
        connect_30: [0., 1., 0.],
        joints: NativeDpPartJoints::Head {
            field_50: [0., 2., 0.],
        },
    }
}

#[test]
fn full_head_pose_preserves_attachment_and_writes_requested_final_orientation() {
    for index in [0, 1] {
        for count in 0..4 {
            for second in 0..2 {
                let part = head(index, count, second);
                let body = NativeDpInitialization::begin(&part.allocation.dna);
                let mut bones = body.bones().clone();
                for (i, bone) in bones.slots.iter_mut().enumerate() {
                    bone.translate([i as f64 * 0.125, 0., -2.]);
                    bone.field_100 = 199;
                }
                let before = bones.clone();
                let attachment = part.current_connect_loc(&bones).unwrap();
                part.tilt_head_to(&mut bones, body.dimensions(), 0.25, -0.125)
                    .unwrap();
                let start = if index == 1 { 12 } else { 7 };
                let mut expected = NativeDpBoneTransform::identity();
                expected.rotate_x(0.25);
                expected.rotate_z(-0.125);
                assert_eq!(
                    &bones.slots[start].matrix_00[..12],
                    &expected.matrix_00[..12]
                );
                for (i, bone) in bones.slots.iter().enumerate() {
                    if (start..=start + usize::from(count + second)).contains(&i) {
                        assert_eq!(bone.field_100, 0);
                    } else {
                        assert_eq!(*bone, before.slots[i]);
                    }
                }
                // With neck segments the attachment bone isn't overwritten by
                // final head-matrix assembly. It must remain fixed.
                if count + second != 0 {
                    let after = part.current_connect_loc(&bones).unwrap();
                    for i in 0..3 {
                        assert!((attachment[i] - after[i]).abs() < 1e-12);
                    }
                }
            }
        }
    }
}

#[test]
fn move_by_premultiplies_translation_including_non_affine_columns() {
    for index in [0, 1] {
        let part = head(index, 3, 1);
        let mut bones = NativeDpBoneTransforms::initialized();
        for bone in &mut bones.slots {
            bone.matrix_00 = std::array::from_fn(|i| (i + 1) as f64);
            bone.field_100 = 199;
        }
        let before = bones.clone();
        let delta = [2., -3., 4.];
        part.move_head_by(&mut bones, delta).unwrap();
        let start = if index == 1 { 12 } else { 7 };
        for (i, bone) in bones.slots.iter().enumerate() {
            if (start..=start + 4).contains(&i) {
                for column in 0..4 {
                    let k = column * 4;
                    let original = &before.slots[i].matrix_00;
                    for row in 0..3 {
                        assert_eq!(
                            bone.matrix_00[k + row],
                            original[k + row] + original[k + 3] * delta[row]
                        );
                    }
                    assert_eq!(bone.matrix_00[k + 3], original[k + 3]);
                }
                assert_eq!(bone.field_100, 0);
            } else {
                assert_eq!(*bone, before.slots[i]);
            }
        }
    }
}

#[test]
fn orientation_uses_actual_up_vector_and_unordered_branches() {
    let mut bones = NativeDpBoneTransforms::initialized();
    assert_eq!(orientation::correction(&bones, false), None);
    assert_eq!(orientation::correction(&bones, true), None);
    bones.slots[0].rotate_x(0.375);
    assert!((orientation::correction(&bones, true).unwrap() - 0.375).abs() < 1e-6);
    assert_eq!(orientation::correction(&bones, false), None);
    bones.slots[0].matrix_00[4] = 0.;
    bones.slots[0].matrix_00[5] = 0.;
    bones.slots[0].matrix_00[6] = -1.;
    assert_eq!(
        orientation::correction(&bones, true).unwrap().to_bits(),
        (-f32::from_bits(0x3fc90fdb)).to_bits()
    );
    bones.slots[0].matrix_00[6] = f64::NAN;
    assert!(orientation::correction(&bones, true).unwrap().is_nan());
}

#[test]
fn move_by_zero_and_invalid_joint_state_preserve_owner() {
    let mut part = head(0, 2, 0);
    let body = NativeDpInitialization::begin(&part.allocation.dna);
    let mut bones = body.bones().clone();
    for bone in &mut bones.slots {
        bone.field_100 = 199;
    }
    let before = bones.clone();
    part.move_head_by(&mut bones, [-0., 0., -0.]).unwrap();
    assert_eq!(bones, before);
    part.joints = NativeDpPartJoints::None;
    assert_eq!(
        part.tilt_head_to(&mut bones, body.dimensions(), 0., 0.)
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(bones, before);
}
