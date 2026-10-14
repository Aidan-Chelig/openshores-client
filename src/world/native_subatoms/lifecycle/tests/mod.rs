use super::*;
use crate::world::native_tick_tock::{RegistrationAction, SolarSystem, store::StoreContext};
use std::{cell::RefCell, rc::Rc};
mod failures;

struct Owner {
    events: Rc<RefCell<Vec<&'static str>>>,
    owners: BTreeMap<u32, bool>,
}
struct Context;
impl ReparentRegistrationContext for Owner {
    type Context<'a> = Context;
    fn context<'a>(&'a mut self, graph: &'a mut NativeNormalSubatoms, handle: u32) -> Context {
        assert_eq!(handle, 70);
        assert_eq!(graph.node(handle).unwrap().field_1c, 6);
        assert!(!self.owners[&handle]);
        assert!(!graph.lists.contains_key(&7));
        self.events.borrow_mut().push("tick");
        Context
    }
}
impl NativeAtomOwnerContext for Owner {
    fn detach_owner(&mut self, handle: u32) {
        assert_eq!(handle, 70);
        self.owners.insert(handle, false);
        self.events.borrow_mut().push("detach");
    }
    fn assign_owner(&mut self, graph: &NativeNormalSubatoms, handle: u32) {
        assert_eq!(handle, 71);
        assert!(graph.lists[&7].available);
        assert!(graph.lists[&7].children.is_empty());
        self.owners.insert(handle, true);
        self.events.borrow_mut().push("assign");
    }
}
impl StoreContext for Context {
    fn must_not_tick_tock(&mut self, store: &RegistrationStore) -> bool {
        assert!(store.ticker(7).unwrap().released);
        true
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
fn node(kind: NativeAtomKind, parent: u32) -> NativeCacheNode {
    let mut position = NativePositionState::default();
    position.position.parent = parent;
    position.environment_cache_valid = true;
    NativeCacheNode {
        kind,
        field_1c: 4,
        position,
    }
}

#[test]
fn both_routes_replace_in_native_stage_order_then_same_handle_is_noop() {
    for kind in [NativeAtomKind::Item, NativeAtomKind::SolarSystem] {
        let mut graph = NativeNormalSubatoms::new(
            [
                (70, node(kind, 0)),
                (71, node(kind, 0)),
                (10, node(NativeAtomKind::Item, 7)),
                (11, node(NativeAtomKind::Item, 7)),
            ],
            [(
                7,
                NativeChildList {
                    available: true,
                    children: vec![10],
                },
            )],
        )
        .unwrap();
        let before = graph.nodes.clone();
        let mut lookup = NativeRecoveryLookup::from_routed_nodes([
            (7, 70, kind),
            (1, 10, NativeAtomKind::Item),
            (2, 11, NativeAtomKind::Item),
        ])
        .unwrap();
        let mut queue = NativeRunaways::default();
        queue.add(7, 2).unwrap();
        let mut store = RegistrationStore::default();
        store
            .apply(RegistrationAction::UpdateOrAddTicker {
                id: 7,
                atom_type: kind as u8,
                solar_id: 99,
            })
            .unwrap();
        let mut retired = NativeRetiredAtoms::default();
        let events = Rc::new(RefCell::new(Vec::new()));
        let mut owner = Owner {
            events: events.clone(),
            owners: BTreeMap::from([(70, true), (71, false)]),
        };
        let request = NativeAtomAddRequest {
            wire_id: 7,
            handle: 71,
            scan_all_memberships: false,
        };
        let now = Instant::now();
        let result = graph
            .add_atom_owner(
                request,
                NativeAtomOwnerStores {
                    lookup: &mut lookup,
                    runaways: &mut queue,
                    registrations: &mut store,
                    retired: &mut retired,
                },
                &mut owner,
                || {
                    events.borrow_mut().push("clock");
                    now
                },
            )
            .unwrap();
        let NativeAtomAdd::Established {
            replaced: Some((old, removal)),
            arrival,
        } = result
        else {
            panic!("replacement expected")
        };
        assert_eq!(old, 70);
        assert_eq!(removal.discarded_child_set, Some(BTreeSet::from([10])));
        assert_eq!(arrival.recovery.inserted_nodes, BTreeSet::from([11]));
        assert_eq!(*events.borrow(), ["detach", "clock", "tick", "assign"]);
        assert_eq!(lookup.routed_entry(7, kind), Some(71));
        assert_eq!(retired.started_at(70), Some(now));
        assert_eq!(graph.node(70).unwrap().field_1c, 6);
        assert_eq!(graph.node(70).unwrap().position, before[&70].position);
        assert_eq!(graph.node(71).unwrap(), &before[&71]);
        assert_eq!(graph.lists[&7].children, BTreeSet::from([11]));
        assert!(store.ticker(7).unwrap().released);
        assert!(queue.is_empty());
        events.borrow_mut().clear();
        assert_eq!(
            graph
                .add_atom_owner(
                    request,
                    NativeAtomOwnerStores {
                        lookup: &mut lookup,
                        runaways: &mut queue,
                        registrations: &mut store,
                        retired: &mut retired,
                    },
                    &mut owner,
                    || panic!("same handle must not sample clock")
                )
                .unwrap(),
            NativeAtomAdd::AlreadyPresent
        );
        assert!(events.borrow().is_empty());
        assert_eq!(retired.started_at(70), Some(now));
        assert_eq!(graph.lists[&7].children, BTreeSet::from([11]));
    }
}

#[test]
fn fresh_add_does_not_retire_or_schedule_incoming_atom() {
    let mut graph = NativeNormalSubatoms::new([(71, node(NativeAtomKind::Item, 0))], []).unwrap();
    let mut lookup = NativeRecoveryLookup::new([], []).unwrap();
    let mut queue = NativeRunaways::default();
    let mut store = RegistrationStore::default();
    let mut retired = NativeRetiredAtoms::default();
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut owner = Owner {
        events: events.clone(),
        owners: BTreeMap::new(),
    };
    let result = graph
        .add_atom_owner(
            NativeAtomAddRequest {
                wire_id: 7,
                handle: 71,
                scan_all_memberships: false,
            },
            NativeAtomOwnerStores {
                lookup: &mut lookup,
                runaways: &mut queue,
                registrations: &mut store,
                retired: &mut retired,
            },
            &mut owner,
            || panic!("fresh add must not sample retirement clock"),
        )
        .unwrap();
    assert!(matches!(
        result,
        NativeAtomAdd::Established { replaced: None, .. }
    ));
    assert_eq!(*events.borrow(), ["assign"]);
    assert!(retired.is_empty());
    assert!(store.ticker(7).is_none());
    assert_eq!(graph.node(71).unwrap().field_1c, 4);
}
