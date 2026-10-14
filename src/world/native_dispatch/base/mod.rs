//! Stateful RxUpdateBase ordering, starting after the identity stage.
use crate::{
    atom_position::{FloatEncoding, PackedPosition},
    protocol::QtReader,
};
use std::io;

/// Required operations on the already selected atom. Inner effect/controller
/// formats and Qt locks remain the adapter's responsibility, not guessed here.
pub trait NativeBaseReceiveContext {
    /// Complete ChangePos and conditional offset-0xa0 reset before returning.
    fn receive_position(&mut self, position: PackedPosition) -> io::Result<()>;
    fn reset_status(&mut self) -> io::Result<()>;
    fn assign_created_time(&mut self, time: i64) -> io::Result<()>;
    fn assign_modified_time(&mut self, time: i64) -> io::Result<()>;
    /// Allocate effects storage if absent, consume count/values and append
    /// using the native comparison policy. No body bytes may be consumed.
    fn receive_effects(&mut self, input: &mut QtReader<'_>) -> io::Result<()>;
    fn assign_motion(&mut self, in_motion: bool) -> io::Result<()>;
    /// Allocate/Append and dispose empty controller storage as established.
    fn receive_controllers(&mut self, input: &mut QtReader<'_>) -> io::Result<()>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeBaseReceiveOutcome {
    pub flags: u8,
    /// Base bytes only; excludes identity and enclosing opcode.
    pub consumed_bytes: usize,
}

/// Perform established base stages in native order. Earlier reader/context
/// effects survive later errors. Unlike observation-prefix decoding this is
/// intentionally not atomic. Rust rejects incomplete packed positions before
/// their callback; that safety policy does not emulate Qt stream error state.
pub fn receive_base(
    input: &mut QtReader<'_>,
    encoding: FloatEncoding,
    context: &mut impl NativeBaseReceiveContext,
) -> io::Result<NativeBaseReceiveOutcome> {
    let length = input.remaining();
    let flags = input.read_u8()?;
    if flags & 1 != 0 {
        context.receive_position(PackedPosition::read(input, encoding)?)?;
    }
    context.reset_status()?;
    if flags & 2 != 0 {
        context.assign_created_time(input.read_i64()?)?;
    }
    if flags & 4 != 0 {
        context.assign_modified_time(input.read_i64()?)?;
    }
    if flags & 0x10 != 0 {
        context.receive_effects(input)?;
    }
    context.assign_motion(flags & 0x20 != 0)?;
    if flags & 0x40 != 0 {
        context.receive_controllers(input)?;
    }
    Ok(NativeBaseReceiveOutcome {
        flags,
        consumed_bytes: length - input.remaining(),
    })
}

#[cfg(test)]
mod tests;
