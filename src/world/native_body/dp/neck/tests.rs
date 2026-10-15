use super::*;
use crate::world::{
    native_body::{NativeDpAllocatedPart, NativeDpPartJoints},
    native_dna::NativeDhDna,
};

fn head(index: u32, count: u8, second: u8) -> NativeDpPartState {
    let mut dna = NativeDhDna([0; 24]);
    dna.0[14] = count;
    dna.0[5] = second << 1;
    NativeDpPartState {
        allocation: NativeDpAllocatedPart {
            kind: NativeDpPartKind::HeadAndNeck,
            index,
            dna,
        },
        connect_30: [0., 2., 0.],
        joints: NativeDpPartJoints::Head { field_50: [0.; 3] },
    }
}

#[test]
fn neck_rotates_around_current_attachment_and_only_writes_inclusive_head_range() {
    for index in [0, 1, 2, u32::MAX] {
        for count in 0..4 {
            for second in 0..2 {
                let head = head(index, count, second);
                let start = if index == 1 { 12 } else { 7 };
                let end = start + usize::from(count + second);
                let mut bones = NativeDpBoneTransforms::initialized();
                for (i, bone) in bones.slots.iter_mut().enumerate() {
                    bone.translate([i as f64 * 0.125, i as f64 * 0.25, -1.]);
                    bone.field_100 = 251;
                }
                let before = bones.clone();
                let pivot = head.current_connect_loc(&bones).unwrap();
                head.rotate_neck(&mut bones, 0.375).unwrap();
                let after = head.current_connect_loc(&bones).unwrap();
                for i in 0..3 {
                    assert!((after[i] - pivot[i]).abs() < 1e-12);
                }
                for (i, bone) in bones.slots.iter().enumerate() {
                    if (start..=end).contains(&i) {
                        assert_eq!(bone.field_100, 0);
                        assert_ne!(bone.matrix_00, before.slots[i].matrix_00);
                    } else {
                        assert_eq!(*bone, before.slots[i]);
                    }
                }
            }
        }
    }
}

#[test]
fn zero_angle_wrapper_preserves_all_flags_and_matrices_but_nan_enters_operation() {
    let head = head(0, 0, 0);
    let mut bones = NativeDpBoneTransforms::initialized();
    for bone in &mut bones.slots {
        bone.field_100 = 199;
    }
    let before = bones.clone();
    for angle in [0., -0.] {
        head.rotate_neck(&mut bones, angle).unwrap();
        assert_eq!(bones, before);
    }
    head.rotate_neck(&mut bones, f32::NAN).unwrap();
    assert_eq!(bones.slots[7].field_100, 0);
    assert!(bones.slots[7].matrix_00.iter().any(|x| x.is_nan()));
    let mut other = head.clone();
    other.allocation.kind = NativeDpPartKind::Torso;
    assert_eq!(
        other.rotate_neck(&mut bones, 1.).unwrap_err().kind(),
        io::ErrorKind::Unsupported
    );
}

#[test]
fn inline_multiply_preserves_cancellation_sensitive_native_grouping() {
    let mut r = [0.; 16];
    r[0] = 1.;
    r[4] = 1.;
    r[8] = 1.;
    r[12] = 1.;
    r[1] = 1.;
    r[5] = 1.;
    r[9] = 1.;
    r[13] = 1.;
    let mut bone = NativeDpBoneTransform {
        matrix_00: [0.; 16],
        field_100: 199,
    };
    // Native row1 grouping Z+(Y+X) retains 1; a Z+Y+X fold loses it.
    bone.matrix_00[..4].copy_from_slice(&[-1e16, 1e16, 1., 2.]);
    premultiply_rotate_neck(&mut bone, &r);
    assert_eq!(bone.matrix_00[0], 3.);
    assert_eq!(bone.matrix_00[1], 3.);
    assert_eq!(bone.field_100, 0);
}
