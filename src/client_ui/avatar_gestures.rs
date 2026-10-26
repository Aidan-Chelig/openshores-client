//! Explicit prototype gesture buttons; no invented original key bindings.
use super::*;
use crate::client_model::{LocalPlayerGesture, LocalPlayerPose};

type Owner = (u64, u32);
#[derive(Default)]
pub(super) struct State {
    owner: Option<Owner>,
    selected: Option<LocalPlayerGesture>,
}
impl State {
    fn sync(&mut self, owner: Option<Owner>, received: Option<[u8; 10]>) -> bool {
        let allowed = owner.is_some() && received == Some([0x24; 10]);
        let changed = self.owner != owner || !allowed && self.selected.is_some();
        if self.owner != owner || !allowed {
            self.selected = None;
        }
        self.owner = owner;
        changed
    }
}
fn owner(ui: &Presentation) -> Option<Owner> {
    (ui.model.phase == Phase::Playing)
        .then(|| ui.model.avatar.map(|id| (ui.model.session_generation, id)))
        .flatten()
}
fn context(ui: &Presentation) -> (Option<Owner>, Option<[u8; 10]>) {
    let received = ui
        .model
        .local_player_pose(0.)
        .ok()
        .flatten()
        .map(|v| v.received);
    (owner(ui), received)
}
pub(super) fn reconcile(ui: &mut Presentation) {
    let (owner, received) = context(ui);
    if ui.gestures.sync(owner, received) {
        ui.dirty = true;
    }
}
pub(super) fn choose(ui: &mut Presentation, gesture: Option<LocalPlayerGesture>) {
    reconcile(ui);
    let (owner, received) = context(ui);
    if owner.is_some() && received == Some([0x24; 10]) {
        ui.gestures.selected = gesture;
        ui.model.status = match gesture {
            Some(LocalPlayerGesture::Wave) => "Local left-arm Wave; neutral ends the gesture",
            Some(LocalPlayerGesture::Point) => "Local left-arm Point; neutral ends the gesture",
            None => "Local gesture cleared",
        }
        .into();
    } else {
        ui.model.status = "Gesture unavailable: waiting for received neutral pose".into();
    }
    ui.dirty = true;
}
impl Presentation {
    pub(super) fn avatar_pose(&self) -> io::Result<Option<LocalPlayerPose>> {
        let gesture = (owner(self) == self.gestures.owner)
            .then_some(self.gestures.selected)
            .flatten();
        Ok(self
            .model
            .local_player_pose(self.avatar_motion_speed)?
            .map(|pose| {
                pose.with_crouch(avatar_posture::crouched(self))
                    .with_gesture(gesture)
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn received(ui: &mut Presentation, poses: Option<[u8; 10]>) {
        let mut body = crate::protocol::QtWriter::default();
        body.write_u8(0);
        body.write_u8(if poses.is_some() { 8 } else { 0 });
        if let Some(poses) = poses {
            for code in poses {
                body.write_u8(code);
            }
            body.write_u32(0);
        }
        for _ in 0..3 {
            body.write_u8(0);
        }
        let mut packet = vec![0x12];
        packet.extend(42u32.to_be_bytes());
        packet.extend(0i64.to_be_bytes());
        packet.push(0);
        packet.extend(body.into_bytes());
        ui.model.apply(Event::Packet(packet)).unwrap();
        reconcile(ui);
    }
    #[test]
    fn sparse_receive_action_dispatch_and_owner_changes_preserve_server_authority() {
        let mut ui = Presentation {
            model: ClientModel {
                phase: Phase::Playing,
                avatar: Some(42),
                session_generation: 1,
                ..default()
            },
            ..default()
        };
        received(&mut ui, Some([0x24; 10]));
        choose(&mut ui, Some(LocalPlayerGesture::Wave));
        assert_eq!(ui.avatar_pose().unwrap().unwrap().native.fields_3c[2], 0x30);
        assert_eq!(
            ui.model
                .local_player_pose(0.)
                .unwrap()
                .unwrap()
                .native
                .fields_3c,
            [0x24; 10]
        );
        received(&mut ui, None);
        assert_eq!(ui.gestures.selected, Some(LocalPlayerGesture::Wave));
        received(&mut ui, Some([0x12; 10]));
        choose(&mut ui, Some(LocalPlayerGesture::Point));
        assert_eq!(ui.gestures.selected, None);
        assert_eq!(
            ui.avatar_pose().unwrap().unwrap().native.fields_3c,
            [0x12; 10]
        );
        received(&mut ui, Some([0x24; 10]));
        assert!(ui.avatar_pose().unwrap().unwrap().is_standing());
        choose(&mut ui, Some(LocalPlayerGesture::Point));
        ui.model.session_generation += 1;
        assert!(
            ui.avatar_pose().unwrap().unwrap().is_standing(),
            "stale owner gesture was applied before reconciliation"
        );
        reconcile(&mut ui);
        assert_eq!(ui.gestures.selected, None);
        choose(&mut ui, Some(LocalPlayerGesture::Wave));
        ui.model.world.entities.remove(&42);
        reconcile(&mut ui);
        assert_eq!(ui.gestures.selected, None);
        received(&mut ui, Some([0x24; 10]));
        assert!(ui.avatar_pose().unwrap().unwrap().is_standing());
        choose(&mut ui, Some(LocalPlayerGesture::Point));
        choose(&mut ui, None);
        assert!(ui.avatar_pose().unwrap().unwrap().is_standing());
    }
    #[test]
    fn gesture_lifetime_is_owner_bound_and_received_actions_cancel_without_resuming() {
        let mut state = State::default();
        let owner = Some((1, 42));
        state.sync(owner, Some([0x24; 10]));
        state.selected = Some(LocalPlayerGesture::Wave);
        assert!(!state.sync(owner, Some([0x24; 10])));
        assert_eq!(state.selected, Some(LocalPlayerGesture::Wave));
        for received in [None, Some([0x12; 10]), Some([0x25; 10])] {
            state.selected = Some(LocalPlayerGesture::Point);
            state.sync(owner, received);
            assert_eq!(state.selected, None);
            state.sync(owner, Some([0x24; 10]));
            assert_eq!(state.selected, None);
        }
        for next in [Some((2, 42)), Some((2, 43)), None] {
            state.selected = Some(LocalPlayerGesture::Wave);
            state.sync(next, Some([0x24; 10]));
            assert_eq!(state.selected, None);
        }
    }
}
