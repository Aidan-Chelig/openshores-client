use super::tests::node;
use super::*;
use crate::world::native_tick_tock::{RegistrationAction, store::RegistrationStore};

#[test]
fn removal_then_arrival_assigns_new_owner_before_recovering_queued_children() {
    let mut graph = NativeNormalSubatoms::new(
        [
            (70, node(NativeAtomKind::SolarSystem, 0)),
            (71, node(NativeAtomKind::SolarSystem, 0)),
            (10, node(NativeAtomKind::Item, 7)),
            (11, node(NativeAtomKind::Item, 7)),
        ],
        [(
            7,
            NativeChildList {
                available: true,
                children: vec![10],
            },
        )],
    )
    .unwrap();
    let nodes = graph.nodes.clone();
    let entries = |parent_handle| {
        [
            (7, parent_handle, NativeAtomKind::SolarSystem),
            (1, 10, NativeAtomKind::Item),
            (2, 11, NativeAtomKind::Item),
        ]
    };
    let mut lookup = NativeRecoveryLookup::from_routed_nodes(entries(70)).unwrap();
    let mut queue = NativeRunaways::default();
    queue.add(7, 2).unwrap();
    let mut store = RegistrationStore::default();
    store
        .apply(RegistrationAction::UpdateOrAddTicker {
            id: 7,
            atom_type: 0x1c,
            solar_id: 7,
        })
        .unwrap();
    let mut owners = BTreeMap::from([(70, Some(42)), (71, None)]);
    let removed = graph
        .remove_atom_owner(7, 70, &mut lookup, &mut store, false, |handle| {
            owners.insert(handle, None);
        })
        .unwrap();
    assert_eq!(removed.discarded_child_set, Some(BTreeSet::from([10])));
    assert!(store.ticker(7).unwrap().released);
    assert!(queue.contains(7, 2));
    // Caller-established map insertion, not the intervening replacement steps.
    let lookup = NativeRecoveryLookup::from_routed_nodes(entries(71)).unwrap();
    let arrived = graph
        .establish_fresh_atom_owner_with_assignment(7, 71, &mut queue, &lookup, |graph, handle| {
            assert_eq!(handle, 71);
            assert!(graph.lists[&7].available);
            assert!(graph.lists[&7].children.is_empty());
            assert_eq!(owners[&70], None);
            owners.insert(handle, Some(42));
        })
        .unwrap();
    assert_eq!(owners[&71], Some(42));
    assert_eq!(arrived.recovery.inserted_nodes, BTreeSet::from([11]));
    // Previous membership alone does not recover a child: only queued IDs do.
    assert_eq!(graph.lists[&7].children, BTreeSet::from([11]));
    assert_eq!(graph.nodes, nodes);
    assert!(queue.is_empty());
    assert!(store.ticker(7).unwrap().released);
}

#[test]
fn invalid_arrival_does_not_assign_owner_or_change_hierarchy() {
    let mut graph =
        NativeNormalSubatoms::new([(71, node(NativeAtomKind::SolarSystem, 0))], []).unwrap();
    let mut queue = NativeRunaways::default();
    let lookup = NativeRecoveryLookup::new([], []).unwrap();
    assert!(
        graph
            .establish_fresh_atom_owner_with_assignment(7, 71, &mut queue, &lookup, |_, _| panic!(
                "assignment before failed preflight"
            ),)
            .is_err()
    );
    assert!(graph.lists.is_empty());
}
