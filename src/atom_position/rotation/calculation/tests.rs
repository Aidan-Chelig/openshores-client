use super::*;
use crate::atom_position::{PackedPosition, native_rotation};

#[test]
fn empty_vector_defaults_and_canonical_directions_match_native_branches() {
    for secondary in [[0.0; 3], [1., 2., 3.]] {
        assert_eq!(
            native_calculate_rotation(&[[0.0; 3], secondary]).unwrap(),
            [0.0; 3]
        );
    }
    assert_eq!(
        native_calculate_rotation(&[[1., 0., 0.], [0., 1., 0.]]).unwrap(),
        [0.0; 3]
    );
    let positive = native_calculate_rotation(&[[0., 5., 0.], [0.; 3]]).unwrap();
    let negative = native_calculate_rotation(&[[0., -5., 0.], [0.; 3]]).unwrap();
    assert_eq!(positive, [0., 0., std::f64::consts::FRAC_PI_2]);
    assert_eq!(negative, [0., 0., -std::f64::consts::FRAC_PI_2]);
    assert_eq!(
        native_calculate_rotation(&[[1., 0., 0.], [0., -1., 0.]]).unwrap(),
        [0., PI, PI]
    );
}

#[test]
fn fresh_matrix_pipeline_roundtrips_basis_except_intentional_angle_snapping() {
    for rotation in [
        [0.3, -0.7, 1.1],
        [3.0, 0.0, 0.0],
        [0.0, 3.0, 1.0],
        [0.0, 0.0, 3.0],
        [1e-6, -1e-6, 1e-6],
        [1e-4, -1e-4, 1e-4],
    ] {
        let record = PackedPosition {
            flags: 0,
            parent: None,
            position: [12345678.125, -2., 3.],
            rotation,
        };
        let original = record.local_matrix().unwrap();
        let angles = native_rotation(&original).unwrap();
        let reconstructed = PackedPosition {
            rotation: angles.map(|angle| angle as f32),
            ..record
        }
        .local_matrix()
        .unwrap();
        for axis in 0..3 {
            for row in 0..3 {
                assert!(
                    (original[axis][row] - reconstructed[axis][row]).abs() < 3e-5,
                    "{rotation:?} -> {angles:?}"
                );
            }
        }
    }
    let record = PackedPosition {
        flags: 0,
        parent: None,
        position: [1e17; 3],
        rotation: [0.3, -0.7, 1.1],
    };
    assert_eq!(
        native_rotation(&record.local_matrix().unwrap()).unwrap(),
        [0.; 3]
    );
}

#[test]
fn invalid_inputs_and_unrepresentable_norms_are_safety_errors() {
    for vector in [
        [f64::NAN, 0., 0.],
        [f64::INFINITY, 0., 0.],
        [f64::MAX, 0., 0.],
        [f64::MIN_POSITIVE, 0., 0.],
    ] {
        assert!(native_calculate_rotation(&[vector, [0., 1., 0.]]).is_err());
        assert!(native_calculate_rotation(&[[1., 0., 0.], vector]).is_err());
    }
}
