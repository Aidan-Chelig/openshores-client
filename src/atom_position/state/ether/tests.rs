use super::*;
use std::time::Duration;

fn record(parent: Option<u32>) -> PackedPosition {
    PackedPosition {
        flags: 0,
        parent,
        position: [12345678.125, -2.0, 3.0],
        rotation: [0.3, -0.7, 1.1],
    }
}

#[test]
fn old_parent_controls_tick_tock_and_cache_invalidation_remains_a_later_step() {
    let now = Instant::now();
    for old in [0, 7, u32::MAX] {
        for incoming in [None, Some(0), Some(7), Some(u32::MAX)] {
            for cache_valid in [false, true] {
                let mut state = NativePositionState::default();
                state.position.parent = old;
                state.environment_cache_valid = cache_valid;
                let record = record(incoming);
                let effects = state.set_fresh_position_by_ether_at(&record, now).unwrap();
                assert_eq!(effects.tick_tock_required, old == 0);
                assert!(effects.restart_movement_timer);
                assert_eq!(state.position, FreshPosition::from_packed(&record).unwrap());
                assert_eq!(state.position.local[3][0], 12345678.125);
                assert_eq!(state.position_update_age_at(now), Some(Duration::ZERO));
                assert_eq!(state.environment_cache_valid, cache_valid);
                // TickTock is a caller obligation, not silently declared done.
                assert_eq!(state.invalidate_environment(), cache_valid);
                assert!(!state.environment_cache_valid);
                assert!(!state.invalidate_environment());
            }
        }
    }
}

#[test]
fn identical_updates_restart_timer_and_invalid_geometry_changes_nothing() {
    let first = Instant::now();
    let later = first + Duration::from_secs(10);
    let mut state = NativePositionState::default();
    let record = record(Some(7));
    assert!(
        state
            .set_fresh_position_by_ether_at(&record, first)
            .unwrap()
            .tick_tock_required
    );
    assert!(
        !state
            .set_fresh_position_by_ether_at(&record, later)
            .unwrap()
            .tick_tock_required
    );
    assert_eq!(state.position_update_age_at(later), Some(Duration::ZERO));
    let before = state.clone();
    let mut invalid = record;
    invalid.position[0] = f64::INFINITY;
    assert!(
        state
            .set_fresh_position_by_ether_at(&invalid, first)
            .is_err()
    );
    assert_eq!(state, before);
}

#[test]
fn oriented_setter_extracts_native_angles_but_position_only_paths_retain_them() {
    let first = Instant::now();
    let mut state = NativePositionState {
        environment_cache_valid: true,
        ..Default::default()
    };
    let original = record(Some(7));
    let effects = state
        .set_fresh_position_and_orientation_by_ether_at(&original, first)
        .unwrap();
    assert!(effects.tick_tock_required);
    let orientation = state.orientation.unwrap();
    assert_eq!(
        orientation,
        crate::atom_position::native_rotation(&original.local_matrix().unwrap()).unwrap()
    );
    assert!(state.environment_cache_valid);
    let mut next = original;
    next.rotation = [1e-6; 3];
    let later = first + Duration::from_secs(1);
    state.set_fresh_position_by_ether_at(&next, later).unwrap();
    assert_eq!(state.orientation, Some(orientation));
    state
        .apply_fresh_wire_position_at(&next, true, later)
        .unwrap();
    assert_eq!(state.orientation, Some(orientation));
    state
        .set_fresh_position_and_orientation_by_ether_at(&next, later)
        .unwrap();
    assert_eq!(state.orientation, Some([0.; 3]));
    assert_eq!(state.position_update_age_at(later), Some(Duration::ZERO));
    let before = state.clone();
    next.rotation[0] = f32::NAN;
    assert!(
        state
            .set_fresh_position_and_orientation_by_ether_at(&next, first)
            .is_err()
    );
    assert_eq!(state, before);
}
