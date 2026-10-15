use super::*;

// Independent factory/vtable ledger: 0 Base, 1 Empire, 2 Animal, 3 Bd.
const FAMILIES: [u8; 43] = [
    1, 2, 1, 1, 1, 1, 0, 1, 0, 0, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 0, 1, 0, 0, 3,
    3, 3, 3, 3, 3, 1, 3, 0, 0, 0, 0,
];

fn fixture(kind: u8, encoding: FloatEncoding, byte: u8, flags: u8) -> Vec<u8> {
    let mut bytes = u32::MAX.to_be_bytes().to_vec();
    for value in [1.0000000001_f64, -0., f64::MAX] {
        if encoding == FloatEncoding::DoublePrecision {
            bytes.extend(value.to_be_bytes());
        } else {
            bytes.extend((value as f32).to_be_bytes());
        }
    }
    match FAMILIES[usize::from(kind - 1)] {
        0 => (),
        1 => bytes.push(byte),
        2 => bytes.extend([byte, flags]),
        3 => bytes.extend([byte, 0xfe]),
        _ => panic!(),
    }
    bytes
}

#[test]
fn every_factory_type_and_profile_obeys_its_actual_minimal_vtable_binding() {
    for encoding in [
        FloatEncoding::TypeWidths,
        FloatEncoding::SinglePrecision,
        FloatEncoding::DoublePrecision,
    ] {
        for kind in 0..=255_u8 {
            if !(1..=43).contains(&kind) {
                let mut input = QtReader::new(&[0xde]);
                assert_eq!(
                    NativeSensorBlipMin::read(kind, &mut input, encoding).unwrap(),
                    None
                );
                assert_eq!(input.remaining(), 1);
                continue;
            }
            for byte in [0_u8, 0x7f, 0x80, 0xff] {
                let mut bytes = fixture(kind, encoding, byte, 0xff);
                let length = bytes.len();
                bytes.push(0xde);
                let mut input = QtReader::new(&bytes);
                let value = NativeSensorBlipMin::read(kind, &mut input, encoding)
                    .unwrap()
                    .unwrap();
                assert_eq!(value.kind, kind);
                assert_eq!(value.id_08, u32::MAX);
                for (actual, expected) in value
                    .position_10
                    .into_iter()
                    .zip([1.0000000001_f64, -0., f64::MAX].map(|value| f64::from(value as f32)))
                {
                    assert_eq!(actual.to_bits(), expected.to_bits());
                }
                assert_eq!(
                    value.body,
                    match FAMILIES[usize::from(kind - 1)] {
                        0 => NativeSensorBlipMinBody::Base,
                        1 => NativeSensorBlipMinBody::Empire { field_2c: byte },
                        2 => NativeSensorBlipMinBody::Animal {
                            field_2c: byte,
                            field_40: 1,
                            field_28: 0
                        },
                        3 => NativeSensorBlipMinBody::Bd {
                            field_2c: byte,
                            field_34: 0xfe
                        },
                        _ => panic!(),
                    }
                );
                assert_eq!(input.read_u8().unwrap(), 0xde);
                for end in 0..length {
                    let mut short = QtReader::new(&bytes[..end]);
                    assert!(NativeSensorBlipMin::read(kind, &mut short, encoding).is_err());
                    assert_eq!(short.remaining(), end);
                }
            }
        }
    }
}

#[test]
fn animal_packed_bits_and_raw_nonfinite_coordinates_are_not_normalized() {
    for flags in 0..=255 {
        let bytes = fixture(2, FloatEncoding::TypeWidths, 0xfe, flags);
        let value =
            NativeSensorBlipMin::read(2, &mut QtReader::new(&bytes), FloatEncoding::TypeWidths)
                .unwrap()
                .unwrap();
        assert_eq!(
            value.body,
            NativeSensorBlipMinBody::Animal {
                field_2c: 0xfe,
                field_40: u8::from(flags & 2 != 0),
                field_28: u32::from(flags & 1 == 0)
            }
        );
    }
    for encoding in [
        FloatEncoding::TypeWidths,
        FloatEncoding::SinglePrecision,
        FloatEncoding::DoublePrecision,
    ] {
        let mut bytes = 0_u32.to_be_bytes().to_vec();
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            if encoding == FloatEncoding::DoublePrecision {
                bytes.extend(value.to_be_bytes());
            } else {
                bytes.extend((value as f32).to_be_bytes());
            }
        }
        let value = NativeSensorBlipMin::read(31, &mut QtReader::new(&bytes), encoding)
            .unwrap()
            .unwrap();
        assert!(value.position_10[0].is_nan());
        assert_eq!(value.position_10[1], f64::INFINITY);
        assert_eq!(value.position_10[2], f64::NEG_INFINITY);
    }
}

