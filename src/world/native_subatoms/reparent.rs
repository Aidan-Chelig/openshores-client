use super::*;
use crate::atom_position::PackedPosition;
use std::time::Instant;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeReparentResult {
    /// Required parent-lock/child-set context was not supplied. No mutations;
    /// native runaway/recovery branches must not be replaced by guessed edges.
    OwnerContextRequired { old_parent: u32, new_parent: u32 },
    Applied {
        cleared_caches: BTreeSet<u32>,
        tick_tock_called: bool,
    },
}

impl NativeNormalSubatoms {
    /// Established normal ChangeParent branch with both existing owner lists.
    /// Callback runs after index moves/copy/timer, before cache invalidation,
    /// only when OLD parent was null. Caller implements actual TickTock effects.
    /// This does not recover missing owners, runaways, secondary atom maps,
    /// diagnostics-driven misplaced-set repair or concurrent native locks.
    pub fn change_parent_in_available_lists_at(
        &mut self,
        id: u32,
        record: &PackedPosition,
        extract_orientation: bool,
        now: Instant,
        mut tick_tock: impl FnMut(&mut Self, u32) -> io::Result<()>,
    ) -> io::Result<NativeReparentResult> {
        let node = self
            .nodes
            .get(&id)
            .ok_or_else(|| invalid("unknown native subatom root"))?;
        let old_parent = node.position.position.parent;
        // Rust safety: all geometry validation happens before index mutation.
        let mut position = node.position.clone();
        let effects = if extract_orientation {
            position.set_fresh_position_and_orientation_by_ether_at(record, now)?
        } else {
            position.set_fresh_position_by_ether_at(record, now)?
        };
        let new_parent = position.position.parent;
        if [old_parent, new_parent]
            .into_iter()
            .any(|parent| !self.lists.get(&parent).is_some_and(|list| list.available))
        {
            return Ok(NativeReparentResult::OwnerContextRequired {
                old_parent,
                new_parent,
            });
        }
        if old_parent != new_parent {
            let removing = self.lists[&old_parent].children.contains(&id);
            let inserting = !self.lists[&new_parent].children.contains(&id);
            if inserting
                && !removing
                && self
                    .lists
                    .values()
                    .map(|list| list.children.len())
                    .sum::<usize>()
                    >= MAX_LINKS
            {
                return Err(invalid("native subatoms exceed link budget"));
            }
            self.lists
                .get_mut(&old_parent)
                .expect("checked owner")
                .children
                .remove(&id);
            self.lists
                .get_mut(&new_parent)
                .expect("checked owner")
                .children
                .insert(id);
        }
        self.nodes.get_mut(&id).expect("checked node").position = position;
        // Callback failures are Rust policy. Preserve earlier successful moves
        // and still fulfill cache cleanup; no invented rollback of native state.
        let callback = if effects.tick_tock_required {
            tick_tock(self, id)
        } else {
            Ok(())
        };
        let cleared_caches = self.invalidate_environment(id)?;
        callback?;
        Ok(NativeReparentResult::Applied {
            cleared_caches,
            tick_tock_called: effects.tick_tock_required,
        })
    }
}

#[cfg(test)]
#[path = "reparent_tests.rs"]
mod tests;
