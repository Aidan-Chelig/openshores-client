//! Minimal connected development UI. Bindings follow the supplied Controls page;
//! primitive markers, camera tuning and movement speed are slice scaffolding.
use crate::{
    client_model::{ClientModel, Phase},
    client_runtime::{ClientRuntime, Event, LocalEndpoint, Request},
    commands::{ChatCommand, SceneCommand},
    terrain_mesh::TerrainMesh,
    world::AtomKind,
};
mod actions;
mod controls;
mod panels;
mod scene;
mod smoke;
use actions::{activate, send};

use bevy::{
    input::{
        ButtonState,
        keyboard::KeyboardInput,
        mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    },
    math::DVec3,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use std::{io, sync::Mutex};
use zeroize::Zeroizing;

#[derive(Resource)]
struct Bridge(Mutex<ClientRuntime>);
#[derive(Resource, Default)]
struct Presentation {
    model: ClientModel,
    username: String,
    password: Zeroizing<String>,
    password_focus: bool,
    dirty: bool,
    gear_open: bool,
    bio_open: bool,
    windows_open: bool,
    hud: bool,
    hand: bool,
    top_down: bool,
    yaw: f32,
    pitch: f32,
    report_timer: f32,
    local_position: Option<[f64; 3]>,
    mappings: [Option<(u8, u8)>; 10],
    terrain: Option<u32>,
    scale: f64,
    last_scene_revision: u64,
    panel_scroll: f32,
    top_height: f32,
}

#[derive(Component)]
struct Panel;
#[derive(Component)]
struct SliceCamera;
#[derive(Component)]
struct TerrainEntity;
#[derive(Component)]
struct Marker(u32);
#[derive(Component, Clone, Copy)]
enum Action {
    Username,
    Password,
    Login,
    Join(u32),
    SelectGear(u8, u8),
    SelectItem(u32),
    Use,
    Eat,
    Drop,
    Move,
    Pickup,
    Disconnect,
    Gear,
    Bio,
}

#[derive(Resource)]
struct Smoke(u32);

pub fn run(endpoint: LocalEndpoint, smoke_frames: Option<u32>) -> io::Result<()> {
    launch(endpoint, smoke_frames, None)
}

/// Explicit synthetic-account integration mode, never enabled by normal UI use.
pub struct LocalSmoke {
    pub username: String,
    pub password: Zeroizing<String>,
    pub avatar_id: u32,
    pub screenshot: Option<std::path::PathBuf>,
}

pub fn run_local_smoke(endpoint: LocalEndpoint, profile: LocalSmoke) -> io::Result<()> {
    if profile.avatar_id == 0 || !profile.username.starts_with("rust-local-") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "local smoke requires an explicit synthetic rust-local-* account and nonzero avatar",
        ));
    }
    if profile
        .screenshot
        .as_ref()
        .is_some_and(|path| path.exists())
    {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "smoke screenshot must be a new file",
        ));
    }
    launch(endpoint, None, Some(profile))
}

fn launch(
    endpoint: LocalEndpoint,
    smoke_frames: Option<u32>,
    profile: Option<LocalSmoke>,
) -> io::Result<()> {
    if smoke_frames.is_some_and(|frames| !(1..=3600).contains(&frames)) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "smoke frame limit must be 1..=3600",
        ));
    }
    let runtime = ClientRuntime::start(endpoint)?;
    let mut app = App::new();
    app.insert_resource(Bridge(Mutex::new(runtime)))
        .insert_resource(Presentation {
            dirty: true,
            gear_open: true,
            bio_open: true,
            windows_open: true,
            hud: true,
            scale: 1.0,
            top_height: 1.2,
            ..default()
        })
        .insert_resource(ClearColor(Color::srgb(0.025, 0.045, 0.07)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "OpenShores Rust • local development slice".into(),
                resolution: (1280, 900).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                actions::network_events,
                actions::buttons,
                controls::keyboard,
                scene::scene,
                panels::rebuild_panels,
            )
                .chain(),
        );
    if let Some(frames) = smoke_frames {
        app.insert_resource(Smoke(frames))
            .add_systems(Update, smoke_exit);
    }
    let completion = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let scripted = profile.is_some();
    if let Some(profile) = profile {
        app.insert_resource(smoke::Workflow::new(profile, completion.clone()))
            .add_systems(Update, smoke::workflow.after(panels::rebuild_panels));
    }
    match app.run() {
        AppExit::Success if !scripted || completion.load(std::sync::atomic::Ordering::Acquire) => {
            Ok(())
        }
        AppExit::Success => Err(io::Error::other("local GUI smoke did not complete")),
        AppExit::Error(code) => Err(io::Error::other(format!("client exited with code {code}"))),
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        bevy::core_pipeline::tonemapping::Tonemapping::Reinhard,
        SliceCamera,
        Transform::from_xyz(5., -5., 4.).looking_at(Vec3::ZERO, Vec3::Z),
    ));
    commands.insert_resource(GlobalAmbientLight {
        brightness: 200.,
        ..default()
    });
    commands.spawn((
        DirectionalLight {
            illuminance: 8000.,
            ..default()
        },
        Transform::from_xyz(3., -4., 5.).looking_at(Vec3::ZERO, Vec3::Z),
    ));
}

fn smoke_exit(mut smoke: ResMut<Smoke>, mut exit: MessageWriter<AppExit>) {
    smoke.0 -= 1;
    if smoke.0 == 0 {
        exit.write(AppExit::Success);
    }
}
