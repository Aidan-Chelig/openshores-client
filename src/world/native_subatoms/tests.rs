use super::*;
use crate::atom_position::PackedPosition;
use std::time::Instant;

fn node(kind: NativeAtomKind, field_1c: i32, valid: bool) -> NativeCacheNode {
    let mut position = NativePositionState::default();
    position.environment_cache_valid = valid;
    NativeCacheNode {
        kind,
        field_1c,
        position,
    }
}
fn list(available: bool, children: &[u32]) -> NativeChildList {
    NativeChildList {
        available,
        children: children.into(),
    }
}

#[test]
fn descendant_filter_is_not_the_environment_boundary_or_universe_opcode() {
    use NativeAtomKind::*;
    let nodes = [
        (1, node(Item, 0, true)),
        (2, node(Arena, 0, true)),
        (3, node(Designer, 0, true)),
        (4, node(Sector, 0, true)),
        (5, node(SolarSystem, 0, true)),
        (6, node(Universe, 0, true)),
        (7, node(Item, 1, true)),
        (8, node(Item, -1, true)),
        (9, node(Item, 0, true)),
    ];
    let mut graph = NativeNormalSubatoms::new(
        nodes,
        [
            (1, list(true, &[2, 3, 4, 5, 6, 7, 8])),
            (2, list(true, &[9])),
        ],
    )
    .unwrap();
    assert_eq!(
        graph.invalidate_environment(1).unwrap(),
        BTreeSet::from([1, 4, 5, 6])
    );
    for id in [2, 3, 7, 8, 9] {
        assert!(graph.node(id).unwrap().position.environment_cache_valid);
    }
    // Direct calls do not use the list's instance-universe exclusion.
    assert_eq!(
        graph.invalidate_environment(2).unwrap(),
        BTreeSet::from([2, 9])
    );
}

#[test]
fn cache_and_owner_availability_gates_stop_traversal_and_cycles_terminate() {
    let mut graph = NativeNormalSubatoms::new(
        [
            (1, node(NativeAtomKind::Item, 0, true)),
            (2, node(NativeAtomKind::Item, 0, false)),
            (3, node(NativeAtomKind::Item, 0, true)),
            (4, node(NativeAtomKind::Item, 0, true)),
        ],
        [
            (1, list(true, &[2, 4])),
            (2, list(true, &[3])),
            (4, list(false, &[3])),
        ],
    )
    .unwrap();
    assert_eq!(
        graph.invalidate_environment(1).unwrap(),
        BTreeSet::from([1, 4])
    );
    assert!(graph.node(3).unwrap().position.environment_cache_valid);
    let mut cyclic = NativeNormalSubatoms::new(
        [
            (1, node(NativeAtomKind::Item, 0, true)),
            (2, node(NativeAtomKind::Item, 0, true)),
        ],
        [(1, list(true, &[2])), (2, list(true, &[1]))],
    )
    .unwrap();
    assert_eq!(
        cyclic.invalidate_environment(1).unwrap(),
        BTreeSet::from([1, 2])
    );
    assert!(cyclic.invalidate_environment(1).unwrap().is_empty());
}

#[test]
fn local_position_update_fulfills_caches_but_reparent_and_bad_geometry_are_atomic() {
    let mut graph = NativeNormalSubatoms::new(
        [
            (1, node(NativeAtomKind::Item, 0, true)),
            (2, node(NativeAtomKind::Item, 0, true)),
        ],
        [(1, list(true, &[2]))],
    )
    .unwrap();
    let now = Instant::now();
    let mut record = PackedPosition {
        flags: 0,
        parent: Some(7),
        position: [12345678.125, -2., 3.],
        rotation: [0.3, -0.7, 1.1],
    };
    let before = graph.nodes.clone();
    assert!(matches!(
        graph.apply_fresh_position_at(1, &record, now).unwrap(),
        NativeHierarchyPositionChange::ReparentRequired(_)
    ));
    assert_eq!(graph.nodes, before);
    record.parent = None;
    assert_eq!(
        graph.apply_fresh_position_at(1, &record, now).unwrap(),
        NativeHierarchyPositionChange::Applied {
            cleared_caches: BTreeSet::from([1, 2])
        }
    );
    assert_eq!(
        graph.node(1).unwrap().position.position.local[3][0],
        12345678.125
    );
    assert_eq!(
        graph.node(2).unwrap().position.position_update_age_at(now),
        None
    );
    let before = graph.nodes.clone();
    record.rotation[0] = f32::NAN;
    assert!(graph.apply_fresh_position_at(1, &record, now).is_err());
    assert!(graph.invalidate_environment(9).is_err());
    assert_eq!(graph.nodes, before);
}

#[test]
fn cleared_root_update_does_not_invalidate_valid_children_or_infer_parent_availability() {
    let mut graph = NativeNormalSubatoms::new(
        [
            (1, node(NativeAtomKind::Item, 0, false)),
            (2, node(NativeAtomKind::Item, 0, true)),
        ],
        [(1, list(true, &[2])), (99, list(true, &[2]))],
    )
    .unwrap();
    let now = Instant::now();
    let record = PackedPosition {
        flags: 0,
        parent: None,
        position: [1., 2., 3.],
        rotation: [0.; 3],
    };
    assert_eq!(
        graph.apply_fresh_position_at(1, &record, now).unwrap(),
        NativeHierarchyPositionChange::Applied {
            cleared_caches: BTreeSet::new()
        }
    );
    assert_eq!(
        graph.node(1).unwrap().position.position_update_age_at(now),
        Some(std::time::Duration::ZERO)
    );
    assert!(graph.node(2).unwrap().position.environment_cache_valid);
    // A list key need not be a node: ownership availability is explicit.
    assert!(graph.lists[&99].available);
    assert_eq!(graph.lists[&99].children, BTreeSet::from([2]));
    assert!(graph.invalidate_environment(99).is_err());
}

#[test]
fn explicit_index_rejects_invalid_context_and_bounds_input_before_building_sets() {
    assert!(
        NativeNormalSubatoms::new(
            [(1, node(NativeAtomKind::Item, 0, true))],
            [(1, list(true, &[2]))]
        )
        .is_err()
    );
    assert!(
        NativeNormalSubatoms::new(
            [
                (1, node(NativeAtomKind::Item, 0, true)),
                (1, node(NativeAtomKind::Item, 0, true))
            ],
            []
        )
        .is_err()
    );
    assert!(
        NativeNormalSubatoms::new(
            [(1, node(NativeAtomKind::Item, 0, true))],
            [(1, list(true, &vec![1; MAX_LINKS + 1]))]
        )
        .is_err()
    );
}
