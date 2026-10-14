//! Bounded source-established AuGear/item tree decoding; no inventory actions.
use crate::{item::AuItemPrefix, protocol::QtReader};
use std::io;

#[derive(Clone, Debug, PartialEq)]
pub struct InventoryItem {
    pub type_id: u8,
    pub common: AuItemPrefix,
    pub raw_body: Vec<u8>,
    pub children: Vec<NestedItem>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NestedItem {
    /// Container two-byte metadata, explicit box key, or empty for implicit box order.
    pub metadata: Vec<u8>,
    pub item: InventoryItem,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GearSlot {
    pub slot_type: u8,
    pub subindex: u8,
    /// Type zero is an explicit empty slot, not an omitted update.
    pub item: Option<InventoryItem>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Gear {
    pub declared_slots: u8,
    pub slots: Vec<GearSlot>,
    pub stop: Option<String>,
    /// Starts at the first incomplete slot; may include subsequent person fields.
    pub unparsed_suffix: Vec<u8>,
}

enum Failure {
    Invalid(io::Error),
    Unknown(String),
}
impl From<io::Error> for Failure {
    fn from(value: io::Error) -> Self {
        Self::Invalid(value)
    }
}
type ParseResult<T> = Result<T, Failure>;

struct Parser<'a> {
    bytes: &'a [u8],
    offset: usize,
    nodes: usize,
}
impl Parser<'_> {
    fn take(&mut self, count: usize) -> ParseResult<&[u8]> {
        let end = self
            .offset
            .checked_add(count)
            .filter(|end| *end <= self.bytes.len())
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::UnexpectedEof, "truncated inventory item")
            })?;
        let result = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(result)
    }
    fn byte(&mut self) -> ParseResult<u8> {
        Ok(self.take(1)?[0])
    }
    fn string(&mut self) -> ParseResult<Option<String>> {
        let mut reader = QtReader::new(&self.bytes[self.offset..]);
        let result = reader.read_string()?;
        self.offset = self.bytes.len() - reader.remaining();
        Ok(result)
    }
    fn item(&mut self, type_id: u8, depth: usize) -> ParseResult<InventoryItem> {
        if depth > 16 || self.nodes >= 4096 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "inventory depth/node budget exceeded",
            )
            .into());
        }
        self.nodes += 1;
        if !matches!(type_id, 1 | 6 | 7 | 8 | 9 | 11 | 12 | 14 | 16 | 17 | 18) {
            return Err(Failure::Unknown(format!(
                "unknown inventory type {type_id:#04x}"
            )));
        }
        let start = self.offset;
        let flags = self.byte()?;
        // Inventory entries have no length prefixes; only quality-bearing seed
        // layouts have a verified boundary. Flag-zero writer discrepancy stays opaque.
        if flags & !0x0c != 0 || flags & 4 == 0 {
            return Err(Failure::Unknown(format!(
                "unsupported inventory flags {flags:#04x} for type {type_id:#04x}"
            )));
        }
        let commodity_id = i16::from_be_bytes(self.take(2)?.try_into().unwrap());
        let byte14 = self.byte()?;
        let name = if flags & 8 != 0 {
            Some(self.string()?)
        } else {
            None
        };
        let quality = Some(self.byte()?);
        let common = AuItemPrefix {
            flags,
            commodity_id,
            byte14,
            name,
            quality,
        };
        let mut children = Vec::new();
        match type_id {
            6 => {
                self.take(1)?;
            }
            8 | 9 | 12 => {
                let weapon_flags = self.byte()?;
                if weapon_flags & !0x3f != 0 {
                    return Err(Failure::Unknown("unsupported weapon flags".into()));
                }
                self.take(2)?;
                let block = if weapon_flags & 0x20 != 0 { 10 } else { 9 };
                for bit in [1, 2] {
                    if weapon_flags & bit != 0 {
                        self.take(block)?;
                    }
                }
                if weapon_flags & 0x1c != 0 {
                    self.take(2)?;
                    for bit in [4, 8] {
                        if weapon_flags & bit != 0 {
                            self.take(block)?;
                        }
                    }
                }
                if type_id == 9 {
                    self.take(4)?;
                }
                if type_id == 12 {
                    self.take(1)?;
                }
            }
            11 | 18 => {
                let (count, explicit) = if type_id == 11 {
                    self.take(8)?;
                    (self.byte()?, true)
                } else {
                    self.take(1)?; // Capacity retained in raw body.
                    let magic = self.byte()?;
                    if magic >= 0xf1 {
                        (self.byte()?, true)
                    } else {
                        (magic, false)
                    }
                };
                for _ in 0..count {
                    let metadata = if type_id == 11 {
                        self.take(2)?.to_vec()
                    } else if explicit {
                        vec![self.byte()?]
                    } else {
                        Vec::new()
                    };
                    let nested_type = self.byte()?;
                    let item = self.item(nested_type, depth + 1)?;
                    children.push(NestedItem { metadata, item });
                }
            }
            14 => {
                self.take(13)?;
                let flora_flags = self.byte()?;
                if flora_flags & !0x80 != 0 {
                    return Err(Failure::Unknown("unsupported flora flags".into()));
                }
                self.take(4)?;
                if flora_flags & 0x80 != 0 {
                    self.take(1)?;
                }
                self.take(14)?;
            }
            16 => {
                self.string()?;
                self.take(1)?;
            }
            17 => {
                self.take(1)?;
                self.string()?;
            }
            _ => {}
        }
        Ok(InventoryItem {
            type_id,
            common,
            children,
            raw_body: self.bytes[start..self.offset].to_vec(),
        })
    }
}

