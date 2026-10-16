//! Source-profile local locomotion reports. Never overwrite received actions.
use super::*;

#[derive(Resource, Default)]
pub(super) struct State {
    owner: Option<(u64, u32)>,
    last: Option<[u8; 10]>,
    elapsed: f32,
}
impl State {
    fn request(
        &mut self,
        owner: Option<(u64, u32)>,
        received: Option<[u8; 10]>,
        speed: f32,
        seconds: f32,
    ) -> Option<[u8; 10]> {
        if self.owner != owner || owner.is_none() {
            *self = Self { owner, ..default() };
        }
        // The supplied server does not echo pose reports. Only its neutral
        // received state authorizes local prediction; preserve mixed/actions.
        if owner.is_none() || received != Some([0x24; 10]) || !speed.is_finite() || speed < 0. {
            self.last = None;
            self.elapsed = 0.;
            return None;
        }
        self.elapsed += seconds.min(0.05);
        let code = if speed >= 8. {
            0x25
        } else if speed > 0. {
            0x26
        } else {
            0x24
        };
        let poses = [code; 10];
        (self.last != Some(poses) || self.elapsed >= 0.5).then_some(poses)
    }
}
pub(super) fn report(
    mut state: ResMut<State>,
    mut ui: ResMut<Presentation>,
    bridge: Res<Bridge>,
    time: Res<Time>,
) {
    let owner = (ui.model.phase == Phase::Playing)
        .then(|| ui.model.avatar.map(|id| (ui.model.session_generation, id)))
        .flatten();
    let received = ui
        .model
        .local_player_pose(ui.avatar_motion_speed)
        .ok()
        .flatten()
        .map(|pose| pose.received);
    if let Some(poses) = state.request(owner, received, ui.avatar_motion_speed, time.delta_secs())
        && send(
            &mut ui,
            &bridge,
            Request::Scene(SceneCommand::PoseReport { poses }),
        )
    {
        state.last = Some(poses);
        state.elapsed = 0.;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reports_transitions_retry_and_heartbeat_without_overwriting_received_actions() {
        let mut state = State::default();
        let owner = Some((1, 42));
        for (speed, code) in [(0., 0x24), (4., 0x26), (12., 0x25), (0., 0x24)] {
            let poses = [code; 10];
            assert_eq!(
                state.request(owner, Some([0x24; 10]), speed, 0.),
                Some(poses)
            );
            // An unsuccessful send must be retried, not marked delivered.
            assert_eq!(
                state.request(owner, Some([0x24; 10]), speed, 0.),
                Some(poses)
            );
            state.last = Some(poses);
            state.elapsed = 0.;
            assert_eq!(state.request(owner, Some([0x24; 10]), speed, 0.), None);
        }
        for _ in 0..11 {
            state.request(owner, Some([0x24; 10]), 0., 0.05);
        }
        assert_eq!(
            state.request(owner, Some([0x24; 10]), 0., 0.),
            Some([0x24; 10])
        );
        for received in [None, Some([0x12; 10]), Some([0x15; 10]), Some([0x25; 10])] {
            assert_eq!(state.request(owner, received, 12., 0.05), None);
        }
        assert_eq!(state.request(None, Some([0x24; 10]), 4., 0.05), None);
        assert!(state.owner.is_none());
        assert_eq!(
            state.request(Some((2, 42)), Some([0x24; 10]), 0., 0.),
            Some([0x24; 10])
        );
    }
}
