//! Opt-in owned-server acceptance through real keyboard controls and live meshes.
use super::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use std::{path::PathBuf, time::Instant};

struct Snapshot {
    gpu: avatar_rendering::GpuSnapshot,
    positions: Vec<Vec<[f32; 3]>>,
    uvs: Vec<Vec<[f32; 2]>>,
    eye: Vec3,
    position: Vec3,
}
#[derive(Resource)]
pub(super) struct Workflow {
    directory: PathBuf,
    started: Instant,
    stage: u8,
    frames: u32,
    initial: Option<Snapshot>,
    last_phase: Option<Vec<Vec<[f32; 3]>>>,
    stopped_position: Option<Vec3>,
}
impl Workflow {
    pub(super) fn new(path: PathBuf) -> Self {
        Self {
            directory: path,
            started: Instant::now(),
            stage: 0,
            frames: 0,
            initial: None,
            last_phase: None,
            stopped_position: None,
        }
    }
    pub(super) fn complete(&self) -> bool {
        self.stage == 4
    }
    pub(super) fn side_view(&self) -> bool {
        matches!(self.stage, 1..=3)
    }
    fn advance(&mut self) {
        self.stage += 1;
        self.frames = 0;
        self.last_phase = None;
    }
}
pub(super) fn input(
    flow: Res<Workflow>,
    rendered: Res<avatar_rendering::State>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
) {
    keys.release(KeyCode::KeyW);
    keys.release(KeyCode::ShiftLeft);
    if rendered.gpu_snapshot().is_some() && matches!(flow.stage, 1 | 2) {
        keys.press(KeyCode::KeyW);
        if flow.stage == 2 {
            keys.press(KeyCode::ShiftLeft);
        }
    }
}
fn snapshot(
    rendered: &avatar_rendering::State,
    meshes: &Assets<Mesh>,
    roots: &Query<&Transform, With<avatar_rendering::LocalAvatar>>,
) -> Option<Snapshot> {
    let gpu = rendered.gpu_snapshot()?;
    let positions = gpu
        .meshes
        .iter()
        .map(|id| {
            let Some(bevy::mesh::VertexAttributeValues::Float32x3(values)) =
                meshes.get(*id).unwrap().attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                panic!("missing animation positions");
            };
            values.clone()
        })
        .collect();
    let uvs = gpu
        .meshes
        .iter()
        .map(|id| {
            let Some(bevy::mesh::VertexAttributeValues::Float32x2(values)) =
                meshes.get(*id).unwrap().attribute(Mesh::ATTRIBUTE_UV_0)
            else {
                panic!("missing animation UVs");
            };
            values.clone()
        })
        .collect();
    let eye = rendered.camera?.1;
    let position = roots.get(gpu.root).unwrap().translation;
    Some(Snapshot {
        gpu,
        positions,
        uvs,
        eye,
        position,
    })
}
pub(super) fn workflow(
    mut flow: ResMut<Workflow>,
    mut ui: ResMut<Presentation>,
    rendered: Res<avatar_rendering::State>,
    meshes: Res<Assets<Mesh>>,
    roots: Query<&Transform, With<avatar_rendering::LocalAvatar>>,
    mut commands: Commands,
) {
    assert!(
        flow.started.elapsed().as_secs() < 145,
        "animation acceptance timed out: {}",
        ui.avatar_animation_status
    );
    if flow.complete() {
        return;
    }
    let Some(current) = snapshot(&rendered, &meshes, &roots) else {
        return;
    };
    assert!(
        !ui.avatar_animation_status.contains("failed"),
        "{}",
        ui.avatar_animation_status
    );
    ui.avatar_inspect = true;
    if ui.windows_open || ui.hud {
        ui.windows_open = false;
        ui.hud = false;
        ui.dirty = true;
    }
    flow.frames += 1;
    let frames = flow.frames;
    if ((flow.stage == 0 && frames == 600) || (matches!(flow.stage, 1 | 2) && frames == 120))
        && let Some(point) = ui.local_position.map(DVec3::from_array)
        && let Some(floor) = ui
            .camera_terrain
            .as_ref()
            .and_then(|terrain| camera::surface_radius(terrain, point.normalize_or_zero()))
    {
        println!(
            "Connected avatar animation: stage {} native origin clearance above rendered terrain = {}",
            flow.stage,
            point.length() - floor
        );
    }
    if flow.stage == 0 {
        if frames < 600 {
            return;
        }
        assert_eq!(ui.avatar_animation_status, "Avatar animation: standing");
        flow.initial = Some(current);
        flow.advance();
        return;
    }
    let initial = flow.initial.as_ref().unwrap();
    assert_eq!(current.gpu.root, initial.gpu.root);
    assert_eq!(current.gpu.meshes, initial.gpu.meshes);
    assert_eq!(current.gpu.image, initial.gpu.image);
    assert_eq!(current.gpu.material, initial.gpu.material);
    assert_eq!(current.uvs, initial.uvs);
    if flow.stage == 3 {
        assert_eq!(ui.avatar_animation_status, "Avatar animation: standing");
        assert_eq!(
            current.positions, initial.positions,
            "stopping did not restore Stand"
        );
        assert_eq!(
            current.eye, initial.eye,
            "stopping did not restore authored eye"
        );
        assert_eq!(
            current.position,
            flow.stopped_position.unwrap(),
            "release continued moving"
        );
        if frames == 30 {
            println!(
                "Connected avatar animation: released input restored exact Stand/eye; persistent mesh/root/material/atlas/UVs preserved"
            );
            flow.advance();
        }
        return;
    }
    let label = if flow.stage == 1 { "walk" } else { "run" };
    assert_eq!(
        ui.avatar_animation_status,
        if flow.stage == 1 {
            "Avatar animation: walking"
        } else {
            "Avatar animation: running"
        }
    );
    assert_ne!(
        current.position, initial.position,
        "keyboard did not move actual player"
    );
    if matches!(frames, 37 | 83) {
        assert_ne!(
            current.positions, initial.positions,
            "mesh remained in Stand"
        );
        if let Some(previous) = &flow.last_phase {
            assert_ne!(
                &current.positions, previous,
                "animation phase did not change"
            );
        }
        let path = flow.directory.join(format!(
            "{label}-{}.png",
            if frames == 37 { "a" } else { "b" }
        ));
        assert!(!path.exists(), "animation capture must be new");
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
        flow.last_phase = Some(current.positions);
    }
    if frames == 120 {
        println!(
            "Connected avatar animation: {label} keyboard input moved player and changed native mesh phases without rebuilding assets"
        );
        if flow.stage == 2 {
            flow.stopped_position = Some(current.position);
        }
        flow.advance();
    }
}
