//! Factory-bound Engineer/Helm station wire bodies; native-consoles.md.
use crate::protocol::QtReader;
use std::io;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeStationEngineer {
    pub field_14: u8,
    pub field_15: u8,
    pub field_16: u8,
    pub field_17: u8,
    pub field_18: u32,
    pub field_1c: u8,
    pub field_1d: u8,
    pub field_1e: u8,
    pub field_1f: u8,
    pub field_20: u32,
    pub field_24: u32,
    pub field_28: u8,
    pub field_29: u8,
    pub field_2a: u16,
}

/// First two bits have precedence, not independent boolean storage. Negative
/// byte state is retained as raw 0xff rather than normalized to one.
fn status(flags: u8) -> (u8, u32) {
    let direction = if flags & 1 != 0 {
        1
    } else if flags & 2 != 0 {
        0xff
    } else {
        0
    };
    let mode = if flags & 4 != 0 {
        1
    } else {
        u32::from((flags >> 3) & 1) * 2
    };
    (direction, mode)
}

impl NativeStationEngineer {
    /// Fresh decoded value, atomically read as Rust safety. Native in-place
    /// mutations on a failed Qt stream are not emulated by this wire reader.
    pub fn read(input: &mut QtReader<'_>) -> io::Result<Self> {
        let mut record = QtReader::new(input.remaining_bytes());
        let flags = record.read_u8()?;
        let (field_16, field_18) = status(flags);
        let packed = record.read_u8()?;
        let field_17 = record.read_u8()?;
        let field_1e = record.read_u8()?;
        let field_1f = record.read_u8()?;
        let field_14 = record.read_u8()?;
        let field_1d = record.read_u8()?;
        let field_15 = record.read_u8()?;
        let field_28 = record.read_u8()?;
        let field_2a = if flags & 0x20 != 0 {
            record.read_u16()?
        } else {
            0
        };
        let value = Self {
            field_14,
            field_15,
            field_16,
            field_17,
            field_18,
            field_1c: (flags >> 4) & 1,
            field_1d,
            field_1e,
            field_1f,
            field_20: u32::from(packed & 3),
            field_24: u32::from(packed >> 2),
            field_28,
            field_29: (flags >> 6) & 1,
            field_2a,
        };
        input.skip(input.remaining() - record.remaining())?;
        Ok(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeStationHelm {
    pub field_14: u8,
    pub field_15: u8,
    pub field_16: u8,
    pub field_18: u32,
    pub field_1c: u8,
    pub field_1d: u8,
    pub field_20: u32,
    pub field_24: u8,
    pub field_25: u8,
    pub field_26: u16,
}

impl NativeStationHelm {
    pub fn read(input: &mut QtReader<'_>) -> io::Result<Self> {
        let mut record = QtReader::new(input.remaining_bytes());
        let flags = record.read_u8()?;
        let (field_15, field_18) = status(flags);
        let field_20 = match flags & 0xc0 {
            0xc0 => 0x39,
            0x40 => 1,
            0x80 => 7,
            _ => 0,
        };
        let field_1d = record.read_u8()?;
        let field_14 = record.read_u8()?;
        let field_16 = record.read_u8()?;
        let field_25 = record.read_u8()?;
        let field_26 = record.read_u16()?;
        let value = Self {
            field_14,
            field_15,
            field_16,
            field_18,
            field_1c: (flags >> 4) & 1,
            field_1d,
            field_20,
            field_24: (flags >> 5) & 1,
            field_25,
            field_26,
        };
        input.skip(input.remaining() - record.remaining())?;
        Ok(value)
    }
}

#[cfg(test)]
mod tests;
