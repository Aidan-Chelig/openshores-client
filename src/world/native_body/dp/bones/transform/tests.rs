use super::*;

fn dense() -> NativeDpBoneTransform {
    NativeDpBoneTransform {
        matrix_00: std::array::from_fn(|i| (i as f64 - 7.) * 0.375),
        field_100: 255,
    }
}

#[test]
fn rotations_update_existing_nonaffine_columns_and_preserve_unaffected_bits() {
    for angle in [0., -0., 0.375, -1.25, std::f64::consts::FRAC_PI_2] {
        let (s, c) = angle.sin_cos();
        for axis in 0..3 {
            let mut actual = dense();
            let before = actual.matrix_00;
            let mut rotation = [0.; 16];
            for i in [0, 5, 10, 15] {
                rotation[i] = 1.;
            }
            let (a, b) = match axis {
                0 => (1, 2),
                1 => (0, 2),
                _ => (0, 1),
            };
            rotation[a * 4 + a] = c;
            rotation[b * 4 + b] = c;
            rotation[a * 4 + b] = if axis == 1 { -s } else { s };
            rotation[b * 4 + a] = if axis == 1 { s } else { -s };
            match axis {
                0 => actual.rotate_x(angle),
                1 => actual.rotate_y(angle),
                _ => actual.rotate_z(angle),
            };
            assert_eq!(actual.field_100, 0);
            for column in 0..4 {
                for row in 0..4 {
                    if column == a || column == b {
                        let expected = before[a * 4 + row] * rotation[column * 4 + a]
                            + before[b * 4 + row] * rotation[column * 4 + b];
                        assert_eq!(
                            actual.matrix_00[column * 4 + row].to_bits(),
                            expected.to_bits()
                        );
                    } else {
                        assert_eq!(
                            actual.matrix_00[column * 4 + row].to_bits(),
                            before[column * 4 + row].to_bits()
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn translation_composes_all_four_rows_including_existing_translation() {
    for point in [[0.; 3], [2., -3., 4.], [-0., 0., -0.]] {
        let mut actual = dense();
        let before = actual.matrix_00;
        actual.translate(point);
        assert_eq!(actual.field_100, 0);
        assert_eq!(actual.matrix_00[..12], before[..12]);
        for row in 0..4 {
            let products: [f64; 3] =
                std::array::from_fn(|column| before[column * 4 + row] * point[column]);
            let expected = ((products[0] + products[1]) + products[2]) + before[12 + row];
            assert_eq!(actual.matrix_00[12 + row].to_bits(), expected.to_bits());
        }
    }
}

#[test]
fn zero_operations_do_not_short_circuit_nonfinite_native_arithmetic() {
    let mut actual = dense();
    actual.matrix_00[0] = f64::INFINITY;
    actual.translate([0.; 3]);
    assert!(actual.matrix_00[12].is_nan());
    let mut actual = dense();
    actual.matrix_00[8] = f64::INFINITY;
    actual.rotate_x(0.);
    assert!(actual.matrix_00[4].is_nan());
    assert_eq!(actual.field_100, 0);
}
