//! AuCommChat body from the supplied protocol/atoms/aucomm.py.
//! This codec is chat-channel-only; 0x0A on scene has a different meaning.
use crate::protocol::QtReader;
use std::io;
mod hail;
mod outgoing;
pub use hail::{Hail, HailReply};
mod narration;
mod story;
pub use narration::StoryText;
pub use outgoing::{Outgoing, Range};
pub use story::{Choice, Reply, StoryControl};

#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq, Eq)]
pub enum Incoming {
    Story(StoryText),
    Hail(HailReply),
    Choice(Choice),
    Message(Message),
    Opaque { kind: u8, body: Vec<u8> },
}

/// Server build_chat_aucomm_v4 emits 0x0A/QByteArray, not the client's
/// timestamp-bearing envelope. These directions must not be conflated.
pub fn decode_incoming(payload: &[u8]) -> io::Result<Incoming> {
    if payload.len() > 128 * 1024 || payload.first() != Some(&0x0a) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid server chat envelope",
        ));
    }
    let mut reader = QtReader::new(&payload[1..]);
    let cipher = reader
        .read_bytes()?
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "null chat ciphertext"))?;
    if reader.remaining() != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "trailing chat envelope",
        ));
    }
    let compressed = crate::protocol::au_crypt(cipher, 0x6e32);
    let plain = crate::har::qdecompress(&compressed, 64 * 1024)?;
    let (&kind, body) = plain
        .split_first()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "empty chat plaintext"))?;
    if matches!(kind, 0x2d | 0x4a | 0x4e) {
        Ok(Incoming::Story(StoryText::decode(kind, body)?))
    } else if kind == 0x4c {
        Ok(Incoming::Choice(Choice::decode_narration(body)?))
    } else if kind == 0x3e {
        Ok(Incoming::Hail(HailReply::decode(body)?))
    } else if kind == 0x2b {
        Ok(Incoming::Choice(Choice::decode(body)?))
    } else if kind == 0x29 || kind == 0x2c {
        Ok(Incoming::Message(Message::decode(body)?))
    } else {
        Ok(Incoming::Opaque {
            kind,
            body: body.to_vec(),
        })
    }
}

pub const CHANNELS: [&str; 22] = [
    "Hail",
    "Galactic",
    "Voice",
    "Intercom",
    "Crew",
    "Trade",
    "Friend",
    "Help",
    "Diplomacy",
    "Empire",
    "Government",
    "Fleet",
    "Company",
    "Recruit",
    "Thoughts",
    "Agent",
    "Programmer",
    "Architect",
    "Galaktika",
    "Galaxie",
    "Galaxia",
    "Galaktischen",
];

#[derive(Debug, PartialEq, Eq)]
pub enum Channel {
    Indexed(u8),
    Named(Option<String>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Message {
    pub channel: Channel,
    pub flags: u8,
    pub sender: u32,
    pub sender_name: Option<String>,
    pub target: u32,
    pub range: Option<(u8, u8)>,
    pub scope: u8,
    pub auxiliary: Option<i32>,
    pub text: Option<String>,
}

impl Message {
    /// Input excludes the envelope and 0x29 type byte. Unknown indexed channels
    /// remain numeric; absent/null and empty QStrings remain distinct.
    pub fn decode(body: &[u8]) -> io::Result<Self> {
        if body.len() > 64 * 1024 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "chat body exceeds budget",
            ));
        }
        let mut input = QtReader::new(body);
        let mut message = header(&mut input)?;
        message.text = input.read_string()?;
        if input.remaining() != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "trailing chat body bytes",
            ));
        }
        Ok(message)
    }
}

fn header(input: &mut QtReader<'_>) -> io::Result<Message> {
    let index = input.read_u8()?;
    let channel = if index == 255 {
        Channel::Named(input.read_string()?)
    } else if index < 128 {
        Channel::Indexed(index)
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid signed chat channel",
        ));
    };
    let flags = input.read_u8()?;
    let sender = input.read_u32()?;
    let sender_name = input.read_string()?;
    let target = input.read_u32()?;
    let range = if flags & 0x40 != 0 {
        let byte = input.read_u8()?;
        Some((byte & 15, byte >> 4))
    } else {
        None
    };
    let scope = input.read_u8()?;
    let auxiliary = if flags & 0x80 != 0 {
        Some(input.read_u32()? as i32)
    } else {
        None
    };
    Ok(Message {
        channel,
        flags,
        sender,
        sender_name,
        target,
        range,
        scope,
        auxiliary,
        text: None,
    })
}
