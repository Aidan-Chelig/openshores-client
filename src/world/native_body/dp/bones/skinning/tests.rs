use super::*;

fn payload(bones: [i32; 4], weights: [f32; 4]) -> NativeBodyVertexWeights {
    NativeBodyVertexWeights {
        words_00_through_24: std::array::from_fn(|i| match i {
            0..4 => bones[i] as u32,
            4..8 => weights[i - 4].to_bits(),
            _ => 0x12345678,
        }),
    }
}

#[test]
fn ordered_bones_blend_positions_and_rotate_normals_without_translation() {
    let mut bones = NativeDpBoneTransforms::initialized();
    bones.slots[1].matrix_00[12] = 2.;
    bones.slots[2].matrix_00 = [
        0., 1., 0., 0., -1., 0., 0., 0., 0., 0., 1., 0., 0., 2., 0., 1.,
    ];
    let input = payload([1, 2, -1, 999], [0.25, 0.75, 0., -0.]);
    let before = input;
    let result = bones
        .skin_shader_vertex([1., 0., 0.], [1., 0., 0.], &input)
        .unwrap();
    assert_eq!(result.position, [0.75, 2.25, 0., 1.]);
    assert!((result.normal[0] - 1_f32 / 10_f32.sqrt()).abs() < 1e-6);
    assert!((result.normal[1] - 3_f32 / 10_f32.sqrt()).abs() < 1e-6);
    assert_eq!(input, before);
}

#[test]
fn position_w_is_not_divided_and_weights_are_not_renormalized() {
    let mut bones = NativeDpBoneTransforms::initialized();
    bones.slots[0].matrix_00[3] = 1.;
    let result = bones
        .skin_shader_vertex(
            [1., 0., 0.],
            [0., 1., 0.],
            &payload([0; 4], [1., 0., 0., 0.]),
        )
        .unwrap();
    assert_eq!(result.position, [1., 0., 0., 2.]);
    bones.slots[0].matrix_00[3] = 0.;
    let result = bones
        .skin_shader_vertex(
            [2., 4., 6.],
            [0., 1., 0.],
            &payload([0; 4], [0.5, 0., 0., 0.]),
        )
        .unwrap();
    assert_eq!(result.position, [1., 2., 3., 0.5]);
    assert_eq!(result.normal, [0., 1., 0.]);
}

#[test]
fn upload_narrows_matrices_and_normals_use_mat3_not_inverse_transpose() {
    let mut bones = NativeDpBoneTransforms::initialized();
    bones.slots[0].matrix_00[0] = 1. + 2_f64.powi(-30);
    let input = payload([0; 4], [1., 0., 0., 0.]);
    assert_eq!(
        bones
            .skin_shader_vertex([1.; 3], [1., 0., 0.], &input)
            .unwrap()
            .position,
        [1., 1., 1., 1.]
    );
    bones.slots[0].matrix_00[0] = 2.;
    let result = bones
        .skin_shader_vertex([0.; 3], [1., 1., 0.], &input)
        .unwrap();
    assert!((result.normal[0] - 2_f32 / 5_f32.sqrt()).abs() < 1e-6);
    assert!((result.normal[1] - 1_f32 / 5_f32.sqrt()).abs() < 1e-6);
}

#[test]
fn undefined_shader_inputs_are_explicitly_unsupported_and_inactive_bones_are_skipped() {
    let mut bones = NativeDpBoneTransforms::initialized();
    let input = payload([0, -1, 999, -123], [1., 0., -0., 0.]);
    bones.slots[60].matrix_00.fill(f64::NAN);
    assert!(
        bones
            .skin_shader_vertex([0.; 3], [0., 0., 1.], &input)
            .is_ok()
    );
    for bad in [
        payload([-1; 4], [1., 0., 0., 0.]),
        payload([61; 4], [1., 0., 0., 0.]),
        payload([0; 4], [0.; 4]),
        payload([0; 4], [f32::NAN, 0., 0., 0.]),
        payload([0; 4], [f32::INFINITY, 0., 0., 0.]),
    ] {
        assert_eq!(
            bones
                .skin_shader_vertex([0.; 3], [0., 1., 0.], &bad)
                .unwrap_err()
                .kind(),
            io::ErrorKind::Unsupported
        );
    }
    assert!(bones.skin_shader_vertex([0.; 3], [0.; 3], &input).is_err());
    assert!(
        bones
            .skin_shader_vertex([f32::NAN; 3], [0., 1., 0.], &input)
            .is_err()
    );
    bones.slots[0].matrix_00[0] = f64::MAX;
    assert!(
        bones
            .skin_shader_vertex([0.; 3], [0., 1., 0.], &input)
            .is_err()
    );
}
