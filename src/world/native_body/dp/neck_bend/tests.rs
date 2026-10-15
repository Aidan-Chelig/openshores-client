use super::*;
use crate::world::{
    native_body::{NativeDpAllocatedPart, NativeDpPartJoints},
    native_dna::NativeDhDna,
};

fn part(index: u32, count: u8, second: u8) -> NativeDpPartState {
    let mut dna = NativeDhDna([0; 24]);
    dna.0[14] = count;
    dna.0[5] = second << 1;
    NativeDpPartState {
        allocation: NativeDpAllocatedPart {
            kind: NativeDpPartKind::HeadAndNeck,
            index,
            dna,
        },
        connect_30: [0.; 3],
        joints: NativeDpPartJoints::Head { field_50: [0.; 3] },
    }
}

#[test]
fn descending_prefix_stages_accumulate_different_angles_per_bone() {
    for index in [0, 1, 2, u32::MAX] {
        for count in 0..4 {
            for second in 0..2 {
                let part = part(index, count, second);
                let dimensions = NativeDpDimensions::from_dna(&part.allocation.dna);
                let start = if index == 1 { 12 } else { 7 };
                let end = start + usize::from(count + second);
                let mut bones = NativeDpBoneTransforms::initialized();
                for bone in &mut bones.slots {
                    bone.field_100 = 199;
                }
                part.bend_neck(&mut bones, &dimensions, 0.125).unwrap();
                for (i, bone) in bones.slots.iter().enumerate() {
                    if (start..=end).contains(&i) {
                        let angle = (end - i + 1) as f64 * 0.125;
                        let (s, c) = angle.sin_cos();
                        assert!((bone.matrix_00[5] - c).abs() < 1e-12);
                        assert!((bone.matrix_00[6] - s).abs() < 1e-12);
                        assert!((bone.matrix_00[9] + s).abs() < 1e-12);
                        assert_eq!(bone.field_100, 0);
                    } else {
                        assert_eq!(
                            *bone,
                            NativeDpBoneTransform {
                                field_100: 199,
                                ..NativeDpBoneTransform::identity()
                            }
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn native_joint_spacing_uses_actual_owner_dimensions() {
    let part = part(0, 1, 0);
    let mut owner_dna = part.allocation.dna.clone();
    owner_dna.0[19] = 255;
    let small = NativeDpDimensions::from_dna(&part.allocation.dna);
    let large = NativeDpDimensions::from_dna(&owner_dna);
    let mut a = NativeDpBoneTransforms::initialized();
    let mut b = a.clone();
    part.bend_neck(&mut a, &small, 0.375).unwrap();
    part.bend_neck(&mut b, &large, 0.375).unwrap();
    assert_eq!(a.slots[8].matrix_00, b.slots[8].matrix_00);
    assert_ne!(a.slots[7].matrix_00[12..], b.slots[7].matrix_00[12..]);
}

#[test]
fn wrapper_keeps_signed_zero_noop_but_nonfinite_angle_is_not_a_noop() {
    let mut part = part(0, 0, 0);
    let dimensions = NativeDpDimensions::from_dna(&part.allocation.dna);
    let mut bones = NativeDpBoneTransforms::initialized();
    for bone in &mut bones.slots {
        bone.field_100 = 251;
    }
    let before = bones.clone();
    for angle in [0., -0.] {
        part.bend_neck(&mut bones, &dimensions, angle).unwrap();
        assert_eq!(bones, before);
    }
    part.bend_neck(&mut bones, &dimensions, f32::NAN).unwrap();
    assert!(bones.slots[7].matrix_00.iter().any(|v| v.is_nan()));
    assert_eq!(bones.slots[7].field_100, 0);
    part.allocation.kind = NativeDpPartKind::Tail;
    assert_eq!(
        part.bend_neck(&mut bones, &dimensions, 1.)
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
}

#[test]
fn expanded_pivot_and_nonaffine_matrix_match_independent_scalar_fixture() {
    let bend = Bend {
        a: 0.,
        b: 0.,
        cos_plus: 0.8,
        cos_minus: 0.8,
        sin_plus: 0.6,
        negative_sin: -0.6,
    };
    let t = bend.about([0.25, 1.5, -2.75]);
    // Filled from independent Python scalar evaluation of the retained SSE
    // stages, not native execution or platform-specific sin/cos constants.
    assert_eq!(
        t.map(f64::to_bits),
        [
            0,
            0xbff599999999999a,
            0xbff7333333333332,
            0x3ff0000000000000
        ]
    );
    let mut bone = NativeDpBoneTransform {
        matrix_00: std::array::from_fn(|i| (i as f64 - 7.) * 0.375),
        field_100: 233,
    };
    bend.apply(&mut bone, t);
    assert_eq!(
        bone.matrix_00.map(f64::to_bits),
        [
            0xc005000000000000,
            0x3ff599999999999b,
            0xbfe5999999999998,
            0xbff8000000000000,
            0xbff2000000000000,
            0xbfd8000000000002,
            0xbfe8000000000000,
            0,
            0x3fd8000000000000,
            0xc000cccccccccccd,
            0xbfea666666666664,
            0x3ff8000000000000,
            0x3ffe000000000000,
            0xc00e99999999999b,
            0xbfecccccccccccc8,
            0x4008000000000000
        ]
    );
    assert_eq!(bone.field_100, 0);
}
