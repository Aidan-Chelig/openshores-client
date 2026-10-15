//! Source-vertex stage of MergeHeadWithInstance, before faces/groups/materials.
use super::{NativeBodyDeformation, NativeBodyMergeBake, native_body_add_if_unique};
use std::{collections::BTreeMap, io};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NativeHeadVertexMapping {
    pub source_to_destination: Vec<usize>,
    pub existing_to_appended: BTreeMap<usize, usize>,
    pub appended_to_connection: BTreeMap<usize, usize>,
}

impl NativeBodyMergeBake {
    /// Destination must have completed its one-time bake. Definition includes
    /// native source connection-field inheritance, performed by the caller.
    /// This mutates source xyz like native code, but not its group keys.
    /// Returned mappings are NOT proof of a complete model merge.
    pub fn append_head_vertices(
        &mut self,
        source: &mut Self,
        definition: &NativeBodyDeformation,
        scale_points: [[f32; 3]; 2],
        tolerance: f64,
    ) -> io::Result<NativeHeadVertexMapping> {
        if !self.field_369
            || !tolerance.is_finite()
            || self
                .vertices
                .len()
                .checked_add(source.vertices.len())
                .is_none_or(|count| count > i32::MAX as usize)
        {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "head vertex mapping requires baked destination, finite tolerance and native index capacity",
            ));
        }
        let first_top = source
            .groups
            .retained_groups()
            .iter()
            .find(|g| g.kind_34 == 8);
        // Native checks existence of a BottomConnect group, not membership
        // of the candidate vertex in that group's map.
        let has_bottom = self.groups.retained_groups().iter().any(|g| g.kind_34 == 9);
        let mut result = NativeHeadVertexMapping::default();
        for vertex in &mut source.vertices {
            let raw = vertex.position.map(f64::from);
            let connection = first_top
                .map(|g| g.matching_weight(raw))
                .transpose()?
                .flatten()
                .is_some();
            let selected = source
                .groups
                .select_vertex(raw, scale_points[0], scale_points[1])?;
            let position = definition.apply_vertex(raw, &selected)?.map(|v| v as f32);
            if position.iter().any(|v| !v.is_finite()) {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "nonfinite native head merge coordinate",
                ));
            }
            vertex.position = position;
            if !connection {
                result.source_to_destination.push(native_body_add_if_unique(
                    &mut self.vertices,
                    *vertex,
                    tolerance,
                    true,
                ));
                continue;
            }
            let appended = self.vertices.len();
            let mut matched = appended;
            for (index, existing) in self.vertices.iter().enumerate() {
                if (0..3).all(|axis| {
                    (f64::from(existing.position[axis]) - f64::from(vertex.position[axis])).abs()
                        <= tolerance
                }) {
                    vertex.position = existing.position;
                    if has_bottom {
                        result.existing_to_appended.entry(index).or_insert(appended);
                        matched = index;
                        break;
                    }
                    // Without BottomConnect, continue from the newly snapped
                    // position: a later match can snap the same vertex again.
                }
            }
            self.vertices.push(*vertex);
            result
                .existing_to_appended
                .entry(appended)
                .or_insert(appended);
            result
                .appended_to_connection
                .entry(appended)
                .or_insert(matched);
            result.source_to_destination.push(appended);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests;
