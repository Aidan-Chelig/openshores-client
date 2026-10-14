use super::*;

impl NativeNormalSubatoms {
    /// Admit a caller-constructed candidate by its explicit stable object
    /// handle. Does not register its wire ID, infer parent memberships, create
    /// owner lists, or replace an existing object. The caller must supply the
    /// actual class's established construction state, not a guessed default.
    pub fn insert_candidate(&mut self, handle: u32, node: NativeCacheNode) -> io::Result<()> {
        if self.nodes.contains_key(&handle) {
            return Err(invalid("native candidate handle already exists"));
        }
        if self.nodes.len() >= MAX_NODES {
            return Err(invalid("native candidates exceed node budget"));
        }
        self.nodes.insert(handle, node);
        Ok(())
    }

    /// RxUpdateBase's common status write, after optional ChangePos has
    /// completed and before later base layers. No hierarchy/cache/timer changes.
    pub fn finish_received_position_stage(&mut self, handle: u32) -> io::Result<()> {
        let node = self
            .nodes
            .get_mut(&handle)
            .ok_or_else(|| invalid("unknown native receiving atom"))?;
        node.field_1c = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::native_item::NativeItemInitialization;
    fn node() -> NativeCacheNode {
        let item = NativeItemInitialization::from_default_item_constructor();
        NativeCacheNode {
            kind: NativeAtomKind::Item,
            field_1c: item.base.field_1c,
            position: item.base.position,
        }
    }
    #[test]
    fn candidate_admission_has_no_owner_indexes_and_never_replaces_handles() {
        let mut graph = NativeNormalSubatoms::new([], []).unwrap();
        graph.insert_candidate(90, node()).unwrap();
        assert_eq!(graph.node(90).unwrap().field_1c, 5);
        assert!(graph.lists.is_empty());
        let before = graph.node(90).unwrap().clone();
        assert!(
            graph
                .insert_candidate(
                    90,
                    NativeCacheNode {
                        field_1c: 6,
                        ..node()
                    }
                )
                .is_err()
        );
        assert_eq!(graph.node(90), Some(&before));
        graph.finish_received_position_stage(90).unwrap();
        assert_eq!(graph.node(90).unwrap().field_1c, 0);
        assert_eq!(graph.node(90).unwrap().position, before.position);
        assert!(graph.lists.is_empty());
        assert!(graph.finish_received_position_stage(91).is_err());
    }
    #[test]
    fn candidate_budget_failure_preserves_known_objects() {
        let mut graph =
            NativeNormalSubatoms::new((0..MAX_NODES as u32).map(|handle| (handle, node())), [])
                .unwrap();
        assert!(graph.insert_candidate(u32::MAX, node()).is_err());
        assert_eq!(graph.nodes.len(), MAX_NODES);
        assert!(graph.node(u32::MAX).is_none());
    }
}
