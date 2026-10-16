//! Explicit owned-server receive/update acceptance, not normal gameplay policy.
use super::*;
use crate::world::native_dna::NativeDhDna;
use bevy::asset::AssetId;

#[derive(Resource, Default)]
pub(super) struct Workflow {
    stage: u8,
    frames: u32,
    original: Option<(Entity, Vec<AssetId<Mesh>>, NativeDhDna)>,
    target: Option<([f64; 3], [f64; 3])>,
}
impl Workflow {
    pub(super) fn complete(&self) -> bool {
        self.stage == 4
    }
}

fn report(ui: &mut Presentation, bridge: &Bridge, yaw: f32) -> ([f64; 3], [f64; 3]) {
    let point = ui.local_position.unwrap();
    let frame = placement::tangent_frame(point, yaw).unwrap();
    let point = (DVec3::from_array(point) + frame.x_axis.as_dvec3() * 2.).to_array();
    let rotation = placement::heading_rotation(point, yaw).unwrap();
    ui.local_position = Some(point);
    ui.local_rotation = Some(rotation);
    ui.yaw = yaw;
    assert!(send(
        ui,
        bridge,
        Request::Scene(SceneCommand::PositionReport {
            projected: point,
            rotation
        })
    ));
    (point, rotation)
}

pub(super) fn workflow(
    mut flow: ResMut<Workflow>,
    mut ui: ResMut<Presentation>,
    bridge: Res<Bridge>,
    state: Res<avatar_rendering::State>,
    roots: Query<(Entity, &Transform), With<avatar_rendering::LocalAvatar>>,
    markers: Query<&Marker>,
    meshes: Res<Assets<Mesh>>,
) {
    if ui.model.phase != Phase::Playing || flow.complete() {
        return;
    }
    ui.hand = true; // isolate the scripted reports from ambient desktop input
    let Some((root, _, _)) = state.focus else {
        return;
    };
    let Some((owner, _)) = state.appearance() else {
        return;
    };
    let Ok((_, body)) = roots.get(root) else {
        return;
    };
    let received = ui.model.local_player_dna().unwrap().unwrap();
    flow.frames += 1;
    match flow.stage {
        0 if flow.frames > 30 => {
            flow.original = Some((root, state.mesh_ids(), received.model));
            flow.target = Some(report(&mut ui, &bridge, 0.7));
            flow.stage = 1;
        }
        1 | 2 => {
            let (point, rotation) = flow.target.unwrap();
            let source = ui.model.world.entities[&owner.entity_id].transform.unwrap();
            let close =
                |a: [f64; 3], b: [f64; 3]| a.into_iter().zip(b).all(|(a, b)| (a - b).abs() < 0.01);
            if !close(source.position.map(f64::from), point)
                || !close(source.rotation.map(f64::from), rotation)
            {
                // Source connection dispatch logs/skips a first-seen opcode.
                // Repeated real reports model the normal movement stream;
                // the server fixture only counts distinct received values.
                if flow.frames.is_multiple_of(30) {
                    assert!(send(
                        &mut ui,
                        &bridge,
                        Request::Scene(SceneCommand::PositionReport {
                            projected: point,
                            rotation,
                        })
                    ));
                }
                return;
            }
            assert!(
                (body.translation - Vec3::from_array(point.map(|v| (v * ui.scale) as f32)))
                    .length()
                    < 1e-6
            );
            let expected =
                placement::local_model(point, rotation.map(|v| v as f32), ui.scale).unwrap();
            assert!(body.rotation.angle_between(expected.rotation) < 1e-3);
            let (original_root, original_meshes, original_dna) = flow.original.as_ref().unwrap();
            if flow.stage == 1 {
                assert_eq!(root, *original_root, "motion rebuilt the avatar root");
                assert_eq!(&state.mesh_ids(), original_meshes, "motion rebuilt meshes");
                assert_eq!(&received.model, original_dna);
                println!(
                    "Connected avatar live: server-received motion/turn echoed; root and meshes reused"
                );
                flow.target = Some(report(&mut ui, &bridge, 1.2));
                flow.stage = 2;
            } else {
                if &received.model == original_dna || owner.content.dna != received.model {
                    return;
                }
                assert_ne!(root, *original_root, "appearance update retained old root");
                assert!(
                    original_meshes.iter().all(|id| !meshes.contains(*id)),
                    "old appearance meshes leaked"
                );
                assert_eq!(roots.iter().count(), 1);
                assert!(!markers.iter().any(|marker| marker.0 == owner.entity_id));
                println!(
                    "Connected avatar live: sparse received DNA update replaced root and released old meshes"
                );
                flow.stage = 3;
                flow.frames = 0;
            }
        }
        3 if flow.frames > 120 => {
            println!("Connected avatar live: update remained attached for 120 frames");
            flow.stage = 4;
        }
        _ => {}
    }
}
