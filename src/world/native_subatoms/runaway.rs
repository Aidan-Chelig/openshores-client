use super::*;
use crate::{atom_position::PackedPosition, world::native_runaways::NativeRunaways};
use std::time::Instant;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeRunawayReparentResult {
    pub parent: u32,
    pub inserted_pair: bool,
    pub cleared_caches: BTreeSet<u32>,
    pub tick_tock_called: bool,
}

impl NativeNormalSubatoms {
    /// Caller-established missing DESTINATION parent lock path. Do not infer
    /// that lock absence merely from a scene atom or child-set lookup failure.
    /// Contradictory available owner context is rejected. Queue pair first,
    /// copy/restart next, then optional TickTock and environment invalidation.
    /// Old child-set membership is deliberately retained, matching this branch.
    /// This does not attach to the missing parent or execute FixRunaways.
    pub fn change_parent_to_missing_destination_at(
        &mut self,
        id: u32,
        record: &PackedPosition,
        extract_orientation: bool,
        now: Instant,
        runaways: &mut NativeRunaways,
        mut tick_tock: impl FnMut(&mut Self, &NativeRunaways, u32) -> io::Result<()>,
    ) -> io::Result<NativeRunawayReparentResult> {
        let (position, effects) =
            self.prepare_ether_position(id, record, extract_orientation, now)?;
        let parent = position.position.parent;
        if self.lists.get(&parent).is_some_and(|list| list.available) {
            return Err(invalid("destination has available owner context"));
        }
        let inserted_pair = runaways.add(parent, id)?;
        let (cleared_caches, tick_tock_called) =
            self.finish_ether_position(id, position, effects, |graph, id| {
                tick_tock(graph, runaways, id)
            })?;
        Ok(NativeRunawayReparentResult {
            parent,
            inserted_pair,
            cleared_caches,
            tick_tock_called,
        })
    }
}

#[cfg(test)]
#[path = "runaway_tests.rs"]
mod tests;
