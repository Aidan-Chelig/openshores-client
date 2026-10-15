//! Normal N/NT body upload inputs, separate from native cache and bone posing.
use super::{Mesh, ModelVariant, NativeBodyMergeGeometry, NativeBodyMeshGeometry, invalid};
use std::{collections::BTreeMap, io};

#[derive(Debug)]
pub struct NativeBodySurfacePreview {
    pub geometry: NativeBodyMeshGeometry,
    /// GetTextureWeight: zero for kind7 membership, one otherwise. Interpolated
    /// by the native body shader when blending the two selected atlas regions.
    pub top_face: Vec<f32>,
    /// Preserve EVERY matching material name, in QString map order. Do not
    /// silently repair native head's post-append-count membership behavior.
    /// Material colors are not texture selectors for the normal body shader.
    pub material_memberships: Vec<Vec<Vec<u16>>>,
}

impl NativeBodySurfacePreview {
    /// Partition without changing face order within each membership set. Empty
    /// membership is an explicit partition, not an invented default material.
    pub fn partitions(&self) -> BTreeMap<Vec<Vec<u16>>, Vec<usize>> {
        let mut partitions = BTreeMap::new();
        for (face, names) in self.material_memberships.iter().enumerate() {
            partitions
                .entry(names.clone())
                .or_insert_with(Vec::new)
                .push(face);
        }
        partitions
    }
}

impl NativeBodyMergeGeometry {
    pub fn textured_baked_preview_geometry(
        &self,
        projection_source: &Mesh,
        variant: ModelVariant,
    ) -> io::Result<NativeBodySurfacePreview> {
        if !matches!(variant, ModelVariant::N | ModelVariant::NT) {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "textured body preview supports normal N/NT upload only",
            ));
        }
        // Native normal-body upload initializes UVs to zero if mapping is absent.
        // A textured diagnostic must report that absence, not display a fabricated skin.
        if self.mapping_coordinates.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "textured body preview has no native mapping coordinates",
            ));
        }
        let mut geometry = self.untextured_baked_preview_geometry(projection_source, variant)?;
        let mut uvs = Vec::with_capacity(self.object.vertices.len());
        let mut top_face = Vec::with_capacity(self.object.vertices.len());
        for vertex in &self.object.vertices {
            let uv = [8, 9].map(|index| f32::from_bits(vertex.payload.words_00_through_24[index]));
            if uv.iter().any(|value| !value.is_finite()) {
                return Err(invalid("nonfinite native body UV payload"));
            }
            uvs.push(uv);
            let mut top = 1.;
            for group in self.object.groups.retained_groups() {
                if group.kind_34 == 7
                    && group
                        .matching_weight(vertex.position.map(f64::from))?
                        .is_some()
                {
                    top = 0.;
                    break;
                }
            }
            top_face.push(top);
        }
        geometry.uvs = Some(uvs);
        let material_memberships = (0..self.faces.len())
            .map(|face| {
                self.metadata
                    .materials
                    .iter()
                    .filter(|(_, faces)| faces.contains(&(face as u16)))
                    .map(|(name, _)| name.clone())
                    .collect()
            })
            .collect();
        Ok(NativeBodySurfacePreview {
            geometry,
            top_face,
            material_memberships,
        })
    }
}

#[cfg(test)]
mod tests;
