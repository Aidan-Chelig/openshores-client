use super::{NativeItemValue, common::bounded_string};
use crate::protocol::QtReader;
use std::io;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OwnershipUpdate {
    Unchanged,
    /// Native flag 0x40 writes ID 1 without resetting the two strings.
    Sentinel,
    /// Flag 0x80 takes precedence over 0x40.
    Explicit {
        id: u32,
        field_3e0: Option<String>,
        field_3e8: Option<String>,
    },
}

/// Leading DaItem::RxUpdate fields, after all base layers are consumed.
/// No state mutation, subtype guessing or source-profile fallback.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeItemBody {
    pub flags: u8,
    /// Omission leaves the existing AuItem pointer unchanged in native code.
    pub item: Option<NativeItemValue>,
    pub ownership: OwnershipUpdate,
    /// Native boolean at 0x39c, replaced on every update.
    pub field_39c: bool,
    pub consumed_bytes: usize,
}

impl NativeItemBody {
    /// Unsupported subtypes return None with the entire input unchanged.
    /// Known malformed bodies return Err atomically. A suffix is never discarded.
    pub fn read(input: &mut QtReader<'_>) -> io::Result<Option<Self>> {
        let length = input.remaining();
        let mut record = QtReader::new(input.remaining_bytes());
        let flags = record.read_u8()?;
        let item = if flags & 1 != 0 {
            let Some(value) = NativeItemValue::read(&mut record)? else {
                return Ok(None);
            };
            Some(value)
        } else {
            None
        };
        let ownership = if flags & 0x80 != 0 {
            OwnershipUpdate::Explicit {
                id: record.read_u32()?,
                field_3e0: bounded_string(&mut record)?,
                field_3e8: bounded_string(&mut record)?,
            }
        } else if flags & 0x40 != 0 {
            OwnershipUpdate::Sentinel
        } else {
            OwnershipUpdate::Unchanged
        };
        let consumed_bytes = length - record.remaining();
        input.skip(consumed_bytes)?;
        Ok(Some(Self {
            flags,
            item,
            ownership,
            field_39c: flags & 0x20 != 0,
            consumed_bytes,
        }))
    }
}
