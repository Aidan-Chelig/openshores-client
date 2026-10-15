use super::*;
mod body;
mod initialization;
mod operator;
mod station;

#[test]
fn marker_update_tests_five_c_zero_not_four_nine_two_and_restarts_only_when_required() {
    let start = Instant::now();
    for old in [0_u16, 0x7fff, 0x8000, 0xffff] {
        for station in 0..=u16::MAX {
            for changed in [false, true] {
                let mut value = NativeCreaturePoseMarkers {
                    field_5c0: old,
                    field_492: old ^ 0xffff,
                    last_restart: Some(start),
                };
                let previous = value.clone();
                let mut samples = 0;
                let now = start + std::time::Duration::from_millis(7);
                let restarted = value.after_body_pose_with_clock(changed, station, || {
                    samples += 1;
                    now
                });
                assert_eq!(restarted, changed || old != station);
                assert_eq!(samples, usize::from(restarted));
                if restarted {
                    assert_eq!(value.field_5c0, station);
                    assert_eq!(value.field_492, station);
                    assert_eq!(value.last_restart, Some(now));
                } else {
                    assert_eq!(value, previous);
                }
            }
        }
    }
}

#[test]
fn real_monotonic_marker_path_starts_invalid_and_restarts_identical_changed_poses() {
    let mut value = NativeCreaturePoseMarkers::default();
    assert_eq!(
        (value.field_5c0, value.field_492, value.last_restart),
        (0, 0, None)
    );
    assert!(!value.after_body_pose(false, 0));
    assert!(value.last_restart.is_none());
    let before = Instant::now();
    assert!(value.after_body_pose(true, 0));
    let first = value.last_restart.unwrap();
    assert!(first >= before);
    assert!(value.after_body_pose(true, 0));
    assert!(value.last_restart.unwrap() >= first);
}
