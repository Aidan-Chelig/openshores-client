//! Native model-space body preview or diagnostic marker; no world conversion.
use super::*;
#[derive(Component)]
pub(super) struct NativeAvatarMarker(
    u32,
    Option<std::sync::Arc<[crate::model::NativeBodyMeshGeometry]>>,
);

pub(super) fn scene(
    mut commands: Commands,
    ui: Res<Presentation>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    markers: Query<(Entity, &NativeAvatarMarker)>,
    mut camera: Query<&mut Transform, With<SliceCamera>>,
) {
    let avatar = if ui.model.phase == Phase::NativePreview {
        ui.model.native_avatar.as_deref()
    } else {
        None
    };
    let Some(avatar) = avatar else {
        for (entity, _) in &markers {
            commands.entity(entity).despawn();
        }
        return;
    };
    let geometry = avatar.model_space_geometry();
    if markers.iter().count() != 1
        || markers.iter().any(|(_, marker)| {
            marker.0 != avatar.wire_id
                || match (&marker.1, geometry) {
                    (None, None) => false,
                    (Some(old), Some(new)) => !std::sync::Arc::ptr_eq(old, new),
                    _ => true,
                }
        })
    {
        for (entity, _) in &markers {
            commands.entity(entity).despawn();
        }
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.8, 1.),
            unlit: true,
            cull_mode: None,
            ..default()
        });
        let mut root = commands.spawn((
            NativeAvatarMarker(avatar.wire_id, geometry.cloned()),
            Transform::default(),
            Visibility::default(),
        ));
        if let Some(parts) = geometry {
            root.with_children(|parent| {
                for part in parts.iter().filter(|part| !part.faces.is_empty()) {
                    let positions: Vec<_> = part
                        .positions
                        .iter()
                        .map(|point| point.map(|v| v as f32))
                        .collect();
                    let triangles: Vec<_> = part
                        .faces
                        .iter()
                        .map(|face| face.vertices.map(u32::from))
                        .collect();
                    let mut mesh = crate::viewer::smooth_mesh(&positions, &triangles);
                    if let Some(uvs) = &part.uvs {
                        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs.clone());
                    }
                    parent.spawn((
                        Mesh3d(meshes.add(mesh)),
                        MeshMaterial3d(material.clone()),
                        Transform::default(),
                    ));
                }
            });
        } else {
            // Missing native model generation stays an explicit diagnostic.
            root.insert((
                Mesh3d(meshes.add(Sphere::new(0.25).mesh().uv(12, 8))),
                MeshMaterial3d(material),
            ));
        }
    }
    for mut transform in &mut camera {
        *transform = Transform::from_xyz(2., -3., 1.5).looking_at(Vec3::ZERO, Vec3::Z);
    }
}

#[cfg(test)]
pub(crate) fn assert_native_geometry_preview(
    avatar: crate::client_model::NativeAvatarPresentation,
) {
    let geometry = avatar.model_space_geometry().expect("supplied geometry");
    let mut expected: Vec<_> = geometry
        .iter()
        .filter(|part| !part.faces.is_empty())
        .map(|part| {
            (
                part.positions
                    .iter()
                    .map(|p| p.map(|v| (v as f32).to_bits()))
                    .collect::<Vec<_>>(),
                part.faces
                    .iter()
                    .flat_map(|face| face.vertices.map(u32::from))
                    .collect::<Vec<_>>(),
                part.uvs.as_ref().map(|uvs| {
                    uvs.iter()
                        .map(|uv| uv.map(f32::to_bits))
                        .collect::<Vec<_>>()
                }),
            )
        })
        .collect();
    assert!(!expected.is_empty());
    let mut model = ClientModel::default();
    model.apply(Event::NativeAvatar(Box::new(avatar))).unwrap();
    let mut app = App::new();
    app.init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .insert_resource(Presentation { model, ..default() })
        .add_systems(Update, scene);
    app.update();
    let handles: Vec<_> = app
        .world_mut()
        .query::<&Mesh3d>()
        .iter(app.world())
        .map(|mesh| mesh.0.clone())
        .collect();
    assert_eq!(handles.len(), expected.len());
    let assets = app.world().resource::<Assets<Mesh>>();
    let mut actual: Vec<_> = handles
        .iter()
        .map(|handle| {
            let mesh = assets.get(handle).unwrap();
            let Some(bevy::mesh::VertexAttributeValues::Float32x3(positions)) =
                mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                panic!("position format");
            };
            let uvs = match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
                Some(bevy::mesh::VertexAttributeValues::Float32x2(uvs)) => Some(
                    uvs.iter()
                        .map(|uv| uv.map(f32::to_bits))
                        .collect::<Vec<_>>(),
                ),
                None => None,
                _ => panic!("UV format"),
            };
            (
                positions
                    .iter()
                    .map(|p| p.map(f32::to_bits))
                    .collect::<Vec<_>>(),
                mesh.indices()
                    .unwrap()
                    .iter()
                    .map(|i| u32::try_from(i).unwrap())
                    .collect::<Vec<_>>(),
                uvs,
            )
        })
        .collect();
    actual.sort_unstable();
    expected.sort_unstable();
    assert_eq!(actual, expected);
    app.world_mut()
        .resource_mut::<Presentation>()
        .model
        .apply(Event::Disconnected)
        .unwrap();
    app.update();
    assert_eq!(
        app.world_mut().query::<&Mesh3d>().iter(app.world()).count(),
        0
    );
}

