use super::*;
use crate::protocol::QtWriter;
use std::io::Write;

/// Source CHAT_RANGE_NAMES and chat_range.py; not inferred distance thresholds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Range {
    #[default]
    Close,
    Hail,
    System,
    Sector,
    Galaxy,
    Universe,
}
impl Range {
    pub fn name(self) -> &'static str {
        ["Close", "Hail", "System", "Sector", "Galaxy", "Universe"][self as usize]
    }
    pub fn next(self) -> Self {
        match self {
            Self::Close => Self::Hail,
            Self::Hail => Self::System,
            Self::System => Self::Sector,
            Self::Sector => Self::Galaxy,
            Self::Galaxy => Self::Universe,
            Self::Universe => Self::Close,
        }
    }
}

pub struct Outgoing {
    pub channel: u8,
    pub range: Range,
    pub text: String,
}
impl Outgoing {
    pub fn encode(&self, sender: u32) -> io::Result<Vec<u8>> {
        if sender == 0
            || self.channel as usize >= CHANNELS.len()
            || self.text.is_empty()
            || self.text.chars().count() > 256
            || self.text.contains('\0')
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "chat requires an authenticated sender, known channel and 1..=256 characters",
            ));
        }
        let mut plain = QtWriter::default();
        plain.write_u8(0x29);
        plain.write_u8(self.channel);
        plain.write_u8(0x40); // Explicit range, no inferred galaxy or auxiliary data.
        plain.write_u32(sender);
        plain.write_string(Some(""))?; // Server supplies the authenticated character name.
        plain.write_u32(0);
        plain.write_u8((self.range as u8) | ((self.range as u8) << 4));
        plain.write_u8(0); // No targeted/auxiliary scope.
        plain.write_string(Some(&self.text))?;
        let plain = plain.into_bytes();
        envelope(&plain)
    }
}

pub(super) fn envelope(plain: &[u8]) -> io::Result<Vec<u8>> {
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(plain)?;
    let mut compressed = (plain.len() as u32).to_be_bytes().to_vec();
    compressed.extend(encoder.finish()?);
    let mut envelope = QtWriter::default();
    envelope.write_u8(0x0a);
    envelope.write_u32(0); // Source handler skips this field; synthetic profile, not native parity.
    envelope.write_bytes(Some(&crate::protocol::au_crypt(&compressed, 0x6e32)))?;
    Ok(envelope.into_bytes())
}
