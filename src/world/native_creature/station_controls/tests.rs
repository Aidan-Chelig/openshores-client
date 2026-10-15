use super::*;

#[test]
fn all_engineer_packed_bytes_preserve_precedence_split_widths_and_wire_order() {
    for flags in 0..=255_u8 {
        for packed in 0..=255_u8 {
            let mut bytes = vec![flags, packed, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87];
            if flags & 0x20 != 0 {
                bytes.extend([0x80, 0xfe]);
            }
            let length = bytes.len();
            bytes.push(0xde);
            let mut input = QtReader::new(&bytes);
            let value = NativeStationEngineer::read(&mut input).unwrap();
            assert_eq!(
                value,
                NativeStationEngineer {
                    field_14: 0x84,
                    field_15: 0x86,
                    field_16: [0, 1, 255, 1][usize::from(flags & 3)],
                    field_17: 0x81,
                    field_18: [0, 1, 2, 1][usize::from((flags >> 2) & 3)],
                    field_1c: u8::from(flags & 16 != 0),
                    field_1d: 0x85,
                    field_1e: 0x82,
                    field_1f: 0x83,
                    field_20: u32::from(packed % 4),
                    field_24: u32::from(packed / 4),
                    field_28: 0x87,
                    field_29: u8::from(flags & 64 != 0),
                    field_2a: if flags & 32 != 0 { 0x80fe } else { 0 }
                }
            );
            assert_eq!(input.read_u8().unwrap(), 0xde);
            if packed == 0xff {
                for end in 0..length {
                    let mut short = QtReader::new(&bytes[..end]);
                    assert!(NativeStationEngineer::read(&mut short).is_err());
                    assert_eq!(short.remaining(), end);
                }
            }
        }
    }
}

#[test]
fn all_helm_flags_preserve_top_bit_mapping_and_exact_following_field() {
    for flags in 0..=255_u8 {
        let bytes = [flags, 0x81, 0x82, 0x83, 0x84, 0x80, 0xfe, 0xde];
        let mut input = QtReader::new(&bytes);
        let value = NativeStationHelm::read(&mut input).unwrap();
        assert_eq!(
            value,
            NativeStationHelm {
                field_14: 0x82,
                field_15: [0, 1, 255, 1][usize::from(flags & 3)],
                field_16: 0x83,
                field_18: [0, 1, 2, 1][usize::from((flags >> 2) & 3)],
                field_1c: u8::from(flags & 16 != 0),
                field_1d: 0x81,
                field_20: [0, 1, 7, 0x39][usize::from(flags >> 6)],
                field_24: u8::from(flags & 32 != 0),
                field_25: 0x84,
                field_26: 0x80fe
            }
        );
        assert_eq!(input.read_u8().unwrap(), 0xde);
        for end in 0..7 {
            let mut short = QtReader::new(&bytes[..end]);
            assert!(NativeStationHelm::read(&mut short).is_err());
            assert_eq!(short.remaining(), end);
        }
    }
}
