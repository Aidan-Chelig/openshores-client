use super::{NativeItemCommon, NativeWeapon, common::bounded_string};
use crate::protocol::QtReader;
use std::io;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeItemExtension {
    Plain,
    Ammo,
    Weapon(NativeWeapon),
    WeaponAmmo {
        weapon: NativeWeapon,
        fields_80: [u8; 4],
    },
    WeaponState {
        weapon: NativeWeapon,
        field_80: u8,
    },
    State {
        field_4c: u8,
    },
    StateMessage {
        field_4c: u8,
        field_50: Option<String>,
    },
}

impl NativeItemExtension {
    pub fn type_id(&self) -> u8 {
        match self {
            Self::Plain => 1,
            Self::State { .. } => 6,
            Self::Ammo => 7,
            Self::Weapon(_) => 8,
            Self::WeaponAmmo { .. } => 9,
            Self::WeaponState { .. } => 12,
            Self::StateMessage { .. } => 17,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeItemValue {
    pub common: NativeItemCommon,
    pub extension: NativeItemExtension,
    /// Includes the leading factory type byte.
    pub consumed_bytes: usize,
}

impl NativeItemValue {
    /// Verified AuItem::Create(byte) null cases, distinct from valid classes
    /// whose body readers are not implemented. See native-item-byte-factory-gnu.txt.
    pub fn factory_is_null(kind: u8) -> bool {
        matches!(kind, 0 | 2 | 3 | 11 | 15 | 21 | 25..=255)
    }

    /// Read type byte and established subtype fields. Unsupported types return
    /// None without consumption; malformed supported types return Err atomically.
    pub fn read(input: &mut QtReader<'_>) -> io::Result<Option<Self>> {
        let length = input.remaining();
        let mut record = QtReader::new(input.remaining_bytes());
        let type_id = record.read_u8()?;
        if !matches!(type_id, 1 | 6 | 7 | 8 | 9 | 12 | 17) {
            return Ok(None);
        }
        let common = NativeItemCommon::read(&mut record)?;
        let extension = match type_id {
            1 => NativeItemExtension::Plain,
            6 => NativeItemExtension::State {
                field_4c: record.read_u8()?,
            },
            7 => NativeItemExtension::Ammo,
            8 => NativeItemExtension::Weapon(NativeWeapon::read(&mut record)?),
            9 => NativeItemExtension::WeaponAmmo {
                weapon: NativeWeapon::read(&mut record)?,
                fields_80: [
                    record.read_u8()?,
                    record.read_u8()?,
                    record.read_u8()?,
                    record.read_u8()?,
                ],
            },
            12 => NativeItemExtension::WeaponState {
                weapon: NativeWeapon::read(&mut record)?,
                field_80: record.read_u8()?,
            },
            17 => NativeItemExtension::StateMessage {
                field_4c: record.read_u8()?,
                field_50: bounded_string(&mut record)?,
            },
            _ => unreachable!("unsupported subtype was rejected before common decoding"),
        };
        let consumed_bytes = length - record.remaining();
        input.skip(consumed_bytes)?;
        Ok(Some(Self {
            common,
            extension,
            consumed_bytes,
        }))
    }
}
