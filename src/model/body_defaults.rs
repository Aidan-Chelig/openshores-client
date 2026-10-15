//! Proven deformation subset of Au3dsGLSLObject's default constructor.
use super::{
    Mesh, ModelVariant, NativeBodyDeformation, NativeBodyModelGeometry, NativeBodyObjectScale,
};
use std::io;

impl NativeBodyDeformation {
    /// Constructor-established geometry only, NOT a full native object.
    pub fn native_initialized() -> Self {
        Self {
            base: NativeBodyObjectScale {
                field_140: 1.,
                fields_1c0: [0.; 3],
                fields_1f4: [0.; 3],
            },
            field_170: 1.,
            field_178: 1.,
            field_180: 1.,
            field_188: 1.,
            field_190: 1.,
            fields_25c: [0.; 3],
            fields_228: [0.; 3],
            fields_2c4: [0.; 3],
        }
    }
}

impl Mesh {
    /// Actual object geometry defaults -> definition writes -> loaded mesh,
    /// group metadata and CPU deformation. Not weights, assembly or rendering.
    pub fn body_defined_positions(
        &self,
        variant: ModelVariant,
        loader_scale: f64,
        geometry: &NativeBodyModelGeometry,
    ) -> io::Result<Vec<[f64; 3]>> {
        let mut definition = NativeBodyDeformation::native_initialized();
        geometry.apply_to_deformation(&mut definition);
        // The constructor explicitly zeroes TopScalePt and BottomScalePt too.
        self.body_deformed_positions(variant, loader_scale, &definition, [0.; 3], [0.; 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{GroupVertex, VertexGroup, VertexGroupChunk};
    #[test]
    fn constructor_subset_has_all_scales_one_and_positive_zero_control_points() {
        let object = NativeBodyDeformation::native_initialized();
        assert_eq!(
            [
                object.base.field_140,
                object.field_170,
                object.field_178,
                object.field_180,
                object.field_188,
                object.field_190
            ],
            [1.; 6]
        );
        for point in [
            object.base.fields_1c0,
            object.base.fields_1f4,
            object.fields_25c,
            object.fields_228,
            object.fields_2c4,
        ] {
            assert_eq!(point.map(f32::to_bits), [0; 3]);
        }
    }
    #[test]
    fn native_defaults_definition_and_serialized_connection_compose_without_fixture_defaults() {
        let mesh = Mesh {
            vertices: vec![[4., 5., 10.]],
            vertex_groups: vec![VertexGroup {
                name: Some(b"ConnectPt".to_vec()),
                members: Some(vec![GroupVertex {
                    position: [1., 2., 3.],
                    weight: 1.,
                }]),
                field_order: vec![VertexGroupChunk::Name, VertexGroupChunk::Members],
            }],
            ..Default::default()
        };
        let geometry = NativeBodyModelGeometry {
            fields_00: [1., 2., 0., 0., 0., 0.],
            fields_40: [10., 20., 30.],
            fields_58: [1.; 5],
        };
        assert_eq!(
            mesh.body_defined_positions(ModelVariant::N, 1., &geometry)
                .unwrap(),
            [[16., 26., 44.]]
        );
        assert_eq!(mesh.vertices, [[4., 5., 10.]]);
    }
    #[test]
    fn malformed_groups_fail_without_fabricating_a_defined_mesh() {
        let mesh = Mesh {
            vertex_groups: vec![VertexGroup {
                name: Some(b"Hand".to_vec()),
                ..Default::default()
            }],
            ..Default::default()
        };
        let geometry = NativeBodyModelGeometry {
            fields_00: [1.; 6],
            fields_40: [0.; 3],
            fields_58: [1.; 5],
        };
        assert_eq!(
            mesh.body_defined_positions(ModelVariant::N, 1., &geometry)
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData
        );
    }
}
