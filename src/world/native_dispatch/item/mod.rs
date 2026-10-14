//! Concrete confirmed Item branch. Not all-class or playable-session completion.
mod context;
use crate::{
    atom_position::PackedPosition,
    world::{
        native_atom::NativeAtomKind,
        native_item::NativeItemObjects,
        native_position_updates::DestinationContext,
        native_runaways::NativeRunaways,
        native_subatoms::{NativeNormalSubatoms, NativeRecoveryLookup, NativeRetiredAtoms},
        native_tick_tock::store::RegistrationStore,
    },
};
use std::{io, time::Instant};

/// Required receive/session observations and post-update effects. No default
/// implementation fabricates clocks, missing-parent decisions or notifications.
pub trait NativeItemDispatchHooks {
    fn allocate_handle(&mut self, kind: NativeAtomKind) -> io::Result<u32>;
    fn position_epoch(&mut self) -> Instant;
    fn destination(&mut self, handle: u32, record: &PackedPosition) -> DestinationContext;
    fn position_auxiliary(&mut self, handle: u32) -> io::Result<()>;
    fn effect_clock(&mut self) -> (i64, i64);
    fn post_update(&mut self, handle: u32, is_new: bool) -> io::Result<()>;
}

pub struct NativeItemDispatch<'a, O, S, H> {
    pub items: &'a mut NativeItemObjects,
    pub graph: &'a mut NativeNormalSubatoms,
    pub lookup: &'a mut NativeRecoveryLookup,
    pub runaways: &'a mut NativeRunaways,
    pub registrations: &'a mut RegistrationStore,
    pub retired: &'a mut NativeRetiredAtoms,
    pub owner: &'a mut O,
    pub connector: &'a mut S,
    pub hooks: &'a mut H,
    pub raw_mode: i32,
}
