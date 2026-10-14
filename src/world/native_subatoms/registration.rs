use super::{NativeNormalSubatoms, NativeReparentResult};
use crate::{
    atom_position::PackedPosition,
    world::native_tick_tock::store::{RegistrationStore, StoreContext},
};
use std::{io, time::Instant};

/// World-owner adapter. Build a context only when the Ether setter actually
/// requires TickTock, borrowing the already-updated hierarchy. Native lookup
/// semantics/predicates stay caller-established instead of guessed defaults.
pub trait ReparentRegistrationContext {
    type Context<'a>: StoreContext
    where
        Self: 'a;
    fn context<'a>(&'a mut self, graph: &'a mut NativeNormalSubatoms, id: u32)
    -> Self::Context<'a>;
}

impl NativeNormalSubatoms {
    /// Normal reparent branch connected to actual registration storage. Context
    /// getters see moved indexes/new position before environment invalidation.
    /// Errors follow the existing partial-effect/cache-cleanup policy, not a
    /// transactional rollback. Missing owner recovery remains explicit.
    pub fn change_parent_with_registrations_at(
        &mut self,
        id: u32,
        record: &PackedPosition,
        extract_orientation: bool,
        now: Instant,
        registrations: &mut RegistrationStore,
        owner: &mut impl ReparentRegistrationContext,
    ) -> io::Result<NativeReparentResult> {
        self.change_parent_in_available_lists_at(
            id,
            record,
            extract_orientation,
            now,
            |graph, id| registrations.apply_tick_tock(id, &mut owner.context(graph, id)),
        )
    }
}

#[cfg(test)]
#[path = "registration_tests.rs"]
mod tests;
