//! Explicit fixture model/context, not the graphical client's runtime model.
use super::*;
use crate::world::{
    native_creature::{
        NativeCreatureBodyKind, NativeCreaturePose, NativeCreatureReceiveContext,
        NativeCreatureReceiveState,
    },
    native_dna::NativeDna,
    native_person::{NativePersonBodyReceive, NativePersonReceiveContext},
    native_sentient::NativeOrderSeed,
};
use std::collections::BTreeMap;

pub(super) struct BodyContext {
    pub kind: NativeCreatureBodyKind,
    pub hp: i16,
    pub time: i64,
}
fn unsupported() -> io::Error {
    io::Error::from(io::ErrorKind::Unsupported)
}
impl NativeCreatureReceiveContext for BodyContext {
    fn select_body(&mut self, kind: NativeCreatureBodyKind, created: bool) -> io::Result<()> {
        if created {
            self.hp = 0;
            self.time = 0;
        }
        self.kind = kind;
        Ok(())
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
    fn set_hit_points(&mut self, kind: NativeCreatureBodyKind, hp: i16) -> io::Result<()> {
        assert_eq!(kind, self.kind);
        self.hp = hp;
        Ok(())
    }
    fn old_console_type(&mut self) -> io::Result<u32> {
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
            Err(unsupported())
        } else {
            Ok(())
        }
    }
    fn wall_clock_ms(&mut self) -> io::Result<i64> {
        Ok(1234)
    }
    fn set_body_timestamp(&mut self, kind: NativeCreatureBodyKind, value: i64) -> io::Result<()> {
        assert_eq!(kind, self.kind);
        self.time = value;
        Ok(())
    }
    fn seat_at_current_location(&mut self) -> io::Result<u8> {
        Ok(0)
    }
}
impl NativePersonReceiveContext for BodyContext {
    fn order_seed(&mut self) -> io::Result<NativeOrderSeed> {
        Ok(NativeOrderSeed {
            translation: [2., 4., 6.],
            denominator_terms: [1., 0., 0., 1.],
            wall_clock_ms: 99,
        })
    }
    fn monotonic_now(&mut self) -> Instant {
        Instant::now()
    }
}

pub(super) struct Hooks {
    pub next: u32,
    pub live: BTreeMap<u32, BodyContext>,
    pub events: Vec<(u32, &'static str)>,
    pub reports: Vec<(u32, NativePersonBodyReceive)>,
    pub post: Vec<(u32, bool)>,
    pub reject_construction: bool,
}
impl Default for Hooks {
    fn default() -> Self {
        Self {
            next: 90,
            live: BTreeMap::new(),
            events: vec![],
            reports: vec![],
            post: vec![],
            reject_construction: false,
        }
    }
}
impl NativePersonDispatchHooks for Hooks {
    type PersonContext = BodyContext;
    fn allocate_handle(&mut self, kind: NativeAtomKind) -> io::Result<u32> {
        assert_eq!(kind, NativeAtomKind::Person);
        let handle = self.next;
        self.next += 1;
        Ok(handle)
    }
    fn construct_runtime(&mut self, handle: u32) -> io::Result<()> {
        self.events.push((handle, "construct"));
        if self.reject_construction {
            return Err(unsupported());
        }
        assert!(!self.live.contains_key(&handle));
        self.live.insert(
            handle,
            BodyContext {
                kind: NativeCreatureBodyKind::DpBody,
                hp: 0,
                time: 0,
            },
        );
        Ok(())
    }
    fn destroy_runtime(&mut self, handle: u32) -> io::Result<()> {
        self.events.push((handle, "destroy"));
        assert!(self.live.remove(&handle).is_some());
        Ok(())
    }
    fn person_context(&mut self, handle: u32) -> io::Result<&mut BodyContext> {
        self.events.push((handle, "body"));
        self.live.get_mut(&handle).ok_or_else(unsupported)
    }
    fn position_epoch(&mut self) -> Instant {
        Instant::now()
    }
    fn destination(
        &mut self,
        _: u32,
        _: &crate::atom_position::PackedPosition,
    ) -> DestinationContext {
        panic!("omitted position")
    }
    fn position_auxiliary(&mut self, _: u32) -> io::Result<()> {
        panic!("omitted position")
    }
    fn effect_clock(&mut self) -> (i64, i64) {
        (0x10000, 0)
    }
    fn received_body(&mut self, handle: u32, report: NativePersonBodyReceive) -> io::Result<()> {
        self.events.push((handle, "report"));
        self.reports.push((handle, report));
        Ok(())
    }
    fn post_update(&mut self, handle: u32, is_new: bool) -> io::Result<()> {
        self.events.push((handle, "post"));
        self.post.push((handle, is_new));
        Ok(())
    }
}
