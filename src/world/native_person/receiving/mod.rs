//! Ordered Person-only suffix. Caller must finish Sentient first.
use super::*;
use std::time::Instant;
mod receive;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NativePersonIdentity {
    pub id_10ec: Option<u32>,
    pub string_10f0: Option<Option<String>>,
    pub id_10f8: Option<u32>,
    pub string_1100: Option<Option<String>>,
    pub globe_1108: Option<NativeGlobeLlf>,
    pub id_1110: Option<u32>,
    pub field_1114: Option<bool>,
    pub string_1118: Option<Option<String>>,
}

/// Observation state, not a full native constructor. Read-modify-write fields
///660 and1130 require actual prior values; None fails rather than guessing zero.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NativePersonSuffixState {
    pub identity: NativePersonIdentity,
    pub hit_notify: NativeHitNotify,
    pub field_660: Option<u16>,
    pub timer_668: Option<Instant>,
    pub field_838: Option<u8>,
    pub field_839: Option<u8>,
    pub field_83b: Option<u8>,
    pub field_83d: Option<u8>,
    pub field_f80: Option<i32>,
    pub field_f84: Option<u8>,
    pub field_f88: Option<u32>,
    pub content_fa0: Option<Vec<u8>>,
    pub field_fa8: Option<u8>,
    pub field_fa9: Option<u8>,
    pub field_fb0: Option<u32>,
    pub gear_1390: Option<NativeGear>,
    pub weight_13a8: Option<f32>,
    pub domain_1018: Option<NativeDomain>,
    pub jobs_13b8: Option<NativeJobList>,
    pub field_10d0: Option<i32>,
    pub field_f58: Option<i16>,
    pub field_1408: Option<f64>,
    pub field_f60: Option<f64>,
    pub field_f68: Option<f64>,
    pub state_1130: Option<u32>,
    pub field_10e8: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[must_use = "inspect gear diagnostics; this suffix is not inherited Person dispatch"]
pub struct NativePersonReceive {
    pub flags_a: u8,
    pub flags_b: u8,
    pub flags_c: u8,
    pub timer_restarted: bool,
    pub gear: Option<NativeGearReceive>,
    pub jobs: Option<NativeJobListReceive>,
    pub consumed_bytes: usize,
}

#[cfg(test)]
mod tests;
