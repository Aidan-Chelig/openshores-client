//! Ordered wire stages only. Capsule/environment augmentation is still required
//! after these readers; they are not complete DpStation::Rx implementations.
use crate::{
    atom_position::FloatEncoding,
    protocol::QtReader,
    world::native_sensor_blip::{
        NativeSensorBlipFull, NativeSensorBlipMinList, NativeSensorBlipMinReceive,
    },
};
use std::io;
mod types;
pub use types::*;
mod receive;

fn float(input: &mut QtReader<'_>, profile: FloatEncoding) -> io::Result<f32> {
    match profile {
        FloatEncoding::DoublePrecision => Ok(input.read_f64()? as f32),
        FloatEncoding::SinglePrecision | FloatEncoding::TypeWidths => input.read_f32(),
    }
}

fn vector(input: &mut QtReader<'_>, profile: FloatEncoding) -> io::Result<[f64; 3]> {
    let mut value = [0.; 3];
    for component in &mut value {
        *component = f64::from(float(input, profile)?);
    }
    Ok(value)
}

fn bits<const N: usize>(flags: u16, shifts: [u8; N]) -> [u8; N] {
    shifts.map(|shift| ((flags >> shift) & 1) as u8)
}

#[cfg(test)]
mod tests;
