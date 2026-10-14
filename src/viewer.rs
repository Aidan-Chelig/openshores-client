//! Optional offline geometry inspector. Controls/shading are debug design choices.
use crate::{model::Model, model_preview};
use bevy::{
    asset::RenderAssetUsages,
    core_pipeline::tonemapping::Tonemapping,
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit},
    math::DVec3,
    prelude::*,
    render::render_resource::PrimitiveTopology,
};
use std::io;

#[derive(Resource)]
struct DisplayGeometry {
    meshes: Vec<DisplayMesh>,
}

struct DisplayMesh {
    positions: Vec<[f32; 3]>,
    triangles: Vec<[u16; 3]>,
}

#[derive(Resource)]
struct Orbit {
    yaw: f32,
    pitch: f32,
    distance: f32,
    wire: bool,
    lit: bool,
}
impl Default for Orbit {
    fn default() -> Self {
        Self {
            yaw: 0.75,
            pitch: 0.35,
            distance: 4.5,
            wire: false,
            lit: true,
        }
    }
}

#[derive(Component)]
struct InspectorCamera;

#[derive(Resource)]
struct SmokeFrames(u32);

#[derive(Resource)]
struct PreviewMaterial(Handle<StandardMaterial>);

fn geometry(model: &Model) -> io::Result<DisplayGeometry> {
    let bounds = model_preview::bounds(model)?.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "cannot view a model without vertices",
        )
    })?;
    if !model.meshes.iter().any(|mesh| !mesh.faces.is_empty()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "cannot view a model without triangles",
        ));
    }
    let center: [f64; 3] =
        std::array::from_fn(|axis| (bounds.minimum[axis] + bounds.maximum[axis]) * 0.5);
    let span = (0..3)
        .map(|axis| bounds.maximum[axis] - bounds.minimum[axis])
        .fold(0.0f64, f64::max);
    let scale = if span == 0.0 { 1.0 } else { 2.0 / span };
    let meshes = model
        .meshes
        .iter()
        .map(|mesh| {
            let positions = mesh
                .vertices
                .iter()
                .map(|vertex| {
                    std::array::from_fn(|axis| {
                        ((f64::from(vertex[axis]) - center[axis]) * scale) as f32
                    })
                })
                .collect();
            DisplayMesh {
                positions,
                triangles: mesh.faces.iter().map(|face| face.vertices).collect(),
            }
        })
        .collect();
    Ok(DisplayGeometry { meshes })
}

/// Launches only this Rust inspector; it does not execute the original client.
pub fn run(model: &Model, smoke_frames: Option<u32>, unlit: bool) -> io::Result<()> {
    if smoke_frames.is_some_and(|frames| !(1..=3600).contains(&frames)) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "smoke frame limit must be 1..=3600",
        ));
    }
    let geometry = geometry(model)?;
    let mut app = App::new();
    app
        .insert_resource(geometry)
        .insert_resource(Orbit { lit: !unlit, ..Default::default() })
        .insert_resource(ClearColor(Color::srgb(0.035, 0.055, 0.075)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "OpenShores geometry preview | Drag: orbit | Wheel: zoom | L: lighting | W: wire | R: camera reset | Esc: quit".into(),
                resolution: (1100, 760).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (controls, lighting_mode, camera, wireframe).chain());
    if let Some(frames) = smoke_frames {
        app.insert_resource(SmokeFrames(frames))
            .add_systems(Update, smoke_exit);
    }
    match app.run() {
        AppExit::Success => Ok(()),
        AppExit::Error(code) => Err(io::Error::other(format!("viewer exited with code {code}"))),
    }
}

