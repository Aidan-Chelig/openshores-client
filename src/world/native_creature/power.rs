use crate::protocol::QtReader;
use std::io;

/// Offsets identify the first entry; later entries use the native byte/word
/// strides. None means unobserved, not a guessed native constructor default.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NativePowerGroup02 {
    pub field_c8: Option<u8>,
    pub field_14: Option<u32>,
    pub field_94: Option<i32>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NativePowerGroup01 {
    pub field_b8: Option<u8>,
    pub field_3c: Option<u32>,
    pub field_34: Option<u8>,
    pub field_74: Option<i32>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NativeStationPower {
    pub field_68: Option<i32>,
    pub fields_c0: [Option<u8>; 8],
    pub field_6c: Option<u8>,
    pub group_02: [NativePowerGroup02; 8],
    pub group_01: [NativePowerGroup01; 8],
}

impl NativeStationPower {
    /// Omitted groups retain prior values. Earlier scalar writes survive later
    /// Rust errors, which stop instead of continuing a failed Qt stream.
    pub fn receive(&mut self, input: &mut QtReader<'_>) -> io::Result<usize> {
        let length = input.remaining();
        self.field_68 = Some(input.read_i32()?);
        let flags = input.read_u8()?;
        if flags & 0x10 != 0 {
            for field in &mut self.fields_c0 {
                *field = Some(input.read_u8()?);
            }
        }
        if flags & 8 != 0 {
            input.read_u8()?;
        } // received but not stored
        if flags & 4 != 0 {
            self.field_6c = Some(input.read_u8()?);
        }
        if flags & 2 != 0 {
            for group in &mut self.group_02 {
                group.field_c8 = Some(input.read_u8()?);
                group.field_14 = Some(u32::from(input.read_u8()?));
                group.field_94 = Some(input.read_i32()?);
            }
        }
        if flags & 1 != 0 {
            for group in &mut self.group_01 {
                group.field_b8 = Some(input.read_u8()?);
                group.field_3c = Some(u32::from(input.read_u8()?));
                group.field_34 = Some(input.read_u8()?);
                group.field_74 = Some(input.read_i32()?);
            }
        }
        Ok(length - input.remaining())
    }
}

#[cfg(test)]
mod tests;
