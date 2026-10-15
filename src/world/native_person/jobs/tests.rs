use super::*;
use crate::protocol::QtWriter;

fn string(value: Option<&str>, bytes: &mut Vec<u8>) {
    let mut writer = QtWriter::default();
    writer.write_string(value).unwrap();
    bytes.extend(writer.into_bytes());
}
fn body(kind: u8, tag: u8) -> Vec<u8> {
    let mut bytes = vec![];
    bytes.extend(2_440_588_u32.to_be_bytes());
    bytes.extend(123_u32.to_be_bytes());
    bytes.push(2);
    string(Some("Rust 🌊"), &mut bytes);
    if kind == 24 {
        string(None, &mut bytes);
        bytes.push(tag);
        bytes.extend(i32::MIN.to_be_bytes());
        string(Some(""), &mut bytes);
    } else {
        bytes.extend(u32::MAX.to_be_bytes());
        string(None, &mut bytes);
        bytes.extend(i32::MIN.to_be_bytes());
    }
    bytes
}

#[test]
fn native_type_zero_is_assassin_and_story24_ignores_every_tag_byte() {
    for kind in [0, 20, 24] {
        for tag in 0..=255_u8 {
            let mut bytes = body(kind, tag);
            let length = bytes.len();
            bytes.push(0xee);
            let mut input = QtReader::new(&bytes);
            let job = NativeJob::read_qt7(kind, &mut input).unwrap();
            assert_eq!(job.kind, kind);
            assert_eq!(job.time_08, 123);
            assert_eq!(input.remaining(), bytes.len() - length);
            assert_eq!(input.read_u8().unwrap(), 0xee);
            match job.body {
                NativeJobBody::Assassin {
                    string_10,
                    id_18,
                    string_20,
                    field_28,
                } => {
                    assert_eq!(string_10.as_deref(), Some("Rust 🌊"));
                    assert_eq!(id_18, u32::MAX);
                    assert_eq!(string_20, None);
                    assert_eq!(field_28, i32::MIN);
                }
                NativeJobBody::InterdictSystem {
                    id_1c, field_18, ..
                } => {
                    assert_eq!(id_1c, u32::MAX);
                    assert_eq!(field_18, i32::MIN);
                }
                NativeJobBody::Story {
                    string_18,
                    ignored_wire_byte,
                    field_20,
                    string_28,
                    ..
                } => {
                    assert_eq!(string_18, None);
                    assert_eq!(ignored_wire_byte, tag);
                    assert_eq!(field_20, i32::MIN);
                    assert_eq!(string_28.as_deref(), Some(""));
                }
            }
            for end in 0..length {
                let mut input = QtReader::new(&bytes[..end]);
                assert_eq!(
                    NativeJob::read_qt7(kind, &mut input).unwrap_err().kind(),
                    io::ErrorKind::UnexpectedEof
                );
                assert_eq!(input.remaining(), end);
            }
        }
    }
}

#[test]
fn all_count_bits_and_unknown_factories_stop_without_skipping() {
    for count in 0..=u16::MAX {
        let [hi, lo] = count.to_be_bytes();
        let bytes = [hi, lo, 31, 0xee];
        let mut input = QtReader::new(&bytes);
        let mut jobs = NativeJobList::default();
        let report = jobs.receive_qt7(&mut input).unwrap();
        assert_eq!(report.declared_count, count);
        assert_eq!(
            report.stop,
            if count == 0 {
                None
            } else {
                Some(NativeJobStop {
                    unknown_type: 31,
                    unreceived_count: count - 1,
                })
            }
        );
        assert_eq!(input.remaining(), if count == 0 { 2 } else { 1 });
    }
    for kind in 27..=255_u8 {
        let mut jobs = NativeJobList::default();
        let report = jobs.receive_qt7(&mut QtReader::new(&[0, 1, kind])).unwrap();
        assert_eq!(report.stop.unwrap().unknown_type, kind);
    }
    for kind in 0..=26 {
        if matches!(kind, 0 | 20 | 24) {
            continue;
        }
        assert_eq!(
            NativeJobList::default()
                .receive_qt7(&mut QtReader::new(&[0, 1, kind]))
                .unwrap_err()
                .kind(),
            io::ErrorKind::Unsupported
        );
    }
}

