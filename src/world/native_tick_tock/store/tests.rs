use super::*;
mod membership;
use crate::world::native_tick_tock::RegistrationAction::*;

#[test]
fn insertion_release_reactivation_and_cleanup_are_distinct() {
    let mut store = RegistrationStore::default();
    store.apply(ReleaseTickTock { id: 7 }).unwrap();
    assert_eq!(store.cleanup_pending(), (false, false));
    store
        .apply(UpdateOrAddTicker {
            id: 7,
            atom_type: 0x16,
            solar_id: 9,
        })
        .unwrap();
    store
        .apply(UpdateOrAddTocker {
            id: 7,
            atom_type: 0x16,
            solar_id: 9,
            raw_interval: u32::MAX,
        })
        .unwrap();
    store.apply(ReleaseTickTock { id: 7 }).unwrap();
    assert!(store.ticker(7).unwrap().released);
    assert!(store.tocker(7).unwrap().common.released);
    assert_eq!(store.cleanup_pending(), (true, true));
    store
        .apply(UpdateOrAddTicker {
            id: 7,
            atom_type: 0x05,
            solar_id: 42,
        })
        .unwrap();
    assert_eq!(store.ticker(7).unwrap().atom_type, 0x16);
    assert_eq!(store.ticker(7).unwrap().timer_words, [i64::MIN; 2]);
    assert_eq!(store.cleanup_pending(), (true, true));
    store
        .apply(UpdateOrAddTicker {
            id: 8,
            atom_type: 0x05,
            solar_id: 42,
        })
        .unwrap();
    assert!(store.ticker(7).is_some());
    assert!(store.tocker(7).is_some());
    assert_eq!(store.cleanup_pending(), (false, true));
    store
        .apply(UpdateOrAddTocker {
            id: 8,
            atom_type: 0x05,
            solar_id: 42,
            raw_interval: 0,
        })
        .unwrap();
    assert!(store.tocker(7).is_none());
    assert_eq!(store.cleanup_pending(), (false, false));
}

#[test]
fn busy_records_survive_cleanup_and_pending_flag_is_still_cleared() {
    let mut store = RegistrationStore::default();
    store
        .apply(UpdateOrAddTicker {
            id: 1,
            atom_type: 2,
            solar_id: 3,
        })
        .unwrap();
    let lock = store.ticker_lock(1).unwrap();
    let guard = lock.lock().unwrap();
    store.apply(ReleaseTicker { id: 1 }).unwrap();
    store
        .apply(UpdateOrAddTicker {
            id: 2,
            atom_type: 2,
            solar_id: 3,
        })
        .unwrap();
    assert!(store.ticker(1).unwrap().released);
    assert_eq!(store.cleanup_pending(), (false, false));
    drop(guard);
    store
        .apply(UpdateOrAddTicker {
            id: 3,
            atom_type: 2,
            solar_id: 3,
        })
        .unwrap();
    assert!(store.ticker(1).is_some()); // no pending flag, so no automatic retry
    store.apply(ReleaseTicker { id: 1 }).unwrap();
    store
        .apply(UpdateOrAddTicker {
            id: 4,
            atom_type: 2,
            solar_id: 3,
        })
        .unwrap();
    assert!(store.ticker(1).is_none());
}

#[test]
fn safety_budget_errors_are_atomic_and_existing_updates_remain_allowed() {
    let mut store = RegistrationStore::with_limit(1).unwrap();
    store
        .apply(UpdateOrAddTicker {
            id: 1,
            atom_type: 2,
            solar_id: 3,
        })
        .unwrap();
    store.apply(ReleaseTicker { id: 1 }).unwrap();
    let before = store.ticker(1).cloned();
    assert!(
        store
            .apply(UpdateOrAddTicker {
                id: 2,
                atom_type: 2,
                solar_id: 3
            })
            .is_err()
    );
    assert_eq!(store.ticker(1).cloned(), before);
    assert_eq!(store.cleanup_pending(), (true, false));
    store
        .apply(UpdateOrAddTicker {
            id: 1,
            atom_type: 9,
            solar_id: 42,
        })
        .unwrap();
    assert_eq!(store.ticker(1).unwrap().solar_id, 42);
    assert!(RegistrationStore::with_limit(20_001).is_err());
}
