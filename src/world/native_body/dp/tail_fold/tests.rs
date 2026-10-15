use super::*;

fn tail(selector: u8, shape: u8, form: u8) -> NativeDpPartState {
    let mut dna = NativeDhDna([0; 24]);
    dna.0[14] = selector << 6 | shape << 4;
    dna.0[16] = form;
    NativeDpPartState {
        allocation: NativeDpAllocatedPart {
            kind: NativeDpPartKind::Tail,
            index: 0,
            dna,
        },
        connect_30: [0., 2., 0.],
        joints: NativeDpPartJoints::None,
    }
}

#[test]
fn fold_accumulates_joint_angles_but_root_uses_a_separate_quarter_turn() {
    for selector in 1..4 {
        for lateral in [false, true] {
            for flag in [false, true] {
                let part = if lateral {
                    tail(selector, 3, 3)
                } else {
                    tail(selector, 3, 0)
                };
                let dims = NativeDpDimensions::from_dna(&part.allocation.dna);
                let mut bones = NativeDpBoneTransforms::initialized();
                for b in &mut bones.slots {
                    b.field_100 = 199;
                }
                let before = bones.clone();
                part.fold_tail(&mut bones, &dims, flag).unwrap();
                let end = 16 + usize::from(selector) * 2;
                for (i, bone) in bones.slots.iter().enumerate() {
                    if (17..=end).contains(&i) {
                        let direction = if lateral && !flag { 1. } else { -1. };
                        let root = f64::from(f32::from_bits(0x3fc90fdb)) * direction;
                        let angle = f64::from(f32::from_bits(0x3f490fdb) / f32::from(selector * 2))
                            * direction;
                        let accumulated = root + (i - 17) as f64 * angle;
                        let (c, s) = if lateral { (0, 1) } else { (5, 6) };
                        assert!((bone.matrix_00[c] - accumulated.cos()).abs() < 1e-12);
                        assert!((bone.matrix_00[s] - accumulated.sin()).abs() < 1e-12);
                        assert_eq!(bone.field_100, 0);
                    } else {
                        assert_eq!(*bone, before.slots[i]);
                    }
                }
            }
        }
    }
}

#[test]
fn root_fold_premultiplies_a_dense_non_affine_bone_around_its_projected_pivot() {
    for flag in [false, true] {
        let part = tail(1, 3, 3);
        let dims = NativeDpDimensions::from_dna(&part.allocation.dna);
        let mut bones = NativeDpBoneTransforms::initialized();
        bones.slots[17].matrix_00 = [
            1., 2., 3., 0.125, 4., 5., 6., -0.25, 7., 8., 9., 0.5, 10., 11., 12., 2.,
        ];
        let before = bones.slots[17];
        let pivot = before.project(part.connect_30);
        let angle = f64::from(f32::from_bits(0x3fc90fdb)) * if flag { -1. } else { 1. };
        let (s, c) = angle.sin_cos();
        let rotation = [[c, -s, 0.], [s, c, 0.], [0., 0., 1.]];
        let t: [f64; 3] = std::array::from_fn(|row| {
            pivot[row] - (0..3).map(|k| rotation[row][k] * pivot[k]).sum::<f64>()
        });
        part.fold_tail(&mut bones, &dims, flag).unwrap();
        for col in 0..4 {
            for row in 0..3 {
                let expected = (0..3)
                    .map(|k| rotation[row][k] * before.matrix_00[col * 4 + k])
                    .sum::<f64>()
                    + t[row] * before.matrix_00[col * 4 + 3];
                assert!((bones.slots[17].matrix_00[col * 4 + row] - expected).abs() < 1e-12);
            }
            assert_eq!(
                bones.slots[17].matrix_00[col * 4 + 3],
                before.matrix_00[col * 4 + 3]
            );
        }
    }
}

#[test]
fn no_segments_noop_and_wrong_kind_rejection_preserve_all_state() {
    let mut part = tail(0, 3, 3);
    let dims = NativeDpDimensions::from_dna(&part.allocation.dna);
    let mut bones = NativeDpBoneTransforms::initialized();
    for b in &mut bones.slots {
        b.field_100 = 199;
    }
    let before = bones.clone();
    part.fold_tail(&mut bones, &dims, true).unwrap();
    assert_eq!(bones, before);
    part.allocation.kind = NativeDpPartKind::Torso;
    assert_eq!(
        part.fold_tail(&mut bones, &dims, false).unwrap_err().kind(),
        io::ErrorKind::Unsupported
    );
    assert_eq!(bones, before);
}
