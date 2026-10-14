use super::{NativeNormalSubatoms, invalid};
use crate::atom_position::{FreshPosition, PackedPosition, PositionChange};
use std::{collections::BTreeSet, io, time::Instant};

#[derive(Clone, Debug, PartialEq)]
pub enum NativeHierarchyPositionChange {
    ReparentRequired(FreshPosition),
    /// Timer restarted. Descendant cache obligation is fulfilled for this
    /// explicit owner context; IDs report caches that were previously valid.
    Applied {
        cleared_caches: BTreeSet<u32>,
    },
}

impl NativeNormalSubatoms {
    fn children(&self, parent: u32) -> Vec<u32> {
        self.lists
            .get(&parent)
            .filter(|list| list.available)
            .map(|list| list.children.iter().copied().collect())
            .unwrap_or_default()
    }

    fn walk(&mut self, mut pending: Vec<(u32, bool)>) -> BTreeSet<u32> {
        let mut cleared = BTreeSet::new();
        while let Some((id, from_list)) = pending.pop() {
            let node = self
                .nodes
                .get_mut(&id)
                .expect("constructor validates child references");
            if from_list && (node.field_1c != 0 || node.kind.is_instance_universe()) {
                continue;
            }
            // Native gate clears BEFORE traversal, terminating cycles/shared
            // paths without recursion or an invented global visited predicate.
            if !node.position.invalidate_environment() {
                continue;
            }
            cleared.insert(id);
            pending.extend(self.children(id).into_iter().map(|id| (id, true)));
        }
        cleared
    }

    /// Direct native invalidator: root bypasses list eligibility filters.
    /// Normal child traversal occurs only when that root's cache was valid.
    pub fn invalidate_environment(&mut self, root: u32) -> io::Result<BTreeSet<u32>> {
        if !self.nodes.contains_key(&root) {
            return Err(invalid("unknown native subatom root"));
        }
        Ok(self.walk(vec![(root, false)]))
    }

    /// Fulfill an explicit already-cleared root's normal-descendant obligation.
    /// Call only when the preceding native cache gate returned true.
    pub fn invalidate_normal_descendants(&mut self, root: u32) -> io::Result<BTreeSet<u32>> {
        if !self.nodes.contains_key(&root) {
            return Err(invalid("unknown native subatom root"));
        }
        Ok(self.walk(
            self.children(root)
                .into_iter()
                .map(|id| (id, true))
                .collect(),
        ))
    }

    /// Local ChangePos branch with normal-descendant invalidation integrated.
    /// Managed parent changes remain explicit and leave all nodes/indexes intact.
    /// No TickTock, runaway handling, owner-index moves or timestamp filtering.
    pub fn apply_fresh_position_at(
        &mut self,
        id: u32,
        record: &PackedPosition,
        now: Instant,
    ) -> io::Result<NativeHierarchyPositionChange> {
        let node = self
            .nodes
            .get_mut(&id)
            .ok_or_else(|| invalid("unknown native subatom root"))?;
        match node
            .position
            .apply_fresh_wire_position_at(record, true, now)?
        {
            PositionChange::ReparentRequired(position) => {
                Ok(NativeHierarchyPositionChange::ReparentRequired(position))
            }
            PositionChange::Applied {
                invalidate_normal_descendants,
                ..
            } => {
                let mut cleared_caches = BTreeSet::new();
                if invalidate_normal_descendants {
                    cleared_caches.insert(id);
                    cleared_caches.extend(self.invalidate_normal_descendants(id)?);
                }
                Ok(NativeHierarchyPositionChange::Applied { cleared_caches })
            }
        }
    }
}
