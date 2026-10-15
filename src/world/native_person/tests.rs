use super::*;

fn float_bytes(value: f64, profile: FloatEncoding, bytes: &mut Vec<u8>) {
    match profile {
        FloatEncoding::DoublePrecision => bytes.extend(value.to_be_bytes()),
        _ => bytes.extend((value as f32).to_be_bytes()),
    }
}
fn profiles() -> [FloatEncoding; 3] {
    [
        FloatEncoding::TypeWidths,
        FloatEncoding::SinglePrecision,
        FloatEncoding::DoublePrecision,
    ]
}
fn string(value: Option<&str>, bytes: &mut Vec<u8>) {
    if let Some(value) = value {
        let units: Vec<_> = value.encode_utf16().collect();
        bytes.extend((units.len() as u32 * 2).to_be_bytes());
        for unit in units {
            bytes.extend(unit.to_be_bytes());
        }
    } else {
        bytes.extend(u32::MAX.to_be_bytes());
    }
}

#[test]
fn globe_wire_order_float_narrowing_and_atomic_truncation() {
    for profile in profiles() {
        let mut bytes = vec![];
        float_bytes(1.00000000001, profile, &mut bytes);
        float_bytes(-0., profile, &mut bytes);
        let mut input = QtReader::new(&bytes);
        let result = NativeGlobeLlf::read(&mut input, profile).unwrap();
        assert_eq!(result.field_04, 1.);
        assert_eq!(result.field_00.to_bits(), (-0_f32).to_bits());
        assert_eq!(input.remaining(), 0);
        for end in 0..bytes.len() {
            let mut input = QtReader::new(&bytes[..end]);
            assert_eq!(
                NativeGlobeLlf::read(&mut input, profile)
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::UnexpectedEof
            );
            assert_eq!(input.remaining(), end);
        }
    }
}

#[test]
fn domain_all_count_bits_duplicates_null_empty_and_replacement() {
    for count in 0..=255_u8 {
        let mut bytes = vec![count];
        for index in 0..count {
            bytes.extend(u32::from(index % 3).to_be_bytes());
            string(
                match index % 3 {
                    0 => None,
                    1 => Some(""),
                    _ => Some("Rust 🌊"),
                },
                &mut bytes,
            );
        }
        bytes.push(0xee);
        let mut state = NativeDomain(BTreeMap::from([(999, Some("old".into()))]));
        let mut input = QtReader::new(&bytes);
        state.receive(&mut input).unwrap();
        assert_eq!(state.0.len(), usize::from(count.min(3)));
        assert!(!state.0.contains_key(&999));
        if count >= 3 {
            assert_eq!(state.0[&0], None);
            assert_eq!(state.0[&1].as_deref(), Some(""));
            assert_eq!(state.0[&2].as_deref(), Some("Rust 🌊"));
        }
        assert_eq!(input.read_u8().unwrap(), 0xee);
    }
}

#[test]
fn domain_failure_clears_before_count_and_keeps_only_completed_entries() {
    let mut bytes = vec![2];
    bytes.extend(7_u32.to_be_bytes());
    string(Some("done"), &mut bytes);
    let boundary = bytes.len();
    bytes.extend(8_u32.to_be_bytes());
    string(Some("later"), &mut bytes);
    for end in 0..bytes.len() {
        let mut state = NativeDomain(BTreeMap::from([(999, None)]));
        assert!(state.receive(&mut QtReader::new(&bytes[..end])).is_err());
        assert!(!state.0.contains_key(&999));
        assert_eq!(state.0.len(), usize::from(end >= boundary));
    }
}

#[test]
fn hit_notify_all_byte_discriminants_and_sparse_integer_retention() {
    for profile in profiles() {
        for first in 0..=255_u8 {
            for second in 0..=255_u8 {
                let mut bytes = vec![];
                for value in [1.00000000001, -2.5, -0.] {
                    float_bytes(value, profile, &mut bytes);
                }
                bytes.push(first);
                if first != 0 {
                    bytes.extend(i32::MIN.to_be_bytes());
                    bytes.extend(0x12345678_i32.to_be_bytes());
                }
                bytes.push(second);
                if second != 0 {
                    bytes.extend(i32::MAX.to_be_bytes());
                    bytes.extend((-123_i32).to_be_bytes());
                }
                bytes.push(0xee);
                let mut state = NativeHitNotify {
                    field_20: Some(20),
                    field_24: Some(24),
                    field_28: Some(28),
                    field_2c: Some(44),
                    ..Default::default()
                };
                let mut input = QtReader::new(&bytes);
                state.receive(&mut input, profile).unwrap();
                assert_eq!(state.field_18, Some(u32::from(first)));
                assert_eq!(state.field_1c, Some(u32::from(second)));
                assert_eq!(state.field_28, Some(if first != 0 { i32::MIN } else { 28 }));
                assert_eq!(
                    state.field_20,
                    Some(if first != 0 { 0x12345678 } else { 20 })
                );
                assert_eq!(
                    state.field_2c,
                    Some(if second != 0 { i32::MAX } else { 44 })
                );
                assert_eq!(state.field_24, Some(if second != 0 { -123 } else { 24 }));
                let vector = state.vector_00.unwrap();
                assert_eq!(vector[..2], [1., -2.5]);
                assert_eq!(vector[2].to_bits(), (-0_f64).to_bits());
                assert_eq!(input.read_u8().unwrap(), 0xee);
            }
        }
    }
}

#[test]
fn domain_duplicate_id_replaces_earlier_different_value() {
    let mut bytes = vec![3];
    for value in [Some("old"), Some("new"), None] {
        bytes.extend(u32::MAX.to_be_bytes());
        string(value, &mut bytes);
    }
    let mut domain = NativeDomain::default();
    domain.receive(&mut QtReader::new(&bytes)).unwrap();
    assert_eq!(domain.0, BTreeMap::from([(u32::MAX, None)]));
}

#[test]
fn hit_notify_each_truncation_retains_exact_completed_writes() {
    for profile in profiles() {
        let mut bytes = vec![];
        for value in [1., 2., 3.] {
            float_bytes(value, profile, &mut bytes);
        }
        let vector_end = bytes.len();
        bytes.push(0x80);
        bytes.extend(28_i32.to_be_bytes());
        bytes.extend(20_i32.to_be_bytes());
        bytes.push(0xff);
        bytes.extend(44_i32.to_be_bytes());
        bytes.extend(24_i32.to_be_bytes());
        for end in 0..bytes.len() {
            let mut state = NativeHitNotify::default();
            assert!(
                state
                    .receive(&mut QtReader::new(&bytes[..end]), profile)
                    .is_err()
            );
            assert_eq!(state.vector_00.is_some(), end >= vector_end);
            assert_eq!(state.field_18.is_some(), end > vector_end);
            assert_eq!(state.field_28.is_some(), end >= vector_end + 5);
            assert_eq!(state.field_20.is_some(), end >= vector_end + 9);
            assert_eq!(state.field_1c.is_some(), end >= vector_end + 10);
            assert_eq!(state.field_2c.is_some(), end >= vector_end + 14);
            assert_eq!(state.field_24, None);
        }
    }
}
