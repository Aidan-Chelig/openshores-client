use super::*;
use crate::protocol::QtWriter;

fn item(kind: u8, commodity: u16) -> Vec<u8> {
    let mut bytes = vec![kind, 0];
    bytes.extend(commodity.to_be_bytes());
    bytes.push(1);
    match kind {
        6 => bytes.push(0x80),
        8 | 9 | 12 => {
            bytes.extend([0, 0x12, 0x34]);
            if kind == 9 {
                bytes.extend([0xff, 0x80, 1, 0]);
            }
            if kind == 12 {
                bytes.push(0x80);
            }
        }
        17 => bytes.extend([0xff; 5]),
        _ => {}
    }
    bytes
}
fn list(items: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = (items.len() as u32).to_be_bytes().to_vec();
    for item in items {
        bytes.extend(item);
    }
    bytes
}

#[test]
fn every_commodity_bit_pattern_matches_native_range_and_bitset_predicate() {
    for commodity in 0..=u16::MAX {
        let raw = u32::from(commodity);
        let expected = raw.wrapping_sub(0x27b) <= 2
            || raw == 0xed
            || if raw <= 0x67 {
                raw >= 0x36 && (0x2000000030001_u64 >> (raw - 0x36)) & 1 != 0
            } else if raw <= 0x119 {
                raw > 0x117
            } else {
                raw == 0x27e
            };
        assert_eq!(
            native_commodity_is_obsolete(commodity),
            expected,
            "{commodity:x}"
        );
    }
}

#[test]
fn every_factory_byte_distinguishes_native_null_from_unsupported_valid_body() {
    for kind in 0..=255_u8 {
        let null = matches!(kind, 0 | 2 | 3 | 11 | 15 | 21 | 25..=255);
        let supported = matches!(kind, 1 | 6 | 7 | 8 | 9 | 12 | 17);
        let mut bytes = 3_u32.to_be_bytes().to_vec();
        if null {
            bytes.push(kind);
        } else {
            bytes.extend(item(kind, 3));
        }
        bytes.extend(item(1, 3));
        bytes.push(0xff);
        bytes.push(0xee);
        let mut input = QtReader::new(&bytes);
        let mut state = NativeItemHash::default();
        let result = state.receive_qt7(&mut input);
        if null || supported {
            let report = result.unwrap();
            assert_eq!(state.0[&3].len(), if null { 1 } else { 2 });
            assert_eq!(
                report.null_factories,
                if null {
                    vec![(0, kind), (2, 0xff)]
                } else {
                    vec![(2, 0xff)]
                }
            );
            assert_eq!(input.read_u8().unwrap(), 0xee);
            assert_eq!(report.consumed_bytes, bytes.len() - 1);
        } else {
            assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Unsupported);
            assert_eq!(input.remaining(), bytes.len() - 4);
        }
    }
}

#[test]
fn obsolete_items_are_fully_received_before_discard_and_duplicates_survive() {
    let obsolete = [
        0x36, 0x46, 0x47, 0x67, 0xed, 0x118, 0x119, 0x27b, 0x27c, 0x27d, 0x27e,
    ];
    for kind in [1, 6, 7, 8, 9, 12, 17] {
        for commodity in obsolete {
            let bytes = list(&[
                item(kind, commodity),
                item(1, 3),
                item(6, 3),
                item(17, 0xffff),
            ]);
            let mut state = NativeItemHash::default();
            let mut input = QtReader::new(&bytes);
            let report = state.receive_qt7(&mut input).unwrap();
            assert_eq!(report.obsolete_items, [(0, commodity)]);
            assert_eq!(state.0.len(), 2);
            assert_eq!(state.0[&3].len(), 2);
            assert_eq!(state.0[&3][0].extension.type_id(), 1);
            assert_eq!(state.0[&3][1].extension.type_id(), 6);
            assert_eq!(state.0[&0xffff][0].common.commodity, -1);
            assert_eq!(input.remaining(), 0);
            let body = item(kind, commodity);
            for end in 0..body.len() {
                let mut bytes = 1_u32.to_be_bytes().to_vec();
                bytes.extend(&body[..end]);
                assert_eq!(
                    NativeItemHash::default()
                        .receive_qt7(&mut QtReader::new(&bytes))
                        .unwrap_err()
                        .kind(),
                    io::ErrorKind::UnexpectedEof
                );
            }
        }
    }
}

