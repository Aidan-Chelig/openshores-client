use super::*;
use crate::world::native_runaways::NativeRunaways;

fn graph(kind: NativeAtomKind) -> NativeNormalSubatoms {
    let nodes = [(10, 7), (11, 99), (12, 1)].map(|(handle, parent)| {
        let mut position = NativePositionState::default();
        position.position.parent = parent;
        position.environment_cache_valid = true;
        (
            handle,
            NativeCacheNode {
                kind,
                field_1c: 4,
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
                    available: false,
                    children: vec![10, 11],
                },
            ),
            (
                8,
                NativeChildList {
                    available: true,
                    children: vec![10],
                },
            ),
            (
                1,
                NativeChildList {
                    available: true,
                    children: vec![12],
                },
            ),
        ],
    )
    .unwrap()
}

#[test]
fn removal_releases_id_but_removes_object_membership_without_destroying_children() {
    for kind in [NativeAtomKind::Item, NativeAtomKind::Universe] {
        for scan in [false, true] {
            let mut graph = graph(kind);
            let nodes = graph.nodes.clone();
            let mut lookup = NativeRecoveryLookup::new([(1, 11)], [(1, 12)]).unwrap();
            let mut store = RegistrationStore::default();
            store
                .apply(RegistrationAction::UpdateOrAddTicker {
                    id: 1,
                    atom_type: 16,
                    solar_id: 99,
                })
                .unwrap();
            store
                .apply(RegistrationAction::UpdateOrAddTocker {
                    id: 1,
                    atom_type: 16,
                    solar_id: 99,
                    raw_interval: 50,
                })
                .unwrap();
            let mut queue = NativeRunaways::default();
            queue.add(1, 12).unwrap();
            let queued = queue.clone();
            let mut owner = Some(42);
            let result = graph
                .remove_atom_owner(1, 10, &mut lookup, &mut store, scan, |handle| {
                    assert_eq!(handle, 10);
                    owner = None;
                })
                .unwrap();
            assert_eq!(owner, None);
            assert_eq!(
                result.erased_map_handle,
                Some(if kind.in_star_field() { 12 } else { 11 })
            );
            assert_eq!(
                lookup.resolve(1),
                Some(if kind.in_star_field() { 11 } else { 12 })
            );
            assert!(result.removed_from_parent);
            assert_eq!(
                result.additional_parent_memberships,
                if scan {
                    BTreeSet::from([8])
                } else {
                    BTreeSet::new()
                }
            );
            assert_eq!(result.discarded_child_set, Some(BTreeSet::from([12])));
            assert_eq!(graph.lists[&7].children, BTreeSet::from([11]));
            assert_eq!(graph.lists[&8].children.contains(&10), !scan);
            assert!(!graph.lists.contains_key(&1));
            assert_eq!(graph.nodes, nodes);
            assert_eq!(queue, queued);
            assert!(store.ticker(1).unwrap().released);
            assert!(store.tocker(1).unwrap().common.released);
        }
    }
}

#[test]
fn missing_handle_is_rejected_before_owner_or_indexes_change() {
    let mut graph = graph(NativeAtomKind::Item);
    let mut lookup = NativeRecoveryLookup::new([(1, 10)], []).unwrap();
    let mut store = RegistrationStore::default();
    assert!(
        graph
            .remove_atom_owner(1, 99, &mut lookup, &mut store, false, |_| {
                panic!("no owner detachment on failed preflight")
            })
            .is_err()
    );
    assert_eq!(lookup.resolve(1), Some(10));
    assert!(graph.lists.contains_key(&1));
}

#[test]
fn absent_entries_and_null_ids_do_not_invent_registration_or_child_lists() {
    let mut graph = graph(NativeAtomKind::Item);
    let mut lookup = NativeRecoveryLookup::new([], []).unwrap();
    let mut store = RegistrationStore::default();
    let result = graph
        .remove_atom_owner(0, 12, &mut lookup, &mut store, false, |_| {})
        .unwrap();
    assert_eq!(result.erased_map_handle, None);
    assert_eq!(result.discarded_child_set, None);
    assert!(result.removed_from_parent);
    assert!(store.ticker(0).is_none());
    assert!(store.tocker(0).is_none());
}
