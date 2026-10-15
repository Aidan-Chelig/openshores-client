use super::*;
use crate::protocol::QtWriter;
mod scalars;

fn single(value: f64, profile: FloatEncoding, bytes: &mut Vec<u8>) {
    if profile == FloatEncoding::DoublePrecision {
        bytes.extend(value.to_be_bytes());
    } else {
        bytes.extend((value as f32).to_be_bytes());
    }
}
fn double(value: f64, profile: FloatEncoding, bytes: &mut Vec<u8>) {
    if profile == FloatEncoding::SinglePrecision {
        bytes.extend((value as f32).to_be_bytes());
    } else {
        bytes.extend(value.to_be_bytes());
    }
}
fn string(value: Option<&str>, bytes: &mut Vec<u8>) {
    let mut writer = QtWriter::default();
    writer.write_string(value).unwrap();
    bytes.extend(writer.into_bytes());
}
fn packet(a: u8, b: u8, c: u8, profile: FloatEncoding) -> Vec<u8> {
    let mut bytes = vec![a];
    if a & 0x80 != 0 {
        bytes.push(b);
    }
    if a & 1 != 0 {
        bytes.extend(u32::MAX.to_be_bytes());
        string(None, &mut bytes);
        bytes.extend(77_u32.to_be_bytes());
        string(Some("Rust 🌊"), &mut bytes);
        single(2.5, profile, &mut bytes);
        single(-1.25, profile, &mut bytes);
        bytes.extend(88_u32.to_be_bytes());
        bytes.push(0xff);
        string(Some(""), &mut bytes);
    }
    if a & 2 != 0 {
        for value in [1., 2., 3.] {
            single(value, profile, &mut bytes);
        }
        bytes.push(1);
        bytes.extend(28_i32.to_be_bytes());
        bytes.extend(20_i32.to_be_bytes());
        bytes.push(0xff);
        bytes.extend(44_i32.to_be_bytes());
        bytes.extend(24_i32.to_be_bytes());
    }
    if a & 4 != 0 {
        bytes.extend(0x9234_u16.to_be_bytes());
    }
    if a & 8 != 0 {
        bytes.push(c);
    }
    if a & 0x10 != 0 {
        bytes.push(0x98);
        bytes.extend(i32::MIN.to_be_bytes());
    }
    if a & 0x20 != 0 {
        bytes.extend([0xff, 0x8f, 3, 0, 0x80, 0xff, 0xfe]);
    }
    if a & 0x40 != 0 {
        bytes.extend([2, 0xab, 0, 0xcd, 0]);
    }
    let b = if a & 0x80 != 0 { b } else { 0 };
    if b & 1 != 0 {
        bytes.push(2);
        bytes.extend(7_u32.to_be_bytes());
        string(Some("old"), &mut bytes);
        bytes.extend(7_u32.to_be_bytes());
        string(Some("new"), &mut bytes);
    }
    if b & 8 != 0 {
        bytes.extend(i32::MIN.to_be_bytes());
        bytes.extend(i16::MIN.to_be_bytes());
    }
    if b & 0x10 != 0 {
        double(1.00000000001, profile, &mut bytes);
    }
    if b & 0x20 != 0 {
        double(-0., profile, &mut bytes);
        double(8., profile, &mut bytes);
    }
    if a & 1 != 0 {
        bytes.push(0xed);
    }
    if a & 8 != 0 && c & 0x40 != 0 {
        bytes.push(0xab);
    }
    bytes
}

