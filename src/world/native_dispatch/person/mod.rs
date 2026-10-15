//! Native Person dispatch with concrete receive storage and required runtime effects.
mod context;
use crate::{
    atom_position::{FloatEncoding, PackedPosition},
    world::{
        native_atom::NativeAtomKind,
        native_person::{NativePersonBodyReceive, NativePersonObjects, NativePersonReceiveContext},
        native_position_updates::DestinationContext,
        native_runaways::NativeRunaways,
        native_subatoms::{NativeNormalSubatoms, NativeRecoveryLookup, NativeRetiredAtoms},
        native_tick_tock::store::RegistrationStore,
    },
};
use std::{io, time::Instant};

/// Required runtime/session operations. No successful no-op defaults fabricate
/// model construction, owner observations, clocks, diagnostics or presentation.
pub trait NativePersonDispatchHooks {
    type PersonContext: NativePersonReceiveContext;
    fn allocate_handle(&mut self, kind: NativeAtomKind) -> io::Result<u32>;
    /// Construct the actual inherited model/context, or explicitly Unsupported.
    /// Called after receive-storage admission, before identity/base reception.
    fn construct_runtime(&mut self, handle: u32) -> io::Result<()>;
    /// Called only after candidate ownership/index guards succeed. This must
    /// dispose the temporary model/context, not the retained wire-ID object.
    fn destroy_runtime(&mut self, handle: u32) -> io::Result<()>;
    fn person_context(&mut self, handle: u32) -> io::Result<&mut Self::PersonContext>;
    fn position_epoch(&mut self) -> Instant;
    fn destination(&mut self, handle: u32, record: &PackedPosition) -> DestinationContext;
    fn position_auxiliary(&mut self, handle: u32) -> io::Result<()>;
    fn effect_clock(&mut self) -> (i64, i64);
    /// Preserve nested null/obsolete factory and other receive diagnostics.
    fn received_body(&mut self, handle: u32, report: NativePersonBodyReceive) -> io::Result<()>;
    fn post_update(&mut self, handle: u32, is_new: bool) -> io::Result<()>;
}

pub struct NativePersonDispatch<'a, O, S, H> {
    pub people: &'a mut NativePersonObjects,
    pub graph: &'a mut NativeNormalSubatoms,
    pub lookup: &'a mut NativeRecoveryLookup,
    pub runaways: &'a mut NativeRunaways,
    pub registrations: &'a mut RegistrationStore,
    pub retired: &'a mut NativeRetiredAtoms,
    pub owner: &'a mut O,
    pub connector: &'a mut S,
    pub hooks: &'a mut H,
    pub raw_mode: i32,
    pub profile: FloatEncoding,
}
