use super::*;

struct Context {
    events: Vec<&'static str>,
    fail: Option<&'static str>,
    now: Instant,
    model: bool,
}
impl Default for Context {
    fn default() -> Self {
        Self {
            events: vec![],
            fail: None,
            now: Instant::now(),
            model: true,
        }
    }
}
impl Context {
    fn event(&mut self, name: &'static str) -> io::Result<()> {
        self.events.push(name);
        if self.fail == Some(name) {
            Err(io::Error::from(io::ErrorKind::Unsupported))
        } else {
            Ok(())
        }
    }
}
// Trace fixture only: these callbacks do not claim real capsule/UI/model effects.
impl NativeSittingContext for Context {
    type Capsule = ();
    fn parent_capsule(&mut self) -> io::Result<Option<()>> {
        self.event("parent")?;
        Ok(Some(()))
    }
    fn destroy_console(&mut self, _: NativeConsoleState) -> io::Result<()> {
        self.event("destroy")
    }
    fn create_console(&mut self, _: &NativeConsoleState) -> io::Result<()> {
        self.event("create")
    }
    fn notify_parent_capsule(&mut self, _: &(), _: &NativeConsoleState) -> io::Result<()> {
        self.event("notify")
    }
    fn dp_model_data_nonempty(&mut self) -> io::Result<bool> {
        self.event("model")?;
        Ok(self.model)
    }
    fn animate_dp(&mut self, _: &NativeDpBodyPoseState, speed: f32) -> io::Result<()> {
        assert_eq!(speed, 1.);
        self.event("dp")
    }
    fn animate_animalia(&mut self, _: &NativeAnimaliaPoseState) -> io::Result<()> {
        self.event("animal")
    }
    fn monotonic_now(&mut self) -> Instant {
        self.events.push("clock");
        self.now
    }
}
fn request(kind: u32) -> NativeSittingRequest {
    NativeSittingRequest {
        station: 7,
        station_type: kind,
        occupant_position: 0,
        variant: 0,
        trailing_station: 9,
    }
}

#[test]
fn create_reuse_and_trailing_marker_have_distinct_conditions() {
    let mut state = NativeSittingState::default();
    let mut context = Context::default();
    let first = state
        .sit_at_console(request(1), false, None, &mut context)
        .unwrap();
    assert_eq!(first.console_change, NativeSittingChange::Created);
    assert_eq!(
        context.events,
        ["parent", "create", "notify", "model", "dp", "clock"]
    );
    assert_eq!(state.console.as_ref().unwrap().station_number().unwrap(), 9);
    context.events.clear();
    let next = NativeSittingRequest {
        trailing_station: 55,
        ..request(1)
    };
    let result = state
        .sit_at_console(next, false, None, &mut context)
        .unwrap();
    assert_eq!(result.console_change, NativeSittingChange::Reused);
    assert!(!result.body_changed);
    assert!(!result.timer_restarted);
    assert_eq!(state.markers.field_492, 55);
    assert_eq!(state.console.as_ref().unwrap().station_number().unwrap(), 9);
    assert_eq!(context.events, ["parent", "notify", "model", "dp"]);
}

#[test]
fn general_pose_alone_reuses_even_a_different_console_class() {
    let mut state = NativeSittingState::default();
    let mut context = Context::default();
    let _ = state
        .sit_at_console(request(0x21), false, None, &mut context)
        .unwrap();
    context.events.clear();
    let result = state
        .sit_at_console(request(0x22), false, None, &mut context)
        .unwrap();
    assert_eq!(result.console_change, NativeSittingChange::Reused);
    assert_eq!(state.console.as_ref().unwrap().factory_type, 0x21);
    assert!(!context.events.contains(&"create"));
    let result = state
        .sit_at_console(request(2), false, None, &mut context)
        .unwrap();
    assert_eq!(result.console_change, NativeSittingChange::Replaced);
    assert_eq!(
        &context.events[4..],
        [
            "parent", "destroy", "create", "notify", "model", "dp", "clock"
        ]
    );
}

