use super::*;
use crate::world::native_atom::NativeAtomKind;

fn graph(old: u32) -> NativeNormalSubatoms {
    let mut position = NativePositionState::default();
    position.position.parent = old;
    position.environment_cache_valid = true;
    let mut child = position.clone();
    child.position.parent = 1;
    NativeNormalSubatoms::new(
        [
            (
                1,
                NativeCacheNode {
                    kind: NativeAtomKind::Item,
                    field_1c: 0,
                    position,
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
                old,
                NativeChildList {
                    available: true,
                    children: vec![1],
                },
            ),
            (
                7,
                NativeChildList {
                    available: true,
                    children: vec![],
                },
            ),
            (
                1,
                NativeChildList {
                    available: true,
                    children: vec![2],
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
fn moves_indexes_then_copies_then_calls_tick_tock_then_clears_descendants() {
    let mut graph = graph(0);
    let now = Instant::now();
    let result = graph
        .change_parent_in_available_lists_at(1, &record(), true, now, |graph, id| {
            assert_eq!(id, 1);
            assert!(!graph.lists[&0].children.contains(&1));
            assert!(graph.lists[&7].children.contains(&1));
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
        })
        .unwrap();
    assert_eq!(
        result,
        NativeReparentResult::Applied {
            cleared_caches: BTreeSet::from([1, 2]),
            tick_tock_called: true
        }
    );
}

#[test]
fn nonnull_parent_skips_callback_and_position_only_retains_orientation() {
    let mut graph = graph(6);
    let orientation = Some([1., 2., 3.]);
    graph.nodes.get_mut(&1).unwrap().position.orientation = orientation;
    let result = graph
        .change_parent_in_available_lists_at(1, &record(), false, Instant::now(), |_, _| {
            panic!("old parent nonnull")
        })
        .unwrap();
    assert!(matches!(
        result,
        NativeReparentResult::Applied {
            tick_tock_called: false,
            ..
        }
    ));
    assert_eq!(graph.node(1).unwrap().position.orientation, orientation);
    assert!(!graph.lists[&6].children.contains(&1));
    assert!(graph.lists[&7].children.contains(&1));
}

#[test]
fn absent_context_and_bad_geometry_do_not_move_indexes_or_call_callbacks() {
    let mut graph = graph(0);
    graph.lists.get_mut(&7).unwrap().available = false;
    let before = graph.nodes.clone();
    assert_eq!(
        graph
            .change_parent_in_available_lists_at(
                1,
                &record(),
                false,
                Instant::now(),
                |_, _| panic!("no owner context")
            )
            .unwrap(),
        NativeReparentResult::OwnerContextRequired {
            old_parent: 0,
            new_parent: 7
        }
    );
    let mut bad = record();
    bad.position[0] = f64::NAN;
    assert!(
        graph
            .change_parent_in_available_lists_at(1, &bad, true, Instant::now(), |_, _| panic!(
                "bad geometry"
            ))
            .is_err()
    );
    assert_eq!(graph.nodes, before);
    assert!(graph.lists[&0].children.contains(&1));
    assert!(!graph.lists[&7].children.contains(&1));
}

#[test]
fn unchanged_null_parent_still_restarts_timer_and_calls_tick_tock_without_index_moves() {
    let mut graph = graph(0);
    let mut record = record();
    record.parent = None;
    let indexes = graph
        .lists
        .iter()
        .map(|(&id, list)| (id, list.children.clone()))
        .collect::<BTreeMap<_, _>>();
    let now = Instant::now();
    let mut calls = 0;
    let result = graph
        .change_parent_in_available_lists_at(1, &record, false, now, |_, _| {
            calls += 1;
            Ok(())
        })
        .unwrap();
    assert_eq!(calls, 1);
    assert!(matches!(
        result,
        NativeReparentResult::Applied {
            tick_tock_called: true,
            ..
        }
    ));
    assert_eq!(
        graph.node(1).unwrap().position.position_update_age_at(now),
        Some(std::time::Duration::ZERO)
    );
    assert_eq!(
        graph
            .lists
            .iter()
            .map(|(&id, list)| (id, list.children.clone()))
            .collect::<BTreeMap<_, _>>(),
        indexes
    );
}

#[test]
fn callback_failure_keeps_prior_changes_and_still_invalidates_caches() {
    let mut graph = graph(0);
    assert!(
        graph
            .change_parent_in_available_lists_at(1, &record(), false, Instant::now(), |_, _| Err(
                invalid("synthetic registration failure")
            ))
            .is_err()
    );
    assert_eq!(graph.node(1).unwrap().position.position.parent, 7);
    assert!(graph.lists[&7].children.contains(&1));
    assert!(!graph.node(1).unwrap().position.environment_cache_valid);
    assert!(!graph.node(2).unwrap().position.environment_cache_valid);
}
