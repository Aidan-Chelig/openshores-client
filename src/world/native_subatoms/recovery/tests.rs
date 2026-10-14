use super::*;
use crate::world::native_atom::NativeAtomKind;
fn graph(available: bool) -> NativeNormalSubatoms {
    let nodes = [(1, 7), (2, 7), (3, 8), (4, 8)].map(|(id, parent)| {
        let mut position = NativePositionState::default();
        position.position.parent = parent;
        position.environment_cache_valid = true;
        (
            id,
            NativeCacheNode {
                kind: NativeAtomKind::Item,
                field_1c: 0,
                position,
            },
        )
    });
    NativeNormalSubatoms::new(
        nodes,
        [
            (
                7,
                NativeChildList {
                    available,
                    children: vec![],
                },
            ),
            (
                0,
                NativeChildList {
                    available: true,
                    children: vec![1],
                },
            ),
        ],
    )
    .unwrap()
}

#[test]
fn batch_is_consumed_missing_and_stale_children_are_not_requeued_and_primary_wins() {
    let mut graph = graph(true);
    let before = graph.nodes.clone();
    let mut queue = NativeRunaways::default();
    for child in [10, 20, 30, 40, 50] {
        queue.add(7, child).unwrap();
    }
    queue.add(8, 10).unwrap();
    let lookup =
        NativeRecoveryLookup::new([(10, 1), (30, 3), (40, 4)], [(20, 2), (40, 1)]).unwrap();
    let result = graph.fix_runaways_in_maps(7, &mut queue, &lookup).unwrap();
    assert_eq!(
        result.consumed_children,
        BTreeSet::from([10, 20, 30, 40, 50])
    );
    assert_eq!(result.inserted_nodes, BTreeSet::from([1, 2]));
    assert_eq!(graph.lists[&7].children, BTreeSet::from([1, 2]));
    assert!(graph.lists[&0].children.contains(&1));
    assert_eq!(graph.nodes, before); // no movement/timer/cache effects
    assert_eq!(queue.len(), 1);
    assert!(queue.contains(8, 10));
    queue.add(7, 10).unwrap();
    assert!(
        graph
            .fix_runaways_in_maps(7, &mut queue, &lookup)
            .unwrap()
            .inserted_nodes
            .is_empty()
    );
}

#[test]
fn unavailable_owner_consumes_batch_without_creating_edges_and_bad_handle_is_atomic() {
    let mut graph = graph(false);
    let mut queue = NativeRunaways::default();
    queue.add(7, 10).unwrap();
    let before = queue.clone();
    let bad = NativeRecoveryLookup::new([(10, 99)], []).unwrap();
    assert!(graph.fix_runaways_in_maps(7, &mut queue, &bad).is_err());
    assert_eq!(queue, before);
    let lookup = NativeRecoveryLookup::new([], [(10, 1)]).unwrap();
    assert_eq!(
        graph
            .fix_runaways_in_maps(7, &mut queue, &lookup)
            .unwrap()
            .consumed_children,
        BTreeSet::from([10])
    );
    assert!(queue.is_empty());
    assert!(graph.lists[&7].children.is_empty());
    queue.add(9, 10).unwrap();
    graph.fix_runaways_in_maps(9, &mut queue, &lookup).unwrap();
    assert!(!graph.lists.contains_key(&9));
    assert!(queue.is_empty());
}

#[test]
fn lookup_rejects_duplicates_but_allows_explicit_overlap_between_maps() {
    assert!(NativeRecoveryLookup::new([(1, 2), (1, 3)], []).is_err());
    assert!(NativeRecoveryLookup::new([], [(1, 2), (1, 3)]).is_err());
    let lookup = NativeRecoveryLookup::new([(1, 2)], [(1, 3)]).unwrap();
    assert_eq!(lookup.resolve(1), Some(2));
    assert_eq!(lookup.resolve(9), None);
}
