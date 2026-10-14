use super::*;

#[test]
fn missing_destination_queues_and_registers_without_moving_child_indexes() {
    for blocked in [false, true] {
        let mut graph = fixture(0);
        graph.lists.get_mut(&7).unwrap().available = false;
        let mut queue = NativeRunaways::default();
        let mut store = RegistrationStore::default();
        if blocked {
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
        }
        let mut owner = Owner { calls: 0, blocked };
        let result = graph
            .change_parent_to_missing_destination_with_registrations_at(
                1,
                &position(),
                true,
                Instant::now(),
                RunawayRegistrationStores {
                    runaways: &mut queue,
                    registrations: &mut store,
                },
                &mut owner,
            )
            .unwrap();
        assert!(result.inserted_pair && result.tick_tock_called);
        assert!(queue.contains(7, 1));
        assert!(graph.lists[&0].children.contains(&1));
        assert!(!graph.lists[&7].children.contains(&1));
        assert_eq!(graph.node(1).unwrap().position.position.parent, 7);
        assert!(graph.node(1).unwrap().position.orientation.is_some());
        assert_eq!(store.ticker(1).unwrap().released, blocked);
        assert_eq!(store.tocker(1).unwrap().common.released, blocked);
        if !blocked {
            assert_eq!(store.tocker(1).unwrap().raw_interval, 123);
        }
        assert!(!graph.node(2).unwrap().position.environment_cache_valid);
    }
}

#[test]
fn failed_registration_retains_queue_and_first_effect_but_cleans_caches() {
    let mut graph = fixture(0);
    graph.lists.get_mut(&7).unwrap().available = false;
    let mut queue = NativeRunaways::default();
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
            .change_parent_to_missing_destination_with_registrations_at(
                1,
                &position(),
                false,
                Instant::now(),
                RunawayRegistrationStores {
                    runaways: &mut queue,
                    registrations: &mut store
                },
                &mut owner,
            )
            .is_err()
    );
    assert!(queue.contains(7, 1));
    assert!(store.ticker(1).is_some());
    assert!(store.tocker(1).is_none());
    assert!(graph.lists[&0].children.contains(&1));
    assert_eq!(graph.node(1).unwrap().position.position.parent, 7);
    assert!(!graph.node(1).unwrap().position.environment_cache_valid);
    assert!(!graph.node(2).unwrap().position.environment_cache_valid);
}

#[test]
fn queue_rejection_skips_context_construction_and_registration_effects() {
    let mut graph = fixture(0);
    graph.lists.get_mut(&7).unwrap().available = false;
    let mut queue = NativeRunaways::with_limit(1).unwrap();
    queue.add(8, 3).unwrap();
    let mut store = RegistrationStore::default();
    let mut owner = Owner {
        calls: 0,
        blocked: false,
    };
    assert!(
        graph
            .change_parent_to_missing_destination_with_registrations_at(
                1,
                &position(),
                false,
                Instant::now(),
                RunawayRegistrationStores {
                    runaways: &mut queue,
                    registrations: &mut store
                },
                &mut owner,
            )
            .is_err()
    );
    assert_eq!(owner.calls, 0);
    assert_eq!(graph.node(1).unwrap().position.position.parent, 0);
    assert!(graph.node(1).unwrap().position.environment_cache_valid);
    assert!(store.ticker(1).is_none());
    assert!(store.tocker(1).is_none());
    assert_eq!(queue.len(), 1);
    assert!(!queue.contains(7, 1));
}
