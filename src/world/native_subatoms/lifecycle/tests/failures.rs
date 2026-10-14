use super::*;

#[test]
fn validation_rejects_null_incoming_and_dangling_old_handle_before_callbacks() {
    let mut graph = NativeNormalSubatoms::new([(71, node(NativeAtomKind::Item, 0))], []).unwrap();
    let mut lookup = NativeRecoveryLookup::new([(7, 999)], []).unwrap();
    let mut queue = NativeRunaways::default();
    let mut store = RegistrationStore::default();
    let mut retired = NativeRetiredAtoms::default();
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut owner = Owner {
        events: events.clone(),
        owners: BTreeMap::new(),
    };
    for (wire_id, handle) in [(0, 71), (7, 999), (7, 71)] {
        assert!(
            graph
                .add_atom_owner(
                    NativeAtomAddRequest {
                        wire_id,
                        handle,
                        scan_all_memberships: false
                    },
                    NativeAtomOwnerStores {
                        lookup: &mut lookup,
                        runaways: &mut queue,
                        registrations: &mut store,
                        retired: &mut retired
                    },
                    &mut owner,
                    || panic!("invalid objects must not sample clock"),
                )
                .is_err()
        );
    }
    assert!(events.borrow().is_empty());
    assert!(retired.is_empty());
    assert_eq!(lookup.resolve(7), Some(999));
    assert!(graph.lists.is_empty());
}

#[test]
fn arrival_safety_error_retains_inserted_map_but_does_not_assign_owner() {
    let mut graph = NativeNormalSubatoms::new([(71, node(NativeAtomKind::Item, 9))], []).unwrap();
    let mut lookup = NativeRecoveryLookup::new([], []).unwrap();
    let mut queue = NativeRunaways::with_limit(1).unwrap();
    queue.add(8, 2).unwrap();
    let mut store = RegistrationStore::default();
    let mut retired = NativeRetiredAtoms::default();
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut owner = Owner {
        events: events.clone(),
        owners: BTreeMap::new(),
    };
    assert!(
        graph
            .add_atom_owner(
                NativeAtomAddRequest {
                    wire_id: 7,
                    handle: 71,
                    scan_all_memberships: false
                },
                NativeAtomOwnerStores {
                    lookup: &mut lookup,
                    runaways: &mut queue,
                    registrations: &mut store,
                    retired: &mut retired
                },
                &mut owner,
                || panic!("fresh add has no retirement clock"),
            )
            .is_err()
    );
    assert_eq!(lookup.resolve(7), Some(71));
    assert!(events.borrow().is_empty());
    assert!(graph.lists.is_empty());
    assert_eq!(queue.len(), 1);
    assert!(queue.contains(8, 2));
    assert!(retired.is_empty());
}

#[test]
fn same_handle_early_return_uses_only_selected_map_without_rebuilding_lists() {
    let mut graph =
        NativeNormalSubatoms::new([(71, node(NativeAtomKind::SolarSystem, 0))], []).unwrap();
    let mut lookup = NativeRecoveryLookup::new([(7, 999)], [(7, 71)]).unwrap();
    let mut queue = NativeRunaways::default();
    let mut store = RegistrationStore::default();
    let mut retired = NativeRetiredAtoms::default();
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut owner = Owner {
        events: events.clone(),
        owners: BTreeMap::new(),
    };
    assert_eq!(
        graph
            .add_atom_owner(
                NativeAtomAddRequest {
                    wire_id: 7,
                    handle: 71,
                    scan_all_memberships: false
                },
                NativeAtomOwnerStores {
                    lookup: &mut lookup,
                    runaways: &mut queue,
                    registrations: &mut store,
                    retired: &mut retired
                },
                &mut owner,
                || panic!("no-op clock"),
            )
            .unwrap(),
        NativeAtomAdd::AlreadyPresent
    );
    assert_eq!(lookup.resolve(7), Some(999));
    assert!(events.borrow().is_empty());
    assert!(graph.lists.is_empty());
}
