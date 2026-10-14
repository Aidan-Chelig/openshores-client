use super::*;
use crate::atom_position::PackedPosition;
use std::time::{Duration, Instant};

#[test]
fn base_initialization_retains_status_orientation_and_invalid_timer_distinctions() {
    let base = NativeBaseAtomInitialization::from_default_base_constructor();
    assert_eq!(base.wire_id, 0);
    assert_eq!(base.field_1c, 5);
    assert!(!base.owner_attached);
    assert_eq!(base.received_time, 0);
    assert_eq!(base.position.position.parent, 0);
    assert_eq!(base.position.orientation, Some([0.; 3]));
    assert_eq!(NativePositionState::default().orientation, None);
    assert!(!base.position.environment_cache_valid);
    assert_eq!(base.position.position_update_age_at(Instant::now()), None);
    for column in 0..4 {
        for row in 0..4 {
            assert_eq!(
                base.position.position.local[column][row],
                f64::from(column == row)
            );
        }
    }
    assert_eq!(base.environment_matrix, base.position.position.local);
}

#[test]
fn first_unowned_position_starts_timer_but_does_not_invent_owner_or_orientation() {
    let mut base = NativeBaseAtomInitialization::from_default_base_constructor();
    let now = Instant::now();
    base.position
        .apply_fresh_wire_position_at(
            &PackedPosition {
                flags: 0xff,
                parent: Some(7),
                position: [12.5, 2., 3.],
                rotation: [0.3, -0.7, 1.1],
            },
            base.owner_attached,
            now,
        )
        .unwrap();
    assert_eq!(base.position.position.parent, 7);
    assert_eq!(base.position.position.local[3][0], 12.5);
    assert_eq!(base.position.orientation, Some([0.; 3]));
    assert_eq!(
        base.position.position_update_age_at(now),
        Some(Duration::ZERO)
    );
    assert_eq!(base.field_1c, 5);
    assert_eq!(base.wire_id, 0);
    assert!(!base.owner_attached);
    assert_eq!(base.received_time, 0);
}
