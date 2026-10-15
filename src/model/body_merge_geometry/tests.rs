use super::*;
use crate::model::{
    GroupVertex, NativeBodyObjectGroups, NativeBodyStoredVertex, NativeBodyVertexGroup,
    VertexGroup, VertexGroupChunk,
};
use std::collections::BTreeMap;

fn geometry(name: &str, x: f32, normal: f64) -> NativeBodyMergeGeometry {
    let mut groups = NativeBodyObjectGroups::default();
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
    NativeBodyMergeGeometry {
        mapping_coordinates: vec![[0.25, 0.75]],
        object: NativeBodyMergeBake {
            vertices: vec![NativeBodyStoredVertex {
                position: [x, 0., 0.],
                payload: Default::default(),
            }],
            groups,
            field_369: false,
        },
        faces: vec![NativeBodyLoadedFace {
            vertices: [0; 3],
            flags_06: 7,
            fields_08: [3.; 9],
            fields_50: [normal; 9],
            field_98: 0.5,
            fields_9c: [9; 3],
        }],
        metadata: NativeBodyFaceMetadata {
            materials: BTreeMap::from([(vec![97], vec![0])]),
            smoothing: vec![42],
        },
        deformation: NativeBodyDeformation::native_initialized(),
        scale_points: [[0.; 3]; 2],
    }
}

#[test]
fn composed_head_geometry_bakes_maps_faces_materials_and_connection_normals() {
    let mut destination = geometry("BottomConnect", 0., 9.);
    let mut source = geometry("TopConnect", 0.5, 2.);
    let report = destination
        .append_head_geometry_stages(&mut source, ModelVariant::N, 1.)
        .unwrap();
    assert!(destination.object.field_369);
    assert!(!source.object.field_369);
    assert_eq!(report.vertices.source_to_destination, [1]);
    assert!(report.fully_connected);
    assert_eq!(report.updated_corners, 6);
    assert_eq!(destination.object.vertices.len(), 2);
    assert_eq!(destination.faces.len(), 2);
    assert_eq!(destination.faces[1].vertices, [1; 3]);
    assert_eq!(destination.faces[1].flags_06, 0);
    assert!(
        destination
            .faces
            .iter()
            .all(|face| face.fields_50 == [2.; 9])
    );
    assert_eq!(destination.metadata.materials[&vec![97]], [0, 2]);
    assert_eq!(destination.metadata.smoothing, [42]);
    assert_eq!(source.faces[0].flags_06, 7);
}

#[test]
fn composed_standard_head_geometry_resets_smoothing_without_normal_replacement() {
    let mut destination = geometry("BottomConnect", 0., 9.);
    let mut source = geometry("TopConnect", 0.5, 2.);
    let report = destination
        .append_head_geometry_stages(&mut source, ModelVariant::Standard, 1.)
        .unwrap();
    assert_eq!(report.updated_corners, 0);
    assert_eq!(destination.faces[0].fields_50, [9.; 9]);
    assert_eq!(destination.faces[1].fields_50, [2.; 9]);
    assert_eq!(destination.metadata.smoothing, [1; 2]);
}

#[test]
fn composed_head_geometry_inherits_destination_connection_before_source_deformation() {
    let mut destination = geometry("BottomConnect", 0., 9.);
    destination.deformation.base.fields_1f4 = [0., 2., 0.];
    let mut source = geometry("TopConnect", 0.5, 2.);
    source.deformation.base.fields_1f4 = [0., 100., 0.];
    destination
        .append_head_geometry_stages(&mut source, ModelVariant::N, 0.01)
        .unwrap();
    assert_eq!(source.deformation.base.fields_1f4, [0., 2., 0.]);
    assert_eq!(source.object.vertices[0].position, [0.5, -2., 0.]);
}

#[test]
fn composed_torso_geometry_retains_mapping_arrays_and_does_not_copy_head_material_rules() {
    let mut destination = geometry("BottomConnect", 0., 9.);
    let mut source = geometry("TopConnect", 0.5, 2.);
    let report = destination
        .append_torso_geometry_stages(&mut source, ModelVariant::N, 1.)
        .unwrap();
    assert!(report.fully_connected);
    assert_eq!(report.updated_corners, 6);
    assert_eq!(report.vertices.source_to_destination, [1]);
    assert_eq!(destination.mapping_coordinates, [[0.25, 0.75], [0., 0.]]);
    assert_eq!(destination.metadata.materials[&vec![97]], [0]);
    assert_eq!(destination.metadata.smoothing, [42]);
    assert_eq!(destination.faces.len(), 2);
    assert_eq!(destination.faces[1].vertices, [1; 3]);
    assert_eq!(destination.faces[0].fields_50, [2.; 9]);
    let mut later = geometry("BottomSkin", 2., 3.);
    let report = destination
        .append_torso_geometry_stages(&mut later, ModelVariant::N, 1.)
        .unwrap();
    assert!(report.vertices.primary.is_empty());
    assert_eq!(report.vertices.last_bottom_group, None);
    assert_eq!(destination.faces.len(), 3);
}

#[test]
fn composed_standard_torso_geometry_applies_final_smoothing_and_group_transfer() {
    let mut destination = geometry("BottomSkin", 0., 9.);
    let mut source = geometry("BottomSkin", 0.5, 2.);
    let report = destination
        .append_torso_geometry_stages(&mut source, ModelVariant::Standard, 1.)
        .unwrap();
    assert_eq!(report.updated_corners, 0);
    assert_eq!(report.group_map_updates, 1);
    assert_eq!(destination.metadata.smoothing, [1; 2]);
    assert_eq!(
        destination.object.groups.retained_groups()[0]
            .matching_weight([0.5, 0., 0.])
            .unwrap(),
        Some(0.)
    );
}