#[test]
fn list_clears_before_count_and_keeps_only_completed_prior_jobs_on_failure() {
    let old = NativeJob::read_qt7(24, &mut QtReader::new(&body(24, 0))).unwrap();
    let mut bytes = vec![0, 2, 24];
    bytes.extend(body(24, 0));
    let boundary = bytes.len();
    bytes.push(0);
    bytes.extend(body(0, 0));
    for end in 0..bytes.len() {
        let mut jobs = NativeJobList(vec![old.clone(), old.clone()]);
        assert_eq!(
            jobs.receive_qt7(&mut QtReader::new(&bytes[..end]))
                .unwrap_err()
                .kind(),
            io::ErrorKind::UnexpectedEof
        );
        assert_eq!(jobs.0.len(), usize::from(end >= boundary));
    }
}

#[test]
fn utc_epoch_math_and_unsupported_local_invalid_datetime_are_explicit() {
    for (day, ms, epoch) in [
        (2_440_588, 0, 0),
        (2_440_587, 86_399_999, -1),
        (2_451_545, 0, 946_684_800_000),
        (1, 0, -210_866_716_800_000),
    ] {
        let mut bytes = body(24, 0);
        bytes[..4].copy_from_slice(&u32::to_be_bytes(day));
        bytes[4..8].copy_from_slice(&u32::to_be_bytes(ms));
        assert_eq!(
            NativeJob::read_qt7(24, &mut QtReader::new(&bytes))
                .unwrap()
                .time_08,
            epoch
        );
    }
    for spec in 0..=255_u8 {
        if spec == 2 {
            continue;
        }
        let mut bytes = body(24, 0);
        bytes[8] = spec;
        assert_eq!(
            NativeJob::read_qt7(24, &mut QtReader::new(&bytes))
                .unwrap_err()
                .kind(),
            io::ErrorKind::Unsupported
        );
    }
    for (range, value) in [(0..4, 0_u32), (4..8, 86_400_000)] {
        let mut bytes = body(24, 0);
        bytes[range].copy_from_slice(&value.to_be_bytes());
        assert_eq!(
            NativeJob::read_qt7(24, &mut QtReader::new(&bytes))
                .unwrap_err()
                .kind(),
            io::ErrorKind::Unsupported
        );
    }
}

#[test]
fn person_composes_job_list_and_keeps_native_null_stop_diagnostic() {
    let mut bytes = vec![0x88, 0x0c, 0x40, 0, 1, 24];
    bytes.extend(body(24, 0xff));
    bytes.extend(i32::MIN.to_be_bytes());
    bytes.extend(i16::MIN.to_be_bytes());
    bytes.extend([0xab, 0xee]);
    let mut state = NativePersonSuffixState::with_prior_fields(0, 0);
    let mut input = QtReader::new(&bytes);
    let report = state
        .receive_suffix(
            &mut input,
            FloatEncoding::TypeWidths,
            std::time::Instant::now,
        )
        .unwrap();
    assert_eq!(report.jobs.unwrap().stop, None);
    assert_eq!(state.jobs_13b8.as_ref().unwrap().0.len(), 1);
    assert_eq!(state.field_10d0, Some(i32::MIN));
    assert_eq!(state.field_f58, Some(i16::MIN));
    assert_eq!(state.field_83d, Some(11));
    assert_eq!(input.read_u8().unwrap(), 0xee);
    let mut bytes = vec![0x88, 0x0c, 0x40, 0, 3, 31];
    bytes.extend(17_i32.to_be_bytes());
    bytes.extend(18_i16.to_be_bytes());
    bytes.push(0xab);
    let report = state
        .receive_suffix(
            &mut QtReader::new(&bytes),
            FloatEncoding::TypeWidths,
            std::time::Instant::now,
        )
        .unwrap();
    assert_eq!(
        report.jobs.unwrap().stop,
        Some(NativeJobStop {
            unknown_type: 31,
            unreceived_count: 2
        })
    );
    assert!(state.jobs_13b8.unwrap().0.is_empty());
    assert_eq!(state.field_10d0, Some(17));
}
