use super::*;
#[test]
fn mesh_cache_evicts_oldest_instead_of_refusing_new_item_types() {
    let mut cache = Cache::default();
    cache.reset((1, 1));
    for index in 0..64 {
        cache.insert(index.to_string(), None);
    }
    assert_eq!(cache.get("0"), Some(None));
    cache.insert("new item".into(), None);
    assert_eq!(cache.models.len(), 64);
    assert!(cache.get("1").is_none());
    assert_eq!(cache.get("0"), Some(None));
    assert_eq!(cache.get("new item"), Some(None));
    cache.reset((2, 1));
    assert!(cache.models.is_empty() && cache.recent.is_empty());
}
#[test]
fn model_geometry_keeps_source_positions_and_rejects_invalid_indices() {
    let mut model = crate::model::Model {
        meshes: vec![crate::model::Mesh {
            vertices: vec![[1.0, 2.0, 3.0], [2.0, 2.0, 3.0], [1.0, 3.0, 3.0]],
            faces: vec![crate::model::Face {
                vertices: [0, 1, 2],
                flags: 0,
                extension: None,
                nt_extension: None,
            }],
            ..default()
        }],
        ..default()
    };
    let mesh = geometry(&model, crate::model::ModelVariant::N, 0.5).unwrap();
    let Some(bevy::mesh::VertexAttributeValues::Float32x3(points)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("positions")
    };
    assert_eq!(
        points,
        &vec![[0.5, 1.0, 1.5], [1.0, 1.0, 1.5], [0.5, 1.5, 1.5]]
    );
    assert!(geometry(&model, crate::model::ModelVariant::N, 0.0).is_err());
    model.meshes[0].faces[0].vertices[2] = 3;
    assert!(geometry(&model, crate::model::ModelVariant::N, 1.0).is_err());
}

#[test]
fn mounted_x_geometry_uses_loader_coordinates_before_planet_display_scaling() {
    let model = crate::model::Model {
        meshes: vec![crate::model::Mesh {
            vertices: vec![[24., 48., 72.], [36., 48., 72.], [24., 60., 72.]],
            faces: vec![crate::model::Face {
                vertices: [0, 1, 2],
                flags: 0,
                extension: None,
                nt_extension: None,
            }],
            matrix: Some([1., 0., 0., 0., 1., 0., 0., 0., 1., 12., 24., 36.]),
            geometry_order: vec![
                crate::model::GeometryChunk::Vertices,
                crate::model::GeometryChunk::Matrix,
            ],
            ..default()
        }],
        ..default()
    };
    let mesh = geometry(&model, crate::model::ModelVariant::X, 0.5).unwrap();
    let Some(bevy::mesh::VertexAttributeValues::Float32x3(points)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("positions")
    };
    assert_eq!(points, &vec![[1., 2., 3.], [1.5, 2., 3.], [1., 2.5, 3.]]);
}

#[test]
fn recognized_helpers_keep_raw_faces_but_produce_no_drawable_geometry() {
    let mut model = crate::model::Model {
        meshes: vec![crate::model::Mesh {
            name: b"!3 weapon".to_vec(),
            vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
            faces: vec![crate::model::Face {
                vertices: [0, 1, 2],
                flags: 0,
                extension: None,
                nt_extension: None,
            }],
            ..default()
        }],
        ..default()
    };
    let mesh = geometry(&model, crate::model::ModelVariant::Standard, 1.).unwrap();
    let Some(bevy::mesh::VertexAttributeValues::Float32x3(points)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("positions")
    };
    assert!(points.is_empty());
    assert_eq!(model.meshes[0].faces.len(), 1);
    model.meshes[0].name = b"-ordinary geometry".to_vec();
    let mesh = geometry(&model, crate::model::ModelVariant::Standard, 1.).unwrap();
    let Some(bevy::mesh::VertexAttributeValues::Float32x3(points)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("positions")
    };
    assert_eq!(points.len(), 3);
}
