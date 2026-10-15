use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeJobPassengerDelivery {
    pub id_10: u32,
    pub string_18: Option<String>,
    pub field_20: i32,
    pub field_24: i32,
    pub string_28: Option<String>,
    pub id_30: u32,
    pub string_38: Option<String>,
}
impl NativeJobPassengerDelivery {
    pub(super) fn read(input: &mut QtReader<'_>) -> io::Result<Self> {
        Ok(Self {
            id_10: input.read_u32()?,
            string_18: input.read_string()?,
            field_20: input.read_i32()?,
            field_24: input.read_i32()?,
            string_28: input.read_string()?,
            id_30: input.read_u32()?,
            string_38: input.read_string()?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeJobPassengerPickup {
    pub id_10: u32,
    pub string_18: Option<String>,
    pub field_20: i32,
    pub string_28: Option<String>,
    pub id_30: u32,
    pub string_38: Option<String>,
    pub string_40: Option<String>,
}
impl NativeJobPassengerPickup {
    pub(super) fn read(input: &mut QtReader<'_>) -> io::Result<Self> {
        Ok(Self {
            id_10: input.read_u32()?,
            string_18: input.read_string()?,
            field_20: input.read_i32()?,
            string_28: input.read_string()?,
            id_30: input.read_u32()?,
            string_38: input.read_string()?,
            string_40: input.read_string()?,
        })
    }
}
