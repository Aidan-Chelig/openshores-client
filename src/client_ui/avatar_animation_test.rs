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
    rotation: Quat,
    scale: Vec3,
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
    local_gestures: bool,
    crouch_only: bool,
    crouch_grounded: bool,
    local_crouch: bool,
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
            local_gestures: std::env::var_os("OPENSHORES_TEST_AVATAR_LOCAL_GESTURES").is_some(),
            crouch_only: std::env::var_os("OPENSHORES_TEST_AVATAR_RECEIVED_CROUCH").is_some(),
            crouch_grounded: std::env::var_os("OPENSHORES_TEST_AVATAR_CROUCH_ROOTS").is_some(),
            local_crouch: std::env::var_os("OPENSHORES_TEST_AVATAR_LOCAL_CROUCH").is_some(),
        }
    }
    pub(super) fn complete(&self) -> bool {
        self.stage
            == if self.local_crouch {
                6
            } else if self.crouch_only {
                4
            } else if self.local_gestures {
                6
            } else if self.pose_control.is_some() {
                19
            } else {
                4
            }
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
    mut ui: ResMut<Presentation>,
    mut motion: ResMut<AccumulatedMouseMotion>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
) {
    // Configure the fixture BEFORE controls/camera evaluation. Doing this in
    // the post-camera checker compared a new pitch against the previous view
    // whenever ambient desktop mouse events arrived during the run.
    motion.delta = Vec2::ZERO;
    keys.reset_all();
    mouse.reset_all();
    if !flow.complete() {
        ui.hand = false;
        ui.chat_focus = false;
        ui.avatar_inspect = !flow.first_person;
        if flow.first_person {
            ui.top_down = false;
            ui.pitch = -0.8;
        }
    }
    keys.release(KeyCode::KeyW);
    keys.release(KeyCode::ShiftLeft);
    if flow.local_crouch {
        keys.release(KeyCode::CapsLock);
        if matches!(flow.stage, 1 | 2 | 4) {
            keys.press(KeyCode::CapsLock);
        }
        if matches!(flow.stage, 2..=4) {
            keys.press(KeyCode::KeyW);
        }
        if flow.stage == 4 {
            keys.press(KeyCode::ShiftLeft);
        }
        return;
    }
    if flow.pose_control.is_none()
        && rendered.gpu_snapshot().is_some()
        && if flow.local_gestures {
            matches!(flow.stage, 3 | 4)
        } else {
            matches!(flow.stage, 1 | 2)
        }
    {
        keys.press(KeyCode::KeyW);
        if flow.stage == if flow.local_gestures { 4 } else { 2 } {
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
    let transform = roots.get(gpu.root).unwrap();
    let position = transform.translation;
    Some(Snapshot {
        gpu,
        positions,
        uvs,
        eye,
        position,
        rotation: transform.rotation,
        scale: transform.scale,
    })
}
#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct TestContext<'w, 's> {
    cameras: Query<'w, 's, &'static Transform, With<SliceCamera>>,
    heads: Query<'w, 's, &'static Visibility, With<avatar_rendering::FirstPersonHead>>,
    bridge: Res<'w, Bridge>,
    prepared: Res<'w, avatar::Preparation>,
}
pub(super) fn workflow(
    mut flow: ResMut<Workflow>,
    mut ui: ResMut<Presentation>,
    rendered: Res<avatar_rendering::State>,
    meshes: Res<Assets<Mesh>>,
    roots: Query<&Transform, With<avatar_rendering::LocalAvatar>>,
    mut commands: Commands,
    views: TestContext,
) {
    if flow.complete() {
        return;
    }
    assert!(
        flow.started.elapsed().as_secs() < 145,
        "animation acceptance timed out: {}",
        ui.avatar_animation_status
    );
    let Some(current) = snapshot(&rendered, &meshes, &roots) else {
        return;
    };
    assert!(
        !ui.avatar_animation_status.contains("failed"),
        "{}",
        ui.avatar_animation_status
    );
    if flow.first_person && flow.frames > 2 {
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
            std::fs::write(path, if flow.crouch_only { "crouch" } else { "mixed" }).unwrap();
        }
        if flow.local_gestures {
            activate(
                Action::Gesture(Some(crate::client_model::LocalPlayerGesture::Wave)),
                &mut ui,
                &views.bridge,
            );
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
    if flow.local_crouch {
        local_crouch(&mut flow, &mut ui, current, &mut commands, &views.prepared);
        return;
    }
    if flow.crouch_only {
        received_crouch(&mut flow, &mut ui, current, &mut commands);
        return;
    }
    if flow.local_gestures {
        local_gestures(&mut flow, &mut ui, &views.bridge, current, &mut commands);
        return;
    }
    if let Some(control) = flow.pose_control.clone() {
        assert_eq!(
            current.position, initial.position,
            "received pose invented world motion"
        );
        assert_eq!(
            current.rotation, initial.rotation,
            "received pose changed actor rotation"
        );
        assert_eq!(
            current.scale, initial.scale,
            "received pose changed actor scale"
        );
        let expected = match flow.stage {
            1 => "Avatar animation: received mixed locomotion channels",
            2 => "Avatar animation: dying (face down)",
            3 => "Avatar animation: dead (face up)",
            4 => "Avatar animation: received wave channels",
            5 => "Avatar animation: received jump pose",
            6 => "Avatar animation: folded-limb pose 0x18",
            7 => "Avatar animation: folded-limb pose 0x19",
            8 => "Avatar animation: folded-limb pose 0x1b",
            9 => "Avatar animation: folded-limb pose 0x35",
            10 => "Avatar animation: received point channels",
            14 => "Avatar animation: received prone pose",
            15 => "Avatar animation: received death pose (left)",
            16 => "Avatar animation: received death pose (right)",
            17 => "Avatar animation unavailable:",
            _ => "Avatar animation: standing",
        };
        if !ui.avatar_animation_status.starts_with(expected) {
            flow.frames = 0;
            return;
        }
        if matches!(flow.stage, 11..=13) {
            let tilt = match flow.stage {
                11 => 0.45,
                12 => -0.45,
                _ => 0.,
            };
            if ui.model.local_player_pose(0.).unwrap().unwrap().head_angles != [tilt, 0.] {
                flow.frames = 0;
                return;
            }
            if flow.stage == 13 {
                assert_eq!(current.positions, initial.positions);
                assert_eq!(current.eye, initial.eye);
            } else {
                assert_ne!(current.positions, initial.positions);
                assert_ne!(current.eye, initial.eye);
                if flow.stage == 12 {
                    assert_ne!(
                        Some(current.eye),
                        flow.stopped_position,
                        "negative tilt retained the previous positive tilt"
                    );
                }
            }
            if let Some(previous) = &flow.last_phase {
                assert_eq!(
                    &current.positions, previous,
                    "stationary head tilt accumulated transforms"
                );
            }
            flow.last_phase = Some(current.positions);
            if frames == 30 {
                flow.stopped_position = Some(current.eye);
                let (label, next) = match flow.stage {
                    11 => ("tilt-up", "tilt-down"),
                    12 => ("tilt-down", "tilt-reset"),
                    _ => ("tilt-reset", "prone"),
                };
                commands
                    .spawn(Screenshot::primary_window())
                    .observe(save_to_disk(flow.directory.join(format!("{label}.png"))));
                std::fs::write(control, next).unwrap();
                flow.advance();
            }
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
            // Both required phases (37/83) have been captured. Keep the full
            // catalog sequence within the unchanged 60-second local session.
            if frames == 90 {
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
                        10 => "tilt-up",
                        _ => "unsupported",
                    },
                )
                .unwrap();
                flow.advance();
            }
        } else if matches!(flow.stage, 2 | 3 | 14..=16) {
            assert_ne!(current.positions, initial.positions);
            assert_ne!(current.eye, initial.eye);
            if flow.stage == 16 {
                assert_ne!(
                    Some(current.eye),
                    flow.stopped_position,
                    "right death retained left death eye"
                );
            }
            if let Some(previous) = &flow.last_phase {
                assert_eq!(
                    &current.positions, previous,
                    "dead pose accumulated transforms or sampled a fabricated rhythm"
                );
            }
            flow.last_phase = Some(current.positions);
            if frames == 30 {
                if flow.stage == 15 {
                    flow.stopped_position = Some(current.eye);
                }
                commands
                    .spawn(Screenshot::primary_window())
                    .observe(save_to_disk(flow.directory.join(match flow.stage {
                        2 => "received-dying.png",
                        3 => "received-dead.png",
                        14 => "received-prone.png",
                        15 => "received-dead-left.png",
                        _ => "received-dead-right.png",
                    })));
                std::fs::write(
                    control,
                    match flow.stage {
                        2 => "dead",
                        3 => "wave",
                        14 => "dead-left",
                        15 => "dead-right",
                        _ => "unsupported",
                    },
                )
                .unwrap();
                flow.advance();
            }
        } else if flow.stage == 17 {
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

fn local_gestures(
    flow: &mut Workflow,
    ui: &mut Presentation,
    bridge: &Bridge,
    current: Snapshot,
    commands: &mut Commands,
) {
    use crate::client_model::LocalPlayerGesture::{Point, Wave};
    let initial = flow.initial.as_ref().unwrap();
    let frames = flow.frames;
    let selected = ui.avatar_pose().unwrap().unwrap();
    assert_eq!(
        selected.received, [0x24; 10],
        "local prediction mutated received channels"
    );
    let gait = match flow.stage {
        3 => 0x26,
        4 => 0x25,
        _ => 0x24,
    };
    let gesture = match flow.stage {
        1 | 4 => 0x30,
        2 | 3 => 0x1f,
        _ => 0x24,
    };
    let mut expected = [gait; 10];
    expected[2] = gesture;
    assert_eq!(selected.native.fields_3c, expected);
    if flow.stage == 5 {
        assert_eq!(current.positions, initial.positions);
        assert_eq!(current.eye, initial.eye);
        if frames == 1 {
            flow.stopped_position = Some(current.position);
        }
        assert_eq!(Some(current.position), flow.stopped_position);
        if frames == 30 {
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(flow.directory.join("gesture-neutral.png")));
            println!(
                "Connected local gestures: action dispatch, Wave/Point with keyboard Walk/Run, persistent meshes and exact neutral recovery verified"
            );
            flow.advance();
        }
        return;
    }
    if matches!(flow.stage, 1 | 2) {
        assert_eq!(current.position, initial.position);
        assert_eq!(current.eye, initial.eye);
    }
    if matches!(frames, 37 | 83) {
        assert_ne!(current.positions, initial.positions);
        if matches!(flow.stage, 3 | 4) {
            assert_ne!(current.position, initial.position);
        }
        if let Some(previous) = &flow.last_phase {
            if flow.stage == 2 {
                assert_eq!(&current.positions, previous);
            } else {
                assert_ne!(&current.positions, previous);
            }
        }
        let label = match flow.stage {
            1 => "gesture-wave",
            2 => "gesture-point",
            3 => "gesture-point-walk",
            _ => "gesture-wave-run",
        };
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(flow.directory.join(format!(
                "{label}-{}.png",
                if frames == 37 { "a" } else { "b" }
            ))));
        flow.last_phase = Some(current.positions);
    }
    if frames == 90 {
        let next = match flow.stage {
            1 => Some(Point),
            2 => Some(Point),
            3 => Some(Wave),
            _ => None,
        };
        activate(Action::Gesture(next), ui, bridge);
        flow.advance();
    }
}

fn received_crouch(
    flow: &mut Workflow,
    ui: &mut Presentation,
    current: Snapshot,
    commands: &mut Commands,
) {
    let initial = flow.initial.as_ref().unwrap();
    if flow.crouch_grounded {
        let source = ui.model.received_player_transform().unwrap();
        let expected = Vec3::from_array(source.position) * ui.scale as f32;
        assert!(
            current.position.distance(expected) < 1e-6,
            "received ground root lost or double-adjusted"
        );
    } else {
        assert_eq!(
            current.position, initial.position,
            "received crouch invented world displacement"
        );
    }
    let code = match flow.stage {
        1 => 0x10,
        2 => 0x11,
        _ => 0x24,
    };
    if ui.model.local_player_pose(0.).unwrap().unwrap().received != [code; 10] {
        flow.frames = 0;
        return;
    }
    let frames = flow.frames;
    if flow.stage == 3 {
        assert_eq!(current.position, initial.position);
        assert_eq!(current.positions, initial.positions);
        assert_eq!(current.eye, initial.eye);
        if frames == 30 {
            if flow.crouch_grounded {
                println!(
                    "Connected received crouch: authoritative ground roots and exact root recovery verified"
                );
            }
            println!(
                "Connected received crouch: fixed/dynamic phases, persistent resources and exact Stand/eye recovery verified"
            );
            flow.advance();
        }
        return;
    }
    if matches!(frames, 37 | 83) {
        if flow.crouch_grounded {
            assert!(current.position.z < initial.position.z);
        }
        assert_ne!(current.positions, initial.positions);
        if let Some(previous) = &flow.last_phase {
            if flow.stage == 1 {
                assert_eq!(&current.positions, previous);
            } else {
                assert_ne!(&current.positions, previous);
            }
        }
        let label = if flow.stage == 1 {
            "crouch"
        } else {
            "crouch-walk"
        };
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(flow.directory.join(format!(
                "{label}-{}.png",
                if frames == 37 { "a" } else { "b" }
            ))));
        flow.last_phase = Some(current.positions);
    }
    if frames == 90 {
        std::fs::write(
            flow.pose_control.as_ref().unwrap(),
            if flow.stage == 1 {
                "crouch-walk"
            } else {
                "stand"
            },
        )
        .unwrap();
        flow.advance();
    }
}

