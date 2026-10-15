use super::*;
use crate::protocol::QtWriter;

#[test]
fn list_count_uses_all_sixteen_bits_without_unknown_body_skipping() {
    for count in [0_u16, 1, 32767, 32768, 65535] {
        let mut bytes = count.to_be_bytes().to_vec();
        bytes.extend(std::iter::repeat_n(6, usize::from(count)));
        bytes.push(0xde);
        let mut input = QtReader::new(&bytes);
        let mut list = NativeConditionList::default();
        list.receive(&mut input, FloatEncoding::TypeWidths).unwrap();
        assert!(list.0.is_empty());
        assert_eq!(input.read_u8().unwrap(), 0xde);
    }
}

// Scalar-only fixtures use the explicit historical type-width profile.
fn read(kind: u8, input: &mut QtReader<'_>) -> io::Result<Option<NativeCondition>> {
    NativeCondition::read(kind, input, FloatEncoding::TypeWidths)
}

#[test]
fn scalar_bodies_preserve_raw_bytes_and_only_normalize_the_last() {
    for byte in 0..=255_u8 {
        for kind in [1, 2, 3, 5, 8, 10, 13] {
            let mut bytes = (-1234567890123_i64).to_be_bytes().to_vec();
            let count = match kind {
                2 => 3,
                1 | 3 | 5 => 2,
                8 => 0,
                _ => 1,
            };
            bytes.extend(std::iter::repeat_n(byte, count));
            bytes.push(0xde);
            let mut input = QtReader::new(&bytes);
            let condition = read(kind, &mut input).unwrap().unwrap();
            assert_eq!(condition.time_08, -1234567890123);
            let last = byte.max(1);
            let expected = match kind {
                1 => NativeConditionBody::Poison {
                    field_10: byte,
                    field_11: last,
                },
                2 => NativeConditionBody::Disease {
                    field_10: byte,
                    field_11: byte,
                    field_12: last,
                },
                3 => NativeConditionBody::Paralysis {
                    field_10: byte,
                    field_11: last,
                },
                5 => NativeConditionBody::Acid {
                    field_10: byte,
                    field_11: last,
                },
                8 => NativeConditionBody::Fire,
                10 => NativeConditionBody::Bandage { field_10: last },
                _ => NativeConditionBody::Antitoxin { field_10: last },
            };
            assert_eq!(condition.body, expected);
            assert_eq!(input.read_u8().unwrap(), 0xde);
            for end in 0..bytes.len() - 1 {
                let mut short = QtReader::new(&bytes[..end]);
                assert_eq!(
                    read(kind, &mut short).unwrap_err().kind(),
                    io::ErrorKind::UnexpectedEof
                );
                assert_eq!(short.remaining(), end);
            }
        }
    }
}

#[test]
fn thought_retains_null_empty_unicode_and_timestamp_extremes() {
    for time in [i64::MIN, -1, 0, i64::MAX] {
        for text in [None, Some(""), Some("thought 🦀")] {
            let mut bytes = time.to_be_bytes().to_vec();
            let mut string = QtWriter::default();
            string.write_string(text).unwrap();
            bytes.extend(string.into_bytes());
            for end in 0..bytes.len() {
                let mut input = QtReader::new(&bytes[..end]);
                assert!(read(7, &mut input).is_err());
                assert_eq!(input.remaining(), end);
            }
            let value = read(7, &mut QtReader::new(&bytes)).unwrap().unwrap();
            assert_eq!(value.time_08, time);
            assert_eq!(
                value.body,
                NativeConditionBody::Thought {
                    field_10: text.map(str::to_owned)
                }
            );
        }
    }
}

#[test]
fn unknown_factory_and_known_unimplemented_are_distinct_and_consume_nothing() {
    for kind in 0..=255 {
        let mut input = QtReader::new(&[]);
        match kind {
            9 => assert_eq!(
                read(kind, &mut input).unwrap_err().kind(),
                io::ErrorKind::Unsupported
            ),
            1..=5 | 7 | 8 | 10..=13 => assert_eq!(
                read(kind, &mut input).unwrap_err().kind(),
                io::ErrorKind::UnexpectedEof
            ),
            _ => assert_eq!(read(kind, &mut input).unwrap(), None),
        }
        assert_eq!(input.remaining(), 0);
    }
}

#[test]
fn bounty_preserves_ids_strings_float_bits_and_following_field() {
    for encoding in [
        FloatEncoding::TypeWidths,
        FloatEncoding::SinglePrecision,
        FloatEncoding::DoublePrecision,
    ] {
        for value in [-0.0_f64, 123.25, f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
            for id in [0_u32, 0x80000000, u32::MAX] {
                let mut bytes = i64::MAX.to_be_bytes().to_vec();
                bytes.extend(id.to_be_bytes());
                let mut writer = QtWriter::default();
                writer.write_string(None).unwrap();
                writer.write_string(Some("bounty 🦀")).unwrap();
                bytes.extend(writer.into_bytes());
                let expected = if encoding == FloatEncoding::SinglePrecision {
                    bytes.extend((value as f32).to_be_bytes());
                    f64::from(value as f32)
                } else {
                    bytes.extend(value.to_be_bytes());
                    value
                };
                let length = bytes.len();
                bytes.push(0xde);
                for end in 0..length {
                    let mut input = QtReader::new(&bytes[..end]);
                    assert!(NativeCondition::read(11, &mut input, encoding).is_err());
                    assert_eq!(input.remaining(), end);
                }
                let mut input = QtReader::new(&bytes);
                let condition = NativeCondition::read(11, &mut input, encoding)
                    .unwrap()
                    .unwrap();
                assert_eq!(condition.time_08, i64::MAX);
                let NativeConditionBody::Bounty {
                    id_10,
                    field_18,
                    field_20,
                    field_28,
                } = condition.body
                else {
                    panic!("wrong factory body")
                };
                assert_eq!(id_10, id);
                assert_eq!(field_18, None);
                assert_eq!(field_20.as_deref(), Some("bounty 🦀"));
                assert_eq!(field_28.to_bits(), expected.to_bits());
                assert_eq!(input.read_u8().unwrap(), 0xde);
            }
        }
    }
}

#[test]
fn implanted_bodies_follow_timestamp_without_a_dynamic_dna_type_byte() {
    for kind in [4, 12] {
        let mut bytes = i64::MIN.to_be_bytes().to_vec();
        let mut writer = QtWriter::default();
        writer.write_bytes(Some(&[0xff; 24])).unwrap();
        bytes.extend(writer.into_bytes());
        let length = bytes.len();
        bytes.push(0xde);
        let mut input = QtReader::new(&bytes);
        let value = read(kind, &mut input).unwrap().unwrap();
        assert_eq!(value.time_08, i64::MIN);
        match value.body {
            NativeConditionBody::ImplantDna { field_10 } => {
                assert_eq!(field_10.0[5], 0x7f);
                assert_eq!(field_10.0[23], 0);
            }
            NativeConditionBody::ImplantSpecies { field_10 } => {
                assert_eq!(field_10.0, Some(vec![0xff; 24]))
            }
            _ => panic!("wrong native factory body"),
        }
        assert_eq!(input.read_u8().unwrap(), 0xde);
        for end in 0..length {
            let mut input = QtReader::new(&bytes[..end]);
            assert!(read(kind, &mut input).is_err());
            assert_eq!(input.remaining(), end);
        }
    }
}
