use super::*;
mod aliases;
mod lifecycle;

#[test]
fn null_wire_id_cannot_be_applied_even_with_contradictory_supplied_node() {
    let bytes = packet(0, 0, 0x16, 1);
    let prefix = NativeAtomPrefix::decode(&bytes).unwrap().unwrap();
    let mut graph = NativeNormalSubatoms::new(
        [(
            0,
            NativeCacheNode {
                kind: NativeAtomKind::Item,
                field_1c: 0,
                position: NativePositionState::default(),
            },
        )],
        [],
    )
    .unwrap();
    let before = graph.node(0).cloned();
    let mut queue = NativeRunaways::default();
    let mut store = RegistrationStore::default();
    let mut owner = Owner::default();
    let mut receiver = NativePositionReceiver {
        graph: &mut graph,
        runaways: &mut queue,
        registrations: &mut store,
        owner: &mut owner,
    };
    assert!(
        receiver
            .apply_at(&prefix, DestinationContext::Unresolved, Instant::now())
            .unwrap_err()
            .to_string()
            .contains("null atom ID")
    );
    assert_eq!(graph.node(0).cloned(), before);
    assert!(queue.is_empty());
    assert!(store.ticker(0).is_none());
    assert_eq!(owner.calls, 0);
}
use crate::{
    atom_position::NativePositionState,
    world::{
        native_atom::NativeAtomKind,
        native_subatoms::{NativeCacheNode, NativeChildList},
        native_tick_tock::{SolarSystem, store::StoreContext},
    },
};

#[derive(Default)]
struct Owner {
    calls: usize,
}
struct Context;
impl ReparentRegistrationContext for Owner {
    type Context<'a> = Context;
    fn context<'a>(&'a mut self, _: &'a mut NativeNormalSubatoms, _: u32) -> Context {
        self.calls += 1;
        Context
    }
}
impl StoreContext for Context {
    fn must_not_tick_tock(&mut self, _: &RegistrationStore) -> bool {
        false
    }
    fn is_ticker(&mut self, _: &RegistrationStore) -> bool {
        true
    }
    fn is_tocker(&mut self, _: &RegistrationStore) -> u32 {
        123
    }
    fn find_parent_solar_system(&mut self, _: &RegistrationStore) -> Option<SolarSystem> {
        Some(SolarSystem {
            id: 99,
            ticking_contents: true,
            field_3a9: 0,
        })
    }
    fn is_ticking_contents(&mut self, _: &RegistrationStore, solar: SolarSystem) -> bool {
        solar.ticking_contents
    }
    fn solar_field_3a9(&mut self, _: &RegistrationStore, solar: SolarSystem) -> u8 {
        solar.field_3a9
    }
    fn atom_type(&mut self, _: &RegistrationStore) -> u8 {
        0x16
    }
}
fn graph(parent: u32, destination: bool) -> NativeNormalSubatoms {
    let mut position = NativePositionState::default();
    position.position.parent = parent;
    position.orientation = Some([1., 2., 3.]);
    position.environment_cache_valid = true;
    NativeNormalSubatoms::new(
        [(
            1,
            NativeCacheNode {
                kind: NativeAtomKind::Item,
                field_1c: 0,
                position,
            },
        )],
        [
            (
                parent,
                NativeChildList {
                    available: true,
                    children: vec![1],
                },
            ),
            (
                7,
                NativeChildList {
                    available: destination,
                    children: vec![],
                },
            ),
        ],
    )
    .unwrap()
}
fn packet(id: u32, parent: u32, opcode: u8, flags: u8) -> Vec<u8> {
    let mut bytes = vec![opcode];
    bytes.extend(id.to_be_bytes());
    bytes.extend(123_i64.to_be_bytes());
    bytes.push(flags);
    if flags & 1 != 0 {
        bytes.push(0xff);
        bytes.extend(parent.to_be_bytes());
        for angle in [0.3_f32, -0.7, 1.1] {
            bytes.extend(angle.to_be_bytes());
        }
        for value in [12345678.125_f64, 2., 3.] {
            bytes.extend(value.to_be_bytes());
        }
    }
    bytes.extend([0xde, 0xad]); // deliberately opaque trailing native layers
    bytes
}

#[test]
fn decoded_wire_position_uses_local_branch_without_owner_and_retains_opaque_tail() {
    let bytes = packet(1, 6, 0x16, 1);
    let prefix = NativeAtomPrefix::decode(&bytes).unwrap().unwrap();
    let mut graph = graph(6, true);
    let mut queue = NativeRunaways::default();
    let mut store = RegistrationStore::default();
    let mut owner = Owner::default();
    let mut receiver = NativePositionReceiver {
        graph: &mut graph,
        runaways: &mut queue,
        registrations: &mut store,
        owner: &mut owner,
    };
    assert_eq!(
        receiver
            .apply_at(&prefix, DestinationContext::Unresolved, Instant::now())
            .unwrap(),
        Some(NativePositionUpdate::Local {
            cleared_caches: BTreeSet::from([1])
        })
    );
    assert_eq!(
        graph.node(1).unwrap().position.position.local[3][0],
        12345678.125
    );
    assert_eq!(
        graph.node(1).unwrap().position.orientation,
        Some([1., 2., 3.])
    );
    assert_eq!(owner.calls, 0);
    assert_eq!(prefix.remainder, [0xde, 0xad]);
    assert!(queue.is_empty() && store.ticker(1).is_none());
}

