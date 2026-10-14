use super::*;

#[test]
fn all_weapon_flags_have_exact_boundaries_and_fresh_overlapping_writes() {
    for flags in 0..=u8::MAX {
        let mut wire = vec![flags];
        wire.extend((-123_i16).to_be_bytes());
        for index in 0..4 {
            if index == 2 && flags & 0x1c != 0 {
                wire.extend((-321_i16).to_be_bytes());
            }
            if flags & (1 << index) == 0 {
                continue;
            }
            wire.push(0x80 + index);
            wire.extend((-100_i16 - i16::from(index)).to_be_bytes());
            if flags & 0x20 != 0 {
                wire.extend((0xf100_u16 + u16::from(index)).to_be_bytes());
            } else {
                wire.push(0xe0 + index);
            }
            wire.extend((-200_i16 - i16::from(index)).to_be_bytes());
            wire.push(0x90 + index);
            wire.extend((-300_i16 - i16::from(index)).to_be_bytes());
        }
        let consumed = wire.len();
        wire.extend([0xde, 0xad]);
        let mut input = QtReader::new(&wire);
        let weapon = NativeWeapon::read(&mut input).unwrap();
        assert_eq!(weapon.flags, flags);
        assert_eq!(weapon.field_4c, -123);
        assert_eq!(weapon.wire_field_66, (flags & 0x1c != 0).then_some(-321));
        assert_eq!(weapon.consumed_bytes, consumed);
        assert_eq!(input.read_u16().unwrap(), 0xdead);
        let fields = weapon.fresh_fields();
        assert_eq!(fields.field_4c, -123);
        assert_eq!(fields.field_66, if flags & 0x1c != 0 { -321 } else { 0 });
        let variable = |index| {
            if flags & 0x20 != 0 {
                0xf100_u16 + index
            } else {
                0xe0 + index
            }
        };
        let low = if flags & 8 != 0 {
            variable(3)
        } else if flags & 4 != 0 {
            variable(2)
        } else {
            0
        };
        let high = if flags & 4 != 0 { (-202_i16) as u16 } else { 0 };
        assert_eq!(fields.field_6c, u32::from(low) | (u32::from(high) << 16));
        assert_eq!(
            fields.field_78,
            if flags & 8 != 0 {
                u32::from((-203_i16) as u16) << 16
            } else {
                0
            }
        );
        for end in 0..consumed {
            let mut input = QtReader::new(&wire[..end]);
            assert!(NativeWeapon::read(&mut input).is_err());
            assert_eq!(input.remaining(), end);
        }
        for index in 0..4 {
            let Some(group) = &weapon.groups[index] else {
                continue;
            };
            let observed = match index {
                0 => (
                    fields.field_4e,
                    fields.field_50 as u16 as i16,
                    fields.field_54,
                    fields.field_56,
                    fields.field_58,
                ),
                1 => (
                    fields.field_5a,
                    fields.field_5c as u16 as i16,
                    fields.field_60,
                    fields.field_62,
                    fields.field_64,
                ),
                2 => (
                    fields.field_68,
                    fields.field_6a,
                    (fields.field_6c >> 16) as u16 as i16,
                    fields.field_70,
                    fields.field_72,
                ),
                3 => (
                    fields.field_74,
                    fields.field_76,
                    (fields.field_78 >> 16) as u16 as i16,
                    fields.field_7c,
                    fields.field_7e,
                ),
                _ => unreachable!(),
            };
            assert_eq!(
                observed,
                (
                    group.byte_0,
                    group.short_1,
                    group.short_3,
                    group.byte_4,
                    group.short_5
                )
            );
        }
    }
}

#[test]
fn weapon_subtype_suffixes_end_before_outer_ownership_and_fail_atomically() {
    use crate::world::native_item::{NativeItemBody, NativeItemValue, OwnershipUpdate};
    for flags in 0..=u8::MAX {
        let mut extension = vec![flags, 0x80, 0];
        for index in 0..4 {
            if index == 2 && flags & 0x1c != 0 {
                extension.extend([0xff, 0xff]);
            }
            if flags & (1 << index) != 0 {
                extension.extend([0x80, 0xff, 0xff]);
                if flags & 0x20 != 0 {
                    extension.extend([0x80, 1]);
                } else {
                    extension.push(0x80);
                }
                extension.extend([0xff, 0xff, 0x80, 0xff, 0xff]);
            }
        }
        for subtype in [8, 9, 12] {
            let mut value = vec![subtype, 0, 0, 1, 1];
            value.extend(&extension);
            match subtype {
                9 => value.extend([0, 0x80, 0xff, 1]),
                12 => value.push(0xfe),
                _ => (),
            }
            let parsed = NativeItemValue::read(&mut QtReader::new(&value))
                .unwrap()
                .unwrap();
            assert_eq!(parsed.extension.type_id(), subtype);
            assert_eq!(parsed.consumed_bytes, value.len());
            let mut body = vec![0x81];
            body.extend(&value);
            body.extend(7_u32.to_be_bytes());
            body.extend(u32::MAX.to_be_bytes());
            body.extend(0_u32.to_be_bytes());
            let consumed = body.len();
            body.extend([0xde, 0xad]);
            let mut input = QtReader::new(&body);
            let item = NativeItemBody::read(&mut input).unwrap().unwrap();
            assert_eq!(item.item, Some(parsed));
            assert_eq!(
                item.ownership,
                OwnershipUpdate::Explicit {
                    id: 7,
                    field_3e0: None,
                    field_3e8: Some("".into()),
                }
            );
            assert_eq!(item.consumed_bytes, consumed);
            assert_eq!(input.read_u16().unwrap(), 0xdead);
            for end in 0..consumed {
                let mut input = QtReader::new(&body[..end]);
                assert!(NativeItemBody::read(&mut input).is_err());
                assert_eq!(input.remaining(), end);
            }
        }
    }
}
