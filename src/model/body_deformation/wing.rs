use super::*;

impl NativeBodyDeformation {
    pub(super) fn apply_wing_vertex(
        &self,
        point: [f64; 3],
        selected: &NativeBodyVertexSelection,
    ) -> [f64; 3] {
        let hand = selected.anatomical[1].expect("matching HandWing branch");
        let top = selected.top.unwrap_or(selected.top_scale_point);
        let bottom = selected.bottom.unwrap_or(selected.bottom_scale_point);
        let midpoint = if self.fields_2c4.iter().all(|v| *v == 0.) {
            self.fields_228
        } else {
            self.fields_2c4
        };
        let scale = self.base.field_140;
        let (mut vertex, mut connection, mut midpoint, wing) = if scale == 0. {
            ([0.; 3], [0.; 3], [0.; 3], [0.; 3])
        } else {
            (
                point.map(|v| v * scale),
                self.base.fields_1f4.map(|v| f64::from(v) * scale),
                midpoint.map(|v| f64::from(v) * scale),
                self.fields_25c.map(|v| f64::from(v) * scale),
            )
        };
        if self.field_178 != 1. {
            let before = midpoint;
            around(&mut midpoint, top, [1., self.field_178, 1.]);
            for axis in 0..3 {
                vertex[axis] += midpoint[axis] - before[axis];
            }
        }
        if self.field_180 != 1. {
            around(&mut vertex, hand, [self.field_180, 1., self.field_180]);
            around(&mut connection, hand, [self.field_180, 1., self.field_180]);
        }
        if self.field_188 != 1. {
            around(&mut vertex, bottom, [1., self.field_188, 1.]);
            let mut transformed = midpoint;
            around(&mut transformed, bottom, [1., self.field_188, 1.]);
            for axis in 0..3 {
                vertex[axis] += midpoint[axis] - transformed[axis];
            }
        }
        if self.field_190 != 1. {
            around(&mut vertex, hand, [1., self.field_190, 1.]);
            // Native Y compensation puts the scale on the LEFT of the delta;
            // keep its actual operand order and the unchanged XZ arithmetic.
            for axis in 0..3 {
                let delta = wing[axis] - hand[axis];
                let transformed = if axis == 1 {
                    self.field_190 * delta + hand[axis]
                } else {
                    delta + hand[axis]
                };
                vertex[axis] += wing[axis] - transformed;
            }
        }
        if self.field_178 != 1. && selected.wing && selected.top.is_some() {
            around(&mut connection, top, [1., self.field_178, 1.]);
        }
        let offset = self.base.fields_1c0.map(f64::from);
        std::array::from_fn(|axis| (offset[axis] - connection[axis]) + vertex[axis])
    }
}
