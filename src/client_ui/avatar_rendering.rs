//! Persistent local-player body. Placement remains the explicit source transform;
//! body-origin and globe-frame acceptance are tracked separately in the goal.
use super::*;
use crate::{
    client_model::AvatarRequest,
    viewer::native_skin::{SkinExtension, SkinMaterial},
};
use bevy::{ecs::system::SystemParam, light::NotShadowCaster};

#[derive(Resource, Default)]
pub(super) struct State {
    owner: Option<AvatarRequest>,
    root: Option<Entity>,
    meshes: Vec<Handle<Mesh>>,
    material: Option<Handle<SkinMaterial>>,
    image: Option<Handle<Image>>,
}

#[derive(Component)]
pub(super) struct LocalAvatar;

#[derive(SystemParam)]
pub(super) struct RenderAssets<'w> {
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<SkinMaterial>>,
    images: ResMut<'w, Assets<Image>>,
}

impl State {
    fn clear(&mut self, commands: &mut Commands, assets: &mut RenderAssets) {
        if let Some(root) = self.root.take()
            && let Ok(mut entity) = commands.get_entity(root)
        {
            entity.despawn();
        }
        for mesh in self.meshes.drain(..) {
            assets.meshes.remove(mesh.id());
        }
        if let Some(material) = self.material.take() {
            assets.materials.remove(material.id());
        }
        if let Some(image) = self.image.take() {
            assets.images.remove(image.id());
        }
        self.owner = None;
    }
}

