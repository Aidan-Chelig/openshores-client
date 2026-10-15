use super::*;
use crate::model::{
    GroupVertex, NativeBodyObjectGroups, NativeBodyStoredVertex, NativeBodyVertexGroup,
    NativeBodyVertexWeights, VertexGroup, VertexGroupChunk,
};

fn object(points: &[f32], group: Option<(&str, f32)>) -> NativeBodyMergeBake {
    let mut groups = NativeBodyObjectGroups::default();
    if let Some((name, x)) = group {
        groups.apply_group(
            NativeBodyVertexGroup::from_group(&VertexGroup {
                name: Some(name.as_bytes().to_vec()),
                members: Some(vec![
                    GroupVertex {
                        position: [x, 0., 0.],
                        weight: 0.,
                    },
                    GroupVertex {
                        position: [99.; 3],
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
            .map(|x| {
                let mut payload = NativeBodyVertexWeights::default();
                payload.words_00_through_24[8] = 0.25_f32.to_bits();
                payload.words_00_through_24[9] = 0.75_f32.to_bits();
                NativeBodyStoredVertex {
                    position: [*x, 0., 0.],
                    payload,
                }
            })
            .collect(),
        groups,
        field_369: true,
    }
}
fn options() -> NativeTorsoVertexOptions {
    NativeTorsoVertexOptions {
        deformation: NativeBodyDeformation::native_initialized(),
        scale_points: [[0.; 3]; 2],
        tolerance: 1.,
        source_mapping_count: 1,
        last_bottom_group: None,
    }
}

#[test]
fn ordinary_torso_vertices_append_even_when_coincident_and_copy_payload_mapping() {
    let mut destination = object(&[0.], None);
    let mut source = object(&[0., 0.], None);
    let result = destination
        .append_torso_vertices(&mut source, options())
        .unwrap();
    assert_eq!(result.source_to_destination, [1, 2]);
    assert_eq!(result.appended_mapping_coordinates, [[0.25, 0.75]]);
    assert!(result.primary.is_empty());
}

#[test]
fn torso_connection_tolerance_is_strict_and_remapping_requires_bottom_membership() {
    let mut destination = object(&[0.], Some(("BottomConnect", 0.)));
    let mut source = object(&[1.], Some(("TopConnect", 1.)));
    let result = destination
        .append_torso_vertices(&mut source, options())
        .unwrap();
    assert_eq!(source.vertices[0].position, [1., 0., 0.]);
    assert!(!result.primary.contains_key(&0));
    let mut source = object(&[0.5], Some(("TopConnect", 0.5)));
    let result = destination
        .append_torso_vertices(&mut source, options())
        .unwrap();
    assert_eq!(source.vertices[0].position, [0.; 3]);
    assert_eq!(result.primary[&0], 0);
    assert_eq!(result.secondary[&2], 0);
    assert_eq!(result.last_bottom_group, Some(0));
}

#[test]
fn ordinary_torso_remapping_depends_on_call_local_bottom_reference() {
    let mut destination = object(&[0.], Some(("BottomConnect", 0.)));
    let mut source = object(&[0.], None);
    let mut configured = options();
    configured.last_bottom_group = Some(0);
    let result = destination
        .append_torso_vertices(&mut source, configured)
        .unwrap();
    assert_eq!(result.primary[&1], 1);
    assert_eq!(result.secondary[&1], 0);
    let result = destination
        .append_torso_vertices(&mut source, options())
        .unwrap();
    assert!(result.primary.is_empty());
}

#[test]
fn torso_first_distance_match_stops_position_search_but_updates_later_bottom_state() {
    let mut destination = object(&[0., 0.8], Some(("BottomConnect", 0.8)));
    let mut source = object(&[0.4, 0.8], Some(("TopConnect", 0.4)));
    let result = destination
        .append_torso_vertices(&mut source, options())
        .unwrap();
    assert_eq!(source.vertices[0].position, [0.; 3]);
    assert!(!result.primary.contains_key(&0));
    assert_eq!(result.last_bottom_group, Some(0));
    assert_eq!(result.primary[&3], 3);
    assert_eq!(result.secondary[&3], 0);
}
