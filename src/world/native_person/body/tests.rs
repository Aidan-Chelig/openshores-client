use super::*;
use crate::world::{
    native_creature::{NativeCreatureBodyKind, NativeCreaturePose},
    native_dna::NativeDna,
};

/// Trace-only context proves orchestration, not actual runtime model effects.
#[derive(Default)]
struct Context {
    events: Vec<&'static str>,
    fail: Option<&'static str>,
}
impl Context {
    fn event(&mut self, name: &'static str) -> io::Result<()> {
        self.events.push(name);
        if self.fail == Some(name) {
            Err(unsupported())
        } else {
            Ok(())
        }
    }
}
fn unsupported() -> io::Error {
    io::Error::from(io::ErrorKind::Unsupported)
}
impl NativeCreatureReceiveContext for Context {
    fn select_body(&mut self, _: NativeCreatureBodyKind, _: bool) -> io::Result<()> {
        self.event("select")
    }
    fn configure_dp_body(&mut self, _: [u8; 2]) -> io::Result<()> {
        Err(unsupported())
    }
    fn set_dna(&mut self, _: &NativeDna, _: u8) -> io::Result<()> {
        Err(unsupported())
    }
    fn set_scalar_1e0(&mut self, _: f32) -> io::Result<()> {
        Err(unsupported())
    }
    fn set_hit_points(&mut self, _: NativeCreatureBodyKind, _: i16) -> io::Result<()> {
        self.event("hp")
    }
    fn old_console_type(&mut self) -> io::Result<u32> {
        self.event("old-console")?;
        Ok(0x3d)
    }
    fn apply_pose(
        &mut self,
        _: NativeCreatureBodyKind,
        _: &NativeCreaturePose,
        _: &NativeCreatureReceiveState,
    ) -> io::Result<()> {
        Err(unsupported())
    }
    fn receive_console(
        &mut self,
        kind: Option<u8>,
        _: u32,
        _: &mut QtReader<'_>,
        _: FloatEncoding,
    ) -> io::Result<()> {
        if kind.is_some() {
            return Err(unsupported());
        }
        self.event("console")
    }
    fn wall_clock_ms(&mut self) -> io::Result<i64> {
        self.event("wall")?;
        Ok(1234)
    }
    fn set_body_timestamp(&mut self, _: NativeCreatureBodyKind, time: i64) -> io::Result<()> {
        assert_eq!(time, 1234);
        self.event("timestamp")
    }
    fn seat_at_current_location(&mut self) -> io::Result<u8> {
        self.event("seat")?;
        Ok(0)
    }
}
impl NativePersonReceiveContext for Context {
    fn order_seed(&mut self) -> io::Result<NativeOrderSeed> {
        self.event("order")?;
        Ok(NativeOrderSeed {
            translation: [2., 4., 6.],
            denominator_terms: [1., 0., 0., 1.],
            wall_clock_ms: 99,
        })
    }
    fn monotonic_now(&mut self) -> Instant {
        self.events.push("monotonic");
        Instant::now()
    }
}
fn state() -> NativePersonBodyState {
    NativePersonBodyState {
        person: NativePersonSuffixState::with_prior_fields(0, 0x8000),
        ..Default::default()
    }
}
fn fixture(profile: FloatEncoding) -> (Vec<u8>, usize, usize) {
    // Unit real scalar suffix, Creature HP and unsigned tail.
    let mut bytes = vec![1, 0xff, 0xff, 0xff, 0xff];
    bytes.extend((-7_i32).to_be_bytes());
    bytes.push(0x80);
    bytes.extend(0x204_u16.to_be_bytes());
    bytes.extend(i16::MIN.to_be_bytes());
    bytes.extend([0xab, 0xff, 0x80]);
    let creature_end = bytes.len();
    // Sentient order initialization + actual skill-map reader.
    bytes.push(6);
    bytes.push(0x29);
    bytes.extend(77_u32.to_be_bytes());
    bytes.extend(u32::MAX.to_be_bytes());
    bytes.push(0xfe);
    bytes.push(0xff);
    bytes.extend(1_u16.to_be_bytes());
    bytes.extend(0x8000_u16.to_be_bytes());
    bytes.extend(i32::MIN.to_be_bytes());
    let sentient_end = bytes.len();
    // Person packed short, counted content IDs, extended double.
    bytes.extend([0xa4, 0x10]);
    bytes.extend(0x8001_u16.to_be_bytes());
    bytes.extend([0xff, 0x8f, 3, 0, 0x80, 0xff, 0xfe]);
    if profile == FloatEncoding::SinglePrecision {
        bytes.extend(1.25_f32.to_be_bytes());
    } else {
        bytes.extend(1.25_f64.to_be_bytes());
    }
    (bytes, creature_end, sentient_end)
}

