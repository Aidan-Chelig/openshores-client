//! AuMatrix4x4's in-place column operations, not a fresh affine shortcut.
use super::NativeDpBoneTransform;

// The native SSE lane pairs use different multiplication operand orders.
// Retain those expressions rather than normalizing them algebraically.
#[allow(clippy::if_same_then_else)]
impl NativeDpBoneTransform {
    pub fn rotate_x(&mut self, angle: f64) {
        let (s, c) = angle.sin_cos();
        self.field_100 = 0;
        let m = self.matrix_00;
        for i in 0..4 {
            self.matrix_00[4 + i] = m[4 + i] * c + if i < 2 { m[8 + i] * s } else { s * m[8 + i] };
            self.matrix_00[8 + i] = if i < 2 {
                m[4 + i] * -s + m[8 + i] * c
            } else {
                -s * m[4 + i] + c * m[8 + i]
            };
        }
    }

    pub fn rotate_y(&mut self, angle: f64) {
        let (s, c) = angle.sin_cos();
        self.field_100 = 0;
        let m = self.matrix_00;
        for i in 0..4 {
            self.matrix_00[i] = m[i] * c + if i < 2 { m[8 + i] * -s } else { -s * m[8 + i] };
            self.matrix_00[8 + i] = m[i] * s + if i < 2 { m[8 + i] * c } else { c * m[8 + i] };
        }
    }

    pub fn rotate_z(&mut self, angle: f64) {
        let (s, c) = angle.sin_cos();
        self.field_100 = 0;
        let m = self.matrix_00;
        for i in 0..4 {
            self.matrix_00[i] = m[i] * c + if i < 2 { m[4 + i] * s } else { s * m[4 + i] };
            self.matrix_00[4 + i] = if i < 2 {
                m[i] * -s + m[4 + i] * c
            } else {
                -s * m[i] + c * m[4 + i]
            };
        }
    }

    /// Native post-translation includes the homogeneous fourth-row dot product.
    /// Zero input still executes the arithmetic and clears field100.
    pub fn translate(&mut self, [x, y, z]: [f64; 3]) {
        self.field_100 = 0;
        let m = self.matrix_00;
        for i in 0..4 {
            self.matrix_00[12 + i] = ((m[i] * x + m[4 + i] * y) + m[8 + i] * z) + m[12 + i];
        }
    }
}

#[cfg(test)]
mod tests;
