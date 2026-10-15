use super::*;

#[derive(Clone, Debug, PartialEq)]
pub(super) enum Event {
    Select(NativeCreatureBodyKind, bool),
    Configure([u8; 2]),
    Dna(NativeDna, u8),
    Scalar(f32),
    Hp(NativeCreatureBodyKind, i16),
    OldConsole,
    Pose(NativeCreatureBodyKind, NativeCreaturePose),
    Console(Option<u8>, u32),
    Clock,
    Timestamp(NativeCreatureBodyKind, i64),
    Seat,
}

/// A trace fixture, NOT the missing concrete runtime context. Console fixtures
/// do use a real NativeStationGame reader; model/pose calls only assert order.
#[derive(Default)]
pub(super) struct Trace {
    pub events: Vec<Event>,
    pub fail_at: Option<usize>,
    pub game: Option<super::super::super::NativeStationGame>,
    pub now: i64,
}

impl Trace {
    fn event(&mut self, value: Event) -> io::Result<()> {
        let index = self.events.len();
        self.events.push(value);
        if self.fail_at == Some(index) {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "fixture context rejects this effect",
            ))
        } else {
            Ok(())
        }
    }
}

impl NativeCreatureReceiveContext for Trace {
    fn select_body(&mut self, kind: NativeCreatureBodyKind, created: bool) -> io::Result<()> {
        self.event(Event::Select(kind, created))
    }
    fn configure_dp_body(&mut self, fields: [u8; 2]) -> io::Result<()> {
        self.event(Event::Configure(fields))
    }
    fn set_dna(&mut self, dna: &NativeDna, packed: u8) -> io::Result<()> {
        self.event(Event::Dna(dna.clone(), packed))
    }
    fn set_scalar_1e0(&mut self, value: f32) -> io::Result<()> {
        self.event(Event::Scalar(value))
    }
    fn set_hit_points(&mut self, kind: NativeCreatureBodyKind, value: i16) -> io::Result<()> {
        self.event(Event::Hp(kind, value))
    }
    fn old_console_type(&mut self) -> io::Result<u32> {
        self.event(Event::OldConsole)?;
        Ok(0x3d)
    }
    fn apply_pose(
        &mut self,
        kind: NativeCreatureBodyKind,
        pose: &NativeCreaturePose,
        state: &NativeCreatureReceiveState,
    ) -> io::Result<()> {
        assert_eq!(state.body.as_ref().unwrap().kind, kind);
        self.event(Event::Pose(kind, pose.clone()))
    }
    fn receive_console(
        &mut self,
        kind: Option<u8>,
        old: u32,
        input: &mut QtReader<'_>,
        _: FloatEncoding,
    ) -> io::Result<()> {
        self.event(Event::Console(kind, old))?;
        match kind {
            None => (),
            Some(0x21) => {
                self.game = Some(super::super::super::NativeStationGame::read(input)?);
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "fixture console context only implements Game",
                ));
            }
        }
        Ok(())
    }
    fn wall_clock_ms(&mut self) -> io::Result<i64> {
        self.event(Event::Clock)?;
        Ok(self.now)
    }
    fn set_body_timestamp(&mut self, kind: NativeCreatureBodyKind, time: i64) -> io::Result<()> {
        self.event(Event::Timestamp(kind, time))
    }
    fn seat_at_current_location(&mut self) -> io::Result<u8> {
        self.event(Event::Seat)?;
        Ok(0xfe)
    }
}
