//! Synthetic dependency observations; not native scheduler/destructor emulation.
use super::*;

struct Cleanup<'a> {
    graph: &'a mut NativeNormalSubatoms,
    lookup: &'a NativeRecoveryLookup,
    selected: Vec<u32>,
    protected: bool,
}
impl NativeRetirementCleanupContext for Cleanup<'_> {
    fn elapsed_reference_millis(&mut self, _: u32, _: Instant, _: Instant) -> i64 {
        20000 // Explicit boundary observation, not an Instant/Qt clock oracle.
    }
    fn in_tick_tock_queue(&mut self, handle: u32, retired: &NativeRetiredAtoms) -> bool {
        assert_eq!(handle, 70);
        assert!(retired.started_at(handle).is_some());
        self.protected // Explicit synthetic queue observation, not registration presence.
    }
    fn is_null_object(&mut self, handle: u32) -> bool {
        assert!(self.graph.node(handle).is_some());
        false
    }
    fn find_same_object_by_id(&mut self, handle: u32) -> bool {
        self.lookup.resolve(7) == Some(handle)
    }
    fn containing_parent_id(&mut self, handle: u32) -> Option<u32> {
        self.graph.lists.iter().find_map(|(&parent, list)| {
            (list.available && list.children.contains(&handle)).then_some(parent)
        })
    }
    fn alternate_map_id(&mut self, handle: u32) -> Option<u32> {
        assert_ne!(self.lookup.resolve(7), Some(handle));
        None // Fixture has no other atom-map keys.
    }
    fn destroy(&mut self, handle: u32, remaining: usize, retired: &NativeRetiredAtoms) {
        assert_eq!(remaining, 0);
        assert!(retired.started_at(handle).is_none());
        // Test-only disposal of a plain graph node, not a derived native destructor.
        self.graph.nodes.remove(&handle).unwrap();
        self.selected.push(handle);
    }
}

#[test]
fn replacement_retention_then_protected_restart_then_disposal_keeps_new_object_alive() {
    let mut graph = NativeNormalSubatoms::new(
        [
            (70, node(NativeAtomKind::Item, 0)),
            (71, node(NativeAtomKind::Item, 0)),
        ],
        [(
            7,
            NativeChildList {
                available: true,
                children: vec![],
            },
        )],
    )
    .unwrap();
    let incoming = graph.node(71).cloned().unwrap();
    let mut lookup =
        NativeRecoveryLookup::from_routed_nodes([(7, 70, NativeAtomKind::Item)]).unwrap();
    let mut queue = NativeRunaways::default();
    let mut store = RegistrationStore::default();
    store
        .apply(RegistrationAction::UpdateOrAddTicker {
            id: 7,
            atom_type: 16,
            solar_id: 99,
        })
        .unwrap();
    let mut retired = NativeRetiredAtoms::default();
    let mut owner = Owner {
        events: Rc::new(RefCell::new(Vec::new())),
        owners: BTreeMap::from([(70, true), (71, false)]),
    };
    let now = Instant::now();
    graph
        .add_atom_owner(
            NativeAtomAddRequest {
                wire_id: 7,
                handle: 71,
                scan_all_memberships: false,
            },
            NativeAtomOwnerStores {
                lookup: &mut lookup,
                runaways: &mut queue,
                registrations: &mut store,
                retired: &mut retired,
            },
            &mut owner,
            || now,
        )
        .unwrap();
    let mut cleanup = Cleanup {
        graph: &mut graph,
        lookup: &lookup,
        selected: Vec::new(),
        protected: true,
    };
    let later = now + std::time::Duration::from_secs(20);
    let result = retired
        .delete_one_in_order(later, &[70], true, &mut cleanup)
        .unwrap();
    assert_eq!(result.selected, None);
    assert_eq!(result.restarted, [70]);
    assert_eq!(retired.started_at(70), Some(later));
    assert!(cleanup.graph.node(70).is_some());
    cleanup.protected = false;
    let result = retired
        .delete_one_in_order(
            later + std::time::Duration::from_secs(20),
            &[70],
            true,
            &mut cleanup,
        )
        .unwrap();
    assert_eq!(
        result.selected,
        Some((70, NativeRetirementDisposition::Destroyed))
    );
    assert_eq!(cleanup.selected, [70]);
    assert!(retired.is_empty());
    assert!(cleanup.graph.node(70).is_none());
    assert_eq!(cleanup.graph.node(71), Some(&incoming));
    assert!(cleanup.graph.lists[&7].available);
    assert_eq!(lookup.resolve(7), Some(71));
    assert!(owner.owners[&71]);
    assert!(store.ticker(7).unwrap().released);
}
