//! Machine-code-derived MovementSpeedMod; the retained decompiler loses f32 return.
use std::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeMovementBody {
    DpBody { hit_points: i16 },
    Animalia { hit_points: i16 },
}

/// Required observations of the actual selected model, growth and burden.
/// No default constant substitutes for unimplemented morphology/item weights.
pub trait NativeMovementContext {
    /// DpBody has priority if present; missing both bodies must fail explicitly.
    fn body(&mut self) -> io::Result<NativeMovementBody>;
    fn growth_byte_494(&mut self) -> io::Result<u8>;
    fn growth_time(&mut self) -> io::Result<u8>;
    fn dp_byte_29(&mut self) -> io::Result<u8>;
    fn max_hit_points(&mut self) -> io::Result<i16>;
    /// Actual virtual+970. Native samples twice on the below-capacity path.
    fn burden(&mut self) -> io::Result<f32>;
}

/// Preserve signed short truncation, two burden observations and ordered f32
/// stages. NaN/infinity/negative results are not normalized. A native integer
/// divide trap for zero GrowthTime is a Rust InvalidData error instead.
pub fn native_movement_speed_modifier(context: &mut impl NativeMovementContext) -> io::Result<f32> {
    let body = context.body()?;
    let mut hp = match body {
        NativeMovementBody::DpBody { hit_points, .. }
        | NativeMovementBody::Animalia { hit_points } => hit_points,
    };
    let growth = context.growth_byte_494()?;
    if growth != 0 && hp > 0 {
        hp = super::native_growth_adjusted_hit_points(hp, growth, context.growth_time()?)?;
    }
    if hp <= 0 {
        return Ok(1.);
    }
    let capacity = match body {
        NativeMovementBody::DpBody { .. } => {
            let byte_29 = context.dp_byte_29()?;
            let max_hp = context.max_hit_points()?;
            (25 * (i32::from(max_hp / 10 + 1) + i32::from(byte_29 >> 4))) as f32
        }
        // AuAnimalia::BurdenMax is a direct load of PE float 0x43480000.
        NativeMovementBody::Animalia { .. } => 200.,
    };
    if context.burden()? >= capacity {
        return Ok(0.);
    }
    let remaining = capacity - context.burden()?;
    let mut modifier = remaining / capacity;
    // Native compares zero >= modifier: unordered NaN continues, not zero.
    if 0. >= modifier {
        return Ok(0.);
    }
    let doubled_hp = hp.wrapping_mul(2);
    let max_hp = context.max_hit_points()?;
    if max_hp > doubled_hp {
        modifier *= f32::from(doubled_hp) / f32::from(max_hp);
    }
    Ok(modifier)
}

#[cfg(test)]
mod tests;
