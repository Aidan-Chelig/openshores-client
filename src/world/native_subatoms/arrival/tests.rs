use super::*;
use crate::world::native_atom::NativeAtomKind;

fn node(kind: NativeAtomKind, parent: u32) -> NativeCacheNode {
    let mut position = NativePositionState::default();
    position.position.parent = parent;
    position.environment_cache_valid = true;
    NativeCacheNode {
        kind,
        field_1c: 0,
        position,
    }
}
fn graph(parent: u32) -> NativeNormalSubatoms {
    NativeNormalSubatoms::new(
        [
            (1, node(NativeAtomKind::Item, 7)),
            (3, node(NativeAtomKind::Item, 8)),
            (7, node(NativeAtomKind::SolarSystem, parent)),
        ],
        [],
    )
    .unwrap()
}
fn lookup() -> NativeRecoveryLookup {
    NativeRecoveryLookup::from_routed_nodes([
        (1, 1, NativeAtomKind::Item),
        (3, 3, NativeAtomKind::Item),
        (7, 7, NativeAtomKind::SolarSystem),
    ])
    .unwrap()
}

#[test]
fn fresh_parent_establishes_owner_before_recovery_without_movement_or_cache_effects() {
    let mut graph = graph(0);
    let before = graph.nodes.clone();
    let mut queue = NativeRunaways::default();
    for child in [1, 3, 99] {
        queue.add(7, child).unwrap();
    }
    queue.add(8, 1).unwrap();
    let result = graph
        .establish_fresh_atom_owner(7, 7, &mut queue, &lookup())
        .unwrap();
    assert_eq!(result.queued_for_parent, None);
    assert!(!result.parent_membership_inserted);
    assert_eq!(
        result.recovery.consumed_children,
        BTreeSet::from([1, 3, 99])
    );
    assert_eq!(result.recovery.inserted_nodes, BTreeSet::from([1]));
    assert!(graph.lists[&7].available);
    assert_eq!(graph.lists[&7].children, BTreeSet::from([1]));
    assert_eq!(graph.nodes, before);
    assert_eq!(queue.len(), 1);
    assert!(queue.contains(8, 1));
}

#[test]
fn parent_child_set_presence_not_parent_lock_controls_initial_attachment() {
    let mut graph = graph(8);
    graph.lists.insert(
        8,
        ChildSet {
            available: false,
            children: BTreeSet::new(),
        },
    );
    let mut queue = NativeRunaways::default();
    let result = graph
        .establish_fresh_atom_owner(7, 7, &mut queue, &lookup())
        .unwrap();
    assert!(result.parent_membership_inserted);
    assert_eq!(result.queued_for_parent, None);
    assert!(graph.lists[&8].children.contains(&7));
    assert!(queue.is_empty());
    assert!(!graph.lists[&8].available);
}

#[test]
fn missing_parent_is_queued_and_self_parent_pair_is_recovered_in_same_tail() {
    for parent in [8, 7] {
        let mut graph = graph(parent);
        let mut queue = NativeRunaways::default();
        let result = graph
            .establish_fresh_atom_owner(7, 7, &mut queue, &lookup())
            .unwrap();
        assert_eq!(result.queued_for_parent, Some(parent));
        if parent == 7 {
            assert!(queue.is_empty());
            assert_eq!(result.recovery.consumed_children, BTreeSet::from([7]));
            assert!(graph.lists[&7].children.contains(&7));
        } else {
            assert!(queue.contains(8, 7));
        }
    }
}

#[test]
fn invalid_lookup_queue_capacity_and_existing_owners_reject_without_mutation() {
    let mut graph = graph(8);
    let before = graph.nodes.clone();
    let mut queue = NativeRunaways::with_limit(1).unwrap();
    queue.add(9, 1).unwrap();
    let pairs = queue.clone();
    assert!(
        graph
            .establish_fresh_atom_owner(7, 7, &mut queue, &lookup())
            .is_err()
    );
    assert_eq!(queue, pairs);
    assert!(graph.lists.is_empty());
    assert_eq!(graph.nodes, before);
    assert!(
        graph
            .establish_fresh_atom_owner(0, 7, &mut queue, &lookup())
            .is_err()
    );
    let wrong_map = NativeRecoveryLookup::new([(7, 7)], []).unwrap();
    assert!(
        graph
            .establish_fresh_atom_owner(7, 7, &mut queue, &wrong_map)
            .is_err()
    );
    let mut queue = NativeRunaways::default();
    queue.add(7, 1).unwrap();
    let dangling = NativeRecoveryLookup::new([(1, 99)], [(7, 7)]).unwrap();
    assert!(
        graph
            .establish_fresh_atom_owner(7, 7, &mut queue, &dangling)
            .is_err()
    );
    assert!(queue.contains(7, 1));
    assert!(graph.lists.is_empty());
    graph
        .establish_fresh_atom_owner(7, 7, &mut queue, &lookup())
        .unwrap();
    let children = graph.lists[&7].children.clone();
    let pairs = queue.clone();
    assert!(
        graph
            .establish_fresh_atom_owner(7, 7, &mut queue, &lookup())
            .is_err()
    );
    assert_eq!(graph.lists[&7].children, children);
    assert_eq!(queue, pairs);
}
