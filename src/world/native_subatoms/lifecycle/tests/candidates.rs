use super::*;
use crate::world::native_item::NativeItemInitialization;

#[test]
fn dynamically_constructed_item_establishes_owner_and_reused_id_discards_only_candidate() {
    let mut graph = NativeNormalSubatoms::new([], []).unwrap();
    let mut item = NativeItemInitialization::from_default_item_constructor();
    item.base
        .apply_received_identity(crate::world::native_prefix::NativeIdentityPrefix {
            id: 7,
            timestamp_ms: -123,
        });
    graph
        .insert_candidate(
            71,
            NativeCacheNode {
                kind: NativeAtomKind::Item,
                field_1c: item.base.field_1c,
                position: item.base.position.clone(),
            },
        )
        .unwrap();
    assert_eq!(graph.node(71).unwrap().field_1c, 5);
    graph.finish_received_position_stage(71).unwrap();
    item.base.finish_received_position_stage();
    let mut lookup = NativeRecoveryLookup::new([], []).unwrap();
    let mut queue = NativeRunaways::default();
    let mut registrations = RegistrationStore::default();
    let mut retired = NativeRetiredAtoms::default();
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut owner = Owner {
        events: events.clone(),
        owners: BTreeMap::from([(71, false)]),
    };
    assert!(lookup.find_atom(7).is_none());
    let outcome = graph
        .add_atom_owner(
            NativeAtomAddRequest {
                wire_id: item.base.wire_id,
                handle: 71,
                scan_all_memberships: false,
            },
            NativeAtomOwnerStores {
                lookup: &mut lookup,
                runaways: &mut queue,
                registrations: &mut registrations,
                retired: &mut retired,
            },
            &mut owner,
            || panic!("fresh object must not start retirement"),
        )
        .unwrap();
    assert!(matches!(
        outcome,
        NativeAtomAdd::Established { replaced: None, .. }
    ));
    assert_eq!(*events.borrow(), ["assign"]);
    assert!(owner.owners[&71]);
    assert_eq!(lookup.find_atom(7), Some(71));
    assert_eq!(graph.node(71).unwrap().field_1c, 0);
    assert!(retired.is_empty());
    assert!(!registrations.in_tick_tock_queue(7));

    // The next packet constructs a temporary before resolving the same ID.
    // Existing selection does not invoke AddAtomLocked's replacement branch.
    let candidate = NativeItemInitialization::from_default_item_constructor();
    graph
        .insert_candidate(
            72,
            NativeCacheNode {
                kind: NativeAtomKind::Item,
                field_1c: candidate.base.field_1c,
                position: candidate.base.position,
            },
        )
        .unwrap();
    assert_eq!(lookup.find_atom(7), Some(71));
    graph
        .discard_candidate(72, false, &lookup, &retired)
        .unwrap();
    assert!(graph.node(72).is_none());
    assert!(graph.node(71).is_some());
    assert_eq!(lookup.find_atom(7), Some(71));
    assert!(graph.lists[&7].available);
    assert!(owner.owners[&71]);
    assert_eq!(*events.borrow(), ["assign"]);
}
