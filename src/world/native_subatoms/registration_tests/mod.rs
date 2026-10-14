use super::*;
mod missing;
use crate::{
    atom_position::NativePositionState,
    world::{
        native_atom::NativeAtomKind,
        native_subatoms::{NativeCacheNode, NativeChildList},
        native_tick_tock::{RegistrationAction, SolarSystem},
    },
};

struct Owner {
    calls: usize,
    blocked: bool,
}
struct Context<'a> {
    owner: &'a mut Owner,
    graph: &'a mut NativeNormalSubatoms,
    id: u32,
}
impl ReparentRegistrationContext for Owner {
    type Context<'a> = Context<'a>;
    fn context<'a>(&'a mut self, graph: &'a mut NativeNormalSubatoms, id: u32) -> Context<'a> {
        self.calls += 1;
        Context {
            owner: self,
            graph,
            id,
        }
    }
}
impl StoreContext for Context<'_> {
    fn must_not_tick_tock(&mut self, _: &RegistrationStore) -> bool {
        let node = self.graph.node(self.id).unwrap();
        assert_eq!(node.position.position.parent, 7);
        assert!(node.position.environment_cache_valid);
        assert!(self.graph.node(2).unwrap().position.environment_cache_valid);
        assert_eq!(
            self.graph.lists[&7].children.contains(&self.id),
            self.graph.lists[&7].available
        );
        self.owner.blocked // explicit synthetic native predicate observation
    }
    fn is_ticker(&mut self, _: &RegistrationStore) -> bool {
        true
    }
    fn is_tocker(&mut self, store: &RegistrationStore) -> u32 {
        assert!(store.ticker(self.id).is_some_and(|fields| !fields.released));
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

fn fixture(old: u32) -> NativeNormalSubatoms {
    let mut root = NativePositionState::default();
    root.position.parent = old;
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
fn position() -> PackedPosition {
    PackedPosition {
        flags: 0,
        parent: Some(7),
        position: [1., 2., 3.],
        rotation: [0.; 3],
    }
}

#[test]
fn null_old_parent_runs_real_store_updates_before_descendant_cleanup() {
    let mut graph = fixture(0);
    let mut store = RegistrationStore::default();
    let mut owner = Owner {
        calls: 0,
        blocked: false,
    };
    graph
        .change_parent_with_registrations_at(
            1,
            &position(),
            false,
            Instant::now(),
            &mut store,
            &mut owner,
        )
        .unwrap();
    assert_eq!(owner.calls, 1);
    assert_eq!(store.ticker(1).unwrap().solar_id, 99);
    assert_eq!(store.tocker(1).unwrap().raw_interval, 123);
    assert!(!graph.node(1).unwrap().position.environment_cache_valid);
    assert!(!graph.node(2).unwrap().position.environment_cache_valid);
}

#[test]
fn nonnull_old_parent_does_not_construct_predicates_or_mutate_registrations() {
    let mut graph = fixture(6);
    let mut store = RegistrationStore::default();
    let mut owner = Owner {
        calls: 0,
        blocked: false,
    };
    graph
        .change_parent_with_registrations_at(
            1,
            &position(),
            true,
            Instant::now(),
            &mut store,
            &mut owner,
        )
        .unwrap();
    assert_eq!(owner.calls, 0);
    assert!(store.ticker(1).is_none());
    assert!(store.tocker(1).is_none());
}

#[test]
fn blocked_predicate_releases_real_records_and_missing_owner_context_skips_all_effects() {
    let mut graph = fixture(0);
    let mut store = RegistrationStore::default();
    store
        .apply(RegistrationAction::UpdateOrAddTicker {
            id: 1,
            atom_type: 2,
            solar_id: 3,
        })
        .unwrap();
    store
        .apply(RegistrationAction::UpdateOrAddTocker {
            id: 1,
            atom_type: 2,
            solar_id: 3,
            raw_interval: 4,
        })
        .unwrap();
    let mut owner = Owner {
        calls: 0,
        blocked: true,
    };
    graph.lists.get_mut(&7).unwrap().available = false;
    assert!(matches!(
        graph
            .change_parent_with_registrations_at(
                1,
                &position(),
                false,
                Instant::now(),
                &mut store,
                &mut owner
            )
            .unwrap(),
        NativeReparentResult::OwnerContextRequired { .. }
    ));
    assert_eq!(owner.calls, 0);
    assert!(!store.ticker(1).unwrap().released);
    assert!(!store.tocker(1).unwrap().common.released);
    assert_eq!(graph.node(1).unwrap().position.position.parent, 0);
    graph.lists.get_mut(&7).unwrap().available = true;
    graph
        .change_parent_with_registrations_at(
            1,
            &position(),
            false,
            Instant::now(),
            &mut store,
            &mut owner,
        )
        .unwrap();
    assert_eq!(owner.calls, 1);
    assert!(store.ticker(1).unwrap().released);
    assert!(store.tocker(1).unwrap().common.released);
    assert_eq!(store.cleanup_pending(), (true, true));
    assert!(!graph.node(2).unwrap().position.environment_cache_valid);
}

#[test]
fn registration_budget_failure_retains_position_and_first_effect_but_clears_caches() {
    let mut graph = fixture(0);
    let mut store = RegistrationStore::with_limit(1).unwrap();
    store
        .apply(RegistrationAction::UpdateOrAddTocker {
            id: 3,
            atom_type: 2,
            solar_id: 4,
            raw_interval: 5,
        })
        .unwrap();
    let mut owner = Owner {
        calls: 0,
        blocked: false,
    };
    assert!(
        graph
            .change_parent_with_registrations_at(
                1,
                &position(),
                false,
                Instant::now(),
                &mut store,
                &mut owner
            )
            .is_err()
    );
    assert_eq!(graph.node(1).unwrap().position.position.parent, 7);
    assert!(store.ticker(1).is_some());
    assert!(store.tocker(1).is_none());
    assert_eq!(store.tocker(3).unwrap().raw_interval, 5);
    assert!(!graph.node(1).unwrap().position.environment_cache_valid);
    assert!(!graph.node(2).unwrap().position.environment_cache_valid);
}