#[test]
fn unresolved_reparent_preserves_state_then_explicit_missing_destination_runs_real_effects() {
    let bytes = packet(1, 7, 0x16, 1);
    let prefix = NativeAtomPrefix::decode(&bytes).unwrap().unwrap();
    let mut graph = graph(0, false);
    let before = graph.node(1).cloned();
    let mut queue = NativeRunaways::default();
    let mut store = RegistrationStore::default();
    let mut owner = Owner::default();
    let mut receiver = NativePositionReceiver {
        graph: &mut graph,
        runaways: &mut queue,
        registrations: &mut store,
        owner: &mut owner,
    };
    assert!(matches!(
        receiver
            .apply_at(&prefix, DestinationContext::Unresolved, Instant::now())
            .unwrap(),
        Some(NativePositionUpdate::ReparentRequired(_))
    ));
    assert_eq!(receiver.graph.node(1).cloned(), before);
    assert!(matches!(
        receiver
            .apply_at(
                &prefix,
                DestinationContext::MissingDestinationLock,
                Instant::now()
            )
            .unwrap(),
        Some(NativePositionUpdate::MissingDestination(_))
    ));
    assert!(queue.contains(7, 1));
    assert_eq!(graph.node(1).unwrap().position.position.parent, 7);
    assert!(!graph.node(1).unwrap().position.environment_cache_valid);
    assert_eq!(store.ticker(1).unwrap().solar_id, 99);
    assert_eq!(store.tocker(1).unwrap().raw_interval, 123);
    assert_eq!(owner.calls, 1);
}

#[test]
fn decoded_wire_reparent_uses_existing_lists_or_preserves_owner_obligation() {
    let bytes = packet(1, 7, 0x16, 1);
    let prefix = NativeAtomPrefix::decode(&bytes).unwrap().unwrap();
    for available in [false, true] {
        let mut graph = graph(0, available);
        let before = graph.node(1).cloned();
        let mut queue = NativeRunaways::default();
        let mut store = RegistrationStore::default();
        let mut owner = Owner::default();
        let mut receiver = NativePositionReceiver {
            graph: &mut graph,
            runaways: &mut queue,
            registrations: &mut store,
            owner: &mut owner,
        };
        let outcome = receiver
            .apply_at(
                &prefix,
                DestinationContext::ExistingParentLists,
                Instant::now(),
            )
            .unwrap();
        if available {
            assert!(matches!(
                outcome,
                Some(NativePositionUpdate::NormalOwner(
                    NativeReparentResult::Applied { .. }
                ))
            ));
            assert_eq!(graph.node(1).unwrap().position.position.parent, 7);
            assert_eq!(owner.calls, 1);
            assert!(store.ticker(1).is_some());
        } else {
            assert!(matches!(
                outcome,
                Some(NativePositionUpdate::NormalOwner(
                    NativeReparentResult::OwnerContextRequired { .. }
                ))
            ));
            assert_eq!(graph.node(1).cloned(), before);
            assert_eq!(owner.calls, 0);
            assert!(store.ticker(1).is_none());
        }
        assert!(queue.is_empty());
    }
}

#[test]
fn no_position_is_noop_and_wrong_kind_or_unknown_object_does_not_mutate_state() {
    let mut graph = graph(0, true);
    let before = graph.node(1).cloned();
    let mut queue = NativeRunaways::default();
    let mut store = RegistrationStore::default();
    let mut owner = Owner::default();
    for (id, opcode, flags) in [(1, 0x16, 0), (1, 0x05, 1), (9, 0x16, 1)] {
        let bytes = packet(id, 7, opcode, flags);
        let prefix = NativeAtomPrefix::decode(&bytes).unwrap().unwrap();
        let mut receiver = NativePositionReceiver {
            graph: &mut graph,
            runaways: &mut queue,
            registrations: &mut store,
            owner: &mut owner,
        };
        let result = receiver.apply_at(
            &prefix,
            DestinationContext::ExistingParentLists,
            Instant::now(),
        );
        if flags == 0 {
            assert_eq!(result.unwrap(), None);
        } else {
            assert!(result.is_err());
        }
    }
    assert_eq!(graph.node(1).cloned(), before);
    assert!(queue.is_empty());
    assert!(store.ticker(1).is_none());
    assert_eq!(owner.calls, 0);
}