pub(super) fn render(
    mut commands: Commands,
    mut ui: ResMut<Presentation>,
    prepared: Res<avatar::Preparation>,
    mut state: ResMut<State>,
    mut assets: RenderAssets,
    mut roots: Query<&mut Transform, With<LocalAvatar>>,
    markers: Query<(Entity, &Marker)>,
) {
    let ready = {
        let preparation = prepared.0.lock().unwrap();
        preparation
            .ready()
            .map(|(request, body)| (request.clone(), body.clone()))
    };
    let placement = ready.as_ref().and_then(|(request, _)| {
        if ui.model.phase != Phase::Playing
            || ui.model.avatar != Some(request.entity_id)
            || ui.model.session_generation != request.session_generation
            || !ui.model.world.visible_ids.contains(&request.entity_id)
        {
            return None;
        }
        let entity = ui.model.world.entities.get(&request.entity_id)?;
        if entity.parent != ui.model.home_globe() || entity.parent.is_none() {
            return None;
        }
        let source = entity.transform?;
        let position = ui.local_position.unwrap_or(source.position.map(f64::from));
        let rotation = ui.model.rotation()?.map(|v| v as f32);
        placement::local_model(position, rotation, ui.scale)
            .map(|transform| transform.with_scale(Vec3::splat(ui.scale as f32)))
    });
    let status = if ready.is_some() && placement.is_none() {
        "Avatar placement unavailable: waiting for a valid visible globe-child transform."
    } else if placement.is_some() {
        "Avatar attached at received origin; ground anchor and globe-facing verification pending."
    } else {
        ""
    };
    if ui.avatar_placement_status != status {
        ui.avatar_placement_status = status.into();
        ui.dirty = true;
    }
    let (Some((request, body)), Some(transform)) = (ready, placement) else {
        state.clear(&mut commands, &mut assets);
        return;
    };
    if state.owner.as_ref() != Some(&request) || state.root.is_none_or(|root| !roots.contains(root))
    {
        state.clear(&mut commands, &mut assets);
        let image = assets.images.add(body.image.clone());
        let material = assets.materials.add(SkinMaterial {
            base: StandardMaterial {
                alpha_mode: AlphaMode::Mask(0.15),
                double_sided: true,
                cull_mode: None,
                perceptual_roughness: 0.55,
                ..default()
            },
            extension: SkinExtension {
                atlas: image.clone(),
                offsets: Vec4::from_array(body.skin_offsets.map(|v| v as f32)),
            },
        });
        let meshes: Vec<_> = body
            .meshes
            .iter()
            .map(|mesh| assets.meshes.add(mesh.clone()))
            .collect();
        let root = commands
            .spawn((LocalAvatar, transform))
            .with_children(|parent| {
                for mesh in &meshes {
                    parent.spawn((
                        Mesh3d(mesh.clone()),
                        MeshMaterial3d(material.clone()),
                        NotShadowCaster,
                    ));
                }
            })
            .id();
        state.owner = Some(request.clone());
        state.root = Some(root);
        state.meshes = meshes;
        state.image = Some(image);
        state.material = Some(material);
    } else if let Some(root) = state.root
        && let Ok(mut current) = roots.get_mut(root)
    {
        *current = transform;
    }
    // Scene diagnostics may be recreated on network revisions; never leave a
    // second marker over the successfully attached local body.
    for (entity, marker) in &markers {
        if marker.0 == request.entity_id {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entity(id: u32, kind: AtomKind, parent: Option<u32>) -> crate::world::Entity {
        crate::world::Entity {
            id,
            kind,
            parent,
            timestamp_ms: 0,
            created_ms: None,
            modified_ms: None,
            transform: Some(crate::world::Transform {
                position: [0., 0., 100.],
                rotation: [0.; 3],
            }),
            name: None,
            world_identity: None,
            globe_surface: None,
            geology: None,
            person: None,
            item: None,
            last_body: vec![],
            last_base_flags: 0,
        }
    }

    #[test]
    fn movement_reuses_root_and_assets_then_removal_cleans_up() {
        use crate::client_model::AvatarContentKey;
        let mut app = App::new();
        let mut model = ClientModel {
            phase: Phase::Playing,
            avatar: Some(42),
            session_generation: 1,
            ..default()
        };
        let mut globe = entity(1, AtomKind::Globe, None);
        globe.globe_surface = Some(crate::terrain::GlobeSurface {
            size: 1,
            atmosphere_type: 0,
            atmosphere_density: 0,
            water: 0,
            terrain: crate::terrain::TerrainParameters::new([0., 1., 0., 0., 0., 0.]).unwrap(),
        });
        model.world.entities.insert(1, globe);
        model
            .world
            .entities
            .insert(42, entity(42, AtomKind::Person, Some(1)));
        model.world.visible_ids.insert(42);
        app.insert_resource(Presentation {
            model,
            scale: 0.01,
            ..default()
        })
        .init_resource::<avatar::Preparation>()
        .init_resource::<State>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<SkinMaterial>>()
        .init_resource::<Assets<Image>>()
        .add_systems(Update, render);
        let request = AvatarRequest {
            session_generation: 1,
            entity_id: 42,
            content: AvatarContentKey::new(
                crate::world::native_dna::NativeDhDna::from_received_bytes(&[0; 24]).unwrap(),
                1,
                60.,
                true,
            )
            .unwrap(),
        };
        {
            let mut preparation = app
                .world()
                .resource::<avatar::Preparation>()
                .0
                .lock()
                .unwrap();
            preparation.select(Some(request));
            preparation.start_next(|_| {
                let (tx, rx) = std::sync::mpsc::channel();
                tx.send(Ok(avatar::PreparedRenderAvatar {
                    surface_count: 1,
                    skin_offsets: [0.; 4],
                    image: Image::default(),
                    meshes: vec![Sphere::new(1.).mesh().uv(8, 4)],
                }))
                .unwrap_or_else(|_| panic!("receiver disappeared"));
                rx
            });
            preparation.poll();
        }
        let marker = app.world_mut().spawn(Marker(42)).id();
        app.update();
        assert!(app.world().get_entity(marker).is_err());
        let root = app.world().resource::<State>().root.unwrap();
        let mesh = app.world().resource::<State>().meshes[0].id();
        assert_eq!(
            app.world().get::<Transform>(root).unwrap().translation,
            Vec3::Z
        );
        {
            let mut ui = app.world_mut().resource_mut::<Presentation>();
            ui.local_position = Some([20., 30., -100.]);
            ui.model.revision += 1;
        }
        app.update();
        assert_eq!(app.world().resource::<State>().root, Some(root));
        assert_eq!(app.world().resource::<State>().meshes[0].id(), mesh);
        let transform = app.world().get::<Transform>(root).unwrap();
        assert!((transform.translation - Vec3::new(0.2, 0.3, -1.)).length() < 1e-6);
        assert_eq!(transform.scale, Vec3::splat(0.01));
        app.world_mut()
            .resource_mut::<Presentation>()
            .model
            .world
            .visible_ids
            .clear();
        app.update();
        assert!(app.world().get_entity(root).is_err());
        assert!(!app.world().resource::<Assets<Mesh>>().contains(mesh));
    }

    #[test]
    fn inactive_renderer_releases_owned_root_children_and_gpu_assets() {
        let mut app = App::new();
        app.init_resource::<Presentation>()
            .init_resource::<avatar::Preparation>()
            .init_resource::<State>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<SkinMaterial>>()
            .init_resource::<Assets<Image>>()
            .add_systems(Update, render);
        let image = app
            .world_mut()
            .resource_mut::<Assets<Image>>()
            .add(Image::default());
        let material = app
            .world_mut()
            .resource_mut::<Assets<SkinMaterial>>()
            .add(SkinMaterial {
                base: StandardMaterial::default(),
                extension: SkinExtension {
                    atlas: image.clone(),
                    offsets: Vec4::ZERO,
                },
            });
        let mesh = app
            .world_mut()
            .resource_mut::<Assets<Mesh>>()
            .add(Sphere::new(1.).mesh().uv(8, 4));
        let child = app
            .world_mut()
            .spawn((Mesh3d(mesh.clone()), MeshMaterial3d(material.clone())))
            .id();
        let root = app
            .world_mut()
            .spawn((LocalAvatar, Transform::default()))
            .add_child(child)
            .id();
        let unrelated = app.world_mut().spawn(Marker(99)).id();
        {
            let mut state = app.world_mut().resource_mut::<State>();
            state.root = Some(root);
            state.meshes.push(mesh.clone());
            state.material = Some(material.clone());
            state.image = Some(image.clone());
        }
        app.update();
        assert!(app.world().get_entity(root).is_err());
        assert!(app.world().get_entity(child).is_err());
        assert!(app.world().get_entity(unrelated).is_ok());
        assert!(!app.world().resource::<Assets<Mesh>>().contains(mesh.id()));
        assert!(
            !app.world()
                .resource::<Assets<SkinMaterial>>()
                .contains(material.id())
        );
        assert!(!app.world().resource::<Assets<Image>>().contains(image.id()));
        assert!(app.world().resource::<State>().root.is_none());
        app.update(); // repeated cleanup is harmless
    }
}
