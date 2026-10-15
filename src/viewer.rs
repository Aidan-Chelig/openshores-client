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
mod auto_smooth;
mod native_skin;
pub use native_skin::{NativeSkinPreviewOptions, run_textured_body};

#[derive(Resource)]
struct DisplayGeometry {
    crease_degrees: Option<f64>,
    meshes: Vec<DisplayMesh>,
    smooth: bool,
    terrain_probe: Option<TerrainProbe>,
}

struct TerrainProbe {
    center: DVec3,
    scale: f64,
    base_radius: f64,
}

#[derive(Resource, Default)]
struct ProbeSelection(Option<Vec3>);

struct DisplayMesh {
    positions: Vec<[f32; 3]>,
    triangles: Vec<[u32; 3]>,
    colors: Option<Vec<[f32; 4]>>,
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
                colors: None,
                positions,
                triangles: mesh
                    .faces
                    .iter()
                    .map(|face| face.vertices.map(u32::from))
                    .collect(),
            }
        })
        .collect();
    Ok(DisplayGeometry {
        crease_degrees: None,
        terrain_probe: None,
        meshes,
        smooth: false,
    })
}

/// Launches only this Rust inspector; it does not execute the original client.
pub fn run(model: &Model, smoke_frames: Option<u32>, unlit: bool) -> io::Result<()> {
    launch(geometry(model)?, smoke_frames, unlit)
}

/// Diagnostic normals only; source topology and native normal records are unchanged.
pub fn run_auto_smooth(
    model: &Model,
    smoke_frames: Option<u32>,
    unlit: bool,
    angle_degrees: f64,
) -> io::Result<()> {
    if !angle_degrees.is_finite() || !(0.0..=180.0).contains(&angle_degrees) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "smoothing angle must be finite and in 0..=180 degrees",
        ));
    }
    let mut display = geometry(model)?;
    display.crease_degrees = Some(angle_degrees);
    launch(display, smoke_frames, unlit)
}

fn terrain_geometry(mesh: &crate::terrain_mesh::TerrainMesh) -> DisplayGeometry {
    let mut minimum = [f64::INFINITY; 3];
    let mut maximum = [f64::NEG_INFINITY; 3];
    for point in mesh.positions() {
        for axis in 0..3 {
            minimum[axis] = minimum[axis].min(point[axis]);
            maximum[axis] = maximum[axis].max(point[axis]);
        }
    }
    let center: [f64; 3] = std::array::from_fn(|axis| (minimum[axis] + maximum[axis]) * 0.5);
    let span = (0..3)
        .map(|axis| maximum[axis] - minimum[axis])
        .fold(0.0f64, f64::max);
    DisplayGeometry {
        terrain_probe: Some(TerrainProbe {
            center: DVec3::from_array(center),
            scale: 2.0 / span,
            base_radius: mesh.statistics().base_radius,
        }),
        crease_degrees: None,
        smooth: true,
        meshes: vec![DisplayMesh {
            colors: Some(elevation_colors(mesh.positions())),
            positions: mesh
                .positions()
                .iter()
                .map(|point| {
                    std::array::from_fn(|axis| ((point[axis] - center[axis]) * (2.0 / span)) as f32)
                })
                .collect(),
            triangles: mesh.triangles().to_vec(),
        }],
    }
}

/// Relative radial elevation ramp in linear RGB. No sea level, biome, water
/// geometry, source textures or source material rules are inferred.
pub(crate) fn elevation_colors(positions: &[[f64; 3]]) -> Vec<[f32; 4]> {
    let radii: Vec<f64> = positions
        .iter()
        .map(|p| p[0].hypot(p[1]).hypot(p[2]))
        .collect();
    let low = radii.iter().copied().fold(f64::INFINITY, f64::min);
    let high = radii.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    // Flat surfaces get the middle color instead of amplifying roundoff.
    let span = high - low;
    let flat = span <= high.abs().max(1.0) * 1e-12;
    let palette = [
        [0.025f32, 0.09, 0.16],
        [0.08, 0.24, 0.13],
        [0.35, 0.25, 0.12],
        [0.8, 0.85, 0.9],
    ];
    radii
        .into_iter()
        .map(|radius| {
            let t = if flat {
                0.5
            } else {
                ((radius - low) / span).clamp(0.0, 1.0)
            } * 3.0;
            let index = (t.floor() as usize).min(2);
            let fraction = (t - index as f64) as f32;
            let rgb: [f32; 3] = std::array::from_fn(|axis| {
                palette[index][axis] * (1.0 - fraction) + palette[index + 1][axis] * fraction
            });
            [rgb[0], rgb[1], rgb[2], 1.0]
        })
        .collect()
}

