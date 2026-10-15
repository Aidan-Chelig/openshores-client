use super::*;

/// None kind means omitted/unobserved. A present kind with no value records
/// the null factory (or a body interrupted by a Rust error). No opaque skipping.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NativeStationBlipSelection {
    pub kind: Option<u8>,
    pub value: Option<Box<NativeSensorBlipFull>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use = "wire-only reception: inspect null factories/list stop and apply capsule augmentation separately"]
pub struct NativeStationBlipWireReceive {
    pub null_full_factory: Option<u8>,
    pub minimal_list: NativeSensorBlipMinReceive,
    pub consumed_bytes: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeStationSensorFlags {
    pub fields_14: [u8; 4],
    pub fields_18: [u8; 2],
    pub field_1a: u8,
    pub fields_40: [u8; 4],
}

/// Options mark unobserved receive state, not native constructor defaults.
/// Packed arrays list native bytes in increasing offset order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NativeStationSensor {
    pub flags: Option<NativeStationSensorFlags>,
    pub blip_50: NativeStationBlipSelection,
    pub vector_20: Option<[f64; 3]>,
    pub field_38: Option<f32>,
    pub field_3c: Option<f32>,
    pub id_1c: Option<u32>,
    pub list_58: NativeSensorBlipMinList,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeStationWeaponFlags {
    pub fields_14: [u8; 4],
    pub field_18: u8,
    pub field_48: u8,
    pub fields_3d: [u8; 4],
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NativeStationWeapon {
    pub flags: Option<NativeStationWeaponFlags>,
    pub field_41: Option<u8>,
    pub blip_70: NativeStationBlipSelection,
    pub vector_20: Option<[f64; 3]>,
    pub field_38: Option<f32>,
    pub field_3c: Option<u8>,
    pub field_4c: Option<f32>,
    pub field_50: Option<u16>,
    pub field_a2: Option<u16>,
    pub field_44: Option<i32>,
    pub id_1c: Option<u32>,
    pub list_78: NativeSensorBlipMinList,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeStationTransporterFlags {
    pub fields_40: [u8; 2],
    pub fields_3c: [u8; 4],
    pub field_42: u8,
    pub field_4c: u8,
    pub fields_48: [u8; 4],
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NativeStationTransporter {
    pub field_18: Option<u32>,
    pub field_1c: Option<u32>,
    pub blip_70: NativeStationBlipSelection,
    pub vector_50: Option<[f64; 3]>,
    pub field_4d: Option<bool>,
    pub vector_20: Option<[f64; 3]>,
    pub field_38: Option<f32>,
    pub flags: Option<NativeStationTransporterFlags>,
    pub field_44: Option<f32>,
    pub field_ae: Option<u16>,
    pub id_14: Option<u32>,
    pub list_78: NativeSensorBlipMinList,
}
