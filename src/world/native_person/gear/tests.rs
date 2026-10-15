use super::*;
use std::time::Instant;

fn slot(packed: u8, kind: u8, commodity: i16) -> Vec<u8> {
    let mut bytes = vec![packed, kind, 0];
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
        17 => bytes.extend([0xff, 0xff, 0xff, 0xff, 0xff]),
        _ => {}
    }
    bytes
}

#[test]
fn packed_slots_use_both_nibbles_without_merging_duplicates() {
    for packed in 0..=255_u8 {
        for kind in [1, 6, 7, 8, 9, 12, 17] {
            let bytes = slot(packed, kind, 3);
            let mut input = QtReader::new(&bytes);
            let result = NativeGearSlot::read(&mut input).unwrap().unwrap();
            assert_eq!(result.field_08, packed & 15);
            assert_eq!(result.field_09, packed >> 4);
            assert_eq!(result.item.extension.type_id(), kind);
            assert_eq!(input.remaining(), 0);
        }
    }
    let mut bytes = vec![2];
    bytes.extend(slot(0xab, 1, 3));
    bytes.extend(slot(0xab, 6, 4));
    let mut gear = NativeGear::default();
    let report = gear.receive(&mut QtReader::new(&bytes)).unwrap();
    assert!(report.null_slots.is_empty());
    assert_eq!(gear.0.len(), 2);
    assert_eq!(gear.0[0].item.common.commodity, 3);
    assert_eq!(gear.0[1].item.common.commodity, 4);
}

#[test]
fn count_uses_all_eight_bits_and_null_slots_continue_without_append() {
    for count in 0..=255_u8 {
        let mut bytes = vec![count];
        for index in 0..count {
            if index % 2 == 0 {
                bytes.extend([index, 0]);
            } else {
                bytes.extend(slot(index, 1, 3));
            }
        }
        let length = bytes.len();
        bytes.push(0xee);
        let mut gear = NativeGear::default();
        let mut input = QtReader::new(&bytes);
        let report = gear.receive(&mut input).unwrap();
        assert_eq!(report.wire_count, count);
        assert_eq!(report.consumed_bytes, length);
        assert_eq!(
            report.null_slots,
            (0..count).filter(|n| n % 2 == 0).collect::<Vec<_>>()
        );
        assert_eq!(gear.0.len(), usize::from(count / 2));
        assert_eq!(input.read_u8().unwrap(), 0xee);
    }
}

#[test]
fn unsupported_factory_and_special_fixup_are_not_silent_null_slots() {
    for bytes in [vec![0xab, 4], slot(0xab, 1, 2)] {
        let mut input = QtReader::new(&bytes);
        assert_eq!(
            NativeGearSlot::read(&mut input).unwrap_err().kind(),
            io::ErrorKind::Unsupported
        );
        assert_eq!(input.remaining(), bytes.len());
    }
}

#[test]
fn every_factory_byte_keeps_null_factories_distinct_from_unimplemented_bodies() {
    for kind in 0..=255_u8 {
        let null = matches!(kind, 0 | 2 | 3 | 11 | 15 | 21 | 25..=255);
        let supported = matches!(kind, 1 | 6 | 7 | 8 | 9 | 12 | 17);
        let mut bytes = vec![3];
        if null {
            bytes.extend([0xab, kind]);
        } else {
            bytes.extend(slot(0xab, kind, 3));
        }
        bytes.extend(slot(0xcd, 1, 4));
        bytes.extend([0xef, 0xff, 0xee]);
        let mut context = NativeGear::default();
        let mut input = QtReader::new(&bytes);
        let result = context.receive(&mut input);
        if null || supported {
            let report = result.unwrap();
            assert_eq!(report.null_slots, if null { vec![0, 2] } else { vec![2] });
            assert_eq!(context.0.len(), if null { 1 } else { 2 });
            assert_eq!(context.0.last().unwrap().item.common.commodity, 4);
            assert_eq!(input.read_u8().unwrap(), 0xee);
        } else {
            assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Unsupported);
            assert!(context.0.is_empty());
            assert_eq!(input.remaining(), bytes.len() - 1);
        }
    }
}

#[test]
fn person_all_null_gear_has_zero_weight_and_reaches_following_domain() {
    use crate::{atom_position::FloatEncoding, world::native_person::NativePersonSuffixState};
    for kind in [0, 2, 3, 11, 15, 21, 25, 255] {
        let bytes = [0xc0, 1, 1, 0xab, kind, 0, 0xee];
        let mut input = QtReader::new(&bytes);
        let mut state = NativePersonSuffixState::with_prior_fields(0, 0);
        let report = state
            .receive_suffix(&mut input, FloatEncoding::TypeWidths, Instant::now)
            .unwrap();
        assert_eq!(report.gear.unwrap().null_slots, [0]);
        assert_eq!(state.weight_13a8, Some(0.));
        assert!(state.domain_1018.unwrap().0.is_empty());
        assert_eq!(input.read_u8().unwrap(), 0xee);
    }
}

#[test]
fn truncation_is_atomic_per_slot_and_preserves_prior_appends() {
    let first = slot(0xab, 1, 3);
    let second = slot(0xcd, 17, 4);
    let old = NativeGearSlot::read(&mut QtReader::new(&first))
        .unwrap()
        .unwrap();
    let mut bytes = vec![2];
    bytes.extend(&first);
    let boundary = bytes.len();
    bytes.extend(second);
    for end in 0..bytes.len() {
        let mut gear = NativeGear(vec![old.clone(), old.clone()]);
        let mut input = QtReader::new(&bytes[..end]);
        assert_eq!(
            gear.receive(&mut input).unwrap_err().kind(),
            io::ErrorKind::UnexpectedEof
        );
        assert_eq!(gear.0.len(), usize::from(end >= boundary));
        assert_eq!(
            input.remaining(),
            if end == 0 {
                0
            } else if end < boundary {
                end - 1
            } else {
                end - boundary
            }
        );
    }
}
