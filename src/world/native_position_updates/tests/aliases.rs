use super::*;
use crate::world::native_subatoms::NativeRecoveryLookup;

#[test]
fn existing_object_kind_survives_different_incoming_candidate_kind() {
    let mut graph = graph(0, true);
    let bytes = packet(1, 7, 0x05, 1);
    let prefix = NativeAtomPrefix::decode(&bytes).unwrap().unwrap();
    assert_eq!(prefix.kind, NativeAtomKind::Person);
    let mut queue = NativeRunaways::default();
    let mut store = RegistrationStore::default();
    let mut owner = Owner::default();
    NativePositionReceiver {
        graph: &mut graph,
        runaways: &mut queue,
        registrations: &mut store,
        owner: &mut owner,
    }
    .apply_at(
        &prefix,
        DestinationContext::ExistingParentLists,
        Instant::now(),
    )
    .unwrap();
    let retained = graph.node(1).unwrap();
    assert_eq!(retained.kind, NativeAtomKind::Item);
    assert_eq!(retained.position.position.parent, 7);
    assert_eq!(retained.position.position.local[3][0], 12345678.125);
    assert_eq!(owner.calls, 1);
    assert_eq!(store.ticker(1).unwrap().atom_type, 0x16);
    assert!(queue.is_empty());
}

#[derive(Default)]
struct AliasOwner {
    calls: usize,
}
impl ReparentRegistrationContext for AliasOwner {
    type Context<'a> = Context;
    fn context<'a>(&'a mut self, graph: &'a mut NativeNormalSubatoms, handle: u32) -> Context {
        assert_eq!(handle, 10);
        assert_eq!(graph.node(handle).unwrap().position.position.parent, 7);
        self.calls += 1;
        Context
    }
}

#[test]
fn explicit_resolved_handle_preserves_wire_registration_and_runaway_identity() {
    for destination in [
        DestinationContext::ExistingParentLists,
        DestinationContext::MissingDestinationLock,
    ] {
        let mut graph = NativeNormalSubatoms::new(
            [(
                10,
                NativeCacheNode {
                    kind: NativeAtomKind::Item,
                    field_1c: 0,
                    position: NativePositionState::default(),
                },
            )],
            [
                (
                    0,
                    NativeChildList {
                        available: true,
                        children: vec![10],
                    },
                ),
                (
                    7,
                    NativeChildList {
                        available: destination == DestinationContext::ExistingParentLists,
                        children: vec![],
                    },
                ),
            ],
        )
        .unwrap();
        let lookup =
            NativeRecoveryLookup::from_routed_nodes([(1, 10, NativeAtomKind::Item)]).unwrap();
        let handle = lookup.routed_entry(1, NativeAtomKind::Item).unwrap();
        let mut queue = NativeRunaways::default();
        let mut store = RegistrationStore::default();
        let mut owner = AliasOwner::default();
        let bytes = packet(1, 7, 0x16, 1);
        let prefix = NativeAtomPrefix::decode(&bytes).unwrap().unwrap();
        NativePositionReceiver {
            graph: &mut graph,
            runaways: &mut queue,
            registrations: &mut store,
            owner: &mut owner,
        }
        .apply_resolved_at(&prefix, handle, destination, Instant::now())
        .unwrap();
        assert_eq!(graph.node(10).unwrap().position.position.parent, 7);
        assert_eq!(
            graph.node(10).unwrap().position.position.local[3][0],
            12345678.125
        );
        assert!(graph.node(1).is_none());
        assert_eq!(owner.calls, 1);
        assert_eq!(store.ticker(1).unwrap().solar_id, 99);
        assert_eq!(store.tocker(1).unwrap().raw_interval, 123);
        assert!(store.ticker(10).is_none());
        if destination == DestinationContext::MissingDestinationLock {
            assert!(queue.contains(7, 1));
            assert!(!queue.contains(7, 10));
            // Parent arrival consumes the wire ID and recovers the actual handle.
            assert_eq!(
                graph
                    .fix_runaways_in_maps(7, &mut queue, &lookup)
                    .unwrap()
                    .inserted_nodes,
                BTreeSet::new()
            );
            assert!(queue.is_empty()); // explicit unavailable destination remains unavailable
        } else {
            assert!(queue.is_empty());
        }
    }
}

#[test]
fn resolved_local_and_deferred_updates_do_not_register_handle_as_wire_id() {
    let mut graph = NativeNormalSubatoms::new(
        [(
            10,
            NativeCacheNode {
                kind: NativeAtomKind::Item,
                field_1c: 0,
                position: NativePositionState::default(),
            },
        )],
        [],
    )
    .unwrap();
    let mut queue = NativeRunaways::default();
    let mut store = RegistrationStore::default();
    let mut owner = AliasOwner::default();
    let bytes = packet(1, 0, 0x16, 1);
    let prefix = NativeAtomPrefix::decode(&bytes).unwrap().unwrap();
    let mut receiver = NativePositionReceiver {
        graph: &mut graph,
        runaways: &mut queue,
        registrations: &mut store,
        owner: &mut owner,
    };
    assert!(matches!(
        receiver
            .apply_resolved_at(&prefix, 10, DestinationContext::Unresolved, Instant::now())
            .unwrap(),
        Some(NativePositionUpdate::Local { .. })
    ));
    let bytes = packet(1, 7, 0x16, 1);
    let prefix = NativeAtomPrefix::decode(&bytes).unwrap().unwrap();
    assert!(matches!(
        receiver
            .apply_resolved_at(&prefix, 10, DestinationContext::Unresolved, Instant::now())
            .unwrap(),
        Some(NativePositionUpdate::ReparentRequired(_))
    ));
    assert_eq!(graph.node(10).unwrap().position.position.parent, 0);
    assert!(store.ticker(1).is_none());
    assert!(store.ticker(10).is_none());
    assert_eq!(owner.calls, 0);
}
