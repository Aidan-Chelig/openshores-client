//! Diagnostic native avatar marker; no source-world or anatomical conversion.
use super::*;
#[derive(Component)]
pub(super) struct NativeAvatarMarker(u32);

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
    if markers.iter().count() != 1 || markers.iter().any(|(_, marker)| marker.0 != avatar.wire_id) {
        for (entity, _) in &markers {
            commands.entity(entity).despawn();
        }
        // A dimensionless diagnostic marker, not a guessed biological model.
        commands.spawn((
            NativeAvatarMarker(avatar.wire_id),
            Mesh3d(meshes.add(Sphere::new(0.25).mesh().uv(12, 8))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.2, 0.8, 1.),
                unlit: true,
                ..default()
            })),
            Transform::default(),
        ));
    }
    for mut transform in &mut camera {
        *transform = Transform::from_xyz(2., -3., 1.5).looking_at(Vec3::ZERO, Vec3::Z);
    }
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
