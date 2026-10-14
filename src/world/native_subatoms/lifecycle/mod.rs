//! AddAtomLocked over already constructed, explicitly identified object nodes.
use super::*;
use crate::world::{native_runaways::NativeRunaways, native_tick_tock::store::RegistrationStore};
use std::time::Instant;

/// Owner metadata remains external; native predicate getters are not guessed.
pub trait NativeAtomOwnerContext: ReparentRegistrationContext {
    fn detach_owner(&mut self, handle: u32);
    fn assign_owner(&mut self, graph: &NativeNormalSubatoms, handle: u32);
}
pub struct NativeAtomOwnerStores<'a> {
    pub lookup: &'a mut NativeRecoveryLookup,
    pub runaways: &'a mut NativeRunaways,
    pub registrations: &'a mut RegistrationStore,
    pub retired: &'a mut NativeRetiredAtoms,
}
#[derive(Clone, Copy, Debug)]
pub struct NativeAtomAddRequest {
    pub wire_id: u32,
    pub handle: u32,
    /// Explicit global 0x49a bit 0x10 used by old-object removal.
    pub scan_all_memberships: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeAtomAdd {
    AlreadyPresent,
    Established {
        replaced: Option<(u32, NativeAtomRemoval)>,
        arrival: NativeFreshOwnerEstablished,
    },
}

impl NativeNormalSubatoms {
    /// Recovered selected-map add/replace sequence. Incoming objects must already
    /// exist in the graph; this does not deserialize or invent constructors.
    /// Samples `retirement_clock` only after old-owner removal. On Rust errors,
    /// completed stages remain applied, with no fabricated rollback. Native
    /// locking, diagnostics and eventual retained-object destruction are separate.
    pub fn add_atom_owner(
        &mut self,
        request: NativeAtomAddRequest,
        stores: NativeAtomOwnerStores<'_>,
        owner: &mut impl NativeAtomOwnerContext,
        retirement_clock: impl FnOnce() -> Instant,
    ) -> io::Result<NativeAtomAdd> {
        let NativeAtomAddRequest {
            wire_id,
            handle,
            scan_all_memberships,
        } = request;
        if wire_id == 0 {
            return Err(invalid("native atom registration rejects null ID"));
        }
        let kind = self
            .nodes
            .get(&handle)
            .ok_or_else(|| invalid("unknown native incoming object handle"))?
            .kind;
        let existing = stores.lookup.routed_entry(wire_id, kind);
        if existing == Some(handle) {
            return Ok(NativeAtomAdd::AlreadyPresent);
        }
        let replaced = if let Some(old_handle) = existing {
            let removal = self.remove_atom_owner(
                wire_id,
                old_handle,
                stores.lookup,
                stores.registrations,
                scan_all_memberships,
                |old| owner.detach_owner(old),
            )?;
            let started = retirement_clock();
            self.retire_replaced_atom_at(
                wire_id,
                old_handle,
                started,
                stores.retired,
                stores.registrations,
                owner,
            )?;
            Some((old_handle, removal))
        } else {
            None
        };
        stores.lookup.insert_routed(wire_id, handle, kind)?;
        let arrival = self.establish_fresh_atom_owner_with_assignment(
            wire_id,
            handle,
            stores.runaways,
            stores.lookup,
            |graph, incoming| owner.assign_owner(graph, incoming),
        )?;
        Ok(NativeAtomAdd::Established { replaced, arrival })
    }
}

#[cfg(test)]
mod tests;
