//! ApplyScaleFactor's ordered membership/pivot collection, before arithmetic.
use super::NativeBodyObjectGroups;
use std::io;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NativeBodyVertexSelection {
    pub any_match: bool,
    pub wing: bool,
    pub midpoint: bool,
    pub top: Option<[f64; 3]>,
    /// Initialized xyz290 even when no Top group matches. Native replaces an
    /// all-zero value with positive zeros; matching Top uses `top` instead.
    pub top_scale_point: [f64; 3],
    pub bottom: Option<[f64; 3]>,
    /// Torso, Hand, Foot, Neck, Tail, Head, corresponding to flags50..55.
    pub anatomical: [Option<[f64; 3]>; 6],
    pub hand_wing: bool,
}

impl NativeBodyObjectGroups {
    /// Overrides are actual initialized object xyz290 and xyz2f8 values, not
    /// inferred defaults. All-zero overrides let the matching group supply its
    /// pivot. The numeric membership weight is deliberately NOT multiplied in
    /// this stage: native checks only that the key exists, including weight0.
    pub fn select_vertex(
        &self,
        point: [f64; 3],
        top_scale: [f32; 3],
        bottom_scale: [f32; 3],
    ) -> io::Result<NativeBodyVertexSelection> {
        let mut result = NativeBodyVertexSelection {
            top_scale_point: if top_scale.iter().all(|v| *v == 0.) {
                [0.; 3]
            } else {
                top_scale.map(f64::from)
            },
            ..Default::default()
        };
        for group in self.retained_groups() {
            if group.matching_weight(point)?.is_none() {
                continue;
            }
            result.any_match = true;
            let wing = group.name().windows(4).any(|word| word == b"Wing");
            result.wing |= wing;
            match group.kind_34 {
                2 => {}
                4 => result.midpoint = true,
                5 => result.top = Some(pivot(top_scale, group.point_28)),
                6 => result.bottom = Some(pivot(bottom_scale, group.point_28)),
                _ => {
                    // Native else-if chain: one group supplies only the first
                    // set anatomical flag, even if several flags are present.
                    if let Some(index) = group.flags_50.iter().position(|flag| *flag) {
                        result.anatomical[index] = Some(group.point_28.map(f64::from));
                        if index == 1 {
                            result.hand_wing |= wing;
                        }
                    }
                }
            }
        }
        Ok(result)
    }
}

fn pivot(override_point: [f32; 3], group_point: [f32; 3]) -> [f64; 3] {
    if override_point.iter().all(|v| *v == 0.) {
        group_point.map(f64::from)
    } else {
        override_point.map(f64::from)
    }
}

#[cfg(test)]
mod tests;
