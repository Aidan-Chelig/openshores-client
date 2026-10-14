use super::*;
use crate::world::{native_subatoms::NativeRecoveryLookup, native_tick_tock::RegistrationAction};

struct MissingParentOwner {
    calls: usize,
}
struct MissingParentContext;
impl ReparentRegistrationContext for MissingParentOwner {
    type Context<'a> = MissingParentContext;
    fn context<'a>(
        &'a mut self,
        graph: &'a mut NativeNormalSubatoms,
        id: u32,
    ) -> MissingParentContext {
        self.calls += 1;
        assert_eq!(graph.node(id).unwrap().position.position.parent, 7);
        MissingParentContext
    }
}
impl StoreContext for MissingParentContext {
    // Explicit native observation: Parent lookup is missing at this stage.
    fn must_not_tick_tock(&mut self, _: &RegistrationStore) -> bool {
        true
    }
    fn is_ticker(&mut self, _: &RegistrationStore) -> bool {
        panic!("blocked predicate")
    }
    fn is_tocker(&mut self, _: &RegistrationStore) -> u32 {
        panic!("blocked predicate")
    }
    fn find_parent_solar_system(&mut self, _: &RegistrationStore) -> Option<SolarSystem> {
        panic!("blocked predicate")
    }
    fn is_ticking_contents(&mut self, _: &RegistrationStore, _: SolarSystem) -> bool {
        panic!("blocked predicate")
    }
    fn solar_field_3a9(&mut self, _: &RegistrationStore, _: SolarSystem) -> u8 {
        panic!("blocked predicate")
    }
    fn atom_type(&mut self, _: &RegistrationStore) -> u8 {
        panic!("blocked predicate")
    }
}

#[test]
fn native_child_position_then_parent_arrival_repairs_indexes_without_replaying_movement() {
    let now = Instant::now();
    let child_bytes = packet(1, 7, 0x16, 1);
    let parent_bytes = packet(7, 0, 0x1c, 1);
    let child_prefix = NativeAtomPrefix::decode(&child_bytes).unwrap().unwrap();
    let parent_prefix = NativeAtomPrefix::decode(&parent_bytes).unwrap().unwrap();
    let mut child = NativePositionState::default();
    child.environment_cache_valid = true;
    let mut parent = NativePositionState::default();
    // Incoming fresh object's position is decoded before its owner tail.
    parent
        .apply_fresh_wire_position_at(parent_prefix.base.position.as_ref().unwrap(), false, now)
        .unwrap();
    let mut graph = NativeNormalSubatoms::new(
        [
            (
                1,
                NativeCacheNode {
                    kind: child_prefix.kind,
                    field_1c: 0,
                    position: child,
                },
            ),
            (
                7,
                NativeCacheNode {
                    kind: parent_prefix.kind,
                    field_1c: 0,
                    position: parent,
                },
            ),
        ],
        [(
            0,
            NativeChildList {
                available: true,
                children: vec![1],
            },
        )],
    )
    .unwrap();
    let mut queue = NativeRunaways::default();
    let mut store = RegistrationStore::default();
    store
        .apply(RegistrationAction::UpdateOrAddTicker {
            id: 1,
            atom_type: 0x16,
            solar_id: 99,
        })
        .unwrap();
    store
        .apply(RegistrationAction::UpdateOrAddTocker {
            id: 1,
            atom_type: 0x16,
            solar_id: 99,
            raw_interval: 123,
        })
        .unwrap();
    let mut owner = MissingParentOwner { calls: 0 };
    let mut receiver = NativePositionReceiver {
        graph: &mut graph,
        runaways: &mut queue,
        registrations: &mut store,
        owner: &mut owner,
    };
    assert!(matches!(
        receiver
            .apply_at(
                &child_prefix,
                DestinationContext::MissingDestinationLock,
                now
            )
            .unwrap(),
        Some(NativePositionUpdate::MissingDestination(_))
    ));
    assert!(queue.contains(7, 1));
    assert!(store.ticker(1).unwrap().released);
    assert!(store.tocker(1).unwrap().common.released);
    let before_child = graph.node(1).cloned();
    let before_parent = graph.node(7).cloned();
    let before_ticker = store.ticker(1).cloned();
    let before_tocker = store.tocker(1).cloned();
    // Explicit native atom-map insertion context for the arriving parent.
    let lookup = NativeRecoveryLookup::from_routed_nodes([
        (1, 1, child_prefix.kind),
        (7, 7, parent_prefix.kind),
    ])
    .unwrap();
    let established = graph
        .establish_fresh_atom_owner(7, 7, &mut queue, &lookup)
        .unwrap();
    assert_eq!(established.recovery.consumed_children, BTreeSet::from([1]));
    assert_eq!(established.recovery.inserted_nodes, BTreeSet::from([1]));
    assert!(queue.is_empty());
    assert_eq!(graph.node(1).cloned(), before_child);
    assert_eq!(graph.node(7).cloned(), before_parent);
    assert_eq!(store.ticker(1).cloned(), before_ticker);
    assert_eq!(store.tocker(1).cloned(), before_tocker);
    assert_eq!(owner.calls, 1);
    assert_eq!(
        graph.node(1).unwrap().position.position_update_age_at(now),
        Some(std::time::Duration::ZERO)
    );
    assert_eq!(child_prefix.remainder, [0xde, 0xad]);
    assert_eq!(parent_prefix.remainder, [0xde, 0xad]);
    assert!(
        graph
            .fix_runaways_in_maps(7, &mut queue, &lookup)
            .unwrap()
            .consumed_children
            .is_empty()
    );
}
