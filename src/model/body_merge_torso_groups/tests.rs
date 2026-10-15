use super::*;
use crate::model::{
    GroupVertex, NativeBodyDeformation, NativeBodyFaceMetadata, NativeBodyMergeBake,
    NativeBodyObjectGroups, NativeBodyVertexGroup, VertexGroup, VertexGroupChunk,
};

fn object(groups: &[(&str, f32, f32)]) -> NativeBodyMergeGeometry {
    let mut state = NativeBodyObjectGroups::default();
    for (name, x, weight) in groups {
        state.apply_group(
            NativeBodyVertexGroup::from_group(&VertexGroup {
                name: Some(name.as_bytes().to_vec()),
                members: Some(vec![
                    GroupVertex {
                        position: [*x; 3],
                        weight: *weight,
                    },
                    GroupVertex {
                        position: [99.; 3],
                        weight: 0.,
                    },
                ]),
                field_order: vec![VertexGroupChunk::Name, VertexGroupChunk::Members],
            })
            .unwrap(),
        );
    }
    NativeBodyMergeGeometry {
        object: NativeBodyMergeBake {
            vertices: vec![],
            groups: state,
            field_369: true,
        },
        faces: vec![],
        metadata: NativeBodyFaceMetadata::default(),
        deformation: NativeBodyDeformation::native_initialized(),
        scale_points: [[0.; 3]; 2],
    }
}

#[test]
fn torso_group_transfer_deforms_connection_maps_then_replaces_or_unions_by_kind() {
    let mut destination = object(&[
        ("BottomSkin", 0., 10.),
        ("BottomConnect", 0., 20.),
        ("Hand", 4., 30.),
    ]);
    let mut source = object(&[
        ("BottomSkin", 1., 11.),
        ("BottomConnect", 2., 12.),
        ("BottomConnect", 3., 13.),
        ("Torso", 4., 14.),
        ("TopConnect", 5., 15.),
    ]);
    source.deformation.base.fields_1c0 = [10.; 3];
    let old_metadata = destination.metadata.clone();
    assert_eq!(
        destination.transfer_torso_group_maps(&mut source).unwrap(),
        4
    );
    let groups = destination.object.groups.retained_groups();
    assert_eq!(groups.len(), 3);
    assert_eq!(groups[0].matching_weight([0.; 3]).unwrap(), Some(10.));
    assert_eq!(groups[0].matching_weight([11.; 3]).unwrap(), Some(11.));
    assert_eq!(groups[1].matching_weight([0.; 3]).unwrap(), None);
    assert_eq!(groups[1].matching_weight([12.; 3]).unwrap(), None);
    assert_eq!(groups[1].matching_weight([13.; 3]).unwrap(), Some(13.));
    assert_eq!(groups[2].name(), b"Hand");
    assert_eq!(groups[2].matching_weight([4.; 3]).unwrap(), Some(14.));
    assert_eq!(
        source.object.groups.retained_groups()[0]
            .matching_weight([1.; 3])
            .unwrap(),
        None
    );
    assert_eq!(
        source.object.groups.retained_groups()[3]
            .matching_weight([4.; 3])
            .unwrap(),
        Some(14.)
    );
    assert_eq!(destination.metadata, old_metadata);
}

#[test]
fn torso_group_transfer_empty_bottom_map_clears_destination_without_adding_groups() {
    let mut destination = object(&[("BottomConnect", 1., 2.)]);
    let mut source = object(&[]);
    source.object.groups.apply_group(
        NativeBodyVertexGroup::from_group(&VertexGroup {
            name: Some(b"BottomConnect".to_vec()),
            members: Some(vec![]),
            field_order: vec![VertexGroupChunk::Name, VertexGroupChunk::Members],
        })
        .unwrap(),
    );
    destination.transfer_torso_group_maps(&mut source).unwrap();
    assert_eq!(
        destination.object.groups.retained_groups()[0]
            .entries()
            .count(),
        0
    );
    assert_eq!(destination.object.groups.retained_groups().len(), 1);
}
