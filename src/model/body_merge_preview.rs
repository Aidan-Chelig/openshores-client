//! Explicit untextured preview adapter for already baked merge geometry.
use super::{Face, Mesh, ModelVariant, NativeBodyMergeGeometry, NativeBodyMeshGeometry};
use std::io;

impl NativeBodyMergeGeometry {
    /// `projection_source` is the ORIGINAL destination/seed object's mesh.
    /// This projects stored merged coordinates once, at loader-scale1, and
    /// retains raw weights. It does not apply a bone pose or world-axis map.
    /// Texturing is intentionally absent; native material/UV publication is
    /// separate. This is not cache/model-instance lifecycle emulation.
    pub fn untextured_baked_preview_geometry(
        &self,
        projection_source: &Mesh,
        variant: ModelVariant,
    ) -> io::Result<NativeBodyMeshGeometry> {
        if !self.object.field_369 {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "merged preview requires completed geometry baking",
            ));
        }
        let points: Vec<_> = self
            .object
            .vertices
            .iter()
            .map(|vertex| vertex.position)
            .collect();
        let positions = projection_source.project_native_stored_positions(variant, &points)?;
        let faces = self
            .faces
            .iter()
            .map(|face| {
                if face
                    .vertices
                    .iter()
                    .any(|index| usize::from(*index) >= positions.len())
                {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "merged preview face index out of bounds",
                    ));
                }
                Ok(Face {
                    vertices: face.vertices,
                    flags: face.flags_06,
                    extension: matches!(variant, ModelVariant::N | ModelVariant::NT)
                        .then(|| face.fields_50.map(|v| v as f32)),
                    nt_extension: (variant == ModelVariant::NT).then(|| {
                        std::array::from_fn(|i| {
                            if i < 9 {
                                face.fields_08[i] as f32
                            } else {
                                face.field_98
                            }
                        })
                    }),
                })
            })
            .collect::<io::Result<Vec<_>>>()?;
        Ok(NativeBodyMeshGeometry {
            positions,
            faces,
            weights: self
                .object
                .vertices
                .iter()
                .map(|vertex| vertex.payload)
                .collect(),
            uvs: None,
        })
    }
}
