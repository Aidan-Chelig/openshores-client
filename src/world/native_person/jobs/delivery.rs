use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeJobCargo {
    /// Signed-short overload, then zero-extended16-bit storage bits.
    pub field_10: u32,
    pub id_14: u32,
    pub string_18: Option<String>,
    pub field_20: i32,
    pub field_24: u8,
    /// Read into a temporary, not stored in the native job.
    pub ignored_wire_byte: u8,
    pub field_28: i32,
    pub string_30: Option<String>,
    pub id_38: u32,
    pub string_40: Option<String>,
}
impl NativeJobCargo {
    pub(super) fn read(input: &mut QtReader<'_>) -> io::Result<Self> {
        Ok(Self {
            field_10: u32::from(input.read_u16()?),
            id_14: input.read_u32()?,
            string_18: input.read_string()?,
            field_20: input.read_i32()?,
            field_24: input.read_u8()?.max(1),
            ignored_wire_byte: input.read_u8()?,
            field_28: input.read_i32()?,
            string_30: input.read_string()?,
            id_38: input.read_u32()?,
            string_40: input.read_string()?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeJobFood {
    pub id_10: u32,
    pub string_18: Option<String>,
    pub field_20: u32,
    pub field_24: i32,
    pub field_28: u8,
    /// Read into a temporary, not stored in the native job.
    pub ignored_wire_byte: u8,
    pub field_2c: i32,
    pub string_30: Option<String>,
    pub id_38: u32,
    pub string_40: Option<String>,
}
impl NativeJobFood {
    pub(super) fn read(input: &mut QtReader<'_>) -> io::Result<Self> {
        Ok(Self {
            id_10: input.read_u32()?,
            string_18: input.read_string()?,
            field_20: u32::from(input.read_u8()?),
            field_24: input.read_i32()?,
            field_28: input.read_u8()?.max(1),
            ignored_wire_byte: input.read_u8()?,
            field_2c: input.read_i32()?,
            string_30: input.read_string()?,
            id_38: input.read_u32()?,
            string_40: input.read_string()?,
        })
    }
}
