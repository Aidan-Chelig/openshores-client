mod lookup;
use super::*;
use crate::world::native_runaways::NativeRunaways;
pub use lookup::NativeRecoveryLookup;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeRunawayRecovery {
    /// Whole batch is consumed, including missing/stale/unavailable children.
    pub consumed_children: BTreeSet<u32>,
    /// Newly inserted node handles; already-present membership is idempotent.
    pub inserted_nodes: BTreeSet<u32>,
}

impl NativeNormalSubatoms {
    /// Stable-owner FixRunaways reconstruction with explicit two-map lookup.
    /// Does not alter position, timers, caches or registration records. It also
    /// does not remove stale old-parent membership or invent a missing set.
    /// Rust preflight rejects dangling handles/link overflow before consuming
    /// the queue; this safety boundary is not native concurrent lock behavior.
    pub fn fix_runaways_in_maps(
        &mut self,
        parent: u32,
        runaways: &mut NativeRunaways,
        lookup: &NativeRecoveryLookup,
    ) -> io::Result<NativeRunawayRecovery> {
        let mut inserted_nodes = BTreeSet::new();
        if let Some(children) = runaways.children(parent) {
            for child in children {
                let Some(handle) = lookup.resolve(*child) else {
                    continue;
                };
                let node = self
                    .nodes
                    .get(&handle)
                    .ok_or_else(|| invalid("native recovery lookup references missing node"))?;
                if node.position.position.parent == parent
                    && let Some(list) = self.lists.get(&parent).filter(|list| list.available)
                    && !list.children.contains(&handle)
                {
                    inserted_nodes.insert(handle);
                }
            }
        }
        let links = self
            .lists
            .values()
            .map(|list| list.children.len())
            .sum::<usize>();
        if inserted_nodes.len() > MAX_LINKS - links {
            return Err(invalid("native recovery exceeds link budget"));
        }
        // Native consumes its batch first, before repair. Preflight above uses
        // immutable stable-owner observations solely to enforce Rust safety.
        let consumed_children = runaways.take_for_recovery(parent);
        if !inserted_nodes.is_empty() {
            self.lists
                .get_mut(&parent)
                .expect("preflight established list")
                .children
                .extend(inserted_nodes.iter().copied());
        }
        Ok(NativeRunawayRecovery {
            consumed_children,
            inserted_nodes,
        })
    }
}

#[cfg(test)]
mod tests;
