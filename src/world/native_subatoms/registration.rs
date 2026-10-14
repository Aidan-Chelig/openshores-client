use super::{NativeNormalSubatoms, NativeReparentResult, NativeRunawayReparentResult};
use crate::{
    atom_position::PackedPosition,
    world::{
        native_runaways::NativeRunaways,
        native_tick_tock::store::{RegistrationStore, StoreContext},
    },
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

/// Distinct caller-owned stores used by the missing-destination branch.
/// Neither store implies native parent-lock availability.
pub struct RunawayRegistrationStores<'a> {
    pub runaways: &'a mut NativeRunaways,
    pub registrations: &'a mut RegistrationStore,
}

impl NativeNormalSubatoms {
    /// Explicit missing-destination-lock branch with real TickTock storage.
    /// Queues the pair before copying and constructing the world context;
    /// retains old child indexes and performs cache cleanup even on callback
    /// errors. Recovery on parent arrival remains separate.
    pub fn change_parent_to_missing_destination_with_registrations_at(
        &mut self,
        id: u32,
        record: &PackedPosition,
        extract_orientation: bool,
        now: Instant,
        stores: RunawayRegistrationStores<'_>,
        owner: &mut impl ReparentRegistrationContext,
    ) -> io::Result<NativeRunawayReparentResult> {
        self.change_parent_to_missing_destination_at(
            id,
            record,
            extract_orientation,
            now,
            stores.runaways,
            |graph, _, id| {
                stores
                    .registrations
                    .apply_tick_tock(id, &mut owner.context(graph, id))
            },
        )
    }

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
#[path = "registration_tests/mod.rs"]
mod tests;
