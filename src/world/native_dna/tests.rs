use super::*;
use crate::protocol::QtWriter;

#[test]
fn chromosome_factory_boundaries_and_scalar_body_lengths_are_explicit() {
    for kind in 0..=255 {
        let length = match kind {
            0 | 1 => Some(0),
            4 | 5 | 7 => Some(22),
            6 => Some(12),
            2 | 3 | 21 | 23 | 24 => Some(1),
            16 | 22 | 25 => Some(2),
            15 | 26 => Some(4),
            27 => Some(5),
            _ => None,
        };
        for byte in [0, 0x7f, 0x80, 0xff] {
            let bytes = vec![byte; length.unwrap_or(0)];
            let mut input = QtReader::new(&bytes);
            match length {
                Some(length) => {
                    let result = NativeChromosome::read_qt7(kind, &mut input)
                        .unwrap()
                        .unwrap();
                    assert_eq!(result.kind, kind);
                    assert_eq!(input.remaining(), 0);
                    for end in 0..length {
                        let mut short = QtReader::new(&bytes[..end]);
                        assert_eq!(
                            NativeChromosome::read_qt7(kind, &mut short)
                                .unwrap_err()
                                .kind(),
                            io::ErrorKind::UnexpectedEof
                        );
                        assert_eq!(short.remaining(), end);
                    }
                }
                None if kind < 28 => assert_eq!(
                    NativeChromosome::read_qt7(kind, &mut input)
                        .unwrap_err()
                        .kind(),
                    io::ErrorKind::Unsupported
                ),
                None => assert_eq!(NativeChromosome::read_qt7(kind, &mut input).unwrap(), None),
            }
        }
    }
}

#[test]
fn qt7_color_has_fixed_shape_for_every_spec_and_survives_chromosome_composition() {
    for spec in 0..=255_u8 {
        let components = [0, 1, 0x7fff, 0x8000, 0xffff];
        let expected = NativeColorQt7 {
            spec: i32::from(spec as i8),
            components,
        };
        let mut bytes = vec![spec];
        for component in components {
            bytes.extend(component.to_be_bytes());
        }
        for end in 0..bytes.len() {
            let mut input = QtReader::new(&bytes[..end]);
            assert!(NativeColorQt7::read(&mut input).is_err());
            assert_eq!(input.remaining(), end);
        }
        for kind in [4, 5, 6, 7] {
            let mut packet = bytes.clone();
            if kind == 6 {
                packet.push(0xfe);
            } else {
                packet.extend(&bytes);
            }
            packet.push(0xde);
            let mut input = QtReader::new(&packet);
            let body = NativeChromosome::read_qt7(kind, &mut input)
                .unwrap()
                .unwrap()
                .body;
            assert_eq!(
                body,
                if kind == 6 {
                    NativeChromosomeBody::Scales {
                        field_08: expected.clone(),
                        field_18: 0xfe,
                    }
                } else {
                    NativeChromosomeBody::Colors {
                        field_08: expected.clone(),
                        field_18: expected.clone(),
                    }
                }
            );
            assert_eq!(input.read_u8().unwrap(), 0xde);
        }
    }
}

#[test]
fn packed_chromosomes_preserve_all_byte_values_and_exact_suffix() {
    for packed in 0..=255 {
        let bytes = [0xfe, 0xab, packed, 0x80, 0xde];
        let mut input = QtReader::new(&bytes);
        assert_eq!(
            NativeChromosome::read_qt7(26, &mut input)
                .unwrap()
                .unwrap()
                .body,
            NativeChromosomeBody::Reproduction {
                field_08: 254,
                field_0c: 0xab,
                field_0d: packed >> 4,
                field_0e: packed & 15,
                field_10: 128
            }
        );
        assert_eq!(input.read_u8().unwrap(), 0xde);
        let bytes = [packed, 0x80, 1, 0xfe, 0xab, 0xde];
        let mut input = QtReader::new(&bytes);
        assert_eq!(
            NativeChromosome::read_qt7(27, &mut input)
                .unwrap()
                .unwrap()
                .body,
            NativeChromosomeBody::Mammary {
                field_08: packed & 1,
                field_10: (packed >> 1) & 1,
                field_12: 0x8001,
                field_0c: 254,
                field_14: 0xab
            }
        );
        assert_eq!(input.read_u8().unwrap(), 0xde);
    }
}

