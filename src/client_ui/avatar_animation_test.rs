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
    pose_control: Option<PathBuf>,
    first_person: bool,
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
            pose_control: std::env::var_os("OPENSHORES_TEST_AVATAR_POSE_CONTROL")
                .map(PathBuf::from),
            first_person: std::env::var_os("OPENSHORES_TEST_AVATAR_ANIMATION_FIRST_PERSON")
                .is_some(),
        }
    }
    pub(super) fn complete(&self) -> bool {
        self.stage == if self.pose_control.is_some() { 13 } else { 4 }
    }
    pub(super) fn side_view(&self) -> bool {
        self.stage > 0 && !self.complete()
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
    if flow.pose_control.is_none()
        && rendered.gpu_snapshot().is_some()
        && matches!(flow.stage, 1 | 2)
    {
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
#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct ViewChecks<'w, 's> {
    cameras: Query<'w, 's, &'static Transform, With<SliceCamera>>,
    heads: Query<'w, 's, &'static Visibility, With<avatar_rendering::FirstPersonHead>>,
}
pub(super) fn workflow(
    mut flow: ResMut<Workflow>,
    mut ui: ResMut<Presentation>,
    rendered: Res<avatar_rendering::State>,
    meshes: Res<Assets<Mesh>>,
    roots: Query<&Transform, With<avatar_rendering::LocalAvatar>>,
    mut commands: Commands,
    views: ViewChecks,
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
    ui.avatar_inspect = !flow.first_person;
    if flow.first_person {
        ui.top_down = false;
        ui.pitch = -0.8;
        if flow.frames > 2 {
            let body = roots.get(current.gpu.root).unwrap();
            let expected = camera::clear_eye(
                body.transform_point(current.eye),
                ui.camera_terrain.as_ref(),
                ui.scale,
            );
            let frame = placement::tangent_frame(ui.local_position.unwrap(), ui.yaw).unwrap();
            let expected_view = camera::view(expected, frame, ui.pitch, 0., ui.scale);
            assert!(!views.cameras.is_empty(), "missing first-person camera");
            for camera in &views.cameras {
                assert!(
                    camera.translation.distance(expected) < 1e-6,
                    "first-person camera lost animated eye"
                );
                assert!(
                    camera.rotation.abs_diff_eq(expected_view.rotation, 1e-5),
                    "received animation changed first-person mouse-look orientation"
                );
            }
            assert!(
                !views.heads.is_empty(),
                "missing local head visibility check"
            );
            for visibility in &views.heads {
                assert_eq!(
                    *visibility,
                    Visibility::Hidden,
                    "animated skull blocks first-person view"
                );
            }
        }
    }
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
        if let Some(path) = &flow.pose_control {
            std::fs::write(path, "mixed").unwrap();
        }
        flow.advance();
        return;
    }
    let initial = flow.initial.as_ref().unwrap();
    assert_eq!(current.gpu.root, initial.gpu.root);
    assert_eq!(current.gpu.meshes, initial.gpu.meshes);
    assert_eq!(current.gpu.image, initial.gpu.image);
    assert_eq!(current.gpu.material, initial.gpu.material);
    assert_eq!(current.uvs, initial.uvs);
    if let Some(control) = flow.pose_control.clone() {
        assert_eq!(
            current.position, initial.position,
            "received pose invented world motion"
        );
        let expected = match flow.stage {
            1 => "Avatar animation: received mixed locomotion channels",
            2 => "Avatar animation: dying (face down)",
            3 => "Avatar animation: dead (face down)",
            4 => "Avatar animation: received wave channels",
            5 => "Avatar animation: received jump pose",
            6 => "Avatar animation: folded-limb pose 0x18",
            7 => "Avatar animation: folded-limb pose 0x19",
            8 => "Avatar animation: folded-limb pose 0x1b",
            9 => "Avatar animation: folded-limb pose 0x35",
            10 => "Avatar animation: received point channels",
            11 => "Avatar animation unavailable:",
            _ => "Avatar animation: standing",
        };
        if !ui.avatar_animation_status.starts_with(expected) {
            flow.frames = 0;
            return;
        }
        if matches!(flow.stage, 1 | 4..=10) {
            if matches!(frames, 37 | 83) {
                assert_ne!(current.positions, initial.positions);
                if let Some(previous) = &flow.last_phase {
                    if matches!(flow.stage, 8 | 10) {
                        assert_eq!(
                            &current.positions, previous,
                            "native fixed pose changed without an active rhythm"
                        );
                    } else {
                        assert_ne!(&current.positions, previous);
                    }
                }
                if matches!(flow.stage, 4 | 10) {
                    assert_eq!(current.eye, initial.eye, "arm-only gesture moved the eye");
                }
                let label = match flow.stage {
                    4 => "wave",
                    5 => "jump",
                    6 => "fold-18",
                    7 => "fold-19",
                    8 => "fold-1b",
                    9 => "fold-35",
                    10 => "point",
                    _ => "received",
                };
                commands
                    .spawn(Screenshot::primary_window())
                    .observe(save_to_disk(flow.directory.join(format!(
                        "{label}-{}.png",
                        if frames == 37 { "a" } else { "b" }
                    ))));
                flow.last_phase = Some(current.positions);
            }
            if frames == 120 {
                std::fs::write(
                    control,
                    match flow.stage {
                        1 => "dying",
                        4 => "jump",
                        5 => "fold-18",
                        6 => "fold-19",
                        7 => "fold-1b",
                        8 => "fold-35",
                        9 => "point",
                        _ => "unsupported",
                    },
                )
                .unwrap();
                flow.advance();
            }
        } else if matches!(flow.stage, 2 | 3) {
            assert_ne!(current.positions, initial.positions);
            assert_ne!(current.eye, initial.eye);
            if let Some(previous) = &flow.last_phase {
                assert_eq!(
                    &current.positions, previous,
                    "dead pose accumulated transforms or sampled a fabricated rhythm"
                );
            }
            flow.last_phase = Some(current.positions);
            if frames == 30 {
                commands
                    .spawn(Screenshot::primary_window())
                    .observe(save_to_disk(flow.directory.join(if flow.stage == 2 {
                        "received-dying.png"
                    } else {
                        "received-dead.png"
                    })));
                std::fs::write(control, if flow.stage == 2 { "dead" } else { "wave" }).unwrap();
                flow.advance();
            }
        } else if flow.stage == 11 {
            if frames == 30 {
                std::fs::write(control, "stand").unwrap();
                flow.advance();
            }
        } else {
            assert_eq!(current.positions, initial.positions);
            assert_eq!(current.eye, initial.eye);
            if frames == 30 {
                println!(
                    "Connected avatar animation: received mixed phases and unsupported/recovery verified; persistent assets and exact Stand/eye preserved"
                );
                if flow.first_person {
                    println!(
                        "Connected avatar animation: received actions followed animated eye and mouse-look with hidden skull"
                    );
                }
                flow.advance();
            }
        }
        return;
    }
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
            if flow.first_person {
                println!(
                    "Connected avatar animation: moving first-person followed animated eye with hidden skull"
                );
            }
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