#[test]
fn all_supported_flag_pairs_profiles_compose_real_readers_and_late_tail() {
    for profile in [
        FloatEncoding::TypeWidths,
        FloatEncoding::SinglePrecision,
        FloatEncoding::DoublePrecision,
    ] {
        for a in 0..=255_u8 {
            for b in 0..=255_u8 {
                if a & 0x80 != 0 && b & 4 != 0 {
                    continue;
                }
                let mut bytes = packet(a, b, 0x7f, profile);
                let length = bytes.len();
                bytes.push(0xee);
                let mut input = QtReader::new(&bytes);
                let mut state = NativePersonSuffixState::with_prior_fields(0, 0x41008001);
                let now = Instant::now();
                let report = state.receive_suffix(&mut input, profile, || now).unwrap();
                let b = if a & 0x80 != 0 { b } else { 0 };
                assert_eq!(report.consumed_bytes, length);
                assert_eq!(
                    (report.flags_a, report.flags_b, report.flags_c),
                    (a, b, if a & 8 != 0 { 0x7f } else { 0 })
                );
                assert_eq!(input.read_u8().unwrap(), 0xee);
                assert_eq!(state.field_838, Some(if a & 8 != 0 { 0x3f } else { 0 }));
                assert_eq!(report.timer_restarted, a & 4 != 0);
                assert_eq!(state.timer_668, if a & 4 != 0 { Some(now) } else { None });
                let expected = (0x41008001 & !0x8000)
                    | (if b & 0x80 != 0 { 0x8000 } else { 0 })
                    | (if b & 0x40 != 0 { 0x200000 } else { 0 });
                assert_eq!(state.state_1130, Some(expected));
                assert_eq!(state.field_f80, (a & 0x10 != 0).then_some(i32::MIN));
                assert_eq!(state.field_f84, (a & 0x10 != 0).then_some(1));
                assert_eq!(state.field_f88, (a & 0x10 != 0).then_some(3));
                if a & 1 != 0 {
                    assert_eq!(state.identity.id_10ec, Some(u32::MAX));
                    assert_eq!(state.identity.string_10f0, Some(None));
                    assert_eq!(
                        state.identity.string_1100.as_ref().unwrap().as_deref(),
                        Some("Rust 🌊")
                    );
                    assert_eq!(
                        state.identity.globe_1108,
                        Some(NativeGlobeLlf {
                            field_00: -1.25,
                            field_04: 2.5
                        })
                    );
                    assert_eq!(state.identity.field_1114, Some(true));
                    assert_eq!(state.field_10e8, Some(0xed));
                }
                if a & 0x20 != 0 {
                    assert_eq!(
                        state.content_fa0.as_deref(),
                        Some([0, 0x80, 0xff].as_slice())
                    );
                    assert_eq!(state.field_fb0, Some(0xfe));
                }
                if a & 0x40 != 0 {
                    assert_eq!(state.weight_13a8, Some(0.));
                    assert_eq!(report.gear.unwrap().null_slots, [0, 1]);
                }
                if b & 1 != 0 {
                    assert_eq!(
                        state.domain_1018.as_ref().unwrap().0[&7].as_deref(),
                        Some("new")
                    );
                }
                if b & 8 != 0 {
                    assert_eq!(state.field_10d0, Some(i32::MIN));
                    assert_eq!(state.field_f58, Some(i16::MIN));
                }
                if b & 0x10 != 0 {
                    assert_eq!(
                        state.field_1408,
                        Some(if profile == FloatEncoding::SinglePrecision {
                            1.
                        } else {
                            1.00000000001
                        })
                    );
                }
                if b & 0x20 != 0 {
                    assert_eq!(state.field_f60.unwrap().to_bits(), (-0_f64).to_bits());
                    assert_eq!(state.field_f68, Some(8.));
                }
            }
        }
    }
}

#[test]
fn every_packed_short_only_restarts_on_low_fifteen_bit_change() {
    let now = Instant::now();
    for packed in 0..=u16::MAX {
        let [hi, lo] = packed.to_be_bytes();
        for old in [packed & 0x7fff, (packed & 0x7fff) ^ 1] {
            let mut state = NativePersonSuffixState::with_prior_fields(old, 0);
            let mut calls = 0;
            let report = state
                .receive_suffix(
                    &mut QtReader::new(&[4, hi, lo]),
                    FloatEncoding::TypeWidths,
                    || {
                        calls += 1;
                        now
                    },
                )
                .unwrap();
            assert_eq!(calls, usize::from(old != packed & 0x7fff));
            assert_eq!(report.timer_restarted, calls != 0);
            assert_eq!(state.field_660, Some(packed & 0x7fff));
            assert_eq!(state.field_839, Some((packed >> 15) as u8));
        }
    }
}

