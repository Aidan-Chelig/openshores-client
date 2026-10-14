use super::*;
use crate::world::native_tick_tock::{RegistrationAction, SolarSystem, store::StoreContext};

struct Owner;
struct Context;
impl ReparentRegistrationContext for Owner {
    type Context<'a> = Context;
    fn context<'a>(&'a mut self, graph: &'a mut NativeNormalSubatoms, handle: u32) -> Context {
        assert_eq!(graph.node(handle).unwrap().field_1c, 6);
        Context
    }
}
impl StoreContext for Context {
    fn must_not_tick_tock(&mut self, store: &RegistrationStore) -> bool {
        assert!(store.ticker(7).unwrap().released);
        true // Explicit status-6 fixture; other native predicates are not needed.
    }
    fn is_ticker(&mut self, _: &RegistrationStore) -> bool {
        unreachable!()
    }
    fn is_tocker(&mut self, _: &RegistrationStore) -> u32 {
        unreachable!()
    }
    fn find_parent_solar_system(&mut self, _: &RegistrationStore) -> Option<SolarSystem> {
        unreachable!()
    }
    fn is_ticking_contents(&mut self, _: &RegistrationStore, _: SolarSystem) -> bool {
        unreachable!()
    }
    fn solar_field_3a9(&mut self, _: &RegistrationStore, _: SolarSystem) -> u8 {
        unreachable!()
    }
    fn atom_type(&mut self, _: &RegistrationStore) -> u8 {
        unreachable!()
    }
}
fn graph() -> NativeNormalSubatoms {
    NativeNormalSubatoms::new(
        [70, 71].map(|handle| {
            (
                handle,
                NativeCacheNode {
                    kind: NativeAtomKind::Item,
                    field_1c: 4,
                    position: NativePositionState::default(),
                },
            )
        }),
        [],
    )
    .unwrap()
}
fn store() -> RegistrationStore {
    let mut store = RegistrationStore::default();
    store
        .apply(RegistrationAction::UpdateOrAddTicker {
            id: 7,
            atom_type: 16,
            solar_id: 99,
        })
        .unwrap();
    store
        .apply(RegistrationAction::ReleaseTickTock { id: 7 })
        .unwrap();
    store
}

#[test]
fn status_precedes_tick_tock_and_retention_is_by_handle_not_wire_id() {
    let mut graph = graph();
    let position = graph.node(70).unwrap().position.clone();
    let mut store = store();
    let mut retired = NativeRetiredAtoms::default();
    let now = Instant::now();
    for handle in [70, 71] {
        graph
            .retire_replaced_atom_at(7, handle, now, &mut retired, &mut store, &mut Owner)
            .unwrap();
        assert_eq!(retired.started_at(handle), Some(now));
    }
    assert_eq!(retired.len(), 2);
    assert_eq!(retired.started_at(7), None);
    assert_eq!(graph.node(70).unwrap().position, position);
    let later = now + std::time::Duration::from_secs(2);
    graph
        .retire_replaced_atom_at(7, 70, later, &mut retired, &mut store, &mut Owner)
        .unwrap();
    assert_eq!(retired.started_at(70), Some(later));
    assert_eq!(retired.len(), 2);
}

#[test]
fn safety_failures_do_not_change_status_or_create_retention() {
    let mut graph = graph();
    let mut retired = NativeRetiredAtoms {
        started: (100..100 + MAX_NODES as u32)
            .map(|h| (h, Instant::now()))
            .collect(),
    };
    let mut store = store();
    assert!(
        graph
            .retire_replaced_atom_at(7, 70, Instant::now(), &mut retired, &mut store, &mut Owner)
            .is_err()
    );
    assert_eq!(graph.node(70).unwrap().field_1c, 4);
    assert_eq!(retired.len(), MAX_NODES);
    assert!(
        graph
            .retire_replaced_atom_at(7, 99, Instant::now(), &mut retired, &mut store, &mut Owner)
            .is_err()
    );
    assert!(retired.started_at(99).is_none());
}
