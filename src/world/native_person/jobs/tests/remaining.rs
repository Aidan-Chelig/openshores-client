use super::*;

fn fixture(kind: u8, profile: FloatEncoding) -> Vec<u8> {
    let mut bytes = vec![];
    bytes.extend(2_440_588_u32.to_be_bytes());
    bytes.extend(123_u32.to_be_bytes());
    bytes.push(2);
    if kind <= 3 {
        string(None, &mut bytes);
        bytes.extend(u32::MAX.to_be_bytes());
        string(Some(""), &mut bytes);
        bytes.extend(3_u32.to_be_bytes());
        for value in [None, Some(""), Some("Bounty 🌊")] {
            string(value, &mut bytes);
        }
        if profile == FloatEncoding::SinglePrecision {
            bytes.extend(1_f32.to_be_bytes());
        } else {
            bytes.extend(1.00000000001_f64.to_be_bytes());
        }
    } else if kind <= 5 {
        string(None, &mut bytes);
        string(Some("Command 🌊"), &mut bytes);
        bytes.extend(i32::MIN.to_be_bytes());
        bytes.extend(u32::MAX.to_be_bytes());
        string(Some(""), &mut bytes);
        bytes.extend(77_u32.to_be_bytes());
        string(None, &mut bytes);
        string(Some("tail"), &mut bytes);
    } else {
        assert_eq!(kind, 25);
        bytes.extend(u32::MAX.to_be_bytes());
        string(None, &mut bytes);
        string(Some(""), &mut bytes);
        bytes.extend(77_u32.to_be_bytes());
        string(Some("Prisoners 🌊"), &mut bytes);
        bytes.extend(i32::MIN.to_be_bytes());
        bytes.extend(i32::MAX.to_be_bytes());
        bytes.extend(88_u32.to_be_bytes());
        string(None, &mut bytes);
        string(Some(""), &mut bytes);
        bytes.extend(99_u32.to_be_bytes());
        string(Some("tail"), &mut bytes);
    }
    bytes
}

#[test]
fn remaining_six_types_decode_real_fields_profiles_tails_and_every_truncation() {
    for profile in [
        FloatEncoding::TypeWidths,
        FloatEncoding::SinglePrecision,
        FloatEncoding::DoublePrecision,
    ] {
        for kind in [1, 2, 3, 4, 5, 25] {
            let mut bytes = fixture(kind, profile);
            let length = bytes.len();
            bytes.push(0xee);
            let mut input = QtReader::new(&bytes);
            let job = NativeJob::read_qt7_with_profile(kind, &mut input, profile).unwrap();
            assert_eq!(job.kind, kind);
            assert_eq!(job.time_08, 123);
            assert_eq!(input.read_u8().unwrap(), 0xee);
            let expected = match kind {
                1..=3 => NativeJobBody::BountyHunter(NativeJobBountyHunter {
                    string_10: None,
                    id_18: u32::MAX,
                    string_20: Some(String::new()),
                    strings_28: vec![None, Some(String::new()), Some("Bounty 🌊".into())],
                    field_30: if profile == FloatEncoding::SinglePrecision {
                        1.
                    } else {
                        1.00000000001
                    },
                }),
                4 | 5 => NativeJobBody::ShipCommand(NativeJobShipCommand {
                    string_10: None,
                    string_18: Some("Command 🌊".into()),
                    ignored_wire_i32: i32::MIN,
                    id_20: u32::MAX,
                    string_28: Some(String::new()),
                    id_30: 77,
                    string_38: None,
                    string_40: Some("tail".into()),
                }),
                25 => NativeJobBody::TransferPrisoners(NativeJobTransferPrisoners {
                    id_10: u32::MAX,
                    string_18: None,
                    string_20: Some(String::new()),
                    id_28: 77,
                    string_30: Some("Prisoners 🌊".into()),
                    field_38: i32::MIN,
                    field_3c: i32::MAX,
                    id_40: 88,
                    string_48: None,
                    string_50: Some(String::new()),
                    id_58: 99,
                    string_60: Some("tail".into()),
                }),
                _ => unreachable!(),
            };
            assert_eq!(job.body, expected);
            for end in 0..length {
                let mut input = QtReader::new(&bytes[..end]);
                assert_eq!(
                    NativeJob::read_qt7_with_profile(kind, &mut input, profile)
                        .unwrap_err()
                        .kind(),
                    io::ErrorKind::UnexpectedEof
                );
                assert_eq!(input.remaining(), end);
            }
        }
    }
}

