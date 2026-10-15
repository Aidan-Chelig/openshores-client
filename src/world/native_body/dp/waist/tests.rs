use super::*;

fn body(waist: u8) -> NativeDpInitialization {
    let mut dna = NativeDhDna([0; 24]);
    dna.0[12] = 4;
    dna.0[20] = waist << 5;
    NativeDpInitialization::begin(&dna)
}

#[test]
fn concrete_torso_rotates_only_the_native_lower_range_and_keeps_its_pivot() {
    for waist in 0..8 {
        let mut body = body(waist);
        for (i, bone) in body.bones.slots.iter_mut().enumerate() {
            bone.translate([i as f64 + 1., 10. - i as f64, 20. + i as f64]);
            bone.field_100 = 199;
        }
        let before = body.bones.clone();
        let pivot = body.current_waist_loc().unwrap();
        body.parts[0]
            .rotate_torso_waist(&mut body.bones, 0.25)
            .unwrap();
        for (i, bone) in body.bones.slots.iter().enumerate() {
            if i < usize::from(waist) {
                assert!((bone.matrix_00[5] - 0.25_f64.cos()).abs() < 1e-12);
                assert!((bone.matrix_00[6] - 0.25_f64.sin()).abs() < 1e-12);
                assert_eq!(bone.field_100, 0);
            } else {
                assert_eq!(*bone, before.slots[i]);
            }
        }
        let after = body.current_waist_loc().unwrap();
        for i in 0..3 {
            assert!((after[i] - pivot[i]).abs() < 1e-12);
        }
    }
}

#[test]
fn current_waist_checks_only_the_first_part_and_projects_the_actual_homogeneous_origin() {
    let mut body = body(2);
    let m = &mut body.bones.slots[1].matrix_00;
    m[3] = 2.;
    m[7] = 3.;
    m[11] = 4.;
    m[15] = 1.;
    m[12] = 10.;
    m[13] = 20.;
    m[14] = 30.;
    assert_eq!(body.current_waist_loc().unwrap(), [1., 2., 3.]);
    body.parts.swap(0, 1);
    assert_eq!(body.current_waist_loc().unwrap(), [0.; 3]);
    body.parts.clear();
    assert_eq!(body.current_waist_loc().unwrap(), [0.; 3]);
}

#[test]
fn dense_nonaffine_bones_are_premultiplied_around_the_projected_joint() {
    let mut body = body(2);
    for bone in &mut body.bones.slots[..2] {
        bone.matrix_00 = [
            1., 2., 3., 0.125, 4., 5., 6., -0.25, 7., 8., 9., 0.5, 10., 11., 12., 2.,
        ];
    }
    let pivot = body.current_waist_loc().unwrap();
    let before = body.bones.clone();
    let (s, c) = 0.25_f64.sin_cos();
    let r = [[1., 0., 0.], [0., c, -s], [0., s, c]];
    let t: [f64; 3] =
        std::array::from_fn(|row| pivot[row] - (0..3).map(|k| r[row][k] * pivot[k]).sum::<f64>());
    body.parts[0]
        .rotate_torso_waist(&mut body.bones, 0.25)
        .unwrap();
    for bone in 0..2 {
        for col in 0..4 {
            for row in 0..3 {
                let m = &before.slots[bone].matrix_00;
                let expected = (0..3).map(|k| r[row][k] * m[col * 4 + k]).sum::<f64>()
                    + m[col * 4 + 3] * t[row];
                assert!((body.bones.slots[bone].matrix_00[col * 4 + row] - expected).abs() < 1e-12);
            }
            assert_eq!(
                body.bones.slots[bone].matrix_00[col * 4 + 3],
                before.slots[bone].matrix_00[col * 4 + 3]
            );
        }
    }
}

#[test]
fn zero_angles_are_noops_nan_enters_and_wrong_parts_fail_before_writes() {
    let mut body = body(2);
    for bone in &mut body.bones.slots {
        bone.field_100 = 199;
    }
    let before = body.bones.clone();
    for angle in [0., -0.] {
        body.parts[0]
            .rotate_torso_waist(&mut body.bones, angle)
            .unwrap();
        assert_eq!(body.bones, before);
    }
    assert_eq!(
        body.parts[1]
            .rotate_torso_waist(&mut body.bones, 0.25)
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
    assert_eq!(body.bones, before);
    body.parts[0]
        .rotate_torso_waist(&mut body.bones, f32::NAN)
        .unwrap();
    assert!(body.bones.slots[0].matrix_00.iter().any(|v| v.is_nan()));
    assert_eq!(body.bones.slots[0].field_100, 0);
    assert_eq!(&body.bones.slots[2..], &before.slots[2..]);
}