#[test]
fn count_is_full_width_and_never_preallocates_from_missing_body_count() {
    for count in [0_u32, 1, 255, 256, 65536] {
        let mut bytes = count.to_be_bytes().to_vec();
        bytes.resize(4 + count as usize, 0);
        bytes.push(0xee);
        let mut state = NativeItemHash::default();
        let mut input = QtReader::new(&bytes);
        let report = state.receive_qt7(&mut input).unwrap();
        assert_eq!(report.declared_count_bits, count);
        assert_eq!(report.null_factories.len(), count as usize);
        assert!(state.0.is_empty());
        assert_eq!(input.read_u8().unwrap(), 0xee);
    }
    for count in [0x7fff_ffff_u32, 0x8000_0000, u32::MAX] {
        let bytes = count.to_be_bytes();
        assert_eq!(
            NativeItemHash::default()
                .receive_qt7(&mut QtReader::new(&bytes))
                .unwrap_err()
                .kind(),
            io::ErrorKind::UnexpectedEof
        );
    }
}

#[test]
fn replacement_clears_before_count_and_keeps_only_completed_prior_inserts() {
    let old = NativeItemValue::read(&mut QtReader::new(&item(1, 3)))
        .unwrap()
        .unwrap();
    let first = item(1, 3);
    let second = item(17, 4);
    let bytes = list(&[first.clone(), second]);
    let boundary = 4 + first.len();
    for end in 0..bytes.len() {
        let mut state = NativeItemHash(BTreeMap::from([(99, vec![old.clone()])]));
        assert_eq!(
            state
                .receive_qt7(&mut QtReader::new(&bytes[..end]))
                .unwrap_err()
                .kind(),
            io::ErrorKind::UnexpectedEof
        );
        assert!(!state.0.contains_key(&99));
        assert_eq!(state.0.contains_key(&3), end >= boundary);
    }
    let bytes = list(&[item(1, 3), item(1, 2), item(1, 4)]);
    let mut input = QtReader::new(&bytes);
    let mut state = NativeItemHash::default();
    assert_eq!(
        state.receive_qt7(&mut input).unwrap_err().kind(),
        io::ErrorKind::Unsupported
    );
    assert_eq!(state.0[&3].len(), 1);
    assert!(!state.0.contains_key(&2));
    assert!(!state.0.contains_key(&4));
    assert_eq!(input.remaining_bytes(), item(1, 4));
}

#[test]
fn person_inventory_runs_after_state_bits_and_before_identity_and_nibble_tails() {
    for profile in [
        FloatEncoding::TypeWidths,
        FloatEncoding::SinglePrecision,
        FloatEncoding::DoublePrecision,
    ] {
        let mut bytes = vec![0x89, 0x40];
        bytes.extend(7_u32.to_be_bytes());
        let mut strings = QtWriter::default();
        strings.write_string(None).unwrap();
        bytes.extend(strings.into_bytes());
        bytes.extend(8_u32.to_be_bytes());
        let mut strings = QtWriter::default();
        strings.write_string(Some("Avatar 🌊")).unwrap();
        bytes.extend(strings.into_bytes());
        for value in [1.25_f64, -2.5] {
            if profile == FloatEncoding::DoublePrecision {
                bytes.extend(value.to_be_bytes());
            } else {
                bytes.extend((value as f32).to_be_bytes());
            }
        }
        bytes.extend(9_u32.to_be_bytes());
        bytes.push(0);
        bytes.extend(u32::MAX.to_be_bytes());
        bytes.push(0xc5);
        bytes.extend(list(&[item(17, 0xed), item(1, 3), item(6, 3)]));
        bytes.extend([0xfe, 0xab, 0xee]);
        let mut input = QtReader::new(&bytes);
        let mut state = NativePersonSuffixState::with_prior_fields(0, 0x8000);
        let report = state
            .receive_suffix(&mut input, profile, std::time::Instant::now)
            .unwrap();
        assert_eq!(report.inventory.unwrap().obsolete_items, [(0, 0xed)]);
        assert_eq!(state.inventory_1458.as_ref().unwrap().0[&3].len(), 2);
        assert_eq!(state.state_1130, Some(0x200000));
        assert_eq!(state.field_10e8, Some(0xfe));
        assert_eq!(state.field_83d, Some(11));
        assert_eq!(state.field_83b, Some(10));
        assert_eq!(input.read_u8().unwrap(), 0xee);
        let old = state.inventory_1458.clone();
        let _ = state
            .receive_suffix(&mut QtReader::new(&[0]), profile, std::time::Instant::now)
            .unwrap();
        assert_eq!(state.inventory_1458, old);
    }
}
