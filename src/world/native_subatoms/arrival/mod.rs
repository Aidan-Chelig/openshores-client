use super::*;
use crate::world::native_runaways::NativeRunaways;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeFreshOwnerEstablished {
    pub parent_membership_inserted: bool,
    pub queued_for_parent: Option<u32>,
    pub recovery: NativeRunawayRecovery,
}

impl NativeNormalSubatoms {
    /// Fresh AddAtomLocked shared tail AFTER caller-established routed atom-map
    /// insertion. Atom identity/handle must already be in that explicit map.
    /// No replacement, same-pointer re-add, raw status defaults or object
    /// construction is inferred. Rejects existing own lists rather than
    /// pretending to implement RemoveAtomLocked/replacement cleanup.
    pub fn establish_fresh_atom_owner(
        &mut self,
        id: u32,
        handle: u32,
        runaways: &mut NativeRunaways,
        lookup: &NativeRecoveryLookup,
    ) -> io::Result<NativeFreshOwnerEstablished> {
        if id == 0 {
            return Err(invalid("native atom registration rejects null ID"));
        }
        let node = self
            .nodes
            .get(&handle)
            .ok_or_else(|| invalid("unknown native arriving atom"))?;
        if lookup.routed_entry(id, node.kind) != Some(handle) {
            return Err(invalid("arriving atom absent from its native routed map"));
        }
        if self.lists.contains_key(&id) {
            return Err(invalid(
                "existing owner list requires registration lifecycle context",
            ));
        }
        if self.lists.len() >= MAX_NODES {
            return Err(invalid("native owner lists exceed budget"));
        }
        let parent = node.position.position.parent;
        // AddAtomLocked checks child-set presence here, NOT its parent lock.
        let queued_for_parent =
            (parent != 0 && !self.lists.contains_key(&parent)).then_some(parent);
        if let Some(parent) = queued_for_parent {
            runaways.check_add(parent, id)?;
        }
        let parent_membership_inserted = parent != 0
            && self
                .lists
                .get(&parent)
                .is_some_and(|list| !list.children.contains(&handle));

        // Stable-owner Rust safety preflight; no mutation until all references
        // and final link growth are valid. Native has no equivalent rollback.
        let mut recovering = BTreeSet::new();
        let children = runaways
            .children(id)
            .into_iter()
            .flat_map(|set| set.iter().copied())
            .chain((queued_for_parent == Some(id)).then_some(id));
        for child in children {
            let Some(child_handle) = lookup.resolve(child) else {
                continue;
            };
            let child_node = self
                .nodes
                .get(&child_handle)
                .ok_or_else(|| invalid("native recovery lookup references missing node"))?;
            if child_node.position.position.parent == id {
                recovering.insert(child_handle);
            }
        }
        let links = self
            .lists
            .values()
            .map(|list| list.children.len())
            .sum::<usize>();
        if recovering.len() + usize::from(parent_membership_inserted) > MAX_LINKS - links {
            return Err(invalid("native owner arrival exceeds link budget"));
        }

        if let Some(parent) = queued_for_parent {
            runaways.add(parent, id)?;
        } else if parent_membership_inserted {
            self.lists
                .get_mut(&parent)
                .expect("preflight established parent set")
                .children
                .insert(handle);
        }
        self.lists.insert(
            id,
            ChildSet {
                available: true,
                children: BTreeSet::new(),
            },
        );
        // Establishing this list is the explicit modeled owner-availability
        // transition, not a guessed pointer field. Recovery sees it immediately.
        let recovery = self.fix_runaways_in_maps(id, runaways, lookup)?;
        Ok(NativeFreshOwnerEstablished {
            parent_membership_inserted,
            queued_for_parent,
            recovery,
        })
    }
}

#[cfg(test)]
mod tests;
