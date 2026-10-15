//! Actual AuMatrix4x4::Project, including zero-coordinate shortcuts and the
//! all-origin denominator. Hidden-result ABI and SIMD grouping checked in GNU.
use super::NativeDpBoneTransform;

impl NativeDpBoneTransform {
    /// Projects using the primary matrix without changing the cached-state byte.
    /// Preserves native nonfinite arithmetic; callers own any safety validation.
    pub fn project(&self, [x, y, z]: [f64; 3]) -> [f64; 3] {
        let m = &self.matrix_00;
        let (mut point, w) = if x == 0. && y == 0. && z == 0. {
            ([m[12], m[13], m[14]], ((m[3] + m[7]) + m[11]) + m[15])
        } else {
            let tx = if x == 0. {
                [0.; 4]
            } else {
                [m[0] * x, m[1] * x, m[2] * x, x * m[3]]
            };
            let ty = if y == 0. {
                [0.; 4]
            } else {
                [y * m[4], y * m[5], m[6] * y, y * m[7]]
            };
            let tz = if z == 0. {
                [0.; 4]
            } else {
                [z * m[8], z * m[9], m[10] * z, z * m[11]]
            };
            // SIMD numerator additions occur before adding translation. The
            // decompiler's left-associated translation+y+x+z is misleading.
            (
                [
                    m[12] + ((ty[0] + tx[0]) + tz[0]),
                    m[13] + ((ty[1] + tx[1]) + tz[1]),
                    m[14] + ((tx[2] + ty[2]) + tz[2]),
                ],
                ((ty[3] + tx[3]) + tz[3]) + m[15],
            )
        };
        if w != 0. && w != 1. {
            for value in &mut point {
                *value /= w;
            }
        }
        point
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_uses_all_four_denominator_terms_and_preserves_zero_w_translation() {
        let mut bone = NativeDpBoneTransform {
            matrix_00: [0.; 16],
            field_100: 231,
        };
        bone.matrix_00[12..].copy_from_slice(&[12., 24., 36., 1.]);
        bone.matrix_00[3] = 2.;
        bone.matrix_00[7] = 3.;
        bone.matrix_00[11] = 6.;
        assert_eq!(bone.project([0.; 3]), [1., 2., 3.]);
        assert_eq!(bone.project([-0.; 3]), [1., 2., 3.]);
        assert_eq!(bone.field_100, 231);
        bone.matrix_00[15] = -11.;
        assert_eq!(bone.project([0.; 3]), [12., 24., 36.]);
    }

    #[test]
    fn coordinate_shortcuts_skip_nonfinite_unused_columns() {
        let mut bone = NativeDpBoneTransform {
            matrix_00: [f64::NAN; 16],
            field_100: 9,
        };
        bone.matrix_00[4..8].copy_from_slice(&[1., 2., 3., 0.]);
        bone.matrix_00[12..].copy_from_slice(&[10., 20., 30., 1.]);
        assert_eq!(bone.project([0., 2., -0.]), [12., 24., 36.]);
        assert_eq!(bone.field_100, 9);
        // Unlike a sparse non-origin input, origin reads every W-row term.
        assert!(bone.project([0.; 3]).iter().all(|v| v.is_nan()));
    }

    #[test]
    fn numerator_grouping_keeps_translation_outside_the_simd_column_sum() {
        let mut bone = NativeDpBoneTransform {
            matrix_00: [0.; 16],
            field_100: 255,
        };
        bone.matrix_00[0] = 1e16;
        bone.matrix_00[4] = -1e16;
        bone.matrix_00[8] = 1.;
        bone.matrix_00[12] = 1.;
        bone.matrix_00[15] = 1.;
        assert_eq!(bone.project([1.; 3])[0], 2.);
        assert_eq!(bone.field_100, 255);
    }

    #[test]
    fn all_coordinate_subsets_match_independent_sparse_column_evaluation() {
        for bits in 1..8 {
            let point = std::array::from_fn(|axis| {
                if bits & (1 << axis) == 0 {
                    0.
                } else {
                    [0.25, -2., 3.][axis]
                }
            });
            for w in [0., 1., 2., -3.] {
                let mut bone = NativeDpBoneTransform {
                    matrix_00: std::array::from_fn(|i| (i as f64 - 3.) * 0.25),
                    field_100: 1,
                };
                // Explicit affine W makes the expected denominator independent
                // of the native origin-only exception covered separately.
                for i in [3, 7, 11] {
                    bone.matrix_00[i] = 0.;
                }
                bone.matrix_00[15] = w;
                let actual = bone.project(point);
                for (row, value) in actual.iter().enumerate() {
                    let terms: [f64; 3] = std::array::from_fn(|axis| {
                        if point[axis] == 0. {
                            0.
                        } else {
                            bone.matrix_00[axis * 4 + row] * point[axis]
                        }
                    });
                    let mut expected =
                        bone.matrix_00[12 + row] + ((terms[0] + terms[1]) + terms[2]);
                    if w != 0. && w != 1. {
                        expected /= w;
                    }
                    assert_eq!(value.to_bits(), expected.to_bits());
                }
            }
        }
    }
}
