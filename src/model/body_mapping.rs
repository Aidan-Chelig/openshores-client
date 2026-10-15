//! LoadMeshChunk's immediate SetMappingCoords call, before AverageWeights.
use super::{GeometryChunk, Mesh, ModelVariant, NativeBodyVertexWeights, invalid};
use std::io;

impl Mesh {
    pub(super) fn initial_native_vertex_payloads(
        &self,
        variant: ModelVariant,
    ) -> io::Result<Vec<NativeBodyVertexWeights>> {
        let mappings = self
            .geometry_order
            .iter()
            .filter(|chunk| **chunk == GeometryChunk::MappingCoordinates)
            .count();
        if mappings != usize::from(self.uvs.is_some()) {
            return Err(invalid(
                "missing or inconsistent native mapping chunk order",
            ));
        }
        let mut result = vec![NativeBodyVertexWeights::default(); self.vertices.len()];
        let mut loaded_vertices = false;
        for chunk in &self.geometry_order {
            match chunk {
                GeometryChunk::Vertices => loaded_vertices = true,
                GeometryChunk::MappingCoordinates
                    if variant != ModelVariant::X && loaded_vertices =>
                {
                    let uvs = self
                        .uvs
                        .as_ref()
                        .ok_or_else(|| invalid("missing native mapping coordinates"))?;
                    // Native does nothing unless the counts are exactly equal.
                    if uvs.len() == result.len() {
                        for (vertex, uv) in result.iter_mut().zip(uvs) {
                            vertex.words_00_through_24[8] = uv[0].to_bits();
                            vertex.words_00_through_24[9] = uv[1].to_bits();
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_mapping_writes_obey_chunk_order_format_and_equal_counts() {
        let mut mesh = Mesh {
            vertices: vec![[1.; 3]],
            uvs: Some(vec![[0.25, -0.5]]),
            geometry_order: vec![GeometryChunk::Vertices, GeometryChunk::MappingCoordinates],
            ..Default::default()
        };
        for variant in [ModelVariant::Standard, ModelVariant::N, ModelVariant::NT] {
            let payload = mesh.native_body_vertex_weights(variant, [0; 3]).unwrap();
            assert_eq!(
                &payload[0].words_00_through_24[8..],
                &[0.25_f32.to_bits(), (-0.5_f32).to_bits()]
            );
        }
        assert_eq!(
            mesh.native_body_vertex_weights(ModelVariant::X, [0; 3])
                .unwrap()[0]
                .words_00_through_24,
            [0; 10]
        );
        mesh.geometry_order.reverse();
        assert_eq!(
            mesh.native_body_vertex_weights(ModelVariant::N, [0; 3])
                .unwrap()[0]
                .words_00_through_24,
            [0; 10]
        );
        mesh.geometry_order.reverse();
        mesh.uvs = Some(vec![]);
        assert_eq!(
            mesh.native_body_vertex_weights(ModelVariant::N, [0; 3])
                .unwrap()[0]
                .words_00_through_24,
            [0; 10]
        );
        mesh.geometry_order.pop();
        assert!(
            mesh.native_body_vertex_weights(ModelVariant::N, [0; 3])
                .is_err()
        );
    }
}
