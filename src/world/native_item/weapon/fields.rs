use super::NativeWeapon;

/// Known fields only: native padding/unwritten storage is not fabricated.
/// Integer pairs preserve raw little-endian x86 destination bits, not semantics.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NativeWeaponFields {
    pub field_4c: i16,
    pub field_4e: u8,
    pub field_50: u32,
    pub field_54: i16,
    pub field_56: u8,
    pub field_58: i16,
    pub field_5a: u8,
    pub field_5c: u32,
    pub field_60: i16,
    pub field_62: u8,
    pub field_64: i16,
    pub field_66: i16,
    pub field_68: u8,
    pub field_6a: i16,
    pub field_6c: u32,
    pub field_70: u8,
    pub field_72: i16,
    pub field_74: u8,
    pub field_76: i16,
    pub field_78: u32,
    pub field_7c: u8,
    pub field_7e: i16,
}

fn pair(low: u16, high: u16) -> u32 {
    u32::from(low) | (u32::from(high) << 16)
}

impl NativeWeaponFields {
    pub(super) fn from_fresh(weapon: &NativeWeapon) -> Self {
        // Native constructor zeroes every modeled destination before ls.
        let mut fields = Self {
            field_4c: weapon.field_4c,
            field_66: weapon.wire_field_66.unwrap_or(0),
            ..Self::default()
        };
        if let Some(group) = &weapon.groups[0] {
            fields.field_4e = group.byte_0;
            fields.field_50 = pair(group.short_1 as u16, group.variable_2);
            fields.field_54 = group.short_3;
            fields.field_56 = group.byte_4;
            fields.field_58 = group.short_5;
        }
        if let Some(group) = &weapon.groups[1] {
            fields.field_5a = group.byte_0;
            fields.field_5c = pair(group.short_1 as u16, group.variable_2);
            fields.field_60 = group.short_3;
            fields.field_62 = group.byte_4;
            fields.field_64 = group.short_5;
        }
        if let Some(group) = &weapon.groups[2] {
            fields.field_68 = group.byte_0;
            fields.field_6a = group.short_1;
            fields.field_6c = pair(group.variable_2, group.short_3 as u16);
            fields.field_70 = group.byte_4;
            fields.field_72 = group.short_5;
        }
        if let Some(group) = &weapon.groups[3] {
            fields.field_74 = group.byte_0;
            fields.field_76 = group.short_1;
            // Native bit-8 group overwrites LOW 16 bits at 0x6c, not 0x78!
            fields.field_6c = (fields.field_6c & 0xffff_0000) | u32::from(group.variable_2);
            fields.field_78 = pair(0, group.short_3 as u16);
            fields.field_7c = group.byte_4;
            fields.field_7e = group.short_5;
        }
        fields
    }
}
