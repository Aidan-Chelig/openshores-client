use super::*;
use crate::protocol::QtWriter;

/// Untargeted Voice greeting: the source server chooses the nearest native.
pub struct Hail;
impl Hail {
    pub fn encode(sender: u32, name: &str) -> io::Result<Vec<u8>> {
        if sender == 0 || name.len() > 1024 || name.contains('\0') {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "hail requires authenticated avatar",
            ));
        }
        let mut writer = QtWriter::default();
        writer.write_u8(0x3f);
        writer.write_u8(2); // Voice.
        writer.write_u8(0x40); // Close range, no galaxy/auxiliary header.
        writer.write_u32(sender);
        writer.write_string(Some(name))?;
        writer.write_u32(0); // Untargeted environment.
        writer.write_u8(0);
        writer.write_u8(15); // Source VOICE_SCOPE.
        writer.write_u32(0); // No claimed empire identity.
        writer.write_u8(0); // No optional greeting text or claimed galaxy.
        outgoing::envelope(&writer.into_bytes())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct HailReply {
    pub header: Message,
    pub environment: u32,
}
impl HailReply {
    pub fn decode(body: &[u8]) -> io::Result<Self> {
        if body.len() > 64 * 1024 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "hail reply exceeds budget",
            ));
        }
        let mut input = QtReader::new(body);
        let mut header = header(&mut input)?;
        let environment = input.read_u32()?;
        header.text = input.read_string()?;
        if input.remaining() != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "trailing hail reply",
            ));
        }
        Ok(Self {
            header,
            environment,
        })
    }
}
