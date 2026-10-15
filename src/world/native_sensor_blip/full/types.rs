use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeSensorBlipFull {
    pub kind: u8,
    pub id_08: u32,
    pub position_10: [f64; 3],
    pub body: NativeBlipFullBody,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeBlipEmpireFields {
    pub field_28: i32,
    pub field_2c: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeBlipCreatureSuffix {
    Animal { field_40: bool },
    Name { field_40: Option<String> },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeBlipCreature {
    pub empire: NativeBlipEmpireFields,
    pub species_30: NativeSpecies,
    pub suffix: NativeBlipCreatureSuffix,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NativeBlipBd {
    pub empire: NativeBlipEmpireFields,
    pub field_35: u8,
    pub field_30: i32,
    pub field_34: u8,
    pub field_38: i32,
    pub field_40: Option<String>,
    pub field_48: f32,
    pub field_4c: u8,
    pub field_50: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NativeBlipSpacecraft {
    pub empire: NativeBlipEmpireFields,
    pub field_30: i32,
    pub field_38: Option<String>,
    pub field_40: bool,
    pub field_41: bool,
    pub field_42: bool,
    pub field_44: i32,
    pub field_48: Option<String>,
    pub field_50: f32,
    pub field_54: u8,
    pub point_58: [f64; 3],
    pub field_70: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NativeBlipFullBody {
    Anomaly,
    Beacon {
        field_28: Option<String>,
    },
    Ring {
        field_28: bool,
        field_2c: f32,
    },
    World {
        field_28: bool,
        field_29: u8,
        field_2a: Option<u8>,
    },
    Asteroid {
        field_28: u16,
        field_2a: u8,
    },
    Creature(NativeBlipCreature),
    Star {
        field_28: u8,
        field_2c: f32,
        field_30: u16,
        field_32: u8,
    },
    Vehicle {
        empire: NativeBlipEmpireFields,
        field_2e: u16,
        field_30: u8,
        field_31: u8,
    },
    Missile {
        empire: NativeBlipEmpireFields,
        field_2e: u16,
        field_30: u8,
        field_31: u8,
        field_32: u8,
    },
    Transporter {
        empire: NativeBlipEmpireFields,
        field_2d: u8,
        field_2e: u16,
        field_30: Option<String>,
    },
    TransporterBd {
        empire: NativeBlipEmpireFields,
        field_30: Option<String>,
        field_38: u8,
        field_39: u8,
        field_3a: u16,
    },
    Bd(NativeBlipBd),
    Spacecraft {
        fields: NativeBlipSpacecraft,
        shield_74: Option<f32>,
    },
}
