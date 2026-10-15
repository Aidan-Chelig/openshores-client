use super::*;

impl NativeBodyDeformation {
    /// Non-anatomical path with no matching Top. Bottom is optional: native
    /// default/ScalePt-only membership still has a distinct connection exit.
    pub(super) fn apply_bottom_vertex(
        &self,
        point: [f64; 3],
        selected: &NativeBodyVertexSelection,
    ) -> [f64; 3] {
        let top = selected.top_scale_point;
        let midpoint = if self.fields_2c4.iter().all(|v| *v == 0.) {
            self.fields_228
        } else {
            self.fields_2c4
        };
        let scale = self.base.field_140;
        let (mut vertex, mut connection, mut midpoint) = if scale == 0. {
            ([0.; 3], [0.; 3], [0.; 3])
        } else {
            (
                point.map(|v| v * scale),
                self.base.fields_1f4.map(|v| f64::from(v) * scale),
                midpoint.map(|v| f64::from(v) * scale),
            )
        };
        if let Some(bottom) = selected.bottom {
            if self.field_178 != 1. && !selected.wing {
                let before = midpoint;
                around(&mut midpoint, top, [1., self.field_178, 1.]);
                for axis in 0..3 {
                    vertex[axis] += midpoint[axis] - before[axis];
                }
            }
            let (radial, pivot, apply) = if selected.midpoint {
                let radial = (self.field_170 + self.field_180) * 0.5;
                (radial, midpoint, radial != 1.)
            } else if self.field_170.partial_cmp(&self.field_180)
                != Some(std::cmp::Ordering::Greater)
                || selected.wing
            {
                (self.field_180, bottom, self.field_180 != 1.)
            } else {
                ((self.field_170 + self.field_180) * 0.5, bottom, true)
            };
            if apply {
                around(&mut vertex, pivot, [radial, 1., radial]);
            }
            if self.field_188 != 1. {
                around(&mut vertex, bottom, [1., self.field_188, 1.]);
                let mut transformed_midpoint = midpoint;
                around(&mut transformed_midpoint, bottom, [1., self.field_188, 1.]);
                for axis in 0..3 {
                    vertex[axis] += midpoint[axis] - transformed_midpoint[axis];
                }
            }
        }
        // No Top means Wing bypasses connection178 at shared5b7->5cd.
        if self.field_178 != 1. && !selected.wing {
            around(&mut connection, top, [1., self.field_178, 1.]);
        }
        let offset = self.base.fields_1c0.map(f64::from);
        std::array::from_fn(|axis| (offset[axis] - connection[axis]) + vertex[axis])
    }
}
