//! AverageWeights on an explicit native vertex-buffer coordinate.
use super::{
    NativeBodyObjectGroups, native_body_is_bone, native_body_neck_bone, native_body_tail_bone,
};
use std::io;

/// Exact first 40 bytes of an Au3dsGLSLVertex, excluding xyz.
/// The native fifth accepted influence overlaps weight zero and word 0x20.
/// Keep that behavior visible instead of silently capping at four influences.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NativeBodyVertexWeights {
    pub words_00_through_24: [u32; 10],
}

impl NativeBodyVertexWeights {
    pub fn bones(&self) -> [i32; 4] {
        std::array::from_fn(|i| self.words_00_through_24[i] as i32)
    }

    pub fn weights(&self) -> [f32; 4] {
        std::array::from_fn(|i| self.weight(i))
    }

    fn weight(&self, index: usize) -> f32 {
        f32::from_bits(self.words_00_through_24[4 + index])
    }

    fn write(&mut self, index: usize, bone: i32, weight: f32) {
        self.words_00_through_24[index] = bone as u32;
        self.words_00_through_24[4 + index] = weight.to_bits();
    }

    /// `point` must be the stored f32 vertex BEFORE triangle-builder projection.
    /// This does not infer that coordinate from a rendered/deformed position.
    /// A false object flag leaves all words untouched; zero sums deliberately
    /// produce the native IEEE NaN/Inf results, not invented fallback weights.
    pub fn average_at_native_vertex(
        &mut self,
        point: [f32; 3],
        groups: &NativeBodyObjectGroups,
        bones: [i32; 3],
    ) -> io::Result<()> {
        if !groups.sets_field_3a9() {
            return Ok(());
        }
        let point = point.map(f64::from);
        let mut top = false;
        let mut bottom = false;
        for group in groups.retained_groups() {
            if matches!(group.kind_34, 8 | 9) && group.matching_weight(point)?.is_some() {
                if group.kind_34 == 8 {
                    top = true;
                } else {
                    bottom = true;
                }
            }
        }
        let mut count = 0;
        let mut sum = 0_f64;
        for group in groups.retained_groups() {
            if count > 4 {
                break;
            }
            if group.kind_34 != 1 {
                continue;
            }
            let Some(weight) = group.matching_weight(point)? else {
                continue;
            };
            let weight = weight as f32;
            let special = group.flags_50[3] || group.flags_50[4];
            let (destination, bone) = if group.flags_50[0] {
                if !native_body_is_bone(group.name(), bones[1]) {
                    continue;
                }
                (1, bones[1])
            } else if group.flags_50[4] {
                (count, native_body_tail_bone(group.name()))
            } else if group.flags_50[3] {
                (count, native_body_neck_bone(group.name()))
            } else {
                let Some(index) = bones
                    .iter()
                    .position(|bone| native_body_is_bone(group.name(), *bone))
                else {
                    continue;
                };
                (index, bones[index])
            };
            if special {
                self.write(destination, bone, weight);
                let mut added = weight;
                if matches!(destination, 1 | 2) {
                    let previous = destination - 1;
                    let previous_bone = self.words_00_through_24[previous] as i32;
                    if bone < previous_bone {
                        added = self.weight(previous);
                        self.write(previous, bone, weight);
                        self.write(destination, previous_bone, added);
                    }
                }
                // Native adds the displaced weight when a pair was swapped.
                sum += f64::from(added);
                count += 1;
            } else {
                sum += f64::from(weight);
                if top || bottom {
                    let pair = if top {
                        [bones[0], bones[1]]
                    } else {
                        [bones[1], bones[2]]
                    };
                    self.write(0, pair[0], weight);
                    self.write(1, pair[0], weight);
                    self.write(2, pair[1], weight);
                    sum += f64::from(weight);
                    sum += f64::from(weight);
                    // This native branch does not increment the accepted count.
                } else {
                    self.write(destination, bone, weight);
                    count += 1;
                }
            }
        }
        for index in 0..4 {
            self.words_00_through_24[4 + index] =
                ((f64::from(self.weight(index)) / sum) as f32).to_bits();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{GroupVertex, NativeBodyVertexGroup, VertexGroup, VertexGroupChunk};

    fn groups(names: &[(&str, f32)]) -> NativeBodyObjectGroups {
        let mut result = NativeBodyObjectGroups::default();
        for (name, weight) in names {
            result.apply_group(
                NativeBodyVertexGroup::from_group(&VertexGroup {
                    name: Some(name.as_bytes().to_vec()),
                    members: Some(vec![
                        GroupVertex {
                            position: [0.; 3],
                            weight: *weight,
                        },
                        GroupVertex {
                            position: [1.; 3],
                            weight: 0.,
                        },
                    ]),
                    field_order: vec![VertexGroupChunk::Name, VertexGroupChunk::Members],
                })
                .unwrap(),
            );
        }
        result
    }

    #[test]
    fn native_body_weights_gate_and_empty_membership() {
        let mut state = NativeBodyVertexWeights {
            words_00_through_24: [7; 10],
        };
        let original = state;
        state
            .average_at_native_vertex([0.; 3], &NativeBodyObjectGroups::default(), [0, 1, 2])
            .unwrap();
        assert_eq!(state, original);
        let mut state = NativeBodyVertexWeights::default();
        state
            .average_at_native_vertex([2.; 3], &groups(&[("Torso", 1.)]), [0, 1, 2])
            .unwrap();
        assert!(state.weights().iter().all(|v| v.is_nan()));
    }

    #[test]
    fn native_body_weights_duplicate_destination_and_connection() {
        let mut state = NativeBodyVertexWeights::default();
        state
            .average_at_native_vertex([0.; 3], &groups(&[("Torso", 1.), ("Torso", 3.)]), [0, 1, 2])
            .unwrap();
        assert_eq!(state.weights(), [0., 0.75, 0., 0.]);
        let mut state = NativeBodyVertexWeights::default();
        state
            .average_at_native_vertex(
                [0.; 3],
                &groups(&[("TopConnect", 0.), ("Torso", 2.)]),
                [0, 1, 2],
            )
            .unwrap();
        assert_eq!(state.bones(), [0, 0, 1, 0]);
        assert_eq!(state.weights(), [1. / 3., 1. / 3., 1. / 3., 0.]);
    }

    #[test]
    fn native_body_weights_sort_accumulates_displaced_weight() {
        let mut state = NativeBodyVertexWeights::default();
        state
            .average_at_native_vertex([0.; 3], &groups(&[("Tail5", 2.), ("Tail0", 6.)]), [17; 3])
            .unwrap();
        assert_eq!(state.bones(), [17, 22, 0, 0]);
        assert_eq!(state.weights(), [1.5, 0.5, 0., 0.]);
    }

    #[test]
    fn native_body_weights_fifth_write_preserves_overlap_and_stops() {
        let mut state = NativeBodyVertexWeights::default();
        state
            .average_at_native_vertex(
                [0.; 3],
                &groups(&[
                    ("Tail0", 1.),
                    ("Tail1", 1.),
                    ("Tail2", 1.),
                    ("Tail3", 1.),
                    ("Tail4", 1.),
                    ("Tail5", 99.),
                ]),
                [17; 3],
            )
            .unwrap();
        assert_eq!(state.bones(), [17, 18, 19, 20]);
        assert_eq!(state.words_00_through_24[8], 1_f32.to_bits());
        assert_eq!(state.weights()[0], f32::from_bits(21) / 5.);
        assert_eq!(state.weights()[1..], [0.2; 3]);
    }
}