fn setup(
    mut commands: Commands,
    geometry: Res<DisplayGeometry>,
    orbit: Res<Orbit>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.28, 0.4, 0.48),
        unlit: !orbit.lit,
        perceptual_roughness: 0.55,
        metallic: 0.0,
        double_sided: true,
        cull_mode: None,
        ..default()
    });
    for DisplayMesh {
        positions,
        triangles,
    } in &geometry.meshes
    {
        if triangles.is_empty() {
            continue;
        }
        let mesh = flat_mesh(positions, triangles);
        commands.spawn((Mesh3d(meshes.add(mesh)), MeshMaterial3d(material.clone())));
    }
    commands.insert_resource(PreviewMaterial(material));
    commands.insert_resource(GlobalAmbientLight {
        brightness: 150.0,
        ..default()
    });
    commands.spawn((
        DirectionalLight {
            illuminance: 8000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(3.0, -4.0, 5.0).looking_at(Vec3::ZERO, Vec3::Z),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 2500.0,
            color: Color::srgb(0.7, 0.8, 1.0),
            ..default()
        },
        Transform::from_xyz(-4.0, 2.0, 2.0).looking_at(Vec3::ZERO, Vec3::Z),
    ));
    commands.spawn((
        Camera3d::default(),
        if orbit.lit {
            Tonemapping::Reinhard
        } else {
            Tonemapping::None
        },
        InspectorCamera,
        Transform::default(),
    ));
}

/// Preview-generated per-triangle normals, not interpreted source extensions.
fn flat_mesh(positions: &[[f32; 3]], triangles: &[[u16; 3]]) -> Mesh {
    let mut expanded = Vec::with_capacity(triangles.len() * 3);
    let mut normals = Vec::with_capacity(triangles.len() * 3);
    for triangle in triangles {
        let points = triangle.map(|index| positions[index as usize]);
        let [a, b, c] = points.map(|point| DVec3::from_array(point.map(f64::from)));
        let normal = (b - a)
            .cross(c - a)
            .try_normalize()
            .map(|normal| normal.as_vec3())
            .unwrap_or(Vec3::Z)
            .to_array();
        expanded.extend(points);
        normals.extend([normal; 3]);
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, expanded)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
}

fn lighting_mode(
    orbit: Res<Orbit>,
    handle: Res<PreviewMaterial>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cameras: Query<&mut Tonemapping, With<InspectorCamera>>,
) {
    if materials
        .get(&handle.0)
        .is_some_and(|material| material.unlit == orbit.lit)
        && let Some(mut material) = materials.get_mut(&handle.0)
    {
        material.unlit = !orbit.lit;
    }
    let desired = if orbit.lit {
        Tonemapping::Reinhard
    } else {
        Tonemapping::None
    };
    for mut tonemapping in &mut cameras {
        if *tonemapping != desired {
            *tonemapping = desired;
        }
    }
}

fn controls(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    mut orbit: ResMut<Orbit>,
    mut exit: MessageWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
    if keys.just_pressed(KeyCode::KeyR) {
        let reset = Orbit::default();
        orbit.yaw = reset.yaw;
        orbit.pitch = reset.pitch;
        orbit.distance = reset.distance;
    }
    if keys.just_pressed(KeyCode::KeyL) {
        orbit.lit = !orbit.lit;
    }
    if keys.just_pressed(KeyCode::KeyW) {
        orbit.wire = !orbit.wire;
    }
    if mouse.pressed(MouseButton::Left) {
        orbit.yaw -= motion.delta.x * 0.005;
        orbit.pitch = (orbit.pitch + motion.delta.y * 0.005).clamp(-1.5, 1.5);
    }
    let wheel = match scroll.unit {
        MouseScrollUnit::Line => scroll.delta.y,
        MouseScrollUnit::Pixel => scroll.delta.y / 40.0,
    };
    orbit.distance = (orbit.distance * (-wheel * 0.12).exp()).clamp(1.8, 30.0);
}

fn smoke_exit(mut frames: ResMut<SmokeFrames>, mut exit: MessageWriter<AppExit>) {
    frames.0 = frames.0.saturating_sub(1);
    if frames.0 == 0 {
        exit.write(AppExit::Success);
    }
}

fn camera(orbit: Res<Orbit>, mut cameras: Query<&mut Transform, With<InspectorCamera>>) {
    let position = Vec3::new(
        orbit.yaw.cos() * orbit.pitch.cos(),
        orbit.yaw.sin() * orbit.pitch.cos(),
        orbit.pitch.sin(),
    ) * orbit.distance;
    for mut transform in &mut cameras {
        *transform = Transform::from_translation(position).looking_at(Vec3::ZERO, Vec3::Z);
    }
}