#[test]
fn all_station_types_select_real_console_or_clear_then_apply_body() {
    for kind in 0..=256 {
        for animal in [false, true] {
            let mut state = NativeSittingState::default();
            if animal {
                state.dp_body = None;
                state.animalia = Some(NativeAnimaliaPoseState::default());
            }
            state.console = Some(NativeConsoleState::from_native_constructor(1, 2).unwrap());
            let mut context = Context::default();
            let mut seed = 123;
            let result = state
                .sit_at_console(request(kind), true, Some(&mut seed), &mut context)
                .unwrap();
            let console = matches!(kind, 0..=5 | 7..=0x10 | 0x12..=0x15 | 0x21..=0x23);
            assert_eq!(state.console.is_some(), console, "{kind:x}");
            assert_eq!(context.events[0], "parent");
            assert_eq!(context.events[1], "destroy");
            assert_eq!(context.events.contains(&"notify"), console);
            assert!(
                context
                    .events
                    .contains(&if animal { "animal" } else { "dp" })
            );
            assert_eq!(state.current_general_pose().unwrap(), result.general_pose);
            assert_eq!(
                result.diagnostic,
                if kind == 0x3c {
                    Some(NativeSittingDiagnostic::IntegratedDesigner)
                } else if kind > 0x3c {
                    Some(NativeSittingDiagnostic::UnknownStationType(kind))
                } else {
                    None
                }
            );
        }
    }
}

#[test]
fn missing_random_state_fails_before_console_mutation() {
    let mut state = NativeSittingState {
        console: Some(NativeConsoleState::from_native_constructor(1, 2).unwrap()),
        ..Default::default()
    };
    let before = state.clone();
    let mut context = Context::default();
    assert_eq!(
        state
            .sit_at_console(request(0x24), false, None, &mut context)
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
    assert_eq!(state, before);
    assert_eq!(context.events, ["parent"]);
}

#[test]
fn required_effect_failures_stop_later_stages_without_rollback() {
    let sequence = ["parent", "create", "notify", "model", "dp"];
    for (index, failure) in sequence.iter().enumerate() {
        let mut state = NativeSittingState::default();
        let mut context = Context {
            fail: Some(failure),
            ..Context::default()
        };
        assert!(
            state
                .sit_at_console(request(1), false, None, &mut context)
                .is_err()
        );
        assert_eq!(context.events, sequence[..=index]);
        assert_eq!(state.console.is_some(), index >= 2);
        assert_eq!(
            state.current_general_pose().unwrap(),
            if index == 4 { 5 } else { 0 }
        );
        assert_eq!(state.markers, NativeCreaturePoseMarkers::default());
    }
}

#[test]
fn empty_dp_model_skips_animation_but_not_pose_or_markers() {
    let mut state = NativeSittingState::default();
    let mut context = Context {
        model: false,
        ..Context::default()
    };
    let result = state
        .sit_at_console(request(1), true, None, &mut context)
        .unwrap();
    assert!(result.body_changed && result.timer_restarted);
    assert_eq!(state.current_general_pose().unwrap(), 5);
    assert_eq!(
        context.events,
        ["parent", "create", "notify", "model", "clock"]
    );
}

#[test]
fn received_station_pose_uses_selected_body_word_and_preserves_fallback() {
    for animal in [false, true] {
        let mut state = NativeSittingState::default();
        if animal {
            state.dp_body = None;
            state.animalia = Some(NativeAnimaliaPoseState::default());
        }
        let mut context = Context::default();
        let mut pose = NativeCreaturePose {
            fields: [0; 10],
            station: 77,
        };
        let before = state.clone();
        assert!(
            state
                .set_station_pose(&pose, false, None, &mut context)
                .unwrap()
                .is_none()
        );
        assert_eq!(state, before);
        assert!(context.events.is_empty());
        pose.fields[if animal { 0 } else { 3 }] = 5;
        let result = state
            .set_station_pose(&pose, false, None, &mut context)
            .unwrap()
            .unwrap();
        assert_eq!(result.general_pose, 5);
        assert_eq!(
            state.console.as_ref().unwrap().station_number().unwrap(),
            77
        );
        assert_eq!(state.markers.field_492, 77);
    }
}

#[test]
fn failed_destruction_removes_old_console_and_does_not_construct() {
    let mut state = NativeSittingState {
        console: Some(NativeConsoleState::from_native_constructor(1, 2).unwrap()),
        ..Default::default()
    };
    let mut context = Context {
        fail: Some("destroy"),
        ..Context::default()
    };
    assert!(
        state
            .sit_at_console(request(2), false, None, &mut context)
            .is_err()
    );
    assert!(state.console.is_none());
    assert_eq!(state.current_general_pose().unwrap(), 0);
    assert_eq!(context.events, ["parent", "destroy"]);
}
