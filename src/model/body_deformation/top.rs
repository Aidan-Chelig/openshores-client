use super::*;

impl NativeBodyDeformation {
    /// Caller establishes matching Top and absence of anatomical groups.
    pub(super) fn apply_top_vertex(
        &self,
        point: [f64; 3],
        selected: &NativeBodyVertexSelection,
    ) -> [f64; 3] {
        let top = selected.top.expect("matching Top branch");
        let scale = self.base.field_140;
        let midpoint = if self.fields_2c4.iter().all(|value| *value == 0.) {
            self.fields_228
        } else {
            self.fields_2c4
        };
        let (mut vertex, mut connection, midpoint) = if scale == 0. {
            ([0.; 3], [0.; 3], [0.; 3])
        } else {
            (
                point.map(|value| value * scale),
                self.base.fields_1f4.map(|value| f64::from(value) * scale),
                midpoint.map(|value| f64::from(value) * scale),
            )
        };
        let (radial, pivot, apply) = if selected.midpoint {
            let radial = (self.field_170 + self.field_180) * 0.5;
            (radial, midpoint, radial != 1.)
        } else if self.field_180.partial_cmp(&self.field_170) != Some(std::cmp::Ordering::Greater)
            || selected.wing
        {
            (self.field_170, top, self.field_170 != 1.)
        } else {
            // Native else arm always performs arithmetic, even if the
            // average is exactly1. Do not collapse this to the first arm.
            ((self.field_180 + self.field_170) * 0.5, top, true)
        };
        if apply {
            around(&mut vertex, pivot, [radial, 1., radial]);
        }
        if self.field_178 != 1. {
            around(&mut vertex, top, [1., self.field_178, 1.]);
            // Both routes from5b7 reach this connection-only operation for
            // matching Top, whether or not Wing was found.
            around(&mut connection, top, [1., self.field_178, 1.]);
        }
        let offset = self.base.fields_1c0.map(f64::from);
        std::array::from_fn(|axis| (offset[axis] - connection[axis]) + vertex[axis])
    }
}
