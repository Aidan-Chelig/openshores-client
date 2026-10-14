use super::*;

fn record(parent: Option<u32>) -> PackedPosition {
    PackedPosition {
        flags: 0,
        parent,
        position: [12345678.125, -2.0, 3.0],
        rotation: [0.3, -0.7, 1.1],
    }
}

#[test]
fn managed_parent_change_delegates_without_mutating_any_local_state() {
    let mut state = NativePositionState {
        environment_cache_valid: true,
        ..Default::default()
    };
    let before = state.clone();
    let proposed = record(Some(7));
    assert_eq!(
        state.apply_fresh_wire_position(&proposed, true).unwrap(),
        PositionChange::ReparentRequired(FreshPosition::from_packed(&proposed).unwrap())
    );
    assert_eq!(state, before);
}

#[test]
fn local_branch_keeps_doubles_and_restarts_timer_even_for_identical_position() {
    let mut state = NativePositionState::default();
    let record = record(None);
    assert_eq!(
        state.apply_fresh_wire_position(&record, true).unwrap(),
        PositionChange::Applied {
            restart_movement_timer: true,
            invalidate_normal_descendants: false
        }
    );
    assert_eq!(state.position.parent, 0);
    assert_eq!(state.position.local[3][0], 12345678.125);
    state.environment_cache_valid = true;
    assert_eq!(
        state.apply_fresh_wire_position(&record, true).unwrap(),
        PositionChange::Applied {
            restart_movement_timer: true,
            invalidate_normal_descendants: true
        }
    );
    assert!(!state.environment_cache_valid);
}

#[test]
fn omitted_fresh_parent_is_null_not_previous_parent_and_unmanaged_change_copies() {
    let mut state = NativePositionState::default();
    state
        .apply_fresh_wire_position(&record(Some(7)), false)
        .unwrap();
    let previous = state.clone();
    assert!(matches!(
        state
            .apply_fresh_wire_position(&record(None), true)
            .unwrap(),
        PositionChange::ReparentRequired(FreshPosition { parent: 0, .. })
    ));
    assert_eq!(state, previous);
    assert!(matches!(
        state
            .apply_fresh_wire_position(&record(None), false)
            .unwrap(),
        PositionChange::Applied { .. }
    ));
    assert_eq!(state.position.parent, 0);
}

#[test]
fn invalid_wire_geometry_is_rejected_atomically() {
    let mut state = NativePositionState::default();
    let before = state.clone();
    let mut record = record(Some(9));
    record.position[0] = f64::NAN;
    assert!(state.apply_fresh_wire_position(&record, false).is_err());
    assert_eq!(state, before);
}

#[test]
fn monotonic_timer_restarts_for_identical_updates_but_not_delegation_or_errors() {
    let mut state = NativePositionState::default();
    let first = Instant::now();
    let later = first + Duration::from_secs(10);
    assert_eq!(state.position_update_age_at(first), None);
    state
        .apply_fresh_wire_position_at(&record(None), true, first)
        .unwrap();
    assert_eq!(
        state.position_update_age_at(later),
        Some(Duration::from_secs(10))
    );
    state
        .apply_fresh_wire_position_at(&record(None), true, later)
        .unwrap();
    assert_eq!(state.position_update_age_at(later), Some(Duration::ZERO));
    assert_eq!(state.position_update_age_at(first), None);
    let check = later + Duration::from_secs(5);
    assert!(matches!(
        state
            .apply_fresh_wire_position_at(&record(Some(7)), true, check)
            .unwrap(),
        PositionChange::ReparentRequired(_)
    ));
    assert_eq!(
        state.position_update_age_at(check),
        Some(Duration::from_secs(5))
    );
    let mut invalid = record(None);
    invalid.rotation[0] = f32::NAN;
    assert!(
        state
            .apply_fresh_wire_position_at(&invalid, true, check)
            .is_err()
    );
    assert_eq!(
        state.position_update_age_at(check),
        Some(Duration::from_secs(5))
    );
}