#[test]
fn actual_four_stages_share_cursor_and_retained_state() {
    for profile in [
        FloatEncoding::TypeWidths,
        FloatEncoding::SinglePrecision,
        FloatEncoding::DoublePrecision,
    ] {
        let (mut bytes, creature_end, sentient_end) = fixture(profile);
        let length = bytes.len();
        bytes.push(0xee);
        let mut input = QtReader::new(&bytes);
        let mut state = state();
        let mut context = Context::default();
        let report = state
            .receive_qt7(&mut input, profile, &mut context)
            .unwrap();
        assert_eq!(report.consumed_bytes, length);
        assert_eq!(report.creature.consumed_bytes, creature_end);
        assert_eq!(report.sentient_bytes, sentient_end - creature_end);
        assert_eq!(report.person.consumed_bytes, length - sentient_end);
        assert_eq!(input.read_u8().unwrap(), 0xee);
        assert_eq!(
            context.events,
            [
                "select",
                "hp",
                "old-console",
                "console",
                "wall",
                "timestamp",
                "seat",
                "order",
                "monotonic"
            ]
        );
        assert_eq!(state.creature.unit.fields.field_3a8, Some(-7));
        assert_eq!(state.creature.body.as_ref().unwrap().hit_points, -30);
        assert_eq!(state.creature.field_5b8, Some(255));
        assert_eq!(state.creature.field_5bc, Some(128));
        assert_eq!(
            state.sentient.orders.as_ref().unwrap()[0].position,
            [1., 2., 3.]
        );
        assert_eq!(state.sentient.skills.as_ref().unwrap().0[&0x8000], i32::MIN);
        assert_eq!(state.person.field_660, Some(1));
        assert_eq!(
            state.person.content_fa0.as_deref(),
            Some([0, 0x80, 0xff].as_slice())
        );
        assert_eq!(state.person.field_1408, Some(1.25));
        context.events.clear();
        // Repeat sparse packet: Unit→Creature→Sentient→Person, no seed/timer.
        let _ = state
            .receive_qt7(
                &mut QtReader::new(&[0, 0, 0, 9, 0, 0]),
                profile,
                &mut context,
            )
            .unwrap();
        assert!(state.sentient.orders.as_ref().unwrap().is_empty());
        assert_eq!(state.person.field_660, Some(1));
        assert_eq!(state.person.field_1408, Some(1.25));
        assert!(!context.events.contains(&"order"));
        assert!(!context.events.contains(&"monotonic"));
    }
}

#[test]
fn required_context_failure_stops_before_later_stages() {
    let (bytes, creature_end, _) = fixture(FloatEncoding::TypeWidths);
    for failure in [
        "select",
        "hp",
        "old-console",
        "console",
        "wall",
        "timestamp",
        "seat",
        "order",
    ] {
        let mut state = state();
        let before = state.clone();
        let mut context = Context {
            fail: Some(failure),
            ..Default::default()
        };
        let mut input = QtReader::new(&bytes);
        assert_eq!(
            state
                .receive_qt7(&mut input, FloatEncoding::TypeWidths, &mut context)
                .unwrap_err()
                .kind(),
            io::ErrorKind::Unsupported
        );
        assert_eq!(context.events.last(), Some(&failure));
        assert_eq!(state.person, before.person);
        if failure != "order" {
            assert_eq!(state.sentient, before.sentient);
            assert!(input.remaining() >= bytes.len() - creature_end);
        } else {
            assert_eq!(state.creature.field_495, Some(0));
            assert_eq!(state.sentient.field_708, Some(0.));
        }
    }
}

#[test]
fn every_truncated_body_rejects_without_running_a_later_stage() {
    for profile in [
        FloatEncoding::TypeWidths,
        FloatEncoding::SinglePrecision,
        FloatEncoding::DoublePrecision,
    ] {
        let (bytes, creature_end, sentient_end) = fixture(profile);
        for end in 0..bytes.len() {
            let mut state = state();
            let before = state.clone();
            let mut context = Context::default();
            assert_eq!(
                state
                    .receive_qt7(&mut QtReader::new(&bytes[..end]), profile, &mut context)
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::UnexpectedEof
            );
            if end < creature_end {
                assert_eq!(state.sentient, before.sentient);
            }
            if end <= sentient_end {
                assert_eq!(state.person, before.person);
            }
        }
    }
}
