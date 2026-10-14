use super::*;

pub(super) fn scene(
    mut commands: Commands,
    mut ui: ResMut<Presentation>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    terrain_entities: Query<Entity, With<TerrainEntity>>,
    mut markers: Query<(Entity, &Marker, &mut Transform), Without<SliceCamera>>,
    mut camera: Query<&mut Transform, (With<SliceCamera>, Without<Marker>)>,
) {
    if ui.model.phase != Phase::Playing {
        for entity in &terrain_entities {
            commands.entity(entity).despawn();
        }
        for (entity, _, _) in &markers {
            commands.entity(entity).despawn();
        }
        ui.terrain = None;
        ui.local_position = None;
        return;
    }
    let Some(home) = ui.model.home_globe() else {
        return;
    };
    if ui.terrain != Some(home) {
        for entity in &terrain_entities {
            commands.entity(entity).despawn();
        }
        let surface = ui.model.world.entities[&home].globe_surface.unwrap();
        match TerrainMesh::generate(surface, 32) {
            Ok(terrain) => {
                ui.scale = 3. / terrain.statistics().base_radius;
                let positions: Vec<_> = terrain
                    .positions()
                    .iter()
                    .map(|point| point.map(|value| (value * ui.scale) as f32))
                    .collect();
                let mut mesh = crate::viewer::smooth_mesh(&positions, terrain.triangles());
                mesh.insert_attribute(
                    Mesh::ATTRIBUTE_COLOR,
                    crate::viewer::elevation_colors(terrain.positions()),
                );
                let material = materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    perceptual_roughness: 0.8,
                    ..default()
                });
                commands.spawn((
                    TerrainEntity,
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(material),
                ));
                ui.terrain = Some(home);
            }
            Err(error) => {
                ui.model.status = error.to_string();
                ui.dirty = true;
                return;
            }
        }
    }
    if ui.last_scene_revision != ui.model.revision {
        ui.last_scene_revision = ui.model.revision;
        for (entity, _, _) in &markers {
            commands.entity(entity).despawn();
        }
        for entity in ui
            .model
            .world
            .entities
            .values()
            .filter(|entity| {
                matches!(entity.kind, AtomKind::Person | AtomKind::Item)
                    && ui.model.world.visible_ids.contains(&entity.id)
                    && entity.parent == Some(home)
            })
            .take(256)
        {
            if let Some(transform) = entity.transform {
                let position = if Some(entity.id) == ui.model.avatar {
                    ui.local_position
                        .unwrap_or(transform.position.map(f64::from))
                } else {
                    transform.position.map(f64::from)
                };
                let radius = if entity.kind == AtomKind::Person {
                    0.003
                } else {
                    0.002
                };
                let mesh = meshes.add(Sphere::new(radius).mesh().uv(12, 8));
                let material = materials.add(StandardMaterial {
                    base_color: if entity.kind == AtomKind::Person {
                        Color::srgb(0.2, 0.8, 1.)
                    } else {
                        Color::srgb(1., 0.7, 0.2)
                    },
                    unlit: true,
                    ..default()
                });
                commands.spawn((
                    Marker(entity.id),
                    Mesh3d(mesh),
                    MeshMaterial3d(material),
                    Transform::from_translation(Vec3::from_array(
                        position.map(|value| (value * ui.scale) as f32),
                    )),
                ));
            }
        }
    }
    if let Some(position) = ui.local_position {
        let point = Vec3::from_array(position.map(|value| (value * ui.scale) as f32));
        for (_, marker, mut transform) in &mut markers {
            if Some(marker.0) == ui.model.avatar {
                transform.translation = point;
            }
        }
        let up = point.normalize_or_zero();
        let up = if up.length_squared() > 0. {
            up
        } else {
            Vec3::Z
        };
        let basis = if up.dot(Vec3::Z).abs() > 0.95 {
            Vec3::Y
        } else {
            Vec3::Z
        };
        let right = basis.cross(up).normalize();
        let north = up.cross(right);
        let forward = north * ui.yaw.cos() + right * ui.yaw.sin();
        let look = forward * ui.pitch.cos() + up * ui.pitch.sin();
        if let Ok(mut transform) = camera.single_mut() {
            *transform = if ui.top_down {
                // Synthetic server fixtures can spawn inside base terrain.
                // Keep the tactical camera outside the generated globe.
                Transform::from_translation(
                    point + up * (3.5 - point.length()).max(ui.top_height.max(0.1)) - forward * 0.5,
                )
                .looking_at(point, forward)
            } else {
                Transform::from_translation(point + up * 0.002)
                    .looking_at(point + up * 0.002 + look, up)
            };
        }
    }
}
