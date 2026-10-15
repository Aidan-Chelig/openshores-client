use super::*;
struct Context {
    events: Vec<&'static str>,
    now: Instant,
    parent_creature: bool,
    parent_vehicle: bool,
    fail: Option<&'static str>,
}
impl Default for Context {
    fn default() -> Self {
        Self {
            events: vec![],
            now: Instant::now(),
            parent_creature: false,
            parent_vehicle: false,
            fail: None,
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
impl NativeHeadTiltContext for Context {
    fn movement_speed_modifier(&mut self) -> io::Result<f32> {
        self.event("speed")?;
        Ok(0.25)
    }
    fn monotonic_now(&mut self) -> Instant {
        self.events.push("clock");
        self.now
    }
    fn parent_is_creature(&mut self) -> io::Result<bool> {
        self.event("parent-creature")?;
        Ok(self.parent_creature)
    }
    fn parent_is_vehicle(&mut self) -> io::Result<bool> {
        self.event("parent-vehicle")?;
        Ok(self.parent_vehicle)
    }
    fn pointer_690_nonnull(&mut self) -> io::Result<bool> {
        self.event("690")?;
        Ok(true)
    }
    fn animate_dp(
        &mut self,
        state: &NativeHeadTiltState,
        mode: bool,
        speed: f32,
    ) -> io::Result<()> {
        assert!(mode);
        assert_eq!(speed, 0.25);
        assert_eq!(state.horizontal, 0.);
        self.event("dp-animation")
    }
    fn animate_animalia(&mut self, state: &NativeHeadTiltState, start: Instant) -> io::Result<()> {
        assert_eq!(state.horizontal, 0.);
        assert_eq!(start, self.now);
        self.event("au-animation")
    }
    fn set_in_motion(&mut self) -> io::Result<()> {
        self.event("motion")
    }
}
fn pose(kind: NativeCreatureBodyKind, general: u32) -> [u32; 10] {
    let mut words = [0x24; 10];
    words[if kind == NativeCreatureBodyKind::DpBody {
        3
    } else {
        0
    }] = general;
    words
}
const KINDS: [NativeCreatureBodyKind; 2] = [
    NativeCreatureBodyKind::DpBody,
    NativeCreatureBodyKind::Animalia,
];

#[test]
fn exact_locked_predicate_and_pose_specific_animation_suppression() {
    for word in (0..=65535).chain([u32::MAX]) {
        assert_eq!(
            native_pose_is_locked(word),
            word == 3
                || (5..16).contains(&word)
                || (57..74).contains(&word)
                || (76..136).contains(&word)
        );
    }
    for kind in KINDS {
        for general in [3, 5, 15, 57, 73, 74, 75, 76, 135] {
            let mut context = Context::default();
            let report = NativeHeadTiltState {
                vertical: -0.25,
                ..Default::default()
            }
            .apply(0.5, kind, &pose(kind, general), &mut context)
            .unwrap();
            assert!(report.changed);
            assert!(!report.animated);
            assert_eq!(context.events, ["speed", "clock"]);
        }
        for index in 0..10 {
            if index
                == if kind == NativeCreatureBodyKind::DpBody {
                    3
                } else {
                    0
                }
            {
                continue;
            }
            let mut words = pose(kind, 0x24);
            words[index] = 0x17;
            let mut context = Context::default();
            let report = NativeHeadTiltState::default()
                .apply(0.5, kind, &words, &mut context)
                .unwrap();
            assert_eq!(
                !report.animated,
                if kind == NativeCreatureBodyKind::DpBody {
                    [1, 6, 8].contains(&index)
                } else {
                    index == 3
                }
            );
        }
    }
}

#[test]
fn clamps_keep_signed_zero_nan_bits_and_compare_requested_before_body_limits() {
    for kind in KINDS {
        for (input, expected) in [
            (0x7f80_0000, 0x3fc9_0fdb),
            (0xff80_0000, 0xbfc9_0fdb),
            (0x8000_0000, 0x8000_0000),
            (0x7fc0_1234, 0x7fc0_1234),
            (0x3f00_0000, 0x3f00_0000),
        ] {
            let mut state = NativeHeadTiltState {
                vertical: -0.25,
                horizontal: 1.,
                timer_5e8: None,
            };
            let mut context = Context::default();
            let report = state
                .apply(f32::from_bits(input), kind, &pose(kind, 0x24), &mut context)
                .unwrap();
            assert!(report.changed && report.animated);
            assert_eq!(state.vertical.to_bits(), expected);
            assert_eq!(state.horizontal.to_bits(), 0);
            assert_eq!(state.timer_5e8, Some(context.now));
            context.events.clear();
            let report = state
                .apply(f32::from_bits(input), kind, &pose(kind, 0x24), &mut context)
                .unwrap();
            assert_eq!(
                report.changed,
                input == 0x7f80_0000 || input == 0xff80_0000 || input == 0x7fc0_1234
            );
            assert_eq!(context.events.is_empty(), !report.changed);
        }
        let mut state = NativeHeadTiltState::default();
        let mut context = Context::default();
        assert!(
            !state
                .apply(-0., kind, &pose(kind, 0x24), &mut context)
                .unwrap()
                .changed
        );
        assert_eq!(state.vertical.to_bits(), 0);
        assert!(context.events.is_empty());
        for (general, requested) in [(0x39, 1.), (0x0f, -1.)] {
            let mut state = NativeHeadTiltState {
                vertical: 0.5,
                ..Default::default()
            };
            let report = state
                .apply(
                    requested,
                    kind,
                    &pose(kind, general),
                    &mut Context::default(),
                )
                .unwrap();
            assert!(report.changed && !report.animated);
            assert_eq!(state.vertical.to_bits(), 0);
        }
    }
}

#[test]
fn parent_gates_animation_order_and_explicit_failures_preserve_completed_angles() {
    for kind in KINDS {
        let mut context = Context::default();
        let _report = NativeHeadTiltState::default()
            .apply(0.5, kind, &pose(kind, 0x24), &mut context)
            .unwrap();
        let expected = context.events;
        assert_eq!(
            expected,
            if kind == NativeCreatureBodyKind::DpBody {
                vec![
                    "speed",
                    "clock",
                    "parent-creature",
                    "parent-vehicle",
                    "690",
                    "dp-animation",
                    "motion",
                ]
            } else {
                vec![
                    "speed",
                    "clock",
                    "parent-creature",
                    "parent-vehicle",
                    "clock",
                    "au-animation",
                    "motion",
                ]
            }
        );
        for (index, name) in expected
            .iter()
            .enumerate()
            .filter(|(_, name)| **name != "clock")
        {
            let mut context = Context {
                fail: Some(name),
                ..Default::default()
            };
            let mut state = NativeHeadTiltState::default();
            assert_eq!(
                state
                    .apply(0.5, kind, &pose(kind, 0x24), &mut context)
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::Unsupported
            );
            assert_eq!(context.events, expected[..=index]);
            assert_eq!(state.vertical, if *name == "speed" { 0. } else { 0.5 });
        }
        for (creature, vehicle) in [(true, false), (true, true), (false, true)] {
            let mut context = Context {
                parent_creature: creature,
                parent_vehicle: vehicle,
                ..Default::default()
            };
            let report = NativeHeadTiltState::default()
                .apply(0.5, kind, &pose(kind, 0x24), &mut context)
                .unwrap();
            assert!(report.changed && !report.animated);
            assert_eq!(
                context.events,
                if creature {
                    vec!["speed", "clock", "parent-creature"]
                } else {
                    vec!["speed", "clock", "parent-creature", "parent-vehicle"]
                }
            );
        }
    }
}
