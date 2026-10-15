use super::*;
use crate::model::{
    GroupVertex, NativeBodyObjectGroups, NativeBodyStoredVertex, NativeBodyVertexGroup,
    NativeBodyVertexWeights, VertexGroup, VertexGroupChunk,
};

fn object(points: &[f32], names: &[(&str, f32)]) -> NativeBodyMergeBake {
    let mut groups = NativeBodyObjectGroups::default();
    for (name, x) in names {
        groups.apply_group(
            NativeBodyVertexGroup::from_group(&VertexGroup {
                name: Some(name.as_bytes().to_vec()),
                members: Some(vec![
                    GroupVertex {
                        position: [*x, 0., 0.],
                        weight: 0.,
                    },
                    GroupVertex {
                        position: [100., 0., 0.],
                        weight: 1.,
                    },
                ]),
                field_order: vec![VertexGroupChunk::Name, VertexGroupChunk::Members],
            })
            .unwrap(),
        );
    }
    NativeBodyMergeBake {
        vertices: points
            .iter()
            .map(|x| NativeBodyStoredVertex {
                position: [*x, 0., 0.],
                payload: NativeBodyVertexWeights::default(),
            })
            .collect(),
        groups,
        field_369: true,
    }
}

#[test]
fn head_connection_vertices_append_at_inclusive_boundary_and_keep_first_mapping() {
    let mut destination = object(&[0.], &[("BottomConnect", 99.)]);
    let mut source = object(&[1., 1.], &[("TopConnect", 1.)]);
    let result = destination
        .append_head_vertices(
            &mut source,
            &NativeBodyDeformation::native_initialized(),
            [[0.; 3]; 2],
            1.,
        )
        .unwrap();
    assert_eq!(result.source_to_destination, [1, 2]);
    assert_eq!(result.existing_to_appended[&0], 1);
    assert_eq!(result.appended_to_connection[&1], 0);
    assert_eq!(result.appended_to_connection[&2], 0);
    assert!(destination.vertices.iter().all(|v| v.position == [0.; 3]));
    assert_eq!(
        source.groups.retained_groups()[0]
            .matching_weight([1., 0., 0.])
            .unwrap(),
        Some(0.)
    );
}

#[test]
fn head_only_first_top_group_controls_connection_and_nonconnection_source_stays_unsnapped() {
    let mut destination = object(&[0.], &[("BottomConnect", 0.)]);
    let mut source = object(&[0.5], &[("TopConnect", 90.), ("TopConnect", 0.5)]);
    let result = destination
        .append_head_vertices(
            &mut source,
            &NativeBodyDeformation::native_initialized(),
            [[0.; 3]; 2],
            1.,
        )
        .unwrap();
    assert_eq!(result.source_to_destination, [0]);
    assert_eq!(destination.vertices.len(), 1);
    assert_eq!(source.vertices[0].position, [0.5, 0., 0.]);
    assert!(result.appended_to_connection.is_empty());
}

#[test]
fn head_without_bottom_group_can_snap_repeatedly_but_still_appends() {
    let mut destination = object(&[0., 0.8], &[]);
    let mut source = object(&[0.1], &[("TopConnect", 0.1)]);
    let result = destination
        .append_head_vertices(
            &mut source,
            &NativeBodyDeformation::native_initialized(),
            [[0.; 3]; 2],
            1.,
        )
        .unwrap();
    assert_eq!(result.source_to_destination, [2]);
    assert_eq!(source.vertices[0].position, [0.8, 0., 0.]);
    assert_eq!(result.appended_to_connection[&2], 2);
}
