use super::*;

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
    for bytes in [vec![0xab, 0xff], slot(0xab, 1, 2)] {
        let mut input = QtReader::new(&bytes);
        assert_eq!(
            NativeGearSlot::read(&mut input).unwrap_err().kind(),
            io::ErrorKind::Unsupported
        );
        assert_eq!(input.remaining(), bytes.len());
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