#[test]
fn repeated_sparse_update_resets_only_the_native_non_sparse_fields() {
    let mut state = NativePersonSuffixState::with_prior_fields(0, 0);
    let _ = state
        .receive_suffix(
            &mut QtReader::new(&packet(0xff, 0xfb, 0x7f, FloatEncoding::TypeWidths)),
            FloatEncoding::TypeWidths,
            Instant::now,
        )
        .unwrap();
    let old = state.clone();
    let _ = state
        .receive_suffix(&mut QtReader::new(&[0]), FloatEncoding::TypeWidths, || {
            panic!("unchanged timer")
        })
        .unwrap();
    let mut expected = old;
    expected.field_838 = Some(0);
    expected.state_1130 = Some(expected.state_1130.unwrap() & !0x8000);
    assert_eq!(state, expected);
    assert_ne!(state.state_1130.unwrap() & 0x200000, 0);
}

#[test]
fn every_truncated_supported_suffix_fails_without_claiming_success() {
    for profile in [
        FloatEncoding::TypeWidths,
        FloatEncoding::SinglePrecision,
        FloatEncoding::DoublePrecision,
    ] {
        let bytes = packet(0xff, 0xfb, 0x7f, profile);
        for end in 0..bytes.len() {
            let mut state = NativePersonSuffixState::with_prior_fields(0, 0);
            assert_eq!(
                state
                    .receive_suffix(&mut QtReader::new(&bytes[..end]), profile, Instant::now)
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::UnexpectedEof,
                "{end}"
            );
        }
    }
}

#[test]
fn unsupported_readers_stop_in_order_and_keep_completed_prior_state() {
    // Jobs stop after domain, before the extended scalars/state bits.
    let bytes = [0x80, 0xcd, 0, 0xaa, 0xbb];
    let mut input = QtReader::new(&bytes);
    let mut state = NativePersonSuffixState::with_prior_fields(0, 0x8000);
    assert_eq!(
        state
            .receive_suffix(&mut input, FloatEncoding::TypeWidths, Instant::now)
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
    assert!(state.domain_1018.unwrap().0.is_empty());
    assert_eq!(state.state_1130, Some(0x8000));
    assert_eq!(input.remaining_bytes(), [0xaa, 0xbb]);
    // Inventory stops after state-bit changes, before its body or late tails.
    let mut state = NativePersonSuffixState::with_prior_fields(0, 0x8000);
    let bytes = [0x88, 0x40, 0xc5, 0xaa, 0xbb];
    let mut input = QtReader::new(&bytes);
    assert_eq!(
        state
            .receive_suffix(&mut input, FloatEncoding::TypeWidths, Instant::now)
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
    assert_eq!(state.field_838, Some(5));
    assert_eq!(state.state_1130, Some(0x200000));
    assert_eq!(state.field_83d, None);
    assert_eq!(input.remaining_bytes(), [0xaa, 0xbb]);
    // Nonempty Gear really receives its item, then fails at weight, not before.
    let bytes = [0xc0, 1, 1, 0xab, 1, 0, 0, 3, 1, 0xaa];
    let mut state = NativePersonSuffixState::with_prior_fields(0, 0x8000);
    let mut input = QtReader::new(&bytes);
    assert_eq!(
        state
            .receive_suffix(&mut input, FloatEncoding::TypeWidths, Instant::now)
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
    assert_eq!(state.gear_1390.as_ref().unwrap().0.len(), 1);
    assert_eq!(state.weight_13a8, None);
    assert_eq!(state.domain_1018, None);
    assert_eq!(input.remaining_bytes(), [0xaa]);
}

#[test]
fn absent_prior_state_is_not_silently_initialized_to_zero() {
    let mut state = NativePersonSuffixState::default();
    assert_eq!(
        state
            .receive_suffix(
                &mut QtReader::new(&[0]),
                FloatEncoding::TypeWidths,
                Instant::now
            )
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
    assert_eq!(state.field_838, Some(0));
    assert_eq!(state.state_1130, None);
    let mut state = NativePersonSuffixState::default();
    assert_eq!(
        state
            .receive_suffix(
                &mut QtReader::new(&[4, 0, 1]),
                FloatEncoding::TypeWidths,
                Instant::now
            )
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
    assert_eq!(state.field_660, None);
    assert_eq!(state.field_838, None);
}
