use super::*;
use crate::atom_position::PackedPosition;

#[test]
fn fresh_affine_inputs_keep_native_translation_cancellation() {
    for position in [[0.0; 3], [12345678.125, -2.0, 3.0], [1e17; 3]] {
        for rotation in [[0.0; 3], [0.3, -0.7, 1.1]] {
            let matrix = PackedPosition {
                flags: 0,
                parent: None,
                position,
                rotation,
            }
            .local_matrix()
            .unwrap();
            let vectors = native_rotation_vectors(&matrix).unwrap();
            for axis in 0..2 {
                for row in 0..3 {
                    assert_eq!(
                        vectors[axis][row],
                        (matrix[axis][row] + position[row]) - position[row]
                    );
                }
            }
            if position == [1e17; 3] {
                assert_eq!(vectors, [[0.0; 3]; 2]);
            }
        }
    }
}

#[test]
fn nonaffine_origin_uses_all_homogeneous_row_terms() {
    let matrix = [
        [1., 2., 3., 2.],
        [4., 5., 6., 3.],
        [7., 8., 9., 4.],
        [10., 11., 12., 5.],
    ];
    let vectors = native_rotation_vectors(&matrix).unwrap();
    assert_eq!(
        vectors[0],
        [
            11. / 7. - 10. / 14.,
            13. / 7. - 11. / 14.,
            15. / 7. - 12. / 14.
        ]
    );
    assert_eq!(
        vectors[1],
        [
            14. / 8. - 10. / 14.,
            16. / 8. - 11. / 14.,
            18. / 8. - 12. / 14.
        ]
    );
}

#[test]
fn exact_zero_and_one_denominators_skip_division_and_invalid_arithmetic_fails() {
    for w in [0., 1.] {
        let matrix = [
            [1., 0., 0., 0.],
            [0., 1., 0., 0.],
            [0., 0., 1., 0.],
            [10., 20., 30., w],
        ];
        assert_eq!(
            native_rotation_vectors(&matrix).unwrap(),
            [[1., 0., 0.], [0., 1., 0.]]
        );
    }
    let mut matrix = [[0.0; 4]; 4];
    matrix[0][0] = f64::NAN;
    assert!(native_rotation_vectors(&matrix).is_err());
    matrix[0][0] = 0.0;
    matrix[0][3] = f64::MAX;
    matrix[1][3] = f64::MAX;
    assert!(native_rotation_vectors(&matrix).is_err());
    matrix = [[0.0; 4]; 4];
    matrix[3][0] = f64::MAX;
    matrix[3][3] = f64::MIN_POSITIVE;
    assert!(native_rotation_vectors(&matrix).is_err());
}
