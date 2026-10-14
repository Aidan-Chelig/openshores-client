use crate::protocol::QtReader;
use std::io;

// Rust safety limit, not a recovered native limit. Check before UTF-16 allocation.
const MAX_NAME_BYTES: usize = 256 * 1024;

pub(super) fn bounded_string(record: &mut QtReader<'_>) -> io::Result<Option<String>> {
    let Some(raw) = record.read_bytes()? else {
        return Ok(None);
    };
    if raw.len() > MAX_NAME_BYTES || raw.len() % 2 != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid or oversized native item string",
        ));
    }
    let units: Vec<_> = raw
        .chunks_exact(2)
        .map(|pair| u16::from_be_bytes([pair[0], pair[1]]))
        .collect();
    String::from_utf16(&units).map(Some).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid native item string UTF-16",
        )
    })
}

/// Common AuItem::ls fields. Offset names intentionally avoid unproven semantics.
/// This is neither a full subtype nor the enclosing DaItem::RxUpdate record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeItemCommon {
    pub flags: u8,
    /// Signed-short stream destination at native offset 0x08.
    pub commodity: i16,
    pub field_0c: i32,
    pub field_10: i32,
    /// Omitted values depend on commodity data/obsolete-item constructor state.
    /// None must not be treated as zero or retained from a previous wire update.
    pub field_14: Option<u8>,
    /// None: omitted (native resets to empty). Some(None): explicit Qt null.
    pub name: Option<Option<String>>,
    /// Unconditional byte after the name. Preserve zero as it appeared on wire.
    pub wire_field_20: u8,
    pub field_24: i32,
    pub field_28: u8,
    pub consumed_bytes: usize,
}

impl NativeItemCommon {
    /// Read a common record only after the caller establishes subtype layout.
    /// Leave subtype suffix untouched; failures do not consume any input.
    pub fn read(input: &mut QtReader<'_>) -> io::Result<Self> {
        let length = input.remaining();
        let mut record = QtReader::new(input.remaining_bytes());
        let flags = record.read_u8()?;
        let commodity = record.read_i16()?;
        let field_0c = if flags & 1 != 0 {
            record.read_i32()?
        } else {
            1
        };
        let field_10 = if flags & 2 != 0 {
            record.read_i32()?
        } else {
            0
        };
        let field_14 = if flags & 4 != 0 {
            Some(record.read_u8()?)
        } else {
            None
        };
        let name = if flags & 8 != 0 {
            Some(bounded_string(&mut record)?)
        } else {
            None
        };
        let wire_field_20 = record.read_u8()?;
        let field_24 = if flags & 0x10 != 0 {
            record.read_i32()?
        } else {
            0
        };
        let field_28 = if flags & 0x20 != 0 {
            record.read_u8()?
        } else {
            0
        };
        let consumed_bytes = length - record.remaining();
        input.skip(consumed_bytes)?;
        Ok(Self {
            flags,
            commodity,
            field_0c,
            field_10,
            field_14,
            name,
            wire_field_20,
            field_24,
            field_28,
            consumed_bytes,
        })
    }

    /// Native reader coerces an unconditional zero byte at offset 0x20 to one.
    pub fn field_20(&self) -> u8 {
        self.wire_field_20.max(1)
    }

    /// Native omitted-name branch resets to empty rather than merging old state.
    pub fn effective_name(&self) -> Option<&str> {
        match &self.name {
            None => Some(""),
            Some(value) => value.as_deref(),
        }
    }
}
