//! Verified Person dependencies, not a complete inherited Person receiver.
//! Unobserved fields are not guessed constructor defaults.
use crate::{atom_position::FloatEncoding, protocol::QtReader};
use std::{collections::BTreeMap, io};
mod gear;
pub use gear::{NativeGear, NativeGearReceive, NativeGearSlot};
mod receiving;
pub use receiving::{NativePersonIdentity, NativePersonReceive, NativePersonSuffixState};
mod atom;
mod body;
mod construction;
pub use atom::NativePersonAtom;
pub use body::{NativePersonBodyReceive, NativePersonBodyState, NativePersonReceiveContext};
mod inventory;
mod jobs;
pub use inventory::{NativeItemHash, NativeItemHashReceive, native_commodity_is_obsolete};
pub use jobs::{
    NativeJob, NativeJobBody, NativeJobBountyHunter, NativeJobCargo, NativeJobFood, NativeJobList,
    NativeJobListReceive, NativeJobPassengerDelivery, NativeJobPassengerPickup,
    NativeJobShipCommand, NativeJobStop, NativeJobTransferPrisoners,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NativeDomain(pub BTreeMap<u32, Option<String>>);

impl NativeDomain {
    /// Native clears before count and replaces duplicate keys. Completed
    /// entries survive later Rust errors; malformed Qt stream parity is not
    /// claimed. Map ordering is not Qt hash iteration order.
    pub fn receive(&mut self, input: &mut QtReader<'_>) -> io::Result<()> {
        self.0.clear();
        let count = input.read_u8()?;
        for _ in 0..count {
            let id = input.read_u32()?;
            let name = input.read_string()?;
            self.0.insert(id, name);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativeGlobeLlf {
    pub field_00: f32,
    pub field_04: f32,
}

impl NativeGlobeLlf {
    /// Wire order is field04 then field00, both float overloads. Fresh-value
    /// atomicity is a Rust safety policy, not native partial-write parity.
    pub fn read(input: &mut QtReader<'_>, profile: FloatEncoding) -> io::Result<Self> {
        let mut record = QtReader::new(input.remaining_bytes());
        let field_04 = float(&mut record, profile)?;
        let field_00 = float(&mut record, profile)?;
        input.skip(input.remaining() - record.remaining())?;
        Ok(Self { field_00, field_04 })
    }
}

/// Observation state, not AuHitNotify's constructor. Omitted integers retain
/// their previous values independently of their current byte discriminants.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NativeHitNotify {
    pub vector_00: Option<[f64; 3]>,
    pub field_18: Option<u32>,
    pub field_1c: Option<u32>,
    pub field_20: Option<i32>,
    pub field_24: Option<i32>,
    pub field_28: Option<i32>,
    pub field_2c: Option<i32>,
}

impl NativeHitNotify {
    /// Earlier writes survive later Rust errors. The vector is assigned only
    /// after all three temporary float reads, then widened to stored doubles.
    pub fn receive(&mut self, input: &mut QtReader<'_>, profile: FloatEncoding) -> io::Result<()> {
        let vector = [
            float(input, profile)?,
            float(input, profile)?,
            float(input, profile)?,
        ];
        self.vector_00 = Some(vector.map(f64::from));
        let first = input.read_u8()?;
        self.field_18 = Some(u32::from(first));
        if first != 0 {
            self.field_28 = Some(input.read_i32()?);
            self.field_20 = Some(input.read_i32()?);
        }
        let second = input.read_u8()?;
        self.field_1c = Some(u32::from(second));
        if second != 0 {
            self.field_2c = Some(input.read_i32()?);
            self.field_24 = Some(input.read_i32()?);
        }
        Ok(())
    }
}

fn float(input: &mut QtReader<'_>, profile: FloatEncoding) -> io::Result<f32> {
    match profile {
        FloatEncoding::DoublePrecision => input.read_f64().map(|value| value as f32),
        FloatEncoding::TypeWidths | FloatEncoding::SinglePrecision => input.read_f32(),
    }
}

#[cfg(test)]
mod tests;
