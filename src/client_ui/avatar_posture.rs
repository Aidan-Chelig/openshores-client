//! Grounded source-profile controller policy, not native collision physics.
use super::*;
use crate::client_model::{AvatarGroundClearances, AvatarRequest};

#[derive(Default)]
pub(super) struct State {
    owner: Option<AvatarRequest>,
    crouched: bool,
    blocked: bool,
    clearances: Option<AvatarGroundClearances>,
    pub(super) frame_start: Option<[f64; 3]>,
}
impl State {
    pub(super) fn cancel(&mut self) {
        self.crouched = false;
        self.blocked = true;
    }
    fn bind(&mut self, owner: &AvatarRequest, held: bool) {
        if self.owner.as_ref() != Some(owner) {
            self.owner = Some(owner.clone());
            self.crouched = false;
            self.blocked = held;
        }
        if !held {
            self.blocked = false;
        }
    }
}
pub(super) fn crouched(ui: &Presentation) -> bool {
    ui.posture.crouched
        && ui.model.phase == Phase::Playing
        && ui.posture.owner.as_ref().is_some_and(|owner| {
            ui.model.avatar == Some(owner.entity_id)
                && ui.model.session_generation == owner.session_generation
                && ui
                    .model
                    .local_player_dna()
                    .ok()
                    .flatten()
                    .is_some_and(|dna| dna.model == owner.content.dna)
        })
}

fn clearance(values: AvatarGroundClearances, crouched: bool) -> f64 {
    f64::from(if crouched {
        values.crouched
    } else {
        values.standing
    })
}
fn grounded(point: DVec3, floor: f64, height: f64) -> bool {
    point.is_finite()
        && floor.is_finite()
        && floor > 0.
        && height.is_finite()
        && height > 0.
        && (point.length() - floor - height).abs() <= 0.02
}
fn ground_root(point: DVec3, floor: f64, height: f64) -> Option<DVec3> {
    (point.is_finite()
        && point.length_squared() > 0.
        && floor.is_finite()
        && floor > 0.
        && height.is_finite()
        && height > 0.)
        .then(|| point.normalize() * (floor + height))
}
fn requested(held: bool, blocked: bool, modal: bool, vertical: bool) -> bool {
    held && !blocked && !modal && !vertical
}

/// Undo only our still-grounded local stance when its prepared body disappears
/// or changes. Never apply old anatomy to another avatar/session or correction.
fn release_local_root(ui: &mut Presentation) {
    let same_actor = ui.posture.owner.as_ref().is_some_and(|owner| {
        ui.model.phase == Phase::Playing
            && ui.model.avatar == Some(owner.entity_id)
            && ui.model.session_generation == owner.session_generation
    });
    if !ui.posture.crouched
        || !same_actor
        || ui.terrain != ui.model.home_globe()
        || ui
            .model
            .local_player_pose(0.)
            .ok()
            .flatten()
            .is_none_or(|pose| pose.received != [0x24; 10])
    {
        return;
    }
    let Some((values, (start, current))) = ui
        .posture
        .clearances
        .zip(ui.posture.frame_start.zip(ui.local_position))
    else {
        return;
    };
    let start = DVec3::from_array(start);
    let current = DVec3::from_array(current);
    let Some(terrain) = &ui.camera_terrain else {
        return;
    };
    let Some(floor) = camera::surface_radius(terrain, start.normalize_or_zero()) else {
        return;
    };
    if !grounded(start, floor, clearance(values, true)) {
        return;
    }
    let Some(floor) = camera::surface_radius(terrain, current.normalize_or_zero()) else {
        return;
    };
    let Some(target) = ground_root(current, floor, clearance(values, false)) else {
        return;
    };
    ui.local_position = Some(target.to_array());
    ui.local_rotation = placement::heading_rotation(target.to_array(), ui.yaw);
    ui.report_dirty = true;
    ui.report_timer = 0.1;
}

