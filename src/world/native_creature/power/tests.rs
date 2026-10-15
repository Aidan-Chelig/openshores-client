use super::*;

fn fixture(flags: u8) -> Vec<u8> {
    let mut bytes = (-17_i32).to_be_bytes().to_vec();
    bytes.push(flags);
    if flags & 16 != 0 {
        bytes.extend(0x80..0x88);
    }
    if flags & 8 != 0 {
        bytes.push(0xde);
    }
    if flags & 4 != 0 {
        bytes.push(0xff);
    }
    if flags & 2 != 0 {
        for index in 0..8_u8 {
            bytes.extend([0xa0 + index, 0xb0 + index]);
            bytes.extend((i32::MIN + i32::from(index)).to_be_bytes());
        }
    }
    if flags & 1 != 0 {
        for index in 0..8_u8 {
            bytes.extend([0xc0 + index, 0xd0 + index, 0xe0 + index]);
            bytes.extend((i32::MAX - i32::from(index)).to_be_bytes());
        }
    }
    bytes
}

#[test]
fn power_all_flags_preserve_wire_order_unsigned_widening_and_omitted_groups() {
    for flags in 0..=255 {
        let mut bytes = fixture(flags);
        let length = bytes.len();
        bytes.push(0xef);
        let mut input = QtReader::new(&bytes);
        let mut state = NativeStationPower::default();
        assert_eq!(state.receive(&mut input).unwrap(), length);
        assert_eq!(input.read_u8().unwrap(), 0xef);
        assert_eq!(state.field_68, Some(-17));
        assert_eq!(
            state.field_6c,
            if flags & 4 != 0 { Some(255) } else { None }
        );
        for index in 0..8 {
            assert_eq!(
                state.fields_c0[index],
                (flags & 16 != 0).then_some(0x80 + index as u8)
            );
            assert_eq!(
                state.group_02[index],
                if flags & 2 != 0 {
                    NativePowerGroup02 {
                        field_c8: Some(0xa0 + index as u8),
                        field_14: Some(0xb0 + index as u32),
                        field_94: Some(i32::MIN + index as i32),
                    }
                } else {
                    Default::default()
                }
            );
            assert_eq!(
                state.group_01[index],
                if flags & 1 != 0 {
                    NativePowerGroup01 {
                        field_b8: Some(0xc0 + index as u8),
                        field_3c: Some(0xd0 + index as u32),
                        field_34: Some(0xe0 + index as u8),
                        field_74: Some(i32::MAX - index as i32),
                    }
                } else {
                    Default::default()
                }
            );
        }
        let mut expected = state.clone();
        expected.field_68 = Some(123);
        let mut bytes = 123_i32.to_be_bytes().to_vec();
        bytes.push(0);
        state.receive(&mut QtReader::new(&bytes)).unwrap();
        assert_eq!(state, expected);
    }
}

#[test]
fn power_truncation_keeps_only_completed_scalar_assignments() {
    let bytes = fixture(0xff);
    assert_eq!(bytes.len(), 119);
    for end in 0..bytes.len() {
        let mut state = NativeStationPower::default();
        assert!(state.receive(&mut QtReader::new(&bytes[..end])).is_err());
        assert_eq!(state.field_68, (end >= 4).then_some(-17));
        for index in 0..8 {
            assert_eq!(
                state.fields_c0[index],
                (end >= 6 + index).then_some(0x80 + index as u8)
            );
        }
        assert_eq!(state.field_6c, (end >= 15).then_some(255));
        assert_eq!(state.group_02[0].field_c8, (end >= 16).then_some(0xa0));
        assert_eq!(state.group_02[0].field_14, (end >= 17).then_some(0xb0));
        assert_eq!(state.group_02[0].field_94, (end >= 21).then_some(i32::MIN));
        assert_eq!(state.group_01[0].field_b8, (end >= 64).then_some(0xc0));
        assert_eq!(state.group_01[0].field_3c, (end >= 65).then_some(0xd0));
        assert_eq!(state.group_01[0].field_34, (end >= 66).then_some(0xe0));
        assert_eq!(state.group_01[0].field_74, (end >= 70).then_some(i32::MAX));
    }
}
