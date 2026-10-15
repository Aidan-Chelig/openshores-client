//! Decoded mesh -> native object geometry consumed by reconstructed merges.
use super::{
    Mesh, ModelVariant, NativeBodyDeformation, NativeBodyFaceMetadata, NativeBodyLoadedFace,
    NativeBodyMergeBake, NativeBodyMergeGeometry, NativeBodyModelGeometry, NativeBodyObjectGroups,
    NativeBodyStoredVertex,
};
use std::io;

impl Mesh {
    /// Initialize represented object geometry from verified constructor defaults,
    /// definition writes and ordered groups. Metadata is actual caller-supplied
    /// native map state, not inferred from raw material-name encodings.
    /// Does not construct a complete model instance or publish a cache entry.
    pub fn body_merge_geometry(
        &self,
        variant: ModelVariant,
        geometry: &NativeBodyModelGeometry,
        bones: [i32; 3],
        metadata: NativeBodyFaceMetadata,
    ) -> io::Result<NativeBodyMergeGeometry> {
        let groups = NativeBodyObjectGroups::from_mesh(self)?;
        let vertices = self
            .native_weight_lookup_positions(variant)?
            .into_iter()
            .zip(self.initial_native_vertex_payloads(variant)?)
            .map(|(position, mut payload)| {
                payload.average_at_native_vertex(position, &groups, bones)?;
                Ok(NativeBodyStoredVertex { position, payload })
            })
            .collect::<io::Result<Vec<_>>>()?;
        let mut deformation = NativeBodyDeformation::native_initialized();
        geometry.apply_to_deformation(&mut deformation);
        if let Some(point) = groups.control_point(0x1cc) {
            deformation.base.fields_1f4 = point;
        }
        if let Some(point) = groups.control_point(0x200) {
            deformation.fields_228 = point;
        }
        if let Some(point) = groups.control_point(0x29c) {
            deformation.fields_2c4 = point;
        }
        if let Some(point) = groups.control_point(0x234) {
            deformation.fields_25c = point;
        }
        let scale_points = [
            groups.control_point(0x268).unwrap_or([0.; 3]),
            groups.control_point(0x2d0).unwrap_or([0.; 3]),
        ];
        let faces = self
            .faces
            .iter()
            .map(|face| NativeBodyLoadedFace::from_serialized(face, variant))
            .collect::<io::Result<Vec<_>>>()?;
        Ok(NativeBodyMergeGeometry {
            object: NativeBodyMergeBake {
                vertices,
                groups,
                field_369: false,
            },
            faces,
            metadata,
            deformation,
            scale_points,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Face, GeometryChunk, GroupVertex, VertexGroup, VertexGroupChunk};

    #[test]
    fn merge_loading_keeps_native_buffer_coordinates_and_actual_control_writes() {
        let mesh = Mesh {
            vertices: vec![[10., 20., 30.]; 3],
            matrix: Some([1., 0., 0., 0., 1., 0., 0., 0., 1., 10., 20., 30.]),
            geometry_order: vec![GeometryChunk::Vertices, GeometryChunk::Matrix],
            uvs: Some(vec![[0.25, 0.75]; 3]),
            faces: vec![Face {
                vertices: [0, 1, 2],
                flags: 7,
                extension: Some([0.25; 9]),
                nt_extension: None,
            }],
            vertex_groups: vec![VertexGroup {
                name: Some(b"ConnectPt".to_vec()),
                members: Some(vec![GroupVertex {
                    position: [5., 6., 7.],
                    weight: 1.,
                }]),
                field_order: vec![VertexGroupChunk::Name, VertexGroupChunk::Members],
            }],
            ..Default::default()
        };
        let mut mesh = mesh;
        mesh.geometry_order.push(GeometryChunk::MappingCoordinates);
        let definition = NativeBodyModelGeometry {
            fields_00: [1.; 6],
            fields_40: [1., 2., 3.],
            fields_58: [1.; 5],
        };
        let result = mesh
            .body_merge_geometry(
                ModelVariant::N,
                &definition,
                [0, 1, 2],
                NativeBodyFaceMetadata::default(),
            )
            .unwrap();
        assert_eq!(result.object.vertices[0].position, [0.; 3]);
        assert_eq!(
            result.object.vertices[0].payload.words_00_through_24,
            [
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0.25_f32.to_bits(),
                0.75_f32.to_bits()
            ]
        );
        assert_eq!(result.deformation.base.fields_1f4, [5., 6., 7.]);
        assert_eq!(result.deformation.base.fields_1c0, [1., 2., 3.]);
        assert_eq!(result.scale_points, [[0.; 3]; 2]);
        assert!(!result.object.field_369);
        assert_eq!(result.faces[0].fields_50, [0.25; 9]);
        assert_eq!(mesh.vertices[0], [10., 20., 30.]);
    }
}
