use super::*;

#[derive(Debug, PartialEq, Eq)]
pub struct StoryText {
    pub kind: u8,
    pub instance: i32,
    pub header: Message,
}
impl StoryText {
    pub fn decode(kind: u8, body: &[u8]) -> io::Result<Self> {
        if !matches!(kind, 0x2d | 0x4a | 0x4e) || body.len() > 64 * 1024 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unsupported or oversized story text",
            ));
        }
        let mut reader = QtReader::new(body);
        let mut header = header(&mut reader)?;
        let instance = reader.read_u32()? as i32;
        header.text = reader.read_string()?;
        if reader.remaining() != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "trailing story text",
            ));
        }
        Ok(Self {
            kind,
            instance,
            header,
        })
    }
}
