use super::*;

/// Bare chat-channel controls consumed by source story_targoss.on_story_op.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoryControl {
    Close,
    Hint,
    Resend,
}
impl StoryControl {
    pub fn encode(self) -> [u8; 1] {
        [match self {
            Self::Close => 0x7d,
            Self::Hint => 0x7e,
            Self::Resend => 0x7f,
        }]
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Choice {
    pub narrated: bool,
    pub header: Message,
    pub index: u8,
    pub instance: i32,
    pub prompt: Option<String>,
    pub offered_to: u32,
}
impl Choice {
    /// Narrated choice has no final offered-to field; the parent target owns it.
    pub fn decode_narration(body: &[u8]) -> io::Result<Self> {
        if body.len() > 64 * 1024 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "narrated choice exceeds budget",
            ));
        }
        let mut input = QtReader::new(body);
        let header = header(&mut input)?;
        let index = input.read_u8()?;
        let instance = input.read_u32()? as i32;
        let prompt = input.read_string()?;
        if input.remaining() != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "trailing narrated choice",
            ));
        }
        let offered_to = header.target;
        Ok(Self {
            narrated: true,
            header,
            index,
            instance,
            prompt,
            offered_to,
        })
    }
    pub fn reply(&self) -> Reply {
        Reply {
            instance: self.instance as u32,
            index: self.index,
            offered: self.header.sender,
        }
    }
    pub fn decode(body: &[u8]) -> io::Result<Self> {
        if body.len() > 64 * 1024 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "choice body exceeds budget",
            ));
        }
        let mut input = QtReader::new(body);
        let header = header(&mut input)?;
        let index = input.read_u8()?;
        let instance = input.read_u32()? as i32;
        let prompt = input.read_string()?;
        let offered_to = input.read_u32()?;
        if input.remaining() != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "trailing choice bytes",
            ));
        }
        Ok(Self {
            narrated: false,
            header,
            index,
            instance,
            prompt,
            offered_to,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Reply {
    pub instance: u32,
    pub index: u8,
    pub offered: u32,
}
impl Reply {
    pub fn encode(self) -> io::Result<Vec<u8>> {
        if self.index > 127 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "choice requires a signed-positive index",
            ));
        }
        let mut writer = crate::protocol::QtWriter::default();
        writer.write_u8(0x7c);
        writer.write_u32(self.instance);
        writer.write_u8(self.index);
        writer.write_u32(self.offered);
        Ok(writer.into_bytes())
    }
}
