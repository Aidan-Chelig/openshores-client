//! Replacement's old-object status, registration and timed-retention stage.
use super::*;
use crate::world::native_tick_tock::store::RegistrationStore;
use std::time::Instant;

/// Native owner 0x48 is keyed by object pointer, not wire ID. Stable graph
/// handles preserve that distinction. No expiry/destructor policy is invented.
#[derive(Clone, Debug, Default)]
pub struct NativeRetiredAtoms {
    started: BTreeMap<u32, Instant>,
}
impl NativeRetiredAtoms {
    pub fn started_at(&self, handle: u32) -> Option<Instant> {
        self.started.get(&handle).copied()
    }
    pub fn len(&self) -> usize {
        self.started.len()
    }
    pub fn is_empty(&self) -> bool {
        self.started.is_empty()
    }
    fn check_insert(&self, handle: u32) -> io::Result<()> {
        if self.started.len() >= MAX_NODES && !self.started.contains_key(&handle) {
            return Err(invalid("native retired atom store exceeds budget"));
        }
        Ok(())
    }
}

impl NativeNormalSubatoms {
    /// Explicit replacement stage AFTER caller has removed old owner indexes.
    /// `started` is the monotonic epoch sampled after removal, before status 6
    /// and TickTock. Context sees the updated old object and current registers.
    /// Record retention only after TickTock succeeds. Rust callback errors retain
    /// status/prior registration effects but stop before retention or new arrival.
    /// This is not AddAtomLocked selection, insertion or same-pointer handling.
    pub fn retire_replaced_atom_at(
        &mut self,
        wire_id: u32,
        handle: u32,
        started: Instant,
        retired: &mut NativeRetiredAtoms,
        registrations: &mut RegistrationStore,
        owner: &mut impl ReparentRegistrationContext,
    ) -> io::Result<()> {
        if !self.nodes.contains_key(&handle) {
            return Err(invalid(
                "native retirement references missing object handle",
            ));
        }
        retired.check_insert(handle)?;
        self.nodes
            .get_mut(&handle)
            .expect("validated object")
            .field_1c = 6;
        registrations.apply_tick_tock(wire_id, &mut owner.context(self, handle))?;
        retired.started.insert(handle, started);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
