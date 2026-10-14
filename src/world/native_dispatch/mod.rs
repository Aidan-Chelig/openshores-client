//! Stable-owner atom branch of native ReadPacket. No source-profile fallback.
use super::{
    native_atom::NativeAtomKind, native_prefix::NativeIdentityPrefix,
    native_subatoms::NativeRecoveryLookup,
};
use crate::protocol::QtReader;
use std::io;

/// Required object/owner operations. No default callbacks silently omit native
/// construction, base layers, signal wiring, body reception or post-update work.
/// The owner must remain stable across this call; Qt locking is not emulated.
pub trait NativeAtomDispatchContext {
    fn lookup(&self) -> &NativeRecoveryLookup;
    fn construct_candidate(&mut self, kind: NativeAtomKind) -> io::Result<u32>;
    fn assign_identity(&mut self, handle: u32, identity: NativeIdentityPrefix) -> io::Result<()>;
    fn assign_time(&mut self, handle: u32, timestamp_ms: i64) -> io::Result<()>;
    /// Consume all base layers, including any effects/controllers, before body.
    fn receive_base(
        &mut self,
        handle: u32,
        identity: NativeIdentityPrefix,
        input: &mut QtReader<'_>,
    ) -> io::Result<()>;
    fn connect_signals(&mut self, handle: u32) -> io::Result<()>;
    fn add_atom(&mut self, handle: u32) -> io::Result<()>;
    fn destroy_candidate(&mut self, handle: u32) -> io::Result<()>;
    /// Dispatch through this object's actual class, never the candidate opcode.
    fn receive_body(
        &mut self,
        handle: u32,
        is_new: bool,
        input: &mut QtReader<'_>,
    ) -> io::Result<()>;
    fn post_update(&mut self, handle: u32, is_new: bool) -> io::Result<()>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeAtomDispatchOutcome {
    pub wire_id: u32,
    pub handle: u32,
    pub is_new: bool,
    pub consumed_bytes: usize,
}

/// Dispatch one unframed payload using the established factory opcode table.
/// Non-atom/unsupported opcodes return None without constructing an object.
/// Errors retain already completed context operations (including construction
/// or time assignment). The adapter owns recovery/disposal for Rust errors;
/// this is not native Qt malformed-stream behavior or a rollback transaction.
pub fn receive_atom(
    payload: &[u8],
    context: &mut impl NativeAtomDispatchContext,
) -> io::Result<Option<NativeAtomDispatchOutcome>> {
    let mut input = QtReader::new(payload);
    let Some(kind) = NativeAtomKind::from_opcode(input.read_u8()?) else {
        return Ok(None);
    };
    let candidate = context.construct_candidate(kind)?;
    let identity = NativeIdentityPrefix::read(&mut input)?;
    context.assign_identity(candidate, identity)?;
    let existing = context.lookup().find_atom(identity.id);
    let handle = if let Some(handle) = existing {
        context.assign_time(handle, identity.timestamp_ms)?;
        context.receive_base(handle, identity, &mut input)?;
        context.destroy_candidate(candidate)?;
        handle
    } else {
        context.receive_base(candidate, identity, &mut input)?;
        context.connect_signals(candidate)?;
        context.add_atom(candidate)?;
        candidate
    };
    let is_new = existing.is_none();
    context.receive_body(handle, is_new, &mut input)?;
    context.post_update(handle, is_new)?;
    Ok(Some(NativeAtomDispatchOutcome {
        wire_id: identity.id,
        handle,
        is_new,
        consumed_bytes: payload.len() - input.remaining(),
    }))
}

#[cfg(test)]
mod tests;
