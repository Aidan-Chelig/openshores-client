use super::*;
use crate::world::native_body::{NativeDpAllocatedPart, NativeDpPartJoints};
use crate::world::native_dna::NativeDhDna;

fn tail(count: u8) -> NativeDpPartState {
    let mut dna = NativeDhDna([0; 24]);
    dna.0[14] = count << 6;
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
fn descending_tail_joints_accumulate_rotations_on_suffixes_only() {
    for count in 0..4 {
        let part = tail(count);
        let dimensions = NativeDpDimensions::from_dna(&part.allocation.dna);
        let mut bones = NativeDpBoneTransforms::initialized();
        for bone in &mut bones.slots {
            bone.field_100 = 199;
        }
        let before = bones.clone();
        part.bend_tail(&mut bones, &dimensions, 0.125, 0., -1.)
            .unwrap();
        let end = if count == 0 {
            17
        } else {
            16 + usize::from(count) * 2
        };
        for (i, bone) in bones.slots.iter().enumerate() {
            if (17..=end).contains(&i) {
                let angle = (i - 16) as f64 * 0.125;
                assert!((bone.matrix_00[5] - angle.cos()).abs() < 1e-12);
                assert!((bone.matrix_00[6] - angle.sin()).abs() < 1e-12);
                assert_eq!(bone.field_100, 0);
            } else {
                assert_eq!(*bone, before.slots[i]);
            }
        }
        // Preserve the actual final joint pivot. Native's initial f32 product
        // followed by repeated f64 additions need not return exactly to the
        // stored connection for the longer tail selections.
        let step = (dimensions.fields_4080_through_40b8()[10] * 2.) / 3.;
        let mut pivot = part.connect_30;
        if count != 0 {
            pivot[1] -= f64::from(f32::from(count * 2 - 1) * step);
            for _ in 17..end {
                pivot[1] += f64::from(step);
            }
        }
        let actual = bones.slots[17].project(pivot);
        for (v, expected) in actual.iter().zip(pivot) {
            assert!((v - expected).abs() < 1e-12);
        }
    }
}

#[test]
fn ground_pass_raises_penetrating_joints_and_negative_clearance_disables_it() {
    for count in 1..4 {
        let part = tail(count);
        let dimensions = NativeDpDimensions::from_dna(&part.allocation.dna);
        let mut unchecked = NativeDpBoneTransforms::initialized();
        part.bend_tail(&mut unchecked, &dimensions, 0.25, 0.125, -1.)
            .unwrap();
        let mut corrected = NativeDpBoneTransforms::initialized();
        part.bend_tail(&mut corrected, &dimensions, 0.25, 0.125, 0.)
            .unwrap();
        assert_ne!(unchecked, corrected);
        let end = 16 + usize::from(count) * 2;
        let length = dimensions.fields_4080_through_40b8()[10];
        let step = (length + length) / 3.;
        let mut point = part.connect_30;
        point[1] -= f64::from(f32::from(count * 2 - 1) * step);
        for joint in (18..=end).rev() {
            assert!(transform::project_z(&corrected.slots[joint], point, end - joint) >= 0.);
            let next_y = f64::from(step) + point[1];
            point[1] = next_y;
        }
        let mut nan_disabled = NativeDpBoneTransforms::initialized();
        part.bend_tail(&mut nan_disabled, &dimensions, 0.25, 0.125, f32::NAN)
            .unwrap();
        assert_eq!(nan_disabled, unchecked);
    }
}

#[test]
fn combined_rotation_is_premultiplied_including_the_homogeneous_row() {
    let part = tail(0);
    let dimensions = NativeDpDimensions::from_dna(&part.allocation.dna);
    let mut bones = NativeDpBoneTransforms::initialized();
    bones.slots[17].matrix_00 = [
        1., 2., 3., 0.125, 4., 5., 6., -0.25, 7., 8., 9., 0.5, 10., 11., 12., 2.,
    ];
    let before = bones.slots[17];
    let pivot = before.project(part.connect_30);
    let (sx, cx) = 0.25_f64.sin_cos();
    let (sz, cz) = (-0.125_f64).sin_cos();
    // Independent row-major product Rx * Rz. This checks the axis/order,
    // projective pivot and premultiplication, not native libm bit equality.
    let rotation = [
        [cz, -sz, 0.],
        [cx * sz, cx * cz, -sx],
        [sx * sz, sx * cz, cx],
    ];
    let t: [f64; 3] = std::array::from_fn(|row| {
        pivot[row]
            - (0..3)
                .map(|col| rotation[row][col] * pivot[col])
                .sum::<f64>()
    });
    part.bend_tail(&mut bones, &dimensions, 0.25, -0.125, -1.)
        .unwrap();
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

#[test]
fn origin_ground_scan_retains_native_homogeneous_sum_instead_of_affine_translation() {
    let mut bone = crate::world::native_body::NativeDpBoneTransform::identity();
    bone.matrix_00[3] = 2.;
    bone.matrix_00[7] = 3.;
    bone.matrix_00[11] = 4.;
    bone.matrix_00[14] = 20.;
    bone.matrix_00[15] = 1.;
    for ordinal in 0..5 {
        assert_eq!(transform::project_z(&bone, [0.; 3], ordinal), 2.);
    }
    bone.matrix_00[7] = 1.;
    bone.matrix_00[6] = 2.;
    assert_eq!(transform::project_z(&bone, [0., 2., 0.], 0), 8.);
    assert_eq!(transform::project_z(&bone, [0., 2., 0.], 4), 8.);
}

#[test]
fn non_tail_is_unsupported_before_any_write() {
    let mut part = tail(3);
    part.allocation.kind = NativeDpPartKind::Torso;
    let dimensions = NativeDpDimensions::from_dna(&part.allocation.dna);
    let mut bones = NativeDpBoneTransforms::initialized();
    let before = bones.clone();
    assert_eq!(
        part.bend_tail(&mut bones, &dimensions, 0.25, 0.125, 0.)
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
    assert_eq!(bones, before);
}

#[test]
fn zero_angles_still_clear_native_flags_and_do_not_touch_neighbors() {
    let part = tail(3);
    let dimensions = NativeDpDimensions::from_dna(&part.allocation.dna);
    let mut bones = NativeDpBoneTransforms::initialized();
    for bone in &mut bones.slots {
        bone.field_100 = 199;
    }
    part.bend_tail(&mut bones, &dimensions, 0., -0., f32::NAN)
        .unwrap();
    for (i, bone) in bones.slots.iter().enumerate() {
        assert_eq!(bone.field_100, if (17..=22).contains(&i) { 0 } else { 199 });
    }
}
