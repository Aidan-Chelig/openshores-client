//! Stable-owner RemoveAtomLocked effects, not recursive object destruction.
use super::*;
use crate::world::native_tick_tock::{RegistrationAction, store::RegistrationStore};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeAtomRemoval {
    /// Entry erased by wire ID, which need not equal the supplied object handle.
    pub erased_map_handle: Option<u32>,
    pub removed_from_parent: bool,
    /// Unexpected memberships removed only when the explicit diagnostic flag is on.
    pub additional_parent_memberships: BTreeSet<u32>,
    /// Children released from the removed set, not destroyed or reparented.
    pub discarded_child_set: Option<BTreeSet<u32>>,
}

impl NativeNormalSubatoms {
    /// Remove established owner indexes for a caller-supplied object identity.
    /// `detach_owner` must clear that object's externally owned owner reference;
    /// this graph deliberately does not infer ownership from list membership.
    /// It runs before registration release and map/set removal, as in native.
    /// `scan_all_memberships` is the observed global 0x49a bit 0x10, not a default.
    /// The combined list representation drops its lock availability and child
    /// set together; separate lock allocation/destruction and Qt concurrency
    /// are not modeled. The object node and its children remain alive unchanged.
    pub fn remove_atom_owner(
        &mut self,
        wire_id: u32,
        handle: u32,
        lookup: &mut NativeRecoveryLookup,
        registrations: &mut RegistrationStore,
        scan_all_memberships: bool,
        detach_owner: impl FnOnce(u32),
    ) -> io::Result<NativeAtomRemoval> {
        let node = self
            .nodes
            .get(&handle)
            .ok_or_else(|| invalid("native removal references missing object handle"))?;
        let kind = node.kind;
        let parent = node.position.position.parent;
        // No wire-ID/pointer equality guard: native's selected-map erase is by ID.
        detach_owner(handle);
        registrations.apply(RegistrationAction::ReleaseTickTock { id: wire_id })?;
        let erased_map_handle = lookup.remove_routed(wire_id, kind);
        let removed_from_parent = self
            .lists
            .get_mut(&parent)
            .is_some_and(|list| list.children.remove(&handle));
        let mut additional_parent_memberships = BTreeSet::new();
        if scan_all_memberships {
            for (parent, list) in &mut self.lists {
                if list.children.remove(&handle) {
                    additional_parent_memberships.insert(*parent);
                }
            }
        }
        let discarded_child_set = self.lists.remove(&wire_id).map(|list| list.children);
        Ok(NativeAtomRemoval {
            erased_map_handle,
            removed_from_parent,
            additional_parent_memberships,
            discarded_child_set,
        })
    }
}

#[cfg(test)]
mod tests;
