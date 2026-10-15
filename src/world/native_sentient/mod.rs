//! Ordered suffix after Creature reception, not a replacement for that stage.
//! Evidence/constructor boundaries: docs/native-sentient.md.
use crate::{atom_position::FloatEncoding, protocol::QtReader};
use std::io;
mod skills;
pub use skills::NativeSkillSet;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativeOrderSeed {
    pub translation: [f64; 3],
    pub denominator_terms: [f64; 4],
    pub wall_clock_ms: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NativeOrder {
    pub position: [f64; 3],
    pub field_18: u16,
    pub id_1c: u32,
    pub field_20: Option<String>,
    pub time_28: i64,
    pub kind_30: u32,
}

impl NativeOrder {
    fn initialize(seed: NativeOrderSeed) -> Self {
        let [a, b, c, d] = seed.denominator_terms;
        let denominator = a + b + c + d;
        let mut position = seed.translation;
        if denominator != 0. && denominator != 1. {
            for value in &mut position {
                *value /= denominator;
            }
        }
        Self {
            position,
            field_18: 0,
            id_1c: 0,
            field_20: Some(String::new()),
            time_28: seed.wall_clock_ms,
            kind_30: 0,
        }
    }
}

/// Options mark unobserved fields, not native constructor defaults.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NativeSentientState {
    pub field_708: Option<f32>,
    pub orders: Option<Vec<NativeOrder>>,
    pub field_6f0: Option<u32>,
    pub skills: Option<NativeSkillSet>,
}

impl NativeSentientState {
    /// Receive-visible DaSentient constructor subset; no inherited allocation.
    /// The actual SkillSet constructor stores an empty shared-null QHash.
    pub fn from_default_constructor_subset() -> Self {
        Self {
            field_708: Some(0.),
            orders: Some(Vec::new()),
            field_6f0: Some(1),
            skills: Some(NativeSkillSet::default()),
        }
    }

    /// Caller must finish Creature first. Seed is required from the real atom
    /// matrix/clock only when an order needs initialization, never on reuse.
    /// Earlier mutations survive later Rust errors; no Qt failed-stream parity
    /// or QObject construction is implied by this suffix-state decoder.
    pub fn receive_suffix(
        &mut self,
        input: &mut QtReader<'_>,
        encoding: FloatEncoding,
        mut order_seed: impl FnMut() -> io::Result<NativeOrderSeed>,
    ) -> io::Result<usize> {
        let length = input.remaining();
        let flags = input.read_u8()?;
        self.field_708 = Some(if flags & 1 == 0 {
            0.
        } else {
            match encoding {
                FloatEncoding::DoublePrecision => input.read_f64()? as f32,
                FloatEncoding::TypeWidths | FloatEncoding::SinglePrecision => input.read_f32()?,
            }
        });
        let orders = self.orders.get_or_insert_with(Vec::new);
        if flags & 2 == 0 {
            orders.clear();
        } else {
            if orders.len() != 1 {
                orders.clear();
                orders.push(NativeOrder::initialize(order_seed()?));
            }
            let order = &mut orders[0];
            let kind = input.read_u8()?;
            order.kind_30 = u32::from(kind);
            order.id_1c = input.read_u32()?;
            if kind == 0x29 {
                order.field_20 = input.read_string()?;
            }
            self.field_6f0 = Some(u32::from(input.read_u8()?));
        }
        if flags & 4 != 0 {
            self.skills
                .get_or_insert_with(Default::default)
                .receive(input)?;
        }
        Ok(length - input.remaining())
    }
}

#[cfg(test)]
mod tests;