impl Gear {
    pub fn read_prefix(bytes: &[u8]) -> io::Result<(Self, usize)> {
        if bytes.len() > 1024 * 1024 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "gear input exceeds byte budget",
            ));
        }
        let mut parser = Parser {
            bytes,
            offset: 0,
            nodes: 0,
        };
        let declared_slots = match parser.byte() {
            Ok(n) => n,
            Err(Failure::Invalid(e)) => return Err(e),
            _ => unreachable!(),
        };
        let mut gear = Self {
            declared_slots,
            slots: Vec::new(),
            stop: None,
            unparsed_suffix: Vec::new(),
        };
        for _ in 0..declared_slots {
            let start = parser.offset;
            let entry = (|| -> ParseResult<GearSlot> {
                let metadata = parser.byte()?;
                let type_id = parser.byte()?;
                let item = if type_id == 0 {
                    None
                } else {
                    Some(parser.item(type_id, 0)?)
                };
                Ok(GearSlot {
                    slot_type: metadata & 15,
                    subindex: metadata >> 4,
                    item,
                })
            })();
            match entry {
                Ok(slot) => gear.slots.push(slot),
                Err(Failure::Invalid(error)) => return Err(error),
                Err(Failure::Unknown(reason)) => {
                    gear.stop = Some(reason);
                    gear.unparsed_suffix = bytes[start..].to_vec();
                    return Ok((gear, start));
                }
            }
        }
        Ok((gear, parser.offset))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn common(cid: u8) -> Vec<u8> {
        vec![4, 0, cid, 5, 61]
    }

    #[test]
    fn slots_empty_entries_and_nested_keys_preserve_wire_identity() {
        let mut box_body = common(10);
        box_body.extend([16, 0xf1, 1, 7, 1]);
        box_body.extend(common(11));
        let mut container = common(12);
        container.extend([0, 0, 0, 4, 0, 0, 0, 4, 1, 9, 4, 7]);
        container.extend(common(13));
        let mut bytes = vec![3, 0x25, 18];
        bytes.extend(&box_body);
        bytes.extend([0x15, 0, 0x12, 11]);
        bytes.extend(&container);
        let (gear, consumed) = Gear::read_prefix(&bytes).unwrap();
        assert_eq!(consumed, bytes.len());
        assert_eq!(gear.declared_slots, 3);
        assert_eq!(gear.slots[0].slot_type, 5);
        assert_eq!(gear.slots[0].subindex, 2);
        let item = gear.slots[0].item.as_ref().unwrap();
        assert_eq!(item.raw_body, box_body);
        assert_eq!(item.children[0].metadata, [7]);
        assert_eq!(item.children[0].item.common.commodity_id, 11);
        assert_eq!(gear.slots[1].item, None);
        let item = gear.slots[2].item.as_ref().unwrap();
        assert_eq!(item.raw_body, container);
        assert_eq!(item.children[0].metadata, [9, 4]);
        for length in 0..bytes.len() {
            assert!(Gear::read_prefix(&bytes[..length]).is_err());
        }
    }

    #[test]
    fn unknown_nested_boundaries_stop_without_seeking_next_slot() {
        let mut bytes = vec![2, 5, 1];
        bytes.extend(common(10));
        let boundary = bytes.len();
        bytes.extend([1, 18]);
        bytes.extend(common(12));
        bytes.extend([16, 0xf1, 1, 7, 0xfe, 0xff, 0xff]);
        let (gear, consumed) = Gear::read_prefix(&bytes).unwrap();
        assert_eq!(consumed, boundary);
        assert_eq!(gear.slots.len(), 1);
        assert!(gear.stop.unwrap().contains("0xfe"));
        assert_eq!(gear.unparsed_suffix, bytes[boundary..]);
        let (empty, consumed) = Gear::read_prefix(&[0, 9, 8]).unwrap();
        assert!(empty.slots.is_empty());
        assert_eq!(consumed, 1); // Later person fields are not folded into a slot.
    }

    #[test]
    fn recursion_and_input_budgets_are_enforced() {
        let mut child = vec![1];
        child.extend(common(10));
        for _ in 0..17 {
            let mut parent = vec![18];
            parent.extend(common(12));
            parent.extend([16, 0xf1, 1, 0]);
            parent.extend(child);
            child = parent;
        }
        let mut bytes = vec![1, 5];
        bytes.extend(child);
        assert!(Gear::read_prefix(&bytes).is_err());
        assert!(Gear::read_prefix(&vec![0; 1024 * 1024 + 1]).is_err());
        let bytes = common(10);
        let mut parser = Parser {
            bytes: &bytes,
            offset: 0,
            nodes: 4095,
        };
        assert!(parser.item(1, 0).is_ok());
        assert_eq!(parser.nodes, 4096);
        assert!(matches!(parser.item(1, 0), Err(Failure::Invalid(_))));
    }
}