pub(super) fn update(
    mut ui: ResMut<Presentation>,
    prepared: Res<avatar::Preparation>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let held = keys.pressed(KeyCode::CapsLock);
    let ready = prepared
        .0
        .lock()
        .unwrap()
        .ready()
        .and_then(|(request, body)| {
            let owner_ok = ui.model.phase == Phase::Playing
                && ui.model.avatar == Some(request.entity_id)
                && ui.model.session_generation == request.session_generation
                && ui.model.world.visible_ids.contains(&request.entity_id)
                && ui
                    .model
                    .world
                    .entities
                    .get(&request.entity_id)
                    .is_some_and(|entity| {
                        entity.parent.is_some()
                            && entity.parent == ui.model.home_globe()
                            && entity
                                .person
                                .as_ref()
                                .is_some_and(|person| person.growth == Some(0))
                    })
                && ui
                    .model
                    .local_player_dna()
                    .ok()
                    .flatten()
                    .is_some_and(|dna| dna.model == request.content.dna);
            owner_ok
                .then(|| {
                    body.animation.as_ref().map(|animation| {
                        (
                            request.clone(),
                            animation.lock().unwrap().ground_clearances(),
                        )
                    })
                })
                .flatten()
        });
    let Some((owner, values)) = ready else {
        release_local_root(&mut ui);
        ui.posture.cancel();
        ui.posture.owner = None;
        return;
    };
    if ui.posture.owner.as_ref() != Some(&owner) {
        release_local_root(&mut ui);
    }
    ui.posture.bind(&owner, held);
    ui.posture.clearances = Some(values);
    let neutral = ui
        .model
        .local_player_pose(0.)
        .ok()
        .flatten()
        .is_some_and(|pose| pose.received == [0x24; 10]);
    if !neutral || ui.terrain != ui.model.home_globe() {
        ui.posture.cancel();
        return;
    }
    let Some((start, current)) = ui.posture.frame_start.zip(ui.local_position) else {
        ui.posture.cancel();
        return;
    };
    let start = DVec3::from_array(start);
    let old = ui.posture.crouched;
    let old_height = clearance(values, old);
    let floor = ui
        .camera_terrain
        .as_ref()
        .and_then(|terrain| camera::surface_radius(terrain, start.normalize_or_zero()));
    if !floor.is_some_and(|floor| grounded(start, floor, old_height)) {
        ui.posture.crouched = false;
        return; // Off-ground/missing terrain: no snap, fabricated gravity or root offset.
    }
    let modal = ui.chat_focus
        || ui.hand
        || keys.pressed(KeyCode::ControlLeft)
        || keys.pressed(KeyCode::ControlRight)
        || keys.pressed(KeyCode::AltLeft)
        || keys.pressed(KeyCode::AltRight);
    let vertical = !modal && (keys.pressed(KeyCode::KeyC) || keys.pressed(KeyCode::KeyZ));
    let next = requested(held, ui.posture.blocked, modal, vertical);
    let mut target = DVec3::from_array(current);
    if next && ui.avatar_motion_speed > 4. {
        // Existing source-slice Walk rate, not a guessed native speed modifier.
        target = start + (target - start) * (4. / f64::from(ui.avatar_motion_speed));
        ui.avatar_motion_speed = 4.;
    }
    let new_height = clearance(values, next);
    if vertical {
        target += start.normalize_or_zero() * (new_height - old_height);
    } else {
        let Some(floor) = ui
            .camera_terrain
            .as_ref()
            .and_then(|terrain| camera::surface_radius(terrain, target.normalize_or_zero()))
        else {
            ui.posture.crouched = false;
            return;
        };
        let Some(point) = ground_root(target, floor, new_height) else {
            ui.posture.cancel();
            return;
        };
        target = point;
    }
    if !target.is_finite() {
        ui.posture.cancel();
        return;
    }
    ui.posture.crouched = next;
    if target.distance_squared(DVec3::from_array(current)) > 1e-12 {
        ui.local_position = Some(target.to_array());
        ui.local_rotation = placement::heading_rotation(target.to_array(), ui.yaw);
        ui.report_dirty = true;
        if old != next {
            ui.report_timer = 0.1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn losing_prepared_body_undoes_only_this_actors_grounded_local_offset() {
        use crate::{client_model::AvatarContentKey, world::native_dna::NativeDhDna};
        let surface = crate::terrain_mesh::demonstration_surface();
        let terrain = TerrainMesh::generate(surface, 16).unwrap();
        let floor = camera::surface_radius(&terrain, DVec3::Z).unwrap();
        let mut ui = Presentation {
            model: ClientModel {
                phase: Phase::Playing,
                avatar: Some(42),
                session_generation: 1,
                ..default()
            },
            terrain: Some(1),
            camera_terrain: Some(terrain),
            ..default()
        };
        let mut packet = vec![0x12];
        packet.extend(42u32.to_be_bytes());
        packet.extend(0i64.to_be_bytes());
        packet.push(1);
        packet.extend(1u32.to_be_bytes());
        packet.extend([0, 8]);
        packet.extend([0x24; 10]);
        packet.extend([0; 7]);
        ui.model.world.ingest(&packet).unwrap();
        let mut globe = ui.model.world.entities[&42].clone();
        globe.id = 1;
        globe.kind = AtomKind::Globe;
        globe.parent = None;
        globe.person = None;
        globe.globe_surface = Some(surface);
        ui.model.world.entities.insert(1, globe);
        ui.posture.owner = Some(AvatarRequest {
            session_generation: 1,
            entity_id: 42,
            content: AvatarContentKey::new(NativeDhDna([0; 24]), 1, 60., true).unwrap(),
        });
        ui.posture.clearances = Some(AvatarGroundClearances {
            standing: 2.,
            crouched: 1.,
        });
        ui.posture.crouched = true;
        let point = [0., 0., floor + 1.];
        ui.posture.frame_start = Some(point);
        ui.local_position = Some(point);
        release_local_root(&mut ui);
        assert!(
            (DVec3::from_array(ui.local_position.unwrap()) - DVec3::Z * (floor + 2.)).length()
                < 1e-8
        );
        assert!(ui.report_dirty);
        ui.model.session_generation += 1;
        ui.local_position = Some(point);
        release_local_root(&mut ui);
        assert_eq!(
            ui.local_position,
            Some(point),
            "old stance moved a new session"
        );
        ui.model.session_generation = 1;
        ui.posture.cancel();
        release_local_root(&mut ui);
        assert_eq!(
            ui.local_position,
            Some(point),
            "cancelled stance moved a correction"
        );
    }
    #[test]
    fn requests_are_owner_bound_and_corrections_require_a_release_before_reapplying() {
        use crate::{client_model::AvatarContentKey, world::native_dna::NativeDhDna};
        let mut owner = AvatarRequest {
            session_generation: 1,
            entity_id: 42,
            content: AvatarContentKey::new(NativeDhDna([0; 24]), 1, 60., true).unwrap(),
        };
        let mut state = State::default();
        state.bind(&owner, true);
        assert!(state.blocked && !state.crouched);
        state.bind(&owner, false);
        assert!(!state.blocked);
        state.crouched = true;
        state.cancel();
        state.bind(&owner, true);
        assert!(state.blocked && !state.crouched);
        state.bind(&owner, false);
        assert!(!state.blocked);
        state.crouched = true;
        owner.session_generation += 1;
        state.bind(&owner, true);
        assert!(state.blocked && !state.crouched);
    }
    #[test]
    fn grounding_uses_nominal_clearance_and_rejects_airborne_or_invalid_points() {
        assert!(grounded(DVec3::new(0., 0., 102.), 100., 2.));
        assert!(grounded(DVec3::new(0., 0., -102.), 100., 2.));
        assert!(!grounded(DVec3::new(0., 0., 107.), 100., 2.));
        assert!(!grounded(DVec3::new(0., 0., 101.), 100., 2.));
        assert!(!grounded(DVec3::ZERO, 100., 2.));
        assert!(!grounded(DVec3::Z * 102., f64::NAN, 2.));
        for up in [DVec3::Z, -DVec3::Z, DVec3::X] {
            let standing = up * 102.;
            let crouched = ground_root(standing, 100., 1.).unwrap();
            assert_eq!(crouched, up * 101.);
            assert_eq!(ground_root(crouched, 100., 2.).unwrap(), standing);
        }
        assert!(ground_root(DVec3::ZERO, 100., 2.).is_none());
        assert!(requested(true, false, false, false));
        for args in [
            (false, false, false, false),
            (true, true, false, false),
            (true, false, true, false),
            (true, false, false, true),
        ] {
            assert!(!requested(args.0, args.1, args.2, args.3));
        }
    }
}
