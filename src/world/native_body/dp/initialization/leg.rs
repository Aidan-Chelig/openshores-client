use super::*;

impl NativeDpPartState {
    pub(super) fn calculate_leg(&mut self, dimensions: &NativeDpDimensions) {
        let raw = &self.allocation.dna.0;
        let d = dimensions.fields_4080_through_40b8();
        let index = self.allocation.index;
        let diff = i32::from(raw[20] >> 5) - native_dh_waist_vertex(&self.allocation.dna) as i32;
        let y = if index == 0 {
            0.
        } else {
            f64::from((if index == 1 { diff } else { diff / 2 }) as f32 * -d[10])
        };
        let mut width = d[12] * 0.5;
        let half = d[8] * 0.5;
        if half < width {
            width = (f64::from(width) - f64::from(half) * 0.5) as f32;
        }
        let mut x = half.max(width);
        if self.allocation.kind == NativeDpPartKind::LeftLegAndFoot {
            x = -x;
        }
        let mut x = f64::from(x);
        if raw[20] & 0xe0 == 0 {
            x /= 3.0;
        }
        self.connect_30 = [x, y, 0.];
        let knee = f64::from(d[7]) + y;
        self.joints = NativeDpPartJoints::Leg {
            splay_48: match self.joints {
                NativeDpPartJoints::Leg { splay_48, .. } => splay_48,
                _ => 0.,
            },
            field_50: [x, knee, 0.],
            field_68: [x, f64::from(d[0]) + knee, 0.],
        };
    }
}
