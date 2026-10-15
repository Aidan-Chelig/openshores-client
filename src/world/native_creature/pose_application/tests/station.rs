use super::*;

#[test]
fn station_requests_match_independently_decoded_native_jump_table_and_branch_immediates() {
    let text = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/docs/evidence/native-station-pose-map.csv"
    ));
    let mut count = 0;
    for line in text.lines().filter(|line| !line.starts_with('#')) {
        let columns: Vec<_> = line.split(',').collect();
        assert_eq!(columns.len(), 5);
        let word: u32 = columns[0].parse().unwrap();
        let occupant: u8 = columns[3].parse().unwrap();
        assert_eq!(native_station_occupant_position(word), occupant);
        let request = NativeStationPoseRequest::for_pose_word(word, 0xff81, 0x8001);
        if columns[1] == "none" {
            assert_eq!(request, None);
        } else {
            assert_eq!(
                request,
                Some(NativeStationPoseRequest {
                    station: 0xff81,
                    station_type: columns[1].parse().unwrap(),
                    occupant_position: occupant,
                    variant: columns[2].parse().unwrap(),
                    trailing_station: 0x8001,
                })
            );
        }
        count += 1;
    }
    assert_eq!(count, 133);
    for word in [0, 1, 2, 136, 255, 256, 0x7fff_ffff, 0x8000_0000, u32::MAX] {
        assert_eq!(native_station_occupant_position(word), 3);
        assert_eq!(NativeStationPoseRequest::for_pose_word(word, 1, 2), None);
    }
}

#[test]
fn body_dependent_pose_pointer_is_not_the_same_word_and_station_type_zero_is_valid() {
    for station in 0..=u16::MAX {
        let pose = NativeCreaturePose {
            fields: [3, 0, 0, 5, 0, 0, 0, 0, 0, 0],
            station,
        };
        let animal =
            NativeStationPoseRequest::from_received_pose(NativeCreatureBodyKind::Animalia, &pose)
                .unwrap();
        let dp =
            NativeStationPoseRequest::from_received_pose(NativeCreatureBodyKind::DpBody, &pose)
                .unwrap();
        assert_eq!(animal.station_type, 5);
        assert_eq!(dp.station_type, 1);
        assert_eq!(
            (animal.station, animal.trailing_station),
            (station, station)
        );
        assert_eq!((dp.station, dp.trailing_station), (station, station));
    }
    let request = NativeStationPoseRequest::for_pose_word(0x41, 0, 0).unwrap();
    assert_eq!(request.station_type, 0);
    assert_eq!(request.variant, 0);
    // These words have occupant positions, but are NOT accepted SetStationPose cases.
    for word in [4, 13, 16, 0x4a, 0x53] {
        assert_eq!(NativeStationPoseRequest::for_pose_word(word, 0, 0), None);
    }
}
