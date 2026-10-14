use super::{NativeItemBody, NativeItemExtension, NativeItemValue, OwnershipUpdate};
use crate::protocol::QtReader;

#[test]
fn subtype_payload_ends_before_ownership_fields() {
    for subtype in [1, 6, 7, 17] {
        for state in [0, 1, 0x80, 0xff] {
            for message in [None, Some(""), Some("message 🚀")] {
                let mut value = vec![subtype, 0, 0, 1, 0];
                if matches!(subtype, 6 | 17) {
                    value.push(state);
                }
                if subtype == 17 {
                    match message {
                        None => value.extend(u32::MAX.to_be_bytes()),
                        Some(message) => {
                            let raw: Vec<_> =
                                message.encode_utf16().flat_map(u16::to_be_bytes).collect();
                            value.extend((raw.len() as u32).to_be_bytes());
                            value.extend(raw);
                        }
                    }
                }
                let parsed = NativeItemValue::read(&mut QtReader::new(&value))
                    .unwrap()
                    .unwrap();
                assert_eq!(parsed.extension.type_id(), subtype);
                assert_eq!(parsed.consumed_bytes, value.len());
                assert_eq!(parsed.common.field_20(), 1);
                assert_eq!(
                    parsed.extension,
                    match subtype {
                        1 => NativeItemExtension::Plain,
                        6 => NativeItemExtension::State { field_4c: state },
                        7 => NativeItemExtension::Ammo,
                        17 => NativeItemExtension::StateMessage {
                            field_4c: state,
                            field_50: message.map(str::to_owned)
                        },
                        _ => unreachable!(),
                    }
                );
                let mut body = vec![0x81];
                body.extend(&value);
                body.extend(0xf1234567_u32.to_be_bytes());
                body.extend(u32::MAX.to_be_bytes());
                body.extend(0_u32.to_be_bytes());
                let consumed = body.len();
                body.extend([0xde, 0xad]);
                let mut reader = QtReader::new(&body);
                let item = NativeItemBody::read(&mut reader).unwrap().unwrap();
                assert_eq!(item.item, Some(parsed));
                assert_eq!(
                    item.ownership,
                    OwnershipUpdate::Explicit {
                        id: 0xf1234567,
                        field_3e0: None,
                        field_3e8: Some("".into()),
                    }
                );
                assert_eq!(item.consumed_bytes, consumed);
                assert_eq!(reader.read_u16().unwrap(), 0xdead);
                for end in 0..consumed {
                    let mut reader = QtReader::new(&body[..end]);
                    assert!(NativeItemBody::read(&mut reader).is_err());
                    assert_eq!(reader.remaining(), end);
                }
            }
        }
    }
}

#[test]
fn malformed_state_message_is_atomic() {
    for raw in [vec![0], vec![0xd8, 0], vec![0; 256 * 1024 + 2]] {
        let mut bytes = vec![17, 0, 0, 1, 1, 0];
        bytes.extend((raw.len() as u32).to_be_bytes());
        bytes.extend(raw);
        let mut reader = QtReader::new(&bytes);
        assert!(NativeItemValue::read(&mut reader).is_err());
        assert_eq!(reader.remaining(), bytes.len());
    }
}