#[cfg(test)]
pub(crate) fn assert_native_preview(avatar: crate::client_model::NativeAvatarPresentation) {
    let id = avatar.wire_id;
    let summary = avatar.summary();
    let mut model = ClientModel::default();
    model.apply(Event::NativeAvatar(Box::new(avatar))).unwrap();
    let mut app = App::new();
    app.init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .insert_resource(Presentation {
            model,
            dirty: true,
            ..default()
        })
        .add_systems(Update, (scene, panels::rebuild_panels).chain());
    app.update();
    assert_eq!(
        app.world_mut()
            .query::<&NativeAvatarMarker>()
            .single(app.world())
            .unwrap()
            .0,
        id
    );
    assert!(
        app.world_mut()
            .query::<&Text>()
            .iter(app.world())
            .any(|text| text.0 == summary)
    );
    // Same wire ID must replace the marker when actual geometry arrives.
    let avatar = app
        .world()
        .resource::<Presentation>()
        .model
        .native_avatar
        .as_ref()
        .unwrap()
        .as_ref()
        .clone();
    let geometry = crate::model::NativeBodyMeshGeometry {
        positions: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        weights: vec![crate::model::NativeBodyVertexWeights::default(); 3],
        faces: vec![crate::model::Face {
            vertices: [0, 1, 2],
            flags: 0,
            extension: None,
            nt_extension: None,
        }],
        uvs: Some(vec![[0., 0.], [1., 0.], [0., 1.]]),
    };
    let avatar = avatar.with_model_space_geometry(vec![geometry]).unwrap();
    app.world_mut()
        .resource_mut::<Presentation>()
        .model
        .apply(Event::NativeAvatar(Box::new(avatar.clone())))
        .unwrap();
    app.update();
    let mesh_handle = app
        .world_mut()
        .query::<&Mesh3d>()
        .single(app.world())
        .unwrap()
        .0
        .clone();
    let mesh = app
        .world()
        .resource::<Assets<Mesh>>()
        .get(&mesh_handle)
        .unwrap();
    assert_eq!(mesh.count_vertices(), 3);
    assert_eq!(mesh.indices().unwrap().len(), 3);
    assert!(mesh.attribute(Mesh::ATTRIBUTE_UV_0).is_some());
    // Successfully empty geometry renders nothing; never invent a sphere.
    let avatar = avatar.with_model_space_geometry(Vec::new()).unwrap();
    app.world_mut()
        .resource_mut::<Presentation>()
        .model
        .apply(Event::NativeAvatar(Box::new(avatar)))
        .unwrap();
    app.update();
    assert_eq!(
        app.world_mut().query::<&Mesh3d>().iter(app.world()).count(),
        0
    );
    app.world_mut()
        .resource_mut::<Presentation>()
        .model
        .apply(Event::Disconnected)
        .unwrap();
    app.update();
    assert_eq!(
        app.world_mut()
            .query::<&NativeAvatarMarker>()
            .iter(app.world())
            .count(),
        0
    );
}