#[test]
fn minimal_list_composes_all_types_and_unknown_factory_stops_without_skipping() {
    let mut bytes = 43_u16.to_be_bytes().to_vec();
    for kind in 1..=43 {
        bytes.push(kind);
        bytes.extend(fixture(kind, FloatEncoding::TypeWidths, 0xff, 0xff));
    }
    let length = bytes.len();
    bytes.push(0xde);
    let mut list = NativeSensorBlipMinList::default();
    let mut input = QtReader::new(&bytes);
    let result = list.receive(&mut input, FloatEncoding::TypeWidths).unwrap();
    assert_eq!(
        result,
        NativeSensorBlipMinReceive {
            declared_count: 43,
            unknown_type: None,
            unread_entries: 0,
            consumed_bytes: length
        }
    );
    assert_eq!(list.0.len(), 43);
    assert_eq!(input.read_u8().unwrap(), 0xde);
    for unknown in [0, 44, 255] {
        let mut bytes = 3_u16.to_be_bytes().to_vec();
        bytes.push(31);
        bytes.extend(fixture(31, FloatEncoding::TypeWidths, 0, 0));
        bytes.extend([unknown, 0xde, 31]);
        let mut input = QtReader::new(&bytes);
        let result = list.receive(&mut input, FloatEncoding::TypeWidths).unwrap();
        assert_eq!(list.0.len(), 1);
        assert_eq!(list.0[0].kind, 31);
        assert_eq!(result.unknown_type, Some(unknown));
        assert_eq!(result.unread_entries, 1);
        assert_eq!(result.declared_count, 3);
        assert_eq!(result.consumed_bytes, 20);
        assert_eq!(input.read_u8().unwrap(), 0xde);
        assert_eq!(input.read_u8().unwrap(), 31);
    }
    assert_eq!(
        list.receive(&mut QtReader::new(&[0, 0]), FloatEncoding::TypeWidths)
            .unwrap()
            .unknown_type,
        None
    );
    assert!(list.0.is_empty());
}

#[test]
fn list_replacement_clear_precedes_count_and_later_errors_keep_completed_blips() {
    let old = NativeSensorBlipMin::read(
        31,
        &mut QtReader::new(&fixture(31, FloatEncoding::TypeWidths, 0, 0)),
        FloatEncoding::TypeWidths,
    )
    .unwrap()
    .unwrap();
    let mut bytes = 2_u16.to_be_bytes().to_vec();
    bytes.push(31);
    bytes.extend(fixture(31, FloatEncoding::TypeWidths, 0, 0));
    bytes.push(2);
    bytes.extend(fixture(2, FloatEncoding::TypeWidths, 0xfe, 0xff));
    for end in 0..bytes.len() {
        let mut list = NativeSensorBlipMinList(vec![old.clone(), old.clone()]);
        assert!(
            list.receive(&mut QtReader::new(&bytes[..end]), FloatEncoding::TypeWidths)
                .is_err()
        );
        assert_eq!(list.0.len(), usize::from(end >= 19));
        if end >= 19 {
            assert_eq!(list.0[0], old);
        }
    }
}

#[test]
fn minimal_count_uses_all_sixteen_bits_without_preallocating_from_count() {
    for count in [32768_u16, 65535] {
        let mut bytes = count.to_be_bytes().to_vec();
        for _ in 0..count {
            bytes.push(31);
            bytes.extend([0; 16]);
        }
        bytes.push(0xde);
        let mut input = QtReader::new(&bytes);
        let mut list = NativeSensorBlipMinList::default();
        let result = list.receive(&mut input, FloatEncoding::TypeWidths).unwrap();
        assert_eq!(result.unknown_type, None);
        assert_eq!(list.0.len(), usize::from(count));
        assert_eq!(input.read_u8().unwrap(), 0xde);
    }
}