fn wireframe(geometry: Res<DisplayGeometry>, orbit: Res<Orbit>, mut gizmos: Gizmos) {
    if !orbit.wire {
        return;
    }
    for DisplayMesh {
        positions,
        triangles,
    } in &geometry.meshes
    {
        for triangle in triangles {
            let [a, b, c] = triangle.map(|index| Vec3::from_array(positions[index as usize]));
            gizmos.linestrip([a, b, c, a], Color::srgb(0.5, 0.8, 0.95));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Face, Mesh as SourceMesh};
    #[test]
    fn normalization_is_display_only_and_preserves_indices() {
        let model = Model {
            meshes: vec![SourceMesh {
                vertices: vec![
                    [-f32::MAX, 0.0, 0.0],
                    [f32::MAX, 0.0, 0.0],
                    [0.0, f32::MAX, 0.0],
                ],
                faces: vec![Face {
                    vertices: [0, 1, 2],
                    flags: 0,
                    extension: None,
                }],
                ..Default::default()
            }],
            ..Default::default()
        };
        let result = geometry(&model).unwrap();
        assert_eq!(result.meshes[0].triangles, vec![[0, 1, 2]]);
        assert_eq!(result.meshes[0].positions[0][0], -1.0);
        assert_eq!(result.meshes[0].positions[1][0], 1.0);
        assert_eq!(model.meshes[0].vertices[0][0], -f32::MAX);
        assert!(
            result.meshes[0]
                .positions
                .iter()
                .flatten()
                .all(|value| value.is_finite())
        );
        assert!(geometry(&Model::default()).is_err());
    }

    #[test]
    fn setup_and_camera_work_without_a_window_or_gpu() {
        let mut app = App::new();
        app.insert_resource(DisplayGeometry {
            meshes: vec![DisplayMesh {
                positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                triangles: vec![[0, 1, 2]],
            }],
        })
        .init_resource::<Orbit>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .add_systems(Startup, setup)
        .add_systems(Update, camera);
        app.update();
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(), 1);
        assert_eq!(app.world().resource::<Assets<StandardMaterial>>().len(), 1);
        let mut cameras = app
            .world_mut()
            .query_filtered::<&Transform, With<InspectorCamera>>();
        let transform = cameras.single(app.world()).unwrap();
        assert!((transform.translation.length() - 4.5).abs() < 1e-5);
        assert!(transform.rotation.is_finite());
        assert!(run(&Model::default(), Some(0), false).is_err());
    }

    #[test]
    fn generated_flat_normals_follow_winding_and_handle_degeneracy() {
        use bevy::mesh::VertexAttributeValues;
        let points = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let mesh = flat_mesh(&points, &[[0, 1, 2], [0, 2, 1], [0, 0, 0]]);
        let Some(VertexAttributeValues::Float32x3(normals)) =
            mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
        else {
            panic!("missing normals");
        };
        assert_eq!(&normals[..3], &[[0.0, 0.0, 1.0]; 3]);
        assert_eq!(&normals[3..6], &[[0.0, 0.0, -1.0]; 3]);
        assert_eq!(&normals[6..], &[[0.0, 0.0, 1.0]; 3]);
        assert!(mesh.indices().is_none());
        let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("missing positions");
        };
        assert_eq!(&positions[..3], &points);
        assert_eq!(&positions[3..6], &[points[0], points[2], points[1]]);
    }

    #[test]
    fn lighting_modes_change_materials_without_a_gpu() {
        let mut app = App::new();
        app.insert_resource(DisplayGeometry {
            meshes: vec![DisplayMesh {
                positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                triangles: vec![[0, 1, 2]],
            }],
        })
        .init_resource::<Orbit>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .add_systems(Startup, setup)
        .add_systems(Update, lighting_mode);
        app.update();
        let handle = app.world().resource::<PreviewMaterial>().0.clone();
        assert!(
            !app.world()
                .resource::<Assets<StandardMaterial>>()
                .get(&handle)
                .unwrap()
                .unlit
        );
        app.world_mut().resource_mut::<Orbit>().lit = false;
        app.update();
        assert!(
            app.world()
                .resource::<Assets<StandardMaterial>>()
                .get(&handle)
                .unwrap()
                .unlit
        );
        let mut lights = app.world_mut().query::<&DirectionalLight>();
        assert_eq!(lights.iter(app.world()).count(), 2);
    }
}
