//! AuItemWeapon::ls extension (after common fields), not a combat simulation.
mod fields;
use crate::protocol::QtReader;
pub use fields::NativeWeaponFields;
use std::io;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeWeaponGroup {
    pub byte_0: u8,
    pub short_1: i16,
    /// Bit 0x20: short-width bits; otherwise a byte zero-extended to 16 bits.
    pub variable_2: u16,
    pub short_3: i16,
    pub byte_4: u8,
    pub short_5: i16,
}

fn group(input: &mut QtReader<'_>, wide: bool) -> io::Result<NativeWeaponGroup> {
    Ok(NativeWeaponGroup {
        byte_0: input.read_u8()?,
        short_1: input.read_i16()?,
        variable_2: if wide {
            input.read_u16()?
        } else {
            u16::from(input.read_u8()?)
        },
        short_3: input.read_i16()?,
        byte_4: input.read_u8()?,
        short_5: input.read_i16()?,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeWeapon {
    pub flags: u8,
    pub field_4c: i16,
    pub groups: [Option<NativeWeaponGroup>; 4],
    /// Present whenever any bit in mask 0x1c is set, even bit 0x10 alone.
    pub wire_field_66: Option<i16>,
    pub consumed_bytes: usize,
}

impl NativeWeapon {
    /// Decode the extension atomically. Does not read common or subtype suffix.
    pub fn read(input: &mut QtReader<'_>) -> io::Result<Self> {
        let length = input.remaining();
        let mut record = QtReader::new(input.remaining_bytes());
        let flags = record.read_u8()?;
        let field_4c = record.read_i16()?;
        let wide = flags & 0x20 != 0;
        let mut groups = [None, None, None, None];
        for (index, slot) in groups.iter_mut().enumerate().take(2) {
            if flags & (1 << index) != 0 {
                *slot = Some(group(&mut record, wide)?);
            }
        }
        let wire_field_66 = if flags & 0x1c != 0 {
            Some(record.read_i16()?)
        } else {
            None
        };
        for (index, slot) in groups.iter_mut().enumerate().skip(2) {
            if flags & (1 << index) != 0 {
                *slot = Some(group(&mut record, wide)?);
            }
        }
        let consumed_bytes = length - record.remaining();
        input.skip(consumed_bytes)?;
        Ok(Self {
            flags,
            field_4c,
            groups,
            wire_field_66,
            consumed_bytes,
        })
    }

    /// Reproduce known native storage writes on a freshly constructed weapon.
    /// No claim about applying ls to arbitrary preexisting storage or padding.
    pub fn fresh_fields(&self) -> NativeWeaponFields {
        NativeWeaponFields::from_fresh(self)
    }
}

#[cfg(test)]
mod tests;
