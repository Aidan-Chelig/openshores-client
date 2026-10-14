//! DeleteTrashAtom selection and conditional guards, using explicit observations.
use super::*;

/// Observations of native dependencies. Queue membership is not inferred from
/// registration records, nor null pointers from a numeric graph handle.
pub trait NativeRetirementCleanupContext {
    /// Signed difference of reference-millisecond observations, not an assumed
    /// floor of Instant elapsed time. The call epoch is sampled once by caller.
    fn elapsed_reference_millis(
        &mut self,
        handle: u32,
        started: Instant,
        call_epoch: Instant,
    ) -> i64;
    fn in_tick_tock_queue(&mut self, handle: u32, retired: &NativeRetiredAtoms) -> bool;
    fn is_null_object(&mut self, handle: u32) -> bool;
    fn find_same_object_by_id(&mut self, handle: u32) -> bool;
    /// First containing child set reached through native parent-lock iteration.
    fn containing_parent_id(&mut self, handle: u32) -> Option<u32>;
    /// Pointer scan of the object's InStarField-selected map, not both maps.
    fn alternate_map_id(&mut self, handle: u32) -> Option<u32>;
    /// Required disposal effect after retention erasure and guard checks. Native
    /// derived destructor implementations remain external to this adapter.
    fn destroy(&mut self, handle: u32, remaining: usize, retired: &NativeRetiredAtoms);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeRetirementDisposition {
    NullObject,
    StillRegisteredById,
    StillInParent(u32),
    StillRegisteredUnderOtherId(u32),
    Destroyed,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeRetirementCleanup {
    pub remaining: usize,
    pub restarted: Vec<u32>,
    pub selected: Option<(u32, NativeRetirementDisposition)>,
}

impl NativeRetiredAtoms {
    /// DeleteTrashAtom with caller-established native hash iteration order.
    /// Order must cover each current handle once; Rust preflight validates it
    /// before any getters/effects. Restarts queued old objects, selects at most
    /// one unqueued old object, and erases it even when diagnostics block disposal.
    /// `check_references` is explicit global 0x49a bit 0x08. No index repair,
    /// requeue, BTree-order substitution or derived destructor is invented.
    pub fn delete_one_in_order(
        &mut self,
        call_epoch: Instant,
        order: &[u32],
        check_references: bool,
        context: &mut impl NativeRetirementCleanupContext,
    ) -> io::Result<NativeRetirementCleanup> {
        if order.len() != self.started.len()
            || order.iter().copied().collect::<BTreeSet<_>>()
                != self.started.keys().copied().collect()
        {
            return Err(invalid(
                "native retirement order must cover every handle exactly once",
            ));
        }
        let mut restarted = Vec::new();
        for &handle in order {
            let started = self.started[&handle];
            if context.elapsed_reference_millis(handle, started, call_epoch) <= 19_999 {
                continue;
            }
            if context.in_tick_tock_queue(handle, self) {
                self.started.insert(handle, call_epoch);
                restarted.push(handle);
                continue;
            }
            self.started.remove(&handle);
            let remaining = self.started.len();
            let disposition = if context.is_null_object(handle) {
                NativeRetirementDisposition::NullObject
            } else if check_references && context.find_same_object_by_id(handle) {
                NativeRetirementDisposition::StillRegisteredById
            } else {
                let parent = if check_references {
                    context.containing_parent_id(handle).unwrap_or(0)
                } else {
                    0
                };
                if parent != 0 {
                    NativeRetirementDisposition::StillInParent(parent)
                } else {
                    let alternate = if check_references {
                        context.alternate_map_id(handle).unwrap_or(0)
                    } else {
                        0
                    };
                    if alternate != 0 {
                        NativeRetirementDisposition::StillRegisteredUnderOtherId(alternate)
                    } else {
                        context.destroy(handle, remaining, self);
                        NativeRetirementDisposition::Destroyed
                    }
                }
            };
            return Ok(NativeRetirementCleanup {
                remaining,
                restarted,
                selected: Some((handle, disposition)),
            });
        }
        Ok(NativeRetirementCleanup {
            remaining: self.started.len(),
            restarted,
            selected: None,
        })
    }
}

#[cfg(test)]
mod tests;
