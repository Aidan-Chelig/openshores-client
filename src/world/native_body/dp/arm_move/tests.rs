use super::*;

#[test]
fn native_operator_uses_primary_operands_left_associative_dots_and_clears_result_flag() {
    let mut a = NativeDpBoneTransform::identity();
    let mut b = NativeDpBoneTransform::identity();
    a.matrix_00 = std::array::from_fn(|i| (i + 1) as f64);
    b.matrix_00 = std::array::from_fn(|i| i as f64 - 10.);
    a.field_100 = 77;
    b.field_100 = 199;
    let before = (a, b);
    let result = a.multiply(&b);
    for col in 0..4 {
        for row in 0..4 {
            let expected = (0..4)
                .map(|k| a.matrix_00[k * 4 + row] * b.matrix_00[col * 4 + k])
                .sum::<f64>();
            assert_eq!(result.matrix_00[col * 4 + row], expected);
        }
    }
    assert_eq!(result.field_100, 0);
    assert_eq!((a, b), before);
    a.matrix_00[0] = 1e16;
    a.matrix_00[4] = 1.;
    a.matrix_00[8] = -1e16;
    a.matrix_00[12] = 1.;
    b.matrix_00[..4].fill(1.);
    assert_eq!(a.multiply(&b).matrix_00[0], 1.);
}

#[test]
fn every_arm_moves_its_three_actual_bones_including_nonaffine_columns() {
    for kind in [
        NativeDpPartKind::LeftArmAndHand,
        NativeDpPartKind::RightArmAndHand,
    ] {
        for index in [0, 1, 2, 99] {
            let part = NativeDpPartState {
                allocation: NativeDpAllocatedPart {
                    kind,
                    index,
                    dna: NativeDhDna([0; 24]),
                },
                connect_30: [0.; 3],
                joints: NativeDpPartJoints::None,
            };
            let mut bones = NativeDpBoneTransforms::initialized();
            for bone in &mut bones.slots {
                bone.matrix_00 = std::array::from_fn(|i| (i + 1) as f64);
                bone.field_100 = 199;
            }
            let before = bones.clone();
            let indices = part.limb_bones().unwrap();
            let delta = [2., -3., 4.];
            part.move_arm_by(&mut bones, delta).unwrap();
            for (i, bone) in bones.slots.iter().enumerate() {
                if indices.contains(&i) {
                    for col in 0..4 {
                        for (row, d) in delta.iter().enumerate() {
                            assert_eq!(
                                bone.matrix_00[col * 4 + row],
                                before.slots[i].matrix_00[col * 4 + row]
                                    + before.slots[i].matrix_00[col * 4 + 3] * d
                            );
                        }
                    }
                    assert_eq!(bone.field_100, 0);
                } else {
                    assert_eq!(*bone, before.slots[i]);
                }
            }
            for bone in &mut bones.slots {
                bone.field_100 = 199;
            }
            part.move_arm_by(&mut bones, [-0., 0., -0.]).unwrap();
            for i in indices {
                assert_eq!(bones.slots[i].field_100, 0);
            }
        }
    }
}

#[test]
fn body_guards_and_wrong_kind_failures_precede_writes() {
    let mut dna = NativeDhDna([0; 24]);
    dna.0[12] = 4;
    dna.0[20] = 0x40;
    let mut body = NativeDpInitialization::begin(&dna);
    let before = body.bones().clone();
    assert!(!body.rotate_waist(0.).unwrap());
    assert!(!body.rotate_waist(-0.).unwrap());
    assert_eq!(
        body.rotate_waist(0.25).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    let torso = body.parts()[0].clone();
    let mut bones = before.clone();
    assert_eq!(
        torso.move_arm_by(&mut bones, [1.; 3]).unwrap_err().kind(),
        io::ErrorKind::Unsupported
    );
    assert_eq!(bones, before);
    assert_eq!(*body.bones(), before);
    let mut no_waist = NativeDpInitialization::begin(&NativeDhDna([0; 24]));
    assert!(!no_waist.rotate_waist(f32::NAN).unwrap());
}
