//! Constructor-established receive-visible subsets, not native UI/bone objects.
//! Keep Default observation states distinct from these explicit constructors.
use super::*;
use crate::world::{
    native_condition::NativeConditionList, native_sensor_blip::NativeSensorBlipMinList,
};

impl NativeStationEngineer {
    pub fn from_native_constructor(station: u16) -> Self {
        Self {
            field_14: 0,
            field_15: 0,
            field_16: 0,
            field_17: 0,
            field_18: 0,
            field_1c: 0,
            field_1d: 0,
            field_1e: 0,
            field_1f: 0,
            field_20: 0,
            field_24: 0,
            field_28: 0,
            field_29: 0,
            field_2a: station,
        }
    }
}
impl NativeStationHelm {
    pub fn from_native_constructor() -> Self {
        Self {
            field_14: 0,
            field_15: 0,
            field_16: 0,
            field_18: 0,
            field_1c: 0,
            field_1d: 0,
            field_20: 0,
            field_24: 0,
            field_25: 0,
            field_26: 0,
        }
    }
}
impl NativeStationGame {
    pub fn from_native_constructor(kind: u8, station: u16) -> Self {
        Self {
            field_14: kind,
            field_16: station,
        }
    }
}
impl NativeStationShield {
    pub fn from_native_constructor() -> Self {
        Self {
            fields_14: [0.; 6],
            field_30: 0.,
        }
    }
}
impl NativeStationNav {
    pub fn from_native_constructor() -> Self {
        Self {
            field_14: 0x01000000,
            field_18: 0,
            field_1c: 0,
            field_64: 0,
            field_20: 3,
            vector_28: [0.; 3],
            vector_40: [0.; 3],
            id_68: 0,
            id_58: 0,
            field_6c: 0,
            field_60: 6,
            vector_70: [0.; 3],
            field_5c: 0,
            route_90: NativeWaypointRoute::default(),
        }
    }
}
impl NativeStationSurgery {
    pub fn from_native_constructor(station: u16) -> Self {
        Self {
            conditions_48: NativeConditionList::default(),
            field_38: 0xffff,
            points_40: vec![],
            field_14: 0,
            field_16: 0xff,
            field_18: 0,
            field_20: None,
            field_28: station,
        }
    }
}
impl NativeStationPower {
    pub fn from_native_constructor() -> Self {
        Self {
            field_68: Some(0),
            fields_c0: [Some(0); 8],
            field_6c: Some(0),
            group_02: [NativePowerGroup02 {
                field_c8: Some(0),
                field_14: Some(17),
                field_94: Some(0),
            }; 8],
            group_01: [NativePowerGroup01 {
                field_b8: Some(0),
                field_3c: Some(17),
                field_34: Some(0),
                field_74: Some(0),
            }; 8],
        }
    }
}
impl NativeStationSensor {
    pub fn from_native_constructor() -> Self {
        Self {
            flags: Some(NativeStationSensorFlags {
                fields_14: [1, 1, 1, 0],
                fields_18: [1, 1],
                field_1a: 1,
                fields_40: [1; 4],
            }),
            blip_50: NativeStationBlipSelection::default(),
            vector_20: Some([0.; 3]),
            field_38: Some(f32::from_bits(0x492c0d47)),
            field_3c: Some(0.),
            id_1c: Some(0),
            list_58: NativeSensorBlipMinList::default(),
        }
    }
}
impl NativeStationWeapon {
    pub fn from_native_constructor(station: u16) -> Self {
        Self {
            flags: Some(NativeStationWeaponFlags {
                fields_14: [1; 4],
                field_18: 1,
                field_48: 0,
                fields_3d: [0, 1, 0, 1],
            }),
            field_41: Some(0),
            blip_70: NativeStationBlipSelection::default(),
            vector_20: Some([0.; 3]),
            field_38: Some(f32::from_bits(0x492c0d47)),
            field_3c: Some(0),
            field_4c: Some(0.),
            field_50: Some(0),
            field_a2: Some(station),
            field_44: Some(0),
            id_1c: Some(0),
            list_78: NativeSensorBlipMinList::default(),
        }
    }
}
impl NativeStationTransporter {
    pub fn from_native_constructor(station: u16) -> Self {
        Self {
            field_18: Some(0),
            field_1c: Some(0),
            blip_70: NativeStationBlipSelection::default(),
            vector_50: Some([0.; 3]),
            field_4d: Some(false),
            vector_20: Some([0.; 3]),
            field_38: Some(1.),
            flags: Some(NativeStationTransporterFlags {
                fields_40: [1; 2],
                fields_3c: [1; 4],
                field_42: 1,
                field_4c: 1,
                fields_48: [0, 1, 1, 1],
            }),
            field_44: Some(0.),
            field_ae: Some(station),
            id_14: Some(0),
            list_78: NativeSensorBlipMinList::default(),
        }
    }
}
