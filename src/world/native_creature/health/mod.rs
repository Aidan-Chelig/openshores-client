//! Native body MaxHitPoints and growth arithmetic; geometry providers are required.
use std::io;
mod dp;
pub use dp::{NativeDpHealthFlags, NativeDpHealthState, NativePartVolumeContext};
mod animalia;
pub use animalia::{NativeAnimaliaHealthContext, native_animalia_max_hit_points};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use = "report native short-overflow diagnostics; this is not complete model generation"]
pub struct NativeHealthResult {
    pub hit_points: i16,
    /// Native warns/replaces when the converted value's LOW signed short is negative.
    pub negative_short_replaced: bool,
}

/// CVTTSS2SI's integer-indefinite result, unlike Rust's saturating float cast.
fn cvtt(value: f32) -> i32 {
    if !value.is_finite() || !(-2_147_483_648_f32..2_147_483_648_f32).contains(&value) {
        i32::MIN
    } else {
        value.trunc() as i32
    }
}
fn volume_hp(volume: f32, scale_bits: u32) -> (u16, bool) {
    let converted = cvtt(volume.sqrt() * f32::from_bits(scale_bits)) as i16;
    let replaced = converted < 0;
    (
        if replaced {
            i16::MAX as u16
        } else {
            converted as u16
        },
        replaced,
    )
}
fn finish(value: u16, negative_short_replaced: bool) -> NativeHealthResult {
    NativeHealthResult {
        hit_points: if value == 0 { 1 } else { value as i16 },
        negative_short_replaced,
    }
}

pub fn native_dp_growth_time(byte_25: u8) -> u8 {
    byte_25 | 7
}

/// AuAnimalia GrowthTime calls its unscaled model MaxHP first. This function
/// takes that actual result, not Creature's already growth-adjusted MaxHP.
pub fn native_animalia_growth_time(model_max_hp: i16) -> u8 {
    let root = f32::from(model_max_hp).sqrt();
    cvtt((root + root).ceil()).min(255) as u8
}

/// DaCreature's growth adjustment. Zero growth bypasses GrowthTime entirely;
/// caller must preserve that conditional observation order. Signed short zero
/// maps to1 only on the adjusted path. Native divide traps become Rust errors.
pub fn native_growth_adjusted_hit_points(base: i16, growth: u8, time: u8) -> io::Result<i16> {
    if growth == 0 {
        return Ok(base);
    }
    if time == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "native health GrowthTime division by zero",
        ));
    }
    let adjusted =
        ((i32::from(time) - (i32::from(growth) - 1)) * i32::from(base) / i32::from(time)) as i16;
    Ok(if adjusted == 0 { 1 } else { adjusted })
}

fn bounded_parts(count: usize) -> io::Result<usize> {
    if count > 20_000 {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "native model-part health safety budget",
        ))
    } else {
        Ok(count)
    }
}

#[cfg(test)]
mod tests;
