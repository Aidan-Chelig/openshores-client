//! Existing native atom position updates, not complete DtClient packet handling.
//! Raw body/timestamps/status metadata and object registration remain separate.
use super::{
    native_atom::NativeAtomPrefix,
    native_runaways::NativeRunaways,
    native_subatoms::{
        NativeAtomIdentity, NativeHierarchyPositionChange, NativeNormalSubatoms,
        NativeReparentResult, NativeRunawayReparentResult, ReparentRegistrationContext,
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
    /// Direct-ID or explicitly resolved stable object nodes. Resolution policy
    /// stays outside the receiver; it does not choose an atom-map fallback.
    pub graph: &'a mut NativeNormalSubatoms,
    pub runaways: &'a mut NativeRunaways,
    pub registrations: &'a mut RegistrationStore,
    pub owner: &'a mut O,
}

impl<O: ReparentRegistrationContext> NativePositionReceiver<'_, O> {
    /// Direct-ID convenience path. Use apply_resolved_at when a caller's native
    /// lookup has explicitly established a different object handle.
    pub fn apply_at(
        &mut self,
        prefix: &NativeAtomPrefix<'_>,
        destination: DestinationContext,
        now: Instant,
    ) -> io::Result<Option<NativePositionUpdate>> {
        self.apply_resolved_at(prefix, prefix.base.id, destination, now)
    }

    /// Apply only the decoded fresh base-position stage. An absent position is
    /// a no-op, not a body/lifecycle acknowledgement. Unknown nodes are explicit
    /// errors before mutation. Existing object kind is retained even if the
    /// incoming opcode constructs a different temporary candidate class: native
    /// ReadPacket applies the common base stage to the object found by ID.
    /// RxUpdateBase uses the position-only ChangePos path, not orientation
    /// extraction. No timestamp filtering or source-profile fallback is added.
    pub fn apply_resolved_at(
        &mut self,
        prefix: &NativeAtomPrefix<'_>,
        handle: u32,
        destination: DestinationContext,
        now: Instant,
    ) -> io::Result<Option<NativePositionUpdate>> {
        let Some(record) = &prefix.base.position else {
            return Ok(None);
        };
        let id = prefix.base.id;
        if id == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "native position receiver rejects null atom ID",
            ));
        }
        self.graph.node(handle).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "native position receiver requires an existing object node",
            )
        })?;
        let outcome = match self.graph.apply_fresh_position_at(handle, record, now)? {
            NativeHierarchyPositionChange::Applied { cleared_caches } => {
                NativePositionUpdate::Local { cleared_caches }
            }
            NativeHierarchyPositionChange::ReparentRequired(position) => match destination {
                DestinationContext::Unresolved => NativePositionUpdate::ReparentRequired(position),
                DestinationContext::ExistingParentLists => NativePositionUpdate::NormalOwner(
                    self.graph.change_parent_in_available_lists_at(
                        handle,
                        record,
                        false,
                        now,
                        |graph, handle| {
                            self.registrations
                                .apply_tick_tock(id, &mut self.owner.context(graph, handle))
                        },
                    )?,
                ),
                DestinationContext::MissingDestinationLock => {
                    NativePositionUpdate::MissingDestination(
                        self.graph
                            .change_parent_to_missing_destination_for_atom_at(
                                NativeAtomIdentity {
                                    wire_id: id,
                                    handle,
                                },
                                record,
                                false,
                                now,
                                self.runaways,
                                |graph, _, handle| {
                                    self.registrations
                                        .apply_tick_tock(id, &mut self.owner.context(graph, handle))
                                },
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