/// View generated base terrain without loading assets or opening a connection.
pub fn run_terrain(
    mesh: &crate::terrain_mesh::TerrainMesh,
    smoke_frames: Option<u32>,
    unlit: bool,
) -> io::Result<()> {
    launch(terrain_geometry(mesh), smoke_frames, unlit)
}

fn launch(geometry: DisplayGeometry, smoke_frames: Option<u32>, unlit: bool) -> io::Result<()> {
    launch_with_skin(geometry, smoke_frames, unlit, None)
}

fn launch_with_skin(
    geometry: DisplayGeometry,
    smoke_frames: Option<u32>,
    unlit: bool,
    skin: Option<native_skin::SkinPreview>,
) -> io::Result<()> {
    if smoke_frames.is_some_and(|frames| !(1..=3600).contains(&frames)) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "smoke frame limit must be 1..=3600",
        ));
    }
    let mut app = App::new();
    app
        .insert_resource(geometry)
        .init_resource::<ProbeSelection>()
        .insert_resource(Orbit { lit: !unlit, ..Default::default() })
        .insert_resource(ClearColor(Color::srgb(0.035, 0.055, 0.075)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "OpenShores geometry preview | Drag: orbit | Wheel: zoom | Right-click: terrain probe | L: lighting | W: wire | R: camera reset | Esc: quit".into(),
                resolution: (1100, 760).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(native_skin::SkinPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (controls, lighting_mode, camera, wireframe).chain())
        .add_systems(PostUpdate, (probe, probe_marker).chain().after(bevy::transform::TransformSystems::Propagate));
    if let Some(skin) = skin {
        app.insert_resource(skin);
    }
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
    skin: Option<Res<native_skin::SkinPreview>>,
) {
    let material = materials.add(StandardMaterial {
        base_color: if geometry.smooth {
            Color::WHITE
        } else {
            Color::srgb(0.28, 0.4, 0.48)
        },
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
        colors,
    } in &geometry.meshes
    {
        if skin.is_some() || triangles.is_empty() {
            continue;
        }
        let mut mesh = if let Some(angle) = geometry.crease_degrees {
            auto_smooth::mesh(positions, triangles, angle)
        } else if geometry.smooth {
            smooth_mesh(positions, triangles)
        } else {
            flat_mesh(positions, triangles)
        };
        if let Some(colors) = colors {
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors.clone());
        }
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
fn flat_mesh(positions: &[[f32; 3]], triangles: &[[u32; 3]]) -> Mesh {
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

/// Area-weighted shared-vertex normals for generated terrain only. Source
/// model smoothing groups and extended normal records are not interpreted.
pub(crate) fn smooth_mesh(positions: &[[f32; 3]], triangles: &[[u32; 3]]) -> Mesh {
    let mut sums = vec![DVec3::ZERO; positions.len()];
    for triangle in triangles {
        let [a, b, c] =
            triangle.map(|index| DVec3::from_array(positions[index as usize].map(f64::from)));
        let normal = (b - a).cross(c - a);
        for index in triangle {
            sums[*index as usize] += normal;
        }
    }
    let normals: Vec<[f32; 3]> = sums
        .into_iter()
        .map(|normal| {
            normal
                .try_normalize()
                .map(|n| n.as_vec3())
                .unwrap_or(Vec3::Z)
                .to_array()
        })
        .collect();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions.to_vec())
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(bevy::mesh::Indices::U32(
        triangles.iter().flatten().copied().collect(),
    ))
}

/// Double-sided nearest triangle hit; bounded by the generated terrain mesh.
fn nearest_hit(mesh: &DisplayMesh, origin: DVec3, direction: DVec3) -> Option<(usize, DVec3)> {
    if !origin.is_finite() || !direction.is_finite() {
        return None;
    }
    let direction = direction.try_normalize()?;
    let mut closest = f64::INFINITY;
    let mut result = None;
    for (index, triangle) in mesh.triangles.iter().enumerate() {
        let [a, b, c] =
            triangle.map(|i| DVec3::from_array(mesh.positions[i as usize].map(f64::from)));
        let e1 = b - a;
        let e2 = c - a;
        let p = direction.cross(e2);
        let determinant = e1.dot(p);
        if determinant.abs() <= 1e-12 * e1.length() * e2.length() {
            continue;
        }
        let offset = origin - a;
        let u = offset.dot(p) / determinant;
        if !(0.0..=1.0).contains(&u) {
            continue;
        }
        let q = offset.cross(e1);
        let v = direction.dot(q) / determinant;
        if v < 0.0 || u + v > 1.0 {
            continue;
        }
        let distance = e2.dot(q) / determinant;
        if distance >= 0.0 && distance < closest {
            closest = distance;
            result = Some((index, origin + distance * direction));
        }
    }
    result
}

fn probe(
    mouse: Res<ButtonInput<MouseButton>>,
    geometry: Res<DisplayGeometry>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<InspectorCamera>>,
    mut selection: ResMut<ProbeSelection>,
) {
    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }
    let Some(mapping) = &geometry.terrain_probe else {
        return;
    };
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok((camera, transform)) = cameras.single() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(transform, cursor) else {
        return;
    };
    selection.0 = None;
    if let Some((triangle, point)) = nearest_hit(
        &geometry.meshes[0],
        ray.origin.as_dvec3(),
        (*ray.direction).as_dvec3(),
    ) {
        let raw = point / mapping.scale + mapping.center;
        let radius = raw.x.hypot(raw.y).hypot(raw.z);
        let latitude = raw.z.atan2(raw.x.hypot(raw.y)).to_degrees();
        let longitude = raw.y.atan2(raw.x).to_degrees();
        selection.0 = Some(point.as_vec3());
        println!(
            "Terrain probe: triangle={triangle}; globe-local={:?}; latitude={latitude:.6}°, longitude={longitude:.6}°; mesh altitude MSL={:.6} (interpolated triangle, not exact terrain sampler).",
            raw.to_array(),
            radius - mapping.base_radius
        );
    } else {
        println!("Terrain probe: no triangle hit.");
    }
}

fn probe_marker(selection: Res<ProbeSelection>, mut gizmos: Gizmos) {
    if let Some(point) = selection.0 {
        for axis in [Vec3::X, Vec3::Y, Vec3::Z] {
            gizmos.line(
                point - axis * 0.025,
                point + axis * 0.025,
                Color::srgb(1.0, 0.25, 0.05),
            );
        }
    }
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
        ..
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
                    nt_extension: None,
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
            terrain_probe: None,
            crease_degrees: None,
            smooth: false,
            meshes: vec![DisplayMesh {
                colors: None,
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
    fn triangle_probe_is_double_sided_nearest_and_rejects_misses() {
        let mesh = DisplayMesh {
            positions: vec![
                [0., 0., 0.],
                [1., 0., 0.],
                [0., 1., 0.],
                [0., 0., -1.],
                [1., 0., -1.],
                [0., 1., -1.],
            ],
            triangles: vec![[3, 4, 5], [0, 1, 2], [0, 0, 0]],
            colors: None,
        };
        let origin = DVec3::new(0.25, 0.25, 2.0);
        let (index, point) = nearest_hit(&mesh, origin, -DVec3::Z * 7.).unwrap();
        assert_eq!(index, 1);
        assert_eq!(point, DVec3::new(0.25, 0.25, 0.));
        assert_eq!(
            nearest_hit(&mesh, DVec3::new(0.25, 0.25, -2.), DVec3::Z)
                .unwrap()
                .0,
            0
        );
        assert!(nearest_hit(&mesh, origin, DVec3::Z).is_none());
        assert!(nearest_hit(&mesh, origin, DVec3::X).is_none());
        assert!(nearest_hit(&mesh, DVec3::new(2., 2., 2.), -DVec3::Z).is_none());
        assert!(nearest_hit(&mesh, origin, DVec3::ZERO).is_none());
        assert!(nearest_hit(&mesh, DVec3::splat(f64::NAN), -DVec3::Z).is_none());
        assert!(nearest_hit(&mesh, origin, DVec3::splat(f64::INFINITY)).is_none());
    }

    #[test]
    fn elevation_ramp_is_relative_finite_and_flat_safe() {
        let points = [[1., 0., 0.], [0., 2., 0.], [0., 0., 3.]];
        let colors = elevation_colors(&points);
        assert_eq!(colors[0], [0.025, 0.09, 0.16, 1.]);
        assert_eq!(colors[2], [0.8, 0.85, 0.9, 1.]);
        assert_eq!(
            colors,
            elevation_colors(&points.map(|p| p.map(|v| v * 1000.)))
        );
        let flat = elevation_colors(&[[1., 0., 0.], [0., 1., 0.], [0., 0., 1. + 1e-14]]);
        assert!(flat.iter().all(|c| *c == flat[0]));
        assert!(
            colors
                .iter()
                .chain(flat.iter())
                .flatten()
                .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
        );
    }

    #[test]
    fn terrain_setup_uploads_colors_without_a_gpu() {
        let source = crate::terrain_mesh::TerrainMesh::generate(
            crate::terrain_mesh::demonstration_surface(),
            3,
        )
        .unwrap();
        let geometry = terrain_geometry(&source);
        let expected = geometry.meshes[0].colors.clone().unwrap();
        let mut app = App::new();
        app.insert_resource(geometry)
            .init_resource::<Orbit>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .add_systems(Startup, setup);
        app.update();
        let assets = app.world().resource::<Assets<Mesh>>();
        let (_, mesh) = assets.iter().next().unwrap();
        let Some(bevy::mesh::VertexAttributeValues::Float32x4(actual)) =
            mesh.attribute(Mesh::ATTRIBUTE_COLOR)
        else {
            panic!("missing colors")
        };
        assert_eq!(actual, &expected);
        let materials = app.world().resource::<Assets<StandardMaterial>>();
        assert_eq!(materials.iter().next().unwrap().1.base_color, Color::WHITE);
    }

    #[test]
    fn smooth_normals_are_area_weighted_and_keep_shared_indices() {
        use bevy::mesh::VertexAttributeValues;
        // Two triangles sharing an edge: Z-facing area 2, Y-facing area 1.
        let points = [
            [0., 0., 0.],
            [2., 0., 0.],
            [0., 2., 0.],
            [0., 0., 1.],
            [3., 3., 3.],
        ];
        let mesh = smooth_mesh(&points, &[[0, 1, 2], [0, 3, 1], [4, 4, 4]]);
        let Some(VertexAttributeValues::Float32x3(normals)) =
            mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
        else {
            panic!("missing normals")
        };
        let expected = Vec3::new(0., 2., 4.).normalize();
        assert!(Vec3::from_array(normals[0]).distance(expected) < 1e-6);
        assert_eq!(normals[0], normals[1]);
        assert_eq!(normals[2], [0., 0., 1.]);
        assert_eq!(normals[3], [0., 1., 0.]);
        assert_eq!(normals[4], [0., 0., 1.]);
        assert!(
            normals
                .iter()
                .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 1e-6)
        );
        assert_eq!(
            mesh.indices(),
            Some(&bevy::mesh::Indices::U32(vec![0, 1, 2, 0, 3, 1, 4, 4, 4]))
        );
        let Some(VertexAttributeValues::Float32x3(actual)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("missing positions")
        };
        assert_eq!(actual, &points);
    }

    #[test]
    fn terrain_display_is_finite_and_preserves_source_topology() {
        use crate::terrain::{GlobeSurface, TerrainParameters};
        let surface = GlobeSurface {
            size: 8,
            atmosphere_type: 0,
            atmosphere_density: 0,
            water: 0,
            terrain: TerrainParameters::new([3.0, 2.26, 0.65, 123.37, 456.81, 0.0]).unwrap(),
        };
        for bands in [3, 32, 128] {
            let source = crate::terrain_mesh::TerrainMesh::generate(surface, bands).unwrap();
            let before = source.positions().to_vec();
            let display = terrain_geometry(&source);
            let mapping = display.terrain_probe.as_ref().unwrap();
            assert!(display.smooth);
            let mesh = &display.meshes[0];
            let rendered = smooth_mesh(&mesh.positions, &mesh.triangles);
            let Some(bevy::mesh::VertexAttributeValues::Float32x3(normals)) =
                rendered.attribute(Mesh::ATTRIBUTE_NORMAL)
            else {
                panic!("missing normals")
            };
            assert_eq!(normals.len(), source.positions().len());
            assert!(normals.iter().all(|n| n.iter().all(|v| v.is_finite())));
            assert_eq!(mesh.triangles, source.triangles());
            assert_eq!(mesh.positions.len(), source.positions().len());
            assert_eq!(source.positions(), before);
            for (raw, normalized) in source.positions().iter().zip(&mesh.positions) {
                let restored =
                    DVec3::from_array(normalized.map(f64::from)) / mapping.scale + mapping.center;
                assert!((restored - DVec3::from_array(*raw)).length() < 0.003);
            }
            assert!(
                mesh.positions
                    .iter()
                    .flatten()
                    .all(|v| v.is_finite() && v.abs() <= 1.00001)
            );
            assert!(run_terrain(&source, Some(0), false).is_err());
        }
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
            terrain_probe: None,
            crease_degrees: None,
            smooth: false,
            meshes: vec![DisplayMesh {
                colors: None,
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
