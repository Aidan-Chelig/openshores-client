use super::*;

impl NativeNormalSubatoms {
    /// Remove only an unowned temporary's graph storage. The adapter must
    /// establish owner absence and dispose other class fields separately.
    /// This is not a reconstructed C++ destructor or RemoveAtomLocked: wire
    /// maps, shared wire-ID owner lists and runaway queues remain untouched.
    pub fn discard_candidate(
        &mut self,
        handle: u32,
        has_owner: bool,
        lookup: &NativeRecoveryLookup,
        retired: &NativeRetiredAtoms,
    ) -> io::Result<NativeCacheNode> {
        if has_owner
            || lookup.references_handle(handle)
            || retired.started_at(handle).is_some()
            || self
                .lists
                .values()
                .any(|list| list.children.contains(&handle))
        {
            return Err(invalid("native candidate is still owned or referenced"));
        }
        self.nodes
            .remove(&handle)
            .ok_or_else(|| invalid("unknown native candidate handle"))
    }
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
    fn candidate_disposal_does_not_erase_retained_objects_with_shared_wire_id() {
        let mut graph = NativeNormalSubatoms::new(
            [(10, node()), (90, node())],
            [(
                7,
                NativeChildList {
                    available: true,
                    children: vec![10],
                },
            )],
        )
        .unwrap();
        let lookup = NativeRecoveryLookup::new([(7, 10)], [(7, 10)]).unwrap();
        let retired = NativeRetiredAtoms::default();
        let before = graph.node(10).unwrap().clone();
        graph
            .discard_candidate(90, false, &lookup, &retired)
            .unwrap();
        assert!(graph.node(90).is_none());
        assert_eq!(graph.node(10), Some(&before));
        assert_eq!(lookup.find_atom(7), Some(10));
        assert!(graph.lists[&7].children.contains(&10));
        assert!(
            graph
                .discard_candidate(10, false, &lookup, &retired)
                .is_err()
        );
        assert_eq!(graph.node(10), Some(&before));
    }
    #[test]
    fn owner_and_child_membership_block_disposal_without_mutation() {
        let mut graph = NativeNormalSubatoms::new(
            [(90, node())],
            [(
                7,
                NativeChildList {
                    available: false,
                    children: vec![90],
                },
            )],
        )
        .unwrap();
        let lookup = NativeRecoveryLookup::new([], []).unwrap();
        let retired = NativeRetiredAtoms::default();
        assert!(
            graph
                .discard_candidate(90, false, &lookup, &retired)
                .is_err()
        );
        graph.lists.clear();
        assert!(
            graph
                .discard_candidate(90, true, &lookup, &retired)
                .is_err()
        );
        assert!(graph.node(90).is_some());
        graph
            .discard_candidate(90, false, &lookup, &retired)
            .unwrap();
        assert!(
            graph
                .discard_candidate(90, false, &lookup, &retired)
                .is_err()
        );
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
