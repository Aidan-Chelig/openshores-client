//! Source-profile local gait/gesture reports. Never overwrite received actions.
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
        desired: Option<[u8; 10]>,
        seconds: f32,
    ) -> Option<[u8; 10]> {
        if self.owner != owner || owner.is_none() {
            *self = Self { owner, ..default() };
        }
        // The supplied server does not echo pose reports. Only its neutral
        // received state authorizes local prediction; preserve mixed/actions.
        if owner.is_none() || received != Some([0x24; 10]) || desired.is_none() {
            self.last = None;
            self.elapsed = 0.;
            return None;
        }
        self.elapsed += seconds.min(0.05);
        let poses = desired?;
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
    let selected = ui.avatar_pose().ok().flatten();
    let received = selected.as_ref().map(|pose| pose.received);
    let desired = selected.and_then(|pose| pose.source_report_channels());
    if let Some(poses) = state.request(owner, received, desired, time.delta_secs())
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
    fn mixed_gesture_reports_do_not_flood_and_neutral_is_reported_immediately() {
        let mut state = State::default();
        let owner = Some((1, 42));
        let received = Some([0x24; 10]);
        for code in [Some(0x30), Some(0x1f), None] {
            let mut poses = [0x26; 10];
            if let Some(code) = code {
                poses[2] = code;
            }
            assert_eq!(state.request(owner, received, Some(poses), 0.), Some(poses));
            state.last = Some(poses);
            state.elapsed = 0.;
            for _ in 0..8 {
                assert_eq!(state.request(owner, received, Some(poses), 0.05), None);
            }
            assert_eq!(state.request(owner, received, None, 0.), None);
            assert_eq!(state.request(owner, received, Some(poses), 0.), Some(poses));
        }
    }
    #[test]
    fn reports_transitions_retry_and_heartbeat_without_overwriting_received_actions() {
        let mut state = State::default();
        let owner = Some((1, 42));
        for code in [0x24, 0x26, 0x25, 0x24] {
            let poses = [code; 10];
            assert_eq!(
                state.request(owner, Some([0x24; 10]), Some(poses), 0.),
                Some(poses)
            );
            // An unsuccessful send must be retried, not marked delivered.
            assert_eq!(
                state.request(owner, Some([0x24; 10]), Some(poses), 0.),
                Some(poses)
            );
            state.last = Some(poses);
            state.elapsed = 0.;
            assert_eq!(
                state.request(owner, Some([0x24; 10]), Some(poses), 0.),
                None
            );
        }
        for _ in 0..11 {
            state.request(owner, Some([0x24; 10]), Some([0x24; 10]), 0.05);
        }
        assert_eq!(
            state.request(owner, Some([0x24; 10]), Some([0x24; 10]), 0.),
            Some([0x24; 10])
        );
        for received in [None, Some([0x12; 10]), Some([0x15; 10]), Some([0x25; 10])] {
            assert_eq!(state.request(owner, received, Some([0x25; 10]), 0.05), None);
        }
        assert_eq!(
            state.request(None, Some([0x24; 10]), Some([0x26; 10]), 0.05),
            None
        );
        assert!(state.owner.is_none());
        assert_eq!(
            state.request(Some((2, 42)), Some([0x24; 10]), Some([0x24; 10]), 0.),
            Some([0x24; 10])
        );
    }
}