#[test]
fn obsolete_jobs_are_decoded_before_discard_and_person_continues_after_them() {
    for profile in [
        FloatEncoding::TypeWidths,
        FloatEncoding::SinglePrecision,
        FloatEncoding::DoublePrecision,
    ] {
        let mut bytes = vec![0x88, 0x0c, 0x40, 0, 6];
        for kind in [1, 4, 2, 5, 3, 25] {
            bytes.push(kind);
            bytes.extend(fixture(kind, profile));
        }
        bytes.extend(17_i32.to_be_bytes());
        bytes.extend(18_i16.to_be_bytes());
        bytes.extend([0xab, 0xee]);
        let mut state = NativePersonSuffixState::with_prior_fields(0, 0);
        let mut input = QtReader::new(&bytes);
        let report = state
            .receive_suffix(&mut input, profile, std::time::Instant::now)
            .unwrap();
        let report = report.jobs.unwrap();
        assert_eq!(report.obsolete_jobs, [(0, 1), (2, 2), (4, 3)]);
        assert_eq!(report.stop, None);
        assert_eq!(
            state
                .jobs_13b8
                .unwrap()
                .0
                .iter()
                .map(|job| job.kind)
                .collect::<Vec<_>>(),
            [4, 5, 25]
        );
        assert_eq!(state.field_10d0, Some(17));
        assert_eq!(state.field_f58, Some(18));
        assert_eq!(state.field_83d, Some(11));
        assert_eq!(input.read_u8().unwrap(), 0xee);
        let bytes = fixture(1, profile);
        for end in 0..bytes.len() {
            let mut list = vec![0, 2, 24];
            list.extend(body(24, 0));
            list.push(1);
            list.extend(&bytes[..end]);
            let mut state = NativeJobList::default();
            assert_eq!(
                state
                    .receive_qt7_with_profile(&mut QtReader::new(&list), profile)
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::UnexpectedEof
            );
            assert_eq!(state.0.len(), 1);
            assert_eq!(state.0[0].kind, 24);
        }
    }
}

#[test]
fn bounty_string_list_is_a_full_width_count_not_a_byte_or_short() {
    for count in [0_u32, 1, 255, 256, 65536] {
        let mut bytes = vec![];
        bytes.extend(2_440_588_u32.to_be_bytes());
        bytes.extend(0_u32.to_be_bytes());
        bytes.push(2);
        string(None, &mut bytes);
        bytes.extend(0_u32.to_be_bytes());
        string(None, &mut bytes);
        bytes.extend(count.to_be_bytes());
        for _ in 0..count {
            string(None, &mut bytes);
        }
        bytes.extend((-0_f64).to_be_bytes());
        let job = NativeJob::read_qt7(1, &mut QtReader::new(&bytes)).unwrap();
        let NativeJobBody::BountyHunter(bounty) = job.body else {
            panic!("wrong job");
        };
        assert_eq!(bounty.strings_28.len(), count as usize);
        assert_eq!(bounty.field_30.to_bits(), (-0_f64).to_bits());
    }
    let mut bytes = vec![];
    bytes.extend(2_440_588_u32.to_be_bytes());
    bytes.extend(0_u32.to_be_bytes());
    bytes.push(2);
    string(None, &mut bytes);
    bytes.extend(0_u32.to_be_bytes());
    string(None, &mut bytes);
    bytes.extend(u32::MAX.to_be_bytes());
    assert_eq!(
        NativeJob::read_qt7(1, &mut QtReader::new(&bytes))
            .unwrap_err()
            .kind(),
        io::ErrorKind::UnexpectedEof
    );
}
