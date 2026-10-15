//! Normal (non-crest) torso source vertex stage; not full instance merging.
use super::{NativeBodyDeformation, NativeBodyMergeBake, native_body_add_if_unique};
use std::{collections::BTreeMap, io};

#[derive(Clone, Copy, Debug)]
pub struct NativeTorsoVertexOptions {
    pub deformation: NativeBodyDeformation,
    pub scale_points: [[f32; 3]; 2],
    pub tolerance: f64,
    pub source_mapping_count: usize,
    /// Call-local result of destination bake, NOT an unconditional group search.
    pub last_bottom_group: Option<usize>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NativeTorsoVertexMapping {
    pub source_to_destination: Vec<usize>,
    pub primary: BTreeMap<usize, usize>,
    pub secondary: BTreeMap<usize, usize>,
    pub appended_mapping_coordinates: Vec<[f32; 2]>,
    pub last_bottom_group: Option<usize>,
}

impl NativeBodyMergeBake {
    pub fn append_torso_vertices(
        &mut self,
        source: &mut Self,
        options: NativeTorsoVertexOptions,
    ) -> io::Result<NativeTorsoVertexMapping> {
        if !self.field_369
            || !options.tolerance.is_finite()
            || self
                .vertices
                .len()
                .checked_add(source.vertices.len())
                .is_none_or(|n| n > i32::MAX as usize)
            || options.last_bottom_group.is_some_and(|index| {
                self.groups
                    .retained_groups()
                    .get(index)
                    .is_none_or(|group| group.kind_34 != 9)
            })
        {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "invalid native torso vertex stage state/capacity",
            ));
        }
        let first_top = source
            .groups
            .retained_groups()
            .iter()
            .find(|group| group.kind_34 == 8);
        let mut result = NativeTorsoVertexMapping {
            last_bottom_group: options.last_bottom_group,
            ..Default::default()
        };
        for (ordinal, vertex) in source.vertices.iter_mut().enumerate() {
            let raw = vertex.position.map(f64::from);
            let connection = first_top
                .map(|group| group.matching_weight(raw))
                .transpose()?
                .flatten()
                .is_some();
            let selected = source.groups.select_vertex(
                raw,
                options.scale_points[0],
                options.scale_points[1],
            )?;
            let position = options
                .deformation
                .apply_vertex(raw, &selected)?
                .map(|v| v as f32);
            if position.iter().any(|v| !v.is_finite()) {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "nonfinite torso source coordinate",
                ));
            }
            vertex.position = position;
            let appended = self.vertices.len();
            let mut connection_index = 0;
            let remap;
            if connection {
                if let Some((candidate, existing)) =
                    self.vertices.iter().enumerate().find(|(_, existing)| {
                        let delta = std::array::from_fn::<_, 3, _>(|i| {
                            f64::from(existing.position[i]) - f64::from(vertex.position[i])
                        });
                        (delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2]).sqrt()
                            < options.tolerance
                    })
                {
                    vertex.position = existing.position;
                    for (index, group) in self
                        .groups
                        .retained_groups()
                        .iter()
                        .enumerate()
                        .filter(|(_, group)| group.kind_34 == 9)
                    {
                        result.last_bottom_group = Some(index);
                        if group
                            .matching_weight(existing.position.map(f64::from))?
                            .is_some()
                        {
                            result.primary.entry(candidate).or_insert(candidate);
                            connection_index = candidate;
                            break;
                        }
                    }
                }
                self.vertices.push(*vertex);
                remap = true;
            } else {
                remap = result
                    .last_bottom_group
                    .map(|index| {
                        self.groups.retained_groups()[index]
                            .matching_weight(vertex.position.map(f64::from))
                    })
                    .transpose()?
                    .flatten()
                    .is_some();
                // Native AddIfUnique has STRICT distance < tolerance; zero
                // therefore appends even an exactly coincident stored vertex.
                let index = native_body_add_if_unique(&mut self.vertices, *vertex, 0., true);
                debug_assert_eq!(index, appended);
            }
            if ordinal < options.source_mapping_count {
                result.appended_mapping_coordinates.push([
                    f32::from_bits(vertex.payload.words_00_through_24[8]),
                    f32::from_bits(vertex.payload.words_00_through_24[9]),
                ]);
            }
            if remap {
                result.primary.entry(appended).or_insert(appended);
                result.secondary.entry(appended).or_insert(connection_index);
            }
            result.source_to_destination.push(appended);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests;
