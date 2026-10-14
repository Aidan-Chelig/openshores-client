//! Existing native atom position updates, not complete DtClient packet handling.
//! Raw body/timestamps/status metadata and object registration remain separate.
use super::{
    native_atom::NativeAtomPrefix,
    native_runaways::NativeRunaways,
    native_subatoms::{
        NativeHierarchyPositionChange, NativeNormalSubatoms, NativeReparentResult,
        NativeRunawayReparentResult, ReparentRegistrationContext, RunawayRegistrationStores,
    },
    native_tick_tock::store::RegistrationStore,
};
use crate::atom_position::FreshPosition;
use std::{collections::BTreeSet, io, time::Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DestinationContext {
    /// Unestablished native destination context; differing parent is deferred.
    Unresolved,
    ExistingParentLists,
    /// Explicit native destination-lock absence, not inferred from wire data.
    MissingDestinationLock,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NativePositionUpdate {
    Local {
        cleared_caches: BTreeSet<u32>,
    },
    ReparentRequired(FreshPosition),
    /// May still contain OwnerContextRequired; not silently counted as applied.
    NormalOwner(NativeReparentResult),
    MissingDestination(NativeRunawayReparentResult),
}

pub struct NativePositionReceiver<'a, O> {
    /// Existing nodes keyed by their wire atom ID. Aliased node-handle lookup
    /// needs an explicit full owner adapter, not ID guessing in this receiver.
    pub graph: &'a mut NativeNormalSubatoms,
    pub runaways: &'a mut NativeRunaways,
    pub registrations: &'a mut RegistrationStore,
    pub owner: &'a mut O,
}

impl<O: ReparentRegistrationContext> NativePositionReceiver<'_, O> {
    /// Apply only the decoded fresh base-position stage. An absent position is
    /// a no-op, not a body/lifecycle acknowledgement. Existing class must match;
    /// unknown nodes/replacement are explicit errors before state mutation.
    /// RxUpdateBase uses the position-only ChangePos path, not orientation
    /// extraction. No timestamp filtering or source-profile fallback is added.
    pub fn apply_at(
        &mut self,
        prefix: &NativeAtomPrefix<'_>,
        destination: DestinationContext,
        now: Instant,
    ) -> io::Result<Option<NativePositionUpdate>> {
        let Some(record) = &prefix.base.position else {
            return Ok(None);
        };
        let id = prefix.base.id;
        let node = self.graph.node(id).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "native position receiver requires an existing wire-ID node",
            )
        })?;
        if node.kind != prefix.kind {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "native atom class change requires object lifecycle handling",
            ));
        }
        let outcome = match self.graph.apply_fresh_position_at(id, record, now)? {
            NativeHierarchyPositionChange::Applied { cleared_caches } => {
                NativePositionUpdate::Local { cleared_caches }
            }
            NativeHierarchyPositionChange::ReparentRequired(position) => match destination {
                DestinationContext::Unresolved => NativePositionUpdate::ReparentRequired(position),
                DestinationContext::ExistingParentLists => NativePositionUpdate::NormalOwner(
                    self.graph.change_parent_with_registrations_at(
                        id,
                        record,
                        false,
                        now,
                        self.registrations,
                        self.owner,
                    )?,
                ),
                DestinationContext::MissingDestinationLock => {
                    NativePositionUpdate::MissingDestination(
                        self.graph
                            .change_parent_to_missing_destination_with_registrations_at(
                                id,
                                record,
                                false,
                                now,
                                RunawayRegistrationStores {
                                    runaways: self.runaways,
                                    registrations: self.registrations,
                                },
                                self.owner,
                            )?,
                    )
                }
            },
        };
        Ok(Some(outcome))
    }
}

#[cfg(test)]
mod tests;
