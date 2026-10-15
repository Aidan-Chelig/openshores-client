//! FootLength/FootHeight share extent/point loading and the scalar pipeline,
//! but select the Foot flag and different DNA bytes. Not a hand-shape fallback.
use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeDpFootGeometry(NativeDpHandGeometry);

impl NativeDpFootGeometry {
    pub fn from_model(model: &Model) -> io::Result<Self> {
        NativeDpHandGeometry::from_model_pivot(model, 3).map(Self)
    }

    /// Actual cached [0, length, height]. Native returns Y for FootLength and
    /// Z for FootHeight, each narrowed to float, not the decompiler's void.
    pub fn offset_for_dna(&self, dna: &NativeDhDna) -> [f64; 3] {
        self.0.offset_for_fields(dna, dna.0[3], dna.0[2], dna.0[10])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn foot_model_selects_foot_pivot_instead_of_earlier_hand_group() {
        use crate::model::{Face, GroupVertex, Mesh};
        let group = |name: &[u8], p| VertexGroup {
            name: Some(name.to_vec()),
            members: Some(vec![
                GroupVertex {
                    position: [0.; 3],
                    weight: 1.,
                },
                GroupVertex {
                    position: p,
                    weight: 1.,
                },
            ]),
            field_order: vec![VertexGroupChunk::Name, VertexGroupChunk::Members],
        };
        let model = Model {
            meshes: vec![Mesh {
                name: b"FootMesh".to_vec(),
                vertices: vec![[1., 2., 3.], [4., 8., 6.], [2., 3., -1.]],
                faces: vec![Face {
                    vertices: [0, 1, 2],
                    flags: 0,
                    extension: Some([0.; 9]),
                    nt_extension: None,
                }],
                vertex_groups: vec![
                    group(b"Hand", [20.; 3]),
                    group(b"Foot", [4., 5., 6.]),
                    group(b"Foot", [30.; 3]),
                    group(b"ConnectPt", [1., 2., 3.]),
                ],
                ..Default::default()
            }],
            ..Default::default()
        };
        let geometry = NativeDpFootGeometry::from_model(&model).unwrap();
        assert_eq!(geometry.0.pivot, [4., 5., 6.]);
        assert_eq!(geometry.0.connect, [1., 2., 3.]);
        assert_eq!(geometry.0.minimum, [1., 2., -1.]);
        assert_eq!(geometry.0.maximum, [4., 8., 6.]);
        assert_eq!(
            NativeDpFootGeometry::from_model(&Model::default())
                .unwrap_err()
                .kind(),
            io::ErrorKind::Unsupported
        );
    }
    #[test]
    fn foot_point_classification_preserves_priority_and_single_record_fast_path() {
        for name in [b"Foot".as_slice(), b"RightFoot"] {
            let mut group = VertexGroup {
                name: Some(name.to_vec()),
                members: Some(vec![crate::model::GroupVertex {
                    position: [1., 2., 3.],
                    weight: 0.,
                }]),
                field_order: vec![VertexGroupChunk::Name, VertexGroupChunk::Members],
            };
            assert_eq!(point(&group).unwrap(), (3, [0.; 3]));
            group
                .members
                .as_mut()
                .unwrap()
                .push(crate::model::GroupVertex {
                    position: [4., 5., 6.],
                    weight: 1.,
                });
            assert_eq!(point(&group).unwrap(), (3, [4., 5., 6.]));
            group.field_order.reverse();
            assert_eq!(point(&group).unwrap(), (3, [0.; 3]));
        }
        assert_eq!(point_kind(b"HandFoot"), 2);
        assert_eq!(point_kind(b"FootConnectPt"), 1);
        assert_eq!(point_kind(b"TorsoFoot"), 0);
    }

    #[test]
    fn foot_scaling_reads_leg_selectors_not_hand_selectors() {
        let geometry = NativeDpFootGeometry(NativeDpHandGeometry {
            minimum: [-1., -2., -3.],
            maximum: [4., 8., 6.],
            connect: [0., 1., 2.],
            pivot: [0.5, 0.75, 1.25],
        });
        let mut dna = NativeDhDna([0; 24]);
        let original = geometry.offset_for_dna(&dna);
        for index in [6, 8, 11] {
            dna.0[index] = 255;
        }
        assert_eq!(geometry.offset_for_dna(&dna), original);
        for index in [2, 3, 10] {
            let mut changed = dna.clone();
            changed.0[index] = 255;
            assert_ne!(geometry.offset_for_dna(&changed), original);
        }
        for (raw, bits) in [
            (0, [0, 0x3fbbc0ec1335cb58, 0x3fd3d2f1c4942388]),
            (0x55, [0, 0x400c98bf0aa80000, 0x40068a0bdb000000]),
            (0xaa, [0, 0x4021a0194b8e79e1, 0x402661a6374f55b0]),
            (255, [0, 0x404c88ec526be000, 0x4040b9460cac0000]),
        ] {
            assert_eq!(
                geometry
                    .offset_for_dna(&NativeDhDna([raw; 24]))
                    .map(f64::to_bits),
                bits
            );
        }
    }
}
