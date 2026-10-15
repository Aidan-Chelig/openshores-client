use super::NativeDpBoneTransform;
impl NativeDpBoneTransform {
    /// Native operator* reads only primary inputs and unconditionally clears
    /// the result flag. Hidden return storage is not the left operand.
    pub fn multiply(&self, right: &Self) -> Self {
        let a = &self.matrix_00;
        let b = &right.matrix_00;
        let mut out = Self::identity();
        for col in 0..4 {
            for row in 0..4 {
                out.matrix_00[col * 4 + row] = ((a[row] * b[col * 4]
                    + a[4 + row] * b[col * 4 + 1])
                    + a[8 + row] * b[col * 4 + 2])
                    + a[12 + row] * b[col * 4 + 3];
            }
        }
        out
    }
}