fn local_crouch(
    flow: &mut Workflow,
    ui: &mut Presentation,
    current: Snapshot,
    commands: &mut Commands,
    prepared: &avatar::Preparation,
) {
    let frames = flow.frames;
    let initial = flow.initial.as_ref().unwrap();
    let crouched = matches!(flow.stage, 1 | 2 | 4);
    let code = match flow.stage {
        1 => 0x10,
        2 | 4 => 0x11,
        3 => 0x26,
        _ => 0x24,
    };
    let pose = ui.avatar_pose().unwrap().unwrap();
    assert_eq!(
        pose.received, [0x24; 10],
        "held crouch changed received state"
    );
    assert_eq!(pose.native.fields_3c, [code; 10]);
    assert_eq!(avatar_posture::crouched(ui), crouched);
    let ready = prepared.0.lock().unwrap();
    let values = ready
        .ready()
        .unwrap()
        .1
        .animation
        .as_ref()
        .unwrap()
        .lock()
        .unwrap()
        .ground_clearances();
    let point = DVec3::from_array(ui.local_position.unwrap());
    let floor =
        camera::surface_radius(ui.camera_terrain.as_ref().unwrap(), point.normalize()).unwrap();
    let height = if crouched {
        values.crouched
    } else {
        values.standing
    };
    assert!(
        (point.length() - floor - f64::from(height)).abs() < 0.002,
        "held crouch lost the nominal ground plane"
    );
    assert!(current.position.distance((point * ui.scale).as_vec3()) < 1e-6);
    if flow.stage == 5 {
        assert_eq!(current.positions, initial.positions);
        assert_eq!(current.eye, initial.eye);
        if frames == 1 {
            flow.stopped_position = Some(current.position);
        }
        assert_eq!(Some(current.position), flow.stopped_position);
        if frames == 30 {
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(
                    flow.directory.join("local-crouch-stopped.png"),
                ));
            println!(
                "Connected local crouch: held Caps Lock, keyboard gait transitions, nominal ground roots, persistent resources and exact Stand geometry/eye verified"
            );
            flow.advance();
        }
        return;
    }
    if matches!(frames, 37 | 83) {
        assert_ne!(current.positions, initial.positions);
        if flow.stage == 1 {
            assert!(current.position.z < initial.position.z);
        }
        if let Some(previous) = &flow.last_phase {
            if flow.stage == 1 {
                assert_eq!(&current.positions, previous);
            } else {
                assert_ne!(&current.positions, previous);
            }
        }
        let label = match flow.stage {
            1 => "local-crouch",
            2 => "local-crouch-walk",
            3 => "local-crouch-release-walk",
            _ => "local-crouch-shift-walk",
        };
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(flow.directory.join(format!(
                "{label}-{}.png",
                if frames == 37 { "a" } else { "b" }
            ))));
        flow.last_phase = Some(current.positions);
    }
    if frames == 90 {
        flow.advance();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixture_view_setup_precedes_camera_and_isolates_desktop_mouse_motion() {
        let mut flow = Workflow::new(PathBuf::new());
        flow.first_person = true;
        let mut app = App::new();
        app.insert_resource(flow)
            .insert_resource(Presentation {
                local_position: Some([0., 0., 100.]),
                scale: 0.001,
                avatar_inspect: true,
                ..default()
            })
            .init_resource::<avatar_rendering::State>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .insert_resource(AccumulatedMouseMotion {
                delta: Vec2::new(20., -30.),
            })
            .add_systems(Update, (input, avatar_camera::view).chain());
        let body = Transform::from_xyz(0., 0., 0.1).with_scale(Vec3::splat(0.001));
        let root = app
            .world_mut()
            .spawn((avatar_rendering::LocalAvatar, body))
            .id();
        let eye = Vec3::new(0., 0.6, 2.5);
        app.world_mut()
            .resource_mut::<avatar_rendering::State>()
            .camera = Some((root, eye));
        let camera = app
            .world_mut()
            .spawn((SliceCamera, Transform::default()))
            .id();
        app.update();
        let ui = app.world().resource::<Presentation>();
        assert_eq!(ui.pitch, -0.8);
        assert!(!ui.avatar_inspect && !ui.top_down);
        assert_eq!(
            app.world().resource::<AccumulatedMouseMotion>().delta,
            Vec2::ZERO
        );
        let frame = placement::tangent_frame([0., 0., 100.], 0.).unwrap();
        let expected = camera::view(body.transform_point(eye), frame, ui.pitch, 0., ui.scale);
        assert!(
            app.world()
                .get::<Transform>(camera)
                .unwrap()
                .rotation
                .abs_diff_eq(expected.rotation, 1e-6)
        );
    }
}
