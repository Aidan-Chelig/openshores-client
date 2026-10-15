//! The first object's one-time preparation in head/torso merge routines.
use super::{NativeBodyDeformation, NativeBodyObjectGroups, NativeBodyStoredVertex};
use std::io;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeBodyMergeBake {
    pub vertices: Vec<NativeBodyStoredVertex>,
    pub groups: NativeBodyObjectGroups,
    pub field_369: bool,
}

fn stored(point: [f64; 3]) -> io::Result<[f32; 3]> {
    let point = point.map(|v| v as f32);
    if point.iter().any(|v| !v.is_finite()) {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "nonfinite native merge-bake coordinate",
        ));
    }
    Ok(point)
}

impl NativeBodyMergeBake {
    /// Inputs are actual object deformation/control points. This is only the
    /// shared preparation stage, not CopyInstance, welding or face remapping.
    /// On error earlier vertex/group writes survive, but field369 stays false.
    pub fn bake_once(
        &mut self,
        definition: &NativeBodyDeformation,
        top: [f32; 3],
        bottom: [f32; 3],
    ) -> io::Result<()> {
        if self.field_369 {
            return Ok(());
        }
        for vertex in &mut self.vertices {
            let point = vertex.position.map(f64::from);
            vertex.position = stored(
                definition.apply_vertex(point, &self.groups.select_vertex(point, top, bottom)?)?,
            )?;
        }
        for index in 0..self.groups.retained_groups().len() {
            if self.groups.retained_groups()[index].kind_34 != 9 {
                continue;
            }
            let group = self.groups.retained_groups()[index].remap_positions(|point| {
                let point = point.map(f64::from);
                stored(
                    definition
                        .apply_vertex(point, &self.groups.select_vertex(point, top, bottom)?)?,
                )
            })?;
            self.groups.replace_retained_group(index, group);
        }
        self.field_369 = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        GroupVertex, NativeBodyVertexGroup, NativeBodyVertexWeights, VertexGroup, VertexGroupChunk,
    };

    #[test]
    fn native_merge_bake_rewrites_only_bottom_connection_keys_and_runs_once() {
        let mut groups = NativeBodyObjectGroups::default();
        for name in ["BottomConnect", "TopConnect"] {
            groups.apply_group(
                NativeBodyVertexGroup::from_group(&VertexGroup {
                    name: Some(name.as_bytes().to_vec()),
                    members: Some(vec![
                        GroupVertex {
                            position: [1.; 3],
                            weight: 2.,
                        },
                        GroupVertex {
                            position: [2.; 3],
                            weight: 3.,
                        },
                    ]),
                    field_order: vec![VertexGroupChunk::Name, VertexGroupChunk::Members],
                })
                .unwrap(),
            );
        }
        let payload = NativeBodyVertexWeights {
            words_00_through_24: [77; 10],
        };
        let mut object = NativeBodyMergeBake {
            vertices: vec![NativeBodyStoredVertex {
                position: [1.; 3],
                payload,
            }],
            groups,
            field_369: false,
        };
        let mut definition = NativeBodyDeformation::native_initialized();
        definition.base.fields_1c0 = [10., 20., 30.];
        object.bake_once(&definition, [0.; 3], [0.; 3]).unwrap();
        assert_eq!(object.vertices[0].position, [11., 21., 31.]);
        assert_eq!(object.vertices[0].payload, payload);
        assert_eq!(
            object.groups.retained_groups()[0]
                .matching_weight([11., 21., 31.])
                .unwrap(),
            Some(2.)
        );
        assert_eq!(
            object.groups.retained_groups()[1]
                .matching_weight([1.; 3])
                .unwrap(),
            Some(2.)
        );
        let baked = object.clone();
        object.bake_once(&definition, [0.; 3], [0.; 3]).unwrap();
        assert_eq!(object, baked);
    }

    #[test]
    fn native_merge_bake_key_collisions_keep_last_weight_without_reclassifying() {
        let group = NativeBodyVertexGroup::from_group(&VertexGroup {
            name: Some(b"BottomConnect".to_vec()),
            members: Some(vec![
                GroupVertex {
                    position: [2.; 3],
                    weight: 2.,
                },
                GroupVertex {
                    position: [1.; 3],
                    weight: 3.,
                },
            ]),
            field_order: vec![VertexGroupChunk::Name, VertexGroupChunk::Members],
        })
        .unwrap();
        let remapped = group.remap_positions(|_| Ok([0.; 3])).unwrap();
        assert_eq!(remapped.entries().collect::<Vec<_>>(), vec![([0.; 3], 2.)]);
        assert_eq!(remapped.kind_34, 9);
        assert_eq!(remapped.name(), group.name());
        assert_eq!(remapped.point_28, group.point_28);
        assert_eq!(group.entries().count(), 2);
    }

    #[test]
    fn native_merge_bake_failure_keeps_prior_writes_and_does_not_set_gate() {
        let vertex = |position| NativeBodyStoredVertex {
            position,
            payload: NativeBodyVertexWeights::default(),
        };
        let mut object = NativeBodyMergeBake {
            vertices: vec![vertex([1.; 3]), vertex([f32::MAX; 3])],
            groups: NativeBodyObjectGroups::default(),
            field_369: false,
        };
        let mut definition = NativeBodyDeformation::native_initialized();
        definition.base.field_140 = 2.;
        assert_eq!(
            object
                .bake_once(&definition, [0.; 3], [0.; 3])
                .unwrap_err()
                .kind(),
            io::ErrorKind::Unsupported
        );
        assert_eq!(object.vertices[0].position, [2.; 3]);
        assert_eq!(object.vertices[1].position, [f32::MAX; 3]);
        assert!(!object.field_369);
    }
}
