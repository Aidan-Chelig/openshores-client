use super::{NativeItemBody, NativeItemCommon, OwnershipUpdate};
use crate::protocol::QtReader;

#[test]
fn all_flags_preserve_native_order_defaults_and_opaque_suffix() {
    for flags in 0..=u8::MAX {
        let mut bytes = vec![flags];
        bytes.extend((-123_i16).to_be_bytes());
        if flags & 1 != 0 {
            bytes.extend(i32::MIN.to_be_bytes());
        }
        if flags & 2 != 0 {
            bytes.extend((-321_i32).to_be_bytes());
        }
        if flags & 4 != 0 {
            bytes.push(0xfe);
        }
        if flags & 8 != 0 {
            bytes.extend(4_u32.to_be_bytes());
            bytes.extend([0xd8, 0x3d, 0xde, 0x80]); // valid surrogate pair
        }
        bytes.push(0); // ALWAYS after the name, not guarded by bit 4
        if flags & 0x10 != 0 {
            bytes.extend(i32::MAX.to_be_bytes());
        }
        if flags & 0x20 != 0 {
            bytes.push(0x81);
        }
        let consumed = bytes.len();
        bytes.extend([0xde, 0xad]);
        let mut reader = QtReader::new(&bytes);
        let item = NativeItemCommon::read(&mut reader).unwrap();
        assert_eq!(item.flags, flags);
        assert_eq!(item.commodity, -123);
        assert_eq!(item.field_0c, if flags & 1 != 0 { i32::MIN } else { 1 });
        assert_eq!(item.field_10, if flags & 2 != 0 { -321 } else { 0 });
        assert_eq!(item.field_14, (flags & 4 != 0).then_some(0xfe));
        assert_eq!(item.name, (flags & 8 != 0).then(|| Some("🚀".into())));
        assert_eq!(
            item.effective_name(),
            Some(if flags & 8 != 0 { "🚀" } else { "" })
        );
        assert_eq!(item.wire_field_20, 0);
        assert_eq!(item.field_20(), 1);
        assert_eq!(item.field_24, if flags & 0x10 != 0 { i32::MAX } else { 0 });
        assert_eq!(item.field_28, if flags & 0x20 != 0 { 0x81 } else { 0 });
        assert_eq!(item.consumed_bytes, consumed);
        assert_eq!(reader.read_u16().unwrap(), 0xdead);
        for end in 0..consumed {
            let mut reader = QtReader::new(&bytes[..end]);
            assert!(NativeItemCommon::read(&mut reader).is_err());
            assert_eq!(reader.remaining(), end);
        }
    }
}

#[test]
fn explicit_null_empty_and_omitted_names_are_distinct() {
    for length in [u32::MAX, 0] {
        let mut bytes = vec![8, 0, 1];
        bytes.extend(length.to_be_bytes());
        bytes.push(0xff);
        let item = NativeItemCommon::read(&mut QtReader::new(&bytes)).unwrap();
        assert_eq!(
            item.effective_name(),
            if length == 0 { Some("") } else { None }
        );
        assert_eq!(
            item.name,
            Some(if length == 0 { Some("".into()) } else { None })
        );
        assert_eq!(item.field_20(), 0xff);
    }
}

#[test]
fn malformed_and_overbudget_names_are_rejected_atomically() {
    for raw in [vec![0], vec![0xd8, 0], vec![0; 256 * 1024 + 2]] {
        let mut bytes = vec![8, 0, 1];
        bytes.extend((raw.len() as u32).to_be_bytes());
        bytes.extend(raw);
        bytes.push(1);
        let mut reader = QtReader::new(&bytes);
        assert!(NativeItemCommon::read(&mut reader).is_err());
        assert_eq!(reader.remaining(), bytes.len());
    }
}

#[test]
fn body_flags_plain_dispatch_and_ownership_precedence_are_atomic() {
    for flags in 0..=u8::MAX {
        let mut bytes = vec![flags];
        if flags & 1 != 0 {
            bytes.extend([1, 0, 0, 1, 1]);
        }
        if flags & 0x80 != 0 {
            bytes.extend(0xf1234567_u32.to_be_bytes());
            bytes.extend(u32::MAX.to_be_bytes());
            bytes.extend(0_u32.to_be_bytes());
        }
        let consumed = bytes.len();
        bytes.extend([0xde, 0xad]);
        let mut reader = QtReader::new(&bytes);
        let item = NativeItemBody::read(&mut reader).unwrap().unwrap();
        assert_eq!(item.flags, flags);
        assert_eq!(item.item.is_some(), flags & 1 != 0);
        assert_eq!(item.field_39c, flags & 0x20 != 0);
        assert_eq!(
            item.ownership,
            if flags & 0x80 != 0 {
                OwnershipUpdate::Explicit {
                    id: 0xf1234567,
                    field_3e0: None,
                    field_3e8: Some("".into()),
                }
            } else if flags & 0x40 != 0 {
                OwnershipUpdate::Sentinel
            } else {
                OwnershipUpdate::Unchanged
            }
        );
        assert_eq!(item.consumed_bytes, consumed);
        assert_eq!(reader.read_u16().unwrap(), 0xdead);
        for end in 0..consumed {
            let mut reader = QtReader::new(&bytes[..end]);
            assert!(NativeItemBody::read(&mut reader).is_err());
            assert_eq!(reader.remaining(), end);
        }
    }
    for subtype in 0..=u8::MAX {
        if matches!(subtype, 1 | 6 | 7 | 8 | 9 | 12 | 17) {
            continue;
        }
        let bytes = [0x81, subtype, 0xde, 0xad];
        let mut reader = QtReader::new(&bytes);
        assert_eq!(NativeItemBody::read(&mut reader).unwrap(), None);
        assert_eq!(reader.remaining(), bytes.len());
    }
}

#[test]
fn atom_dispatch_never_decodes_unknown_base_layers_as_items() {
    use crate::world::native_atom::NativeAtomPrefix;
    for opcode in [0x16, 0x05, 0x24] {
        for flags in [0, 0x10, 0x40, 0x50] {
            let mut bytes = vec![opcode];
            bytes.extend(7_u32.to_be_bytes());
            bytes.extend(123_i64.to_be_bytes());
            bytes.extend([flags, 1, 1, 0, 0, 1, 1]);
            let atom = NativeAtomPrefix::decode(&bytes).unwrap().unwrap();
            assert_eq!(
                atom.item_body().unwrap().is_some(),
                opcode == 0x16 && flags == 0
            );
            assert_eq!(atom.remainder, [1, 1, 0, 0, 1, 1]);
        }
    }
}
