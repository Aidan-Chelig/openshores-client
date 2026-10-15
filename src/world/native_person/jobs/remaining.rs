use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeJobBountyHunter {
    pub string_10: Option<String>,
    pub id_18: u32,
    pub string_20: Option<String>,
    pub strings_28: Vec<Option<String>>,
    pub field_30: f64,
}
impl NativeJobBountyHunter {
    pub(super) fn read(input: &mut QtReader<'_>, profile: FloatEncoding) -> io::Result<Self> {
        let string_10 = input.read_string()?;
        let id_18 = input.read_u32()?;
        let string_20 = input.read_string()?;
        let count = input.read_u32()?;
        let mut strings_28 = vec![];
        // No reservation from an untrusted count. Qt status/reset/reserve and
        // copy behavior are not emulated by this fresh-value Rust reader.
        for _ in 0..count {
            strings_28.push(input.read_string()?);
        }
        let field_30 = match profile {
            FloatEncoding::SinglePrecision => f64::from(input.read_f32()?),
            FloatEncoding::TypeWidths | FloatEncoding::DoublePrecision => input.read_f64()?,
        };
        Ok(Self {
            string_10,
            id_18,
            string_20,
            strings_28,
            field_30,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeJobShipCommand {
    pub string_10: Option<String>,
    pub string_18: Option<String>,
    /// Temporary wire value, not a native stored field.
    pub ignored_wire_i32: i32,
    pub id_20: u32,
    pub string_28: Option<String>,
    pub id_30: u32,
    pub string_38: Option<String>,
    pub string_40: Option<String>,
}
impl NativeJobShipCommand {
    pub(super) fn read(input: &mut QtReader<'_>) -> io::Result<Self> {
        Ok(Self {
            string_10: input.read_string()?,
            string_18: input.read_string()?,
            ignored_wire_i32: input.read_i32()?,
            id_20: input.read_u32()?,
            string_28: input.read_string()?,
            id_30: input.read_u32()?,
            string_38: input.read_string()?,
            string_40: input.read_string()?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeJobTransferPrisoners {
    pub id_10: u32,
    pub string_18: Option<String>,
    pub string_20: Option<String>,
    pub id_28: u32,
    pub string_30: Option<String>,
    pub field_38: i32,
    pub field_3c: i32,
    pub id_40: u32,
    pub string_48: Option<String>,
    pub string_50: Option<String>,
    pub id_58: u32,
    pub string_60: Option<String>,
}
impl NativeJobTransferPrisoners {
    pub(super) fn read(input: &mut QtReader<'_>) -> io::Result<Self> {
        Ok(Self {
            id_10: input.read_u32()?,
            string_18: input.read_string()?,
            string_20: input.read_string()?,
            id_28: input.read_u32()?,
            string_30: input.read_string()?,
            field_38: input.read_i32()?,
            field_3c: input.read_i32()?,
            id_40: input.read_u32()?,
            string_48: input.read_string()?,
            string_50: input.read_string()?,
            id_58: input.read_u32()?,
            string_60: input.read_string()?,
        })
    }
}
