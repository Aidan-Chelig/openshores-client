//! Opt-in connected removal/restore and same-player reconnect acceptance.
use super::*;
use crate::viewer::native_skin::SkinMaterial;
use bevy::ecs::system::SystemParam;

#[derive(Resource)]
pub(super) struct Workflow {
    username: String,
    password: Zeroizing<String>,
    avatar: u32,
    stage: u8,
    frames: u32,
    generation: u64,
    snapshot: Option<avatar_rendering::GpuSnapshot>,
    cached_body: usize,
}
impl Workflow {
    pub(super) fn new(profile: &LocalAutoLogin) -> Self {
        Self {
            username: profile.username.clone(),
            password: profile.password.clone(),
            avatar: profile.avatar_id,
            stage: 0,
            frames: 0,
            generation: 0,
            snapshot: None,
            cached_body: 0,
        }
    }
    pub(super) fn complete(&self) -> bool {
        self.stage == 6
    }
}

#[derive(SystemParam)]
pub(super) struct Evidence<'w, 's> {
    animation: Res<'w, avatar_animation::State>,
    meshes: Res<'w, Assets<Mesh>>,
    materials: Res<'w, Assets<SkinMaterial>>,
    images: Res<'w, Assets<Image>>,
    roots: Query<'w, 's, Entity, With<avatar_rendering::LocalAvatar>>,
    draws: Query<'w, 's, &'static Mesh3d>,
}
impl Evidence<'_, '_> {
    fn released(&self, snapshot: &avatar_rendering::GpuSnapshot) {
        assert!(
            self.animation.owner().is_none(),
            "released avatar retained animation owner"
        );
        assert!(!self.roots.contains(snapshot.root));
        assert!(snapshot.meshes.iter().all(|id| !self.meshes.contains(*id)));
        assert!(!self.materials.contains(snapshot.material));
        assert!(!self.images.contains(snapshot.image));
        assert!(
            !self
                .draws
                .iter()
                .any(|mesh| snapshot.meshes.contains(&mesh.0.id())),
            "orphaned child still references released avatar geometry"
        );
    }
}

pub(super) fn workflow(
    mut flow: ResMut<Workflow>,
    mut ui: ResMut<Presentation>,
    bridge: Res<Bridge>,
    state: Res<avatar_rendering::State>,
    evidence: Evidence,
    preparation: Res<avatar::Preparation>,
    live: Res<avatar_live::Workflow>,
) {
    if !live.complete() || flow.complete() {
        return;
    }
    ui.hand = true;
    flow.frames += 1;
    match flow.stage {
        0 => {
            let Some(snapshot) = state.gpu_snapshot() else {
                return;
            };
            let prepared = preparation.0.lock().unwrap();
            let Some((_, body)) = prepared.ready() else {
                return;
            };
            flow.cached_body = std::sync::Arc::as_ptr(body) as usize;
            flow.snapshot = Some(snapshot);
            flow.generation = ui.model.session_generation;
            // A third distinct real report asks the explicit server fixture to
            // withdraw then restore ONLY this player from its actual manifest.
            let mut point = ui.local_position.unwrap();
            point[2] += 0.25;
            ui.local_position = Some(point);
            let rotation = ui.local_rotation.or_else(|| ui.model.rotation()).unwrap();
            assert!(send(
                &mut ui,
                &bridge,
                Request::Scene(SceneCommand::PositionReport {
                    projected: point,
                    rotation
                })
            ));
            flow.stage = 1;
        }
        1 if !ui.model.world.visible_ids.contains(&flow.avatar) => {
            evidence.released(flow.snapshot.as_ref().unwrap());
            assert!(state.appearance().is_none());
            println!(
                "Connected avatar lifecycle: received manifest removal released root, children, meshes, material and atlas"
            );
            flow.stage = 2;
        }
        2 if ui.model.world.visible_ids.contains(&flow.avatar) && state.focus.is_some() => {
            let prepared = preparation.0.lock().unwrap();
            let (_, body) = prepared.ready().unwrap();
            assert_eq!(
                std::sync::Arc::as_ptr(body) as usize,
                flow.cached_body,
                "visibility restore regenerated CPU geometry"
            );
            assert_eq!(evidence.roots.iter().count(), 1);
            let snapshot = state.gpu_snapshot().unwrap();
            assert_eq!(
                evidence.animation.owner(),
                Some((ui.model.session_generation, flow.avatar, snapshot.root))
            );
            assert_eq!(ui.avatar_animation_status, "Avatar animation: standing");
            assert_ne!(snapshot.root, flow.snapshot.as_ref().unwrap().root);
            flow.snapshot = Some(snapshot);
            println!(
                "Connected avatar lifecycle: received visibility restore reused completed CPU body with one fresh GPU root"
            );
            activate(Action::Disconnect, &mut ui, &bridge);
            flow.stage = 3;
            flow.frames = 0;
        }
        3 if ui.model.phase == Phase::Disconnected => {
            evidence.released(flow.snapshot.as_ref().unwrap());
            assert!(state.appearance().is_none());
            if flow.frames < 30 {
                return;
            }
            println!("Connected avatar lifecycle: disconnect cleanup remained empty for 30 frames");
            ui.username = flow.username.clone();
            ui.password = flow.password.clone();
            activate(Action::Login, &mut ui, &bridge);
            flow.stage = 4;
        }
        3 => flow.frames = 0,
        4 if ui.model.phase == Phase::Roster => {
            activate(Action::Join(flow.avatar), &mut ui, &bridge);
            flow.stage = 5;
        }
        5 if ui.model.phase == Phase::Playing && state.focus.is_some() => {
            assert_eq!(ui.model.avatar, Some(flow.avatar));
            assert!(ui.model.session_generation > flow.generation);
            let received = ui.model.local_player_dna().unwrap().unwrap();
            let (owner, _) = state.appearance().unwrap();
            assert_eq!(owner.session_generation, ui.model.session_generation);
            assert_eq!(owner.content.dna, received.model);
            assert_eq!(
                Some(&received.model),
                live.initial_dna(),
                "old sparse-update appearance survived server rebootstrap"
            );
            assert_ne!(state.focus.unwrap().0, flow.snapshot.as_ref().unwrap().root);
            assert_eq!(evidence.roots.iter().count(), 1);
            assert_eq!(
                evidence.animation.owner(),
                Some((
                    ui.model.session_generation,
                    flow.avatar,
                    state.focus.unwrap().0
                ))
            );
            assert_eq!(ui.avatar_animation_status, "Avatar animation: standing");
            println!(
                "Connected avatar lifecycle: animation owner cleared and rebound to restored/reconnected generation"
            );
            println!(
                "Connected avatar lifecycle: same-player authenticated reconnect advanced generation and attached received bootstrap DNA once"
            );
            flow.stage = 6;
            // Drop the previous zeroizing allocation now, not just its length.
            flow.password = Zeroizing::new(String::new());
        }
        _ => {}
    }
}
