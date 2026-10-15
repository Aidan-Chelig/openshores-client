use super::*;

#[test]
fn initialize_poses_keeps_general_and_animation_only_domains_separate() {
    for raw in (0..=65535).chain([0x7fff_ffff, 0x8000_0000, u32::MAX]) {
        let animal = native_animalia_initialize_poses(raw);
        if (0x8b..=0x9f).contains(&raw) {
            assert_eq!(&animal[..9], &[0x24; 9]);
            assert_eq!(animal[9], raw);
        } else {
            assert_eq!(&animal[..9], &[raw; 9]);
            assert_eq!(animal[9], 0x8c);
        }
        assert_eq!(
            native_station_body_pose(NativeCreatureBodyKind::Animalia, raw),
            animal
        );
        assert_eq!(
            native_station_body_pose(NativeCreatureBodyKind::DpBody, raw),
            [raw; 10]
        );
    }
}

#[test]
fn current_general_pose_prioritizes_dp_index_three_not_array_zero_or_animal_animation() {
    for raw in [0, 3, 0x8c, 255, 0x7fff_ffff, 0x8000_0000, u32::MAX] {
        let dp = NativeDpBodyPoseState {
            fields_3c: [11, 12, 13, raw, 15, 16, 17, 18, 19, 20],
            field_6c: true,
        };
        let animal = NativeAnimaliaPoseState {
            fields_6c: [raw, 22, 23, 24, 25, 26, 27, 28, 29, 0x9f],
            field_40: true,
            field_94: 1.,
        };
        assert_eq!(native_current_general_pose(Some(&dp), None).unwrap(), raw);
        assert_eq!(
            native_current_general_pose(None, Some(&animal)).unwrap(),
            raw
        );
        let other = NativeAnimaliaPoseState {
            fields_6c: [raw ^ 0xffff_ffff; 10],
            ..animal
        };
        assert_eq!(
            native_current_general_pose(Some(&dp), Some(&other)).unwrap(),
            raw
        );
    }
    assert_eq!(
        native_current_general_pose(None, None).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
}

#[test]
fn console_marker_always_sets_trailing_short_without_forcing_a_timer_restart() {
    let start = Instant::now();
    let now = start + std::time::Duration::from_millis(9);
    for station in 0..=u16::MAX {
        for changed in [false, true] {
            let trailing = station ^ 0x8001;
            let mut value = NativeCreaturePoseMarkers {
                field_5c0: station,
                field_492: 0,
                last_restart: Some(start),
            };
            let mut calls = 0;
            assert_eq!(
                value.after_console_pose_with_clock(changed, station, trailing, || {
                    calls += 1;
                    now
                }),
                changed
            );
            assert_eq!(calls, usize::from(changed));
            assert_eq!(value.field_5c0, station);
            assert_eq!(value.field_492, trailing);
            assert_eq!(value.last_restart, Some(if changed { now } else { start }));
            assert!(value.after_console_pose_with_clock(false, station ^ 1, trailing, || now));
            assert_eq!(value.field_5c0, station ^ 1);
            assert_eq!(value.field_492, trailing);
            assert_eq!(value.last_restart, Some(now));
        }
    }
    let mut value = NativeCreaturePoseMarkers::default();
    assert!(!value.after_console_pose(false, 0, 0xffff));
    assert_eq!(value.field_492, 0xffff);
    assert_eq!(value.last_restart, None);
}
