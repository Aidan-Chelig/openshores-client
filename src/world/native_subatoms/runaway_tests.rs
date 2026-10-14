use super::*;
use crate::world::native_atom::NativeAtomKind;

fn fixture() -> NativeNormalSubatoms {
    let mut root = NativePositionState::default();
    root.environment_cache_valid = true;
    let mut child = root.clone();
    child.position.parent = 1;
    NativeNormalSubatoms::new(
        [
            (
                1,
                NativeCacheNode {
                    kind: NativeAtomKind::Item,
                    field_1c: 0,
                    position: root,
                },
            ),
            (
                2,
                NativeCacheNode {
                    kind: NativeAtomKind::Item,
                    field_1c: 0,
                    position: child,
                },
            ),
        ],
        [
            (
                0,
                NativeChildList {
                    available: true,
                    children: vec![1],
                },
            ),
            (
                1,
                NativeChildList {
                    available: true,
                    children: vec![2],
                },
            ),
            (
                7,
                NativeChildList {
                    available: false,
                    children: vec![],
                },
            ),
        ],
    )
    .unwrap()
}
fn record() -> PackedPosition {
    PackedPosition {
        flags: 0,
        parent: Some(7),
        position: [12345678.125, 2., 3.],
        rotation: [0.3, -0.7, 1.1],
    }
}

#[test]
fn queue_precedes_copy_callback_and_cache_cleanup_without_any_index_moves() {
    let mut graph = fixture();
    let mut queue = NativeRunaways::default();
    let now = Instant::now();
    let result = graph
        .change_parent_to_missing_destination_at(
            1,
            &record(),
            true,
            now,
            &mut queue,
            |graph, queue, id| {
                assert!(queue.contains(7, id));
                assert!(graph.lists[&0].children.contains(&id));
                assert!(!graph.lists[&7].children.contains(&id));
                let node = graph.node(id).unwrap();
                assert_eq!(node.position.position.parent, 7);
                assert_eq!(node.position.position.local[3][0], 12345678.125);
                assert!(node.position.orientation.is_some());
                assert_eq!(
                    node.position.position_update_age_at(now),
                    Some(std::time::Duration::ZERO)
                );
                assert!(node.position.environment_cache_valid);
                assert!(graph.node(2).unwrap().position.environment_cache_valid);
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(
        result,
        NativeRunawayReparentResult {
            parent: 7,
            inserted_pair: true,
            cleared_caches: BTreeSet::from([1, 2]),
            tick_tock_called: true
        }
    );
    let orientation = graph.node(1).unwrap().position.orientation;
    let result = graph
        .change_parent_to_missing_destination_at(1, &record(), false, now, &mut queue, |_, _, _| {
            panic!("old parent now nonnull")
        })
        .unwrap();
    assert!(!result.inserted_pair);
    assert!(!result.tick_tock_called);
    assert!(result.cleared_caches.is_empty());
    assert_eq!(queue.len(), 1);
    assert_eq!(graph.node(1).unwrap().position.orientation, orientation);
}

#[test]
fn queue_capacity_bad_geometry_and_contradictory_owner_are_atomic() {
    let mut graph = fixture();
    let mut queue = NativeRunaways::with_limit(1).unwrap();
    queue.add(8, 3).unwrap();
    let before = graph.nodes.clone();
    let pairs = queue.clone();
    assert!(
        graph
            .change_parent_to_missing_destination_at(
                1,
                &record(),
                false,
                Instant::now(),
                &mut queue,
                |_, _, _| panic!("queue full")
            )
            .is_err()
    );
    let mut bad = record();
    bad.rotation[0] = f32::NAN;
    assert!(
        graph
            .change_parent_to_missing_destination_at(
                1,
                &bad,
                true,
                Instant::now(),
                &mut queue,
                |_, _, _| panic!("invalid geometry")
            )
            .is_err()
    );
    graph.lists.get_mut(&7).unwrap().available = true;
    assert!(
        graph
            .change_parent_to_missing_destination_at(
                1,
                &record(),
                false,
                Instant::now(),
                &mut queue,
                |_, _, _| panic!("contradictory owner")
            )
            .is_err()
    );
    assert_eq!(queue, pairs);
    assert_eq!(graph.nodes, before);
    assert!(graph.lists[&0].children.contains(&1));
    assert!(!graph.lists[&7].children.contains(&1));
}

#[test]
fn callback_error_retains_queued_pair_and_updated_position_but_clears_caches() {
    let mut graph = fixture();
    let mut queue = NativeRunaways::default();
    assert!(
        graph
            .change_parent_to_missing_destination_at(
                1,
                &record(),
                false,
                Instant::now(),
                &mut queue,
                |_, _, _| Err(invalid("synthetic callback failure"))
            )
            .is_err()
    );
    assert!(queue.contains(7, 1));
    assert_eq!(graph.node(1).unwrap().position.position.parent, 7);
    assert!(graph.lists[&0].children.contains(&1));
    assert!(!graph.node(1).unwrap().position.environment_cache_valid);
    assert!(!graph.node(2).unwrap().position.environment_cache_valid);
}
