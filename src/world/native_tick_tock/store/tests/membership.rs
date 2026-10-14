use super::*;

#[test]
fn queue_membership_keeps_released_entries_until_each_table_actually_sweeps() {
    let mut store = RegistrationStore::default();
    assert!(!store.in_tick_tock_queue(7));
    store
        .apply(UpdateOrAddTicker {
            id: 7,
            atom_type: 16,
            solar_id: 99,
        })
        .unwrap();
    assert!(store.in_tick_tock_queue(7));
    store.apply(ReleaseTicker { id: 7 }).unwrap();
    assert!(store.in_tick_tock_queue(7));
    store
        .apply(UpdateOrAddTocker {
            id: 7,
            atom_type: 16,
            solar_id: 99,
            raw_interval: 0,
        })
        .unwrap();
    store.apply(ReleaseTocker { id: 7 }).unwrap();
    store
        .apply(UpdateOrAddTicker {
            id: 8,
            atom_type: 16,
            solar_id: 99,
        })
        .unwrap();
    assert!(store.ticker(7).is_none());
    assert!(store.in_tick_tock_queue(7));
    store
        .apply(UpdateOrAddTocker {
            id: 8,
            atom_type: 16,
            solar_id: 99,
            raw_interval: 1,
        })
        .unwrap();
    assert!(!store.in_tick_tock_queue(7));
    assert!(store.in_tick_tock_queue(8));
}

#[test]
fn busy_released_record_stays_in_queue_even_after_cleanup_flag_clears() {
    let mut store = RegistrationStore::default();
    store
        .apply(UpdateOrAddTicker {
            id: 0,
            atom_type: 16,
            solar_id: 99,
        })
        .unwrap();
    let lock = store.ticker_lock(0).unwrap();
    let guard = lock.lock().unwrap();
    store.apply(ReleaseTicker { id: 0 }).unwrap();
    store
        .apply(UpdateOrAddTicker {
            id: u32::MAX,
            atom_type: 16,
            solar_id: 99,
        })
        .unwrap();
    assert!(store.in_tick_tock_queue(0));
    assert!(!store.cleanup_pending().0);
    drop(guard);
    assert!(store.in_tick_tock_queue(0));
    store.apply(ReleaseTicker { id: 0 }).unwrap();
    store
        .apply(UpdateOrAddTicker {
            id: 1,
            atom_type: 16,
            solar_id: 99,
        })
        .unwrap();
    assert!(!store.in_tick_tock_queue(0));
}
