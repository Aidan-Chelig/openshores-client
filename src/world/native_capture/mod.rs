//! Offline native payloads in the existing length-prefixed capture container.
//! Does not assert complete native transport/body parity or mutate world state.
mod items;
use super::{
    native_atom::{NativeAtomKind, NativeAtomPrefix},
    native_prefix::NativeBasePrefix,
};
use crate::protocol::read_frame;
pub use items::NativeItemReplay;
use std::io::{self, BufRead};

#[derive(Clone, Copy, Debug)]
pub struct CaptureLimits {
    pub frames: usize,
    pub frame_bytes: usize,
    pub payload_bytes: usize,
}

impl Default for CaptureLimits {
    fn default() -> Self {
        Self {
            frames: 20_000,
            frame_bytes: 16 * 1024 * 1024,
            payload_bytes: 128 * 1024 * 1024,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CapturedNativeFrame {
    payload: Vec<u8>,
    prefix: Option<(NativeAtomKind, NativeBasePrefix)>,
}

impl CapturedNativeFrame {
    /// Decode only established item bodies. Unknown types/base layers stay
    /// opaque; known malformed bodies fail. The retained payload never changes.
    pub fn item_body(&self) -> io::Result<Option<super::native_item::NativeItemBody>> {
        let Some((kind, base)) = &self.prefix else {
            return Ok(None);
        };
        super::native_atom::item_body(*kind, base.tail_kind, self.remainder())
    }
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
    pub fn prefix(&self) -> Option<&(NativeAtomKind, NativeBasePrefix)> {
        self.prefix.as_ref()
    }
    /// Untouched remaining layers, or the whole payload for unclassified frames.
    pub fn remainder(&self) -> &[u8] {
        let offset = self
            .prefix
            .as_ref()
            .map(|(_, base)| 1 + base.consumed_bytes)
            .unwrap_or(0);
        &self.payload[offset..]
    }
}

#[derive(Clone, Debug, Default)]
pub struct NativeCapture {
    pub frames: Vec<CapturedNativeFrame>,
    pub payload_bytes: usize,
}

impl NativeCapture {
    /// Decode prefixes only. Unknown packets stay byte-exact; malformed known
    /// prefixes fail explicitly. Limits may tighten, never raise, hard budgets.
    pub fn read(reader: &mut impl BufRead, limits: CaptureLimits) -> io::Result<Self> {
        let hard = CaptureLimits::default();
        if limits.frames > hard.frames
            || limits.frame_bytes > hard.frame_bytes
            || limits.payload_bytes > hard.payload_bytes
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "native capture limits exceed hard budgets",
            ));
        }
        let mut capture = Self::default();
        while !reader.fill_buf()?.is_empty() {
            if capture.frames.len() >= limits.frames {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "native capture exceeds frame budget",
                ));
            }
            let remaining = limits.payload_bytes - capture.payload_bytes;
            // Enforce aggregate allocation budget before reading each payload.
            let payload = read_frame(reader, limits.frame_bytes.min(remaining))?;
            let prefix = NativeAtomPrefix::decode(&payload)?.map(|value| (value.kind, value.base));
            capture.payload_bytes += payload.len();
            capture.frames.push(CapturedNativeFrame { payload, prefix });
        }
        Ok(capture)
    }
}

#[cfg(test)]
mod tests;