#[test]
fn scalar_chromosomes_widen_without_sign_extension_or_boolean_clamping() {
    for byte in 0..=255_u8 {
        for (kind, body, bytes) in [
            (
                2,
                NativeChromosomeBody::WideByte08 {
                    field_08: u32::from(byte),
                },
                vec![byte],
            ),
            (
                3,
                NativeChromosomeBody::WideByte08 {
                    field_08: u32::from(byte),
                },
                vec![byte],
            ),
            (
                23,
                NativeChromosomeBody::WideByte08 {
                    field_08: u32::from(byte),
                },
                vec![byte],
            ),
            (
                21,
                NativeChromosomeBody::Byte08 { field_08: byte },
                vec![byte],
            ),
            (
                24,
                NativeChromosomeBody::Byte08 { field_08: byte },
                vec![byte],
            ),
            (
                25,
                NativeChromosomeBody::WideShort08 {
                    field_08: u32::from(u16::from_be_bytes([byte, 0xab])),
                },
                vec![byte, 0xab],
            ),
            (
                15,
                NativeChromosomeBody::Antenna {
                    field_08: u16::from_be_bytes([byte, 0xab]),
                    field_0c: u32::from(byte),
                    field_10: 0xfe,
                },
                vec![byte, 0xab, byte, 0xfe],
            ),
            (
                16,
                NativeChromosomeBody::Horn {
                    field_08: u32::from(byte),
                    field_0c: 0xfe,
                },
                vec![byte, 0xfe],
            ),
            (
                22,
                NativeChromosomeBody::Gills {
                    field_08: u32::from(byte),
                    field_0c: 254,
                },
                vec![byte, 0xfe],
            ),
        ] {
            let mut bytes = bytes;
            bytes.push(0xde);
            let mut input = QtReader::new(&bytes);
            assert_eq!(
                NativeChromosome::read_qt7(kind, &mut input)
                    .unwrap()
                    .unwrap()
                    .body,
                body
            );
            assert_eq!(input.read_u8().unwrap(), 0xde);
        }
    }
}

#[test]
fn dhdna_normalizes_only_two_bytes_and_leaves_following_fields() {
    for byte in 0..=255 {
        let mut writer = QtWriter::default();
        writer.write_bytes(Some(&[byte; 24])).unwrap();
        writer.write_u8(0xde);
        let bytes = writer.into_bytes();
        let mut input = QtReader::new(&bytes);
        let value = NativeDhDna::read(&mut input).unwrap();
        let mut expected = [byte; 24];
        expected[5] &= 0x7f;
        expected[23] = 0;
        assert_eq!(value.0, expected);
        assert_eq!(input.read_u8().unwrap(), 0xde);
        for end in 0..28 {
            let mut truncated = QtReader::new(&bytes[..end]);
            assert!(NativeDhDna::read(&mut truncated).is_err());
            assert_eq!(truncated.remaining(), end);
        }
    }
}

#[test]
fn null_wrong_length_and_truncated_dhdna_fail_without_consumption() {
    for raw in [None, Some(vec![]), Some(vec![1; 23]), Some(vec![2; 25])] {
        let mut writer = QtWriter::default();
        writer.write_bytes(raw.as_deref()).unwrap();
        let bytes = writer.into_bytes();
        let mut input = QtReader::new(&bytes);
        assert_eq!(
            NativeDhDna::read(&mut input).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        assert_eq!(input.remaining(), bytes.len());
    }
}

#[test]
fn species_preserves_null_empty_and_raw_non_text_bytes_with_budget() {
    for raw in [
        None,
        Some(vec![]),
        Some(vec![0xff, 0, 0xfe]),
        Some(vec![7; 1_048_576]),
    ] {
        let mut writer = QtWriter::default();
        writer.write_bytes(raw.as_deref()).unwrap();
        writer.write_u8(0xde);
        let bytes = writer.into_bytes();
        let mut input = QtReader::new(&bytes);
        assert_eq!(NativeSpecies::read(&mut input).unwrap().0, raw);
        assert_eq!(input.read_u8().unwrap(), 0xde);
    }
    let mut writer = QtWriter::default();
    writer.write_bytes(Some(&vec![0; 1_048_577])).unwrap();
    let bytes = writer.into_bytes();
    let mut input = QtReader::new(&bytes);
    assert_eq!(
        NativeSpecies::read(&mut input).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(input.remaining(), bytes.len());
}
