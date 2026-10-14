use super::*;

/// Column-major binary64 state as stored by AuMatrix4x4.
#[derive(Clone, Copy)]
pub(super) struct Matrix([[f64; 4]; 4]);

impl Matrix {
    pub(super) fn identity() -> Self {
        Self(std::array::from_fn(|column| {
            std::array::from_fn(|row| f64::from(column == row))
        }))
    }

    pub(super) fn is_identity(&self) -> bool {
        self.0 == Self::identity().0
    }

    pub(super) fn from_serialized(values: [f32; 12], translation_scale: f64) -> io::Result<Self> {
        if values.iter().any(|value| !value.is_finite()) {
            return Err(invalid("non-finite mesh matrix"));
        }
        let mut result = Self::identity();
        for column in 0..4 {
            for row in 0..3 {
                result.0[column][row] = f64::from(values[column * 3 + row]);
            }
        }
        result.scale_translation(translation_scale);
        Ok(result)
    }

    pub(super) fn scale_translation(&mut self, scale: f64) {
        for value in &mut self.0[3][..3] {
            *value *= scale;
        }
    }

    pub(super) fn project(&self, point: [f64; 3]) -> [f64; 3] {
        // Native Project skips zero-coordinate products and adds translation
        // after the x/y and then z terms. No fused multiply-add is introduced.
        std::array::from_fn(|row| {
            if point == [0.0; 3] {
                return self.0[3][row];
            }
            let terms: [f64; 3] = std::array::from_fn(|column| {
                if point[column] == 0.0 {
                    0.0
                } else {
                    self.0[column][row] * point[column]
                }
            });
            self.0[3][row] + ((terms[1] + terms[0]) + terms[2])
        })
    }

    pub(super) fn inverse(self) -> io::Result<Self> {
        let mut matrix = self.0;
        let mut swaps = [0; 4];
        // Native in-place pivoted inversion: swap columns for pivots, then undo
        // those permutations on rows, in reverse order. Reject zero pivots;
        // the native code instead leaves partial results and still projects.
        for pivot in 0..4 {
            let mut selected = pivot;
            for column in pivot + 1..4 {
                if matrix[column][pivot].abs() > matrix[selected][pivot].abs() {
                    selected = column;
                }
            }
            if matrix[selected][pivot] == 0.0 {
                return Err(invalid("singular mesh matrix"));
            }
            if selected != pivot {
                matrix.swap(selected, pivot);
                swaps[pivot] = selected;
            }
            let reciprocal = 1.0 / matrix[pivot][pivot];
            for (row, value) in matrix[pivot].iter_mut().enumerate() {
                *value = if row == pivot {
                    reciprocal
                } else {
                    *value * reciprocal
                };
            }
            let normalized = matrix[pivot];
            for (column, values) in matrix.iter_mut().enumerate() {
                if column == pivot || values[pivot] == 0.0 {
                    continue;
                }
                let factor = values[pivot];
                for (row, value) in values.iter_mut().enumerate() {
                    if row != pivot && normalized[row] != 0.0 {
                        *value -= normalized[row] * factor;
                    }
                }
                values[pivot] = -factor * reciprocal;
            }
        }
        for row in (0..3).rev() {
            if swaps[row] != 0 {
                for column in &mut matrix {
                    column.swap(row, swaps[row]);
                }
            }
        }
        if matrix.iter().flatten().any(|value| !value.is_finite()) {
            return Err(invalid("non-finite inverse mesh matrix"));
        }
        Ok(Self(matrix))
    }
}
