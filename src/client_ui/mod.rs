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
mod assets;
mod auto_login;
mod avatar;
mod avatar_inspection;
mod avatar_rendering;
mod camera;
mod chat;
mod controls;
mod models;
mod native_scene;
#[cfg(test)]
pub(crate) use native_scene::assert_native_geometry_preview;
#[cfg(test)]
pub(crate) use native_scene::assert_native_preview;
mod panels;
mod placement;
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
    asset_focus: bool,
    asset_path: String,
    asset_status: String,
    asset_request: bool,
    asset_browse: bool,
    assets_ready: bool,
    avatar_status: String,
    avatar_placement_status: String,
    avatar_inspect: bool,
    chat_focus: bool,
    chat_draft: String,
    chat_channel: u8,
    chat_range: crate::chat::Range,
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
    report_dirty: bool,
    local_position: Option<[f64; 3]>,
    local_rotation: Option<[f64; 3]>,
    mappings: [Option<(u8, u8)>; 10],
    terrain: Option<u32>,
    camera_terrain: Option<TerrainMesh>,
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
    StoryControl(crate::chat::StoryControl),
    Hail,
    Reply(crate::chat::Reply),
    ChatFocus,
    SendMessage,
    NextChannel,
    NextRange,
    AssetsPath,
    BrowseAssets,
    MountAssets,
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
struct Smoke(u32, bool);

pub fn run(endpoint: LocalEndpoint, smoke_frames: Option<u32>) -> io::Result<()> {
    launch(Some(endpoint), smoke_frames, None, None, None)
}

/// Offline presentation of an explicitly dispatched native Person snapshot.
/// No network worker is created and source gameplay requests remain disabled.
pub fn run_native_preview(
    avatar: crate::client_model::NativeAvatarPresentation,
    smoke_frames: Option<u32>,
) -> io::Result<()> {
    launch(None, smoke_frames, None, None, Some(avatar))
}

/// Login and select a synthetic local avatar, then leave control to the user.
pub struct LocalAutoLogin {
    pub username: String,
    pub password: Zeroizing<String>,
    pub avatar_id: u32,
}

pub fn run_auto_login(
    endpoint: LocalEndpoint,
    profile: LocalAutoLogin,
    smoke_frames: Option<u32>,
) -> io::Result<()> {
    if profile.avatar_id == 0 || !profile.username.starts_with("rust-local-") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "auto-login requires a synthetic rust-local-* account and nonzero avatar",
        ));
    }
    launch(Some(endpoint), smoke_frames, None, Some(profile), None)
}

/// Explicit synthetic-account integration mode, never enabled by normal UI use.
pub struct LocalSmoke {
    pub conversation: bool,
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
    launch(Some(endpoint), None, Some(profile), None, None)
}

fn launch(
    endpoint: Option<LocalEndpoint>,
    smoke_frames: Option<u32>,
    profile: Option<LocalSmoke>,
    auto_profile: Option<LocalAutoLogin>,
    native_avatar: Option<crate::client_model::NativeAvatarPresentation>,
) -> io::Result<()> {
    if smoke_frames.is_some_and(|frames| !(1..=3600).contains(&frames)) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "smoke frame limit must be 1..=3600",
        ));
    }
    let runtime = if let Some(endpoint) = endpoint {
        ClientRuntime::start(endpoint)?
    } else {
        ClientRuntime::offline()
    };
    let mut model = ClientModel::default();
    if let Some(avatar) = native_avatar {
        model.apply(Event::NativeAvatar(Box::new(avatar)))?;
    }
    let asset_path = std::env::var("HAZERON_INSTALL").unwrap_or_default();
    // Explicit scripted fixture mode can still test without proprietary assets.
    let bypass_assets = profile.is_some() && asset_path.is_empty();
    let mut app = App::new();
    let capture = avatar_inspection::Capture::from_environment(auto_profile.is_some())?;
    let inspect = capture.is_some();
    app.insert_resource(Bridge(Mutex::new(runtime)))
        .init_resource::<assets::Setup>()
        .init_resource::<models::Cache>()
        .init_resource::<avatar::Preparation>()
        .init_resource::<avatar_rendering::State>()
        .insert_resource(Presentation {
            model,
            chat_channel: 2,
            asset_request: !asset_path.is_empty(),
            asset_path,
            asset_status: "Select your Hazeron installation to prepare its assets.".into(),
            assets_ready: bypass_assets,
            dirty: true,
            gear_open: true,
            bio_open: true,
            windows_open: true,
            hud: true,
            avatar_inspect: inspect,
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
        .add_plugins(crate::viewer::native_skin::SkinPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                actions::network_events,
                assets::preparation,
                actions::buttons,
                avatar::preparation,
                chat::typing,
                controls::keyboard,
                scene::scene,
                avatar_rendering::render,
                avatar_inspection::camera,
                avatar_inspection::ground_reference,
                native_scene::scene,
                panels::rebuild_panels,
            )
                .chain(),
        );
    if let Some(capture) = capture {
        app.insert_resource(capture).add_systems(
            Update,
            avatar_inspection::capture.after(avatar_inspection::camera),
        );
    }
    if let Some(frames) = smoke_frames {
        app.insert_resource(Smoke(frames, auto_profile.is_some()))
            .add_systems(Update, smoke_exit);
    }
    if let Some(profile) = auto_profile {
        app.insert_resource(auto_login::Workflow::new(profile, smoke_frames.is_some()))
            .add_systems(Update, auto_login::workflow.after(actions::network_events));
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

fn smoke_exit(mut smoke: ResMut<Smoke>, ui: Res<Presentation>, mut exit: MessageWriter<AppExit>) {
    if smoke.1 && ui.model.phase != Phase::Playing {
        return;
    }
    smoke.0 -= 1;
    if smoke.0 == 0 {
        exit.write(AppExit::Success);
    }
}
