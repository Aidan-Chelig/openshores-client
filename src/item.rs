//! DaItem identity and known AuItem prefixes; subtype payloads remain opaque.
use crate::protocol::QtReader;
use std::io;

#[derive(Clone, Debug, PartialEq)]
pub struct AuItemPrefix {
    pub flags: u8,
    pub commodity_id: i16,
    pub byte14: u8,
    pub name: Option<Option<String>>,
    pub quality: Option<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ItemValue {
    pub type_id: u8,
    pub common: Option<AuItemPrefix>,
    pub raw_payload: Vec<u8>,
    pub opaque_tail: Vec<u8>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ItemState {
    pub flags: u8,
    pub value: Option<ItemValue>,
}

impl ItemState {
    pub(crate) fn decode_body(body: &[u8]) -> io::Result<Self> {
        let mut input = QtReader::new(body);
        let flags = input.read_u8()?;
        let mut result = Self { flags, value: None };
        if flags & 1 == 0 {
            return Ok(result);
        }
        let type_id = input.read_u8()?;
        let raw_payload = body[body.len() - input.remaining()..].to_vec();
        let mut common = None;
        if matches!(type_id, 1 | 6 | 7 | 8 | 9 | 11 | 12 | 14 | 16 | 17 | 18) {
            let item_flags = input.read_u8()?;
            // Only flag-zero and the source-established name/quality layouts.
            // Unknown flags could change field boundaries; don't seek past them.
            if item_flags & !0x0c == 0 {
                let commodity_id = input.read_u16()? as i16;
                let byte14 = input.read_u8()?;
                let name = if item_flags & 8 != 0 {
                    Some(input.read_string()?)
                } else {
                    None
                };
                let quality = if item_flags & 4 != 0 {
                    Some(input.read_u8()?)
                } else {
                    None
                };
                common = Some(AuItemPrefix {
                    flags: item_flags,
                    commodity_id,
                    byte14,
                    name,
                    quality,
                });
            }
        }
        let opaque_tail = if common.is_some() {
            body[body.len() - input.remaining()..].to_vec()
        } else {
            raw_payload.clone()
        };
        result.value = Some(ItemValue {
            type_id,
            common,
            raw_payload,
            opaque_tail,
        });
        Ok(result)
    }

    pub fn merge(&mut self, mut update: Self) {
        self.flags = update.flags;
        if let Some(value) = &mut update.value {
            if let Some(previous) = &self.value
                && value.type_id == previous.type_id
                && let (Some(new), Some(old)) = (&mut value.common, &previous.common)
            {
                if new.name.is_none() {
                    new.name = old.name.clone();
                }
                if new.quality.is_none() {
                    new.quality = old.quality;
                }
            }
            self.value = update.value;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::WorldState;

    fn packet(body: &[u8]) -> Vec<u8> {
        let mut out = vec![0x11];
        out.extend(42u32.to_be_bytes());
        out.extend(1000i64.to_be_bytes());
        out.push(0);
        out.extend(body);
        out
    }

    #[test]
    fn sparse_common_fields_and_unknown_types_remain_distinct() {
        let mut world = WorldState::default();
        let mut body = vec![1, 1, 12, 0xff, 0xfe, 5];
        body.extend(2u32.to_be_bytes());
        body.extend([0, b'A', 61]);
        world.ingest(&packet(&body)).unwrap();
        let before = world.entities[&42].clone();
        for length in 0..body.len() {
            assert!(world.ingest(&packet(&body[..length])).is_err());
            assert_eq!(world.entities[&42], before);
        }
        world.ingest(&packet(&[1, 1, 0, 0xff, 0xfe, 7])).unwrap();
        let common = world.entities[&42]
            .item
            .as_ref()
            .unwrap()
            .value
            .as_ref()
            .unwrap()
            .common
            .as_ref()
            .unwrap();
        assert_eq!(common.name, Some(Some("A".into())));
        assert_eq!(common.quality, Some(61));
        assert_eq!(common.commodity_id, -2);
        assert_eq!(common.byte14, 7);
        let value = world.entities[&42].item.as_ref().unwrap().value.clone();
        world.ingest(&packet(&[0])).unwrap();
        assert_eq!(world.entities[&42].item.as_ref().unwrap().value, value);
        world.ingest(&packet(&[1, 0xfe, 9, 8, 7])).unwrap();
        let value = world.entities[&42]
            .item
            .as_ref()
            .unwrap()
            .value
            .as_ref()
            .unwrap();
        assert_eq!(value.type_id, 0xfe);
        assert_eq!(value.common, None);
        assert_eq!(value.raw_payload, [9, 8, 7]);
        assert_eq!(value.opaque_tail, [9, 8, 7]);
    }

    #[test]
    fn null_empty_names_and_subtype_tails_are_not_guessed() {
        for (length, name) in [(u32::MAX, None), (0, Some(String::new()))] {
            let mut body = vec![1, 8, 12, 0, 31, 5];
            body.extend(length.to_be_bytes());
            body.extend([61, 0x99, 0x77]);
            let value = ItemState::decode_body(&body).unwrap().value.unwrap();
            assert_eq!(value.common.unwrap().name, Some(name));
            assert_eq!(value.opaque_tail, [0x99, 0x77]);
        }
        let value = ItemState::decode_body(&[1, 1, 0x80, 9, 8])
            .unwrap()
            .value
            .unwrap();
        assert_eq!(value.common, None);
        assert_eq!(value.opaque_tail, [0x80, 9, 8]);
    }
}
