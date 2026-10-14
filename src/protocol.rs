//! Derived from reference/OpenShores-server/src/openshores/protocol/{framing,stream,encryption}.py.
use std::io::{self, Read, Write};

pub const VERSION: &str = "13.110.60";
pub const MAX_WIRE_SIZE: usize = 0x3fff_ffff;

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

/// The first two bits encode how many additional length bytes follow.
pub fn encode_size(size: usize) -> io::Result<Vec<u8>> {
    if size > MAX_WIRE_SIZE {
        return Err(invalid("frame exceeds the 30-bit wire length"));
    }
    let count = match size {
        0..0x40 => 1,
        0x40..0x4000 => 2,
        0x4000..0x400000 => 3,
        _ => 4,
    };
    let bytes = (size as u32).to_be_bytes();
    let mut result = bytes[4 - count..].to_vec();
    result[0] |= ((count - 1) as u8) << 6;
    Ok(result)
}

pub fn read_size(reader: &mut impl Read) -> io::Result<usize> {
    let mut first = [0];
    reader.read_exact(&mut first)?;
    let count = usize::from(first[0] >> 6);
    let mut extra = [0; 3];
    reader.read_exact(&mut extra[..count])?;
    Ok(extra[..count]
        .iter()
        .fold(usize::from(first[0] & 0x3f), |n, b| {
            (n << 8) | usize::from(*b)
        }))
}

/// Bound allocation before reading an untrusted payload.
pub fn read_frame(reader: &mut impl Read, max_payload: usize) -> io::Result<Vec<u8>> {
    let size = read_size(reader)?;
    if size > max_payload {
        return Err(invalid("frame exceeds the configured payload limit"));
    }
    let mut payload = vec![0; size];
    reader.read_exact(&mut payload)?;
    Ok(payload)
}

pub fn write_frame(writer: &mut impl Write, payload: &[u8]) -> io::Result<()> {
    writer.write_all(&encode_size(payload.len())?)?;
    writer.write_all(payload)
}

pub fn net_crypt_key(per_packet: u16) -> u16 {
    let key = 0x6e31u16.wrapping_add(per_packet);
    if key == 0 { per_packet } else { key }
}

/// Original reversible byte obfuscation; this provides no transport security.
pub fn au_crypt(data: &[u8], key: u16) -> Vec<u8> {
    let mut mask = ((key & 0x0f0f) >> 8) as u8;
    let step = (key & 0x0f) as u8;
    data.iter()
        .map(|byte| {
            let result = byte ^ mask;
            mask = mask.wrapping_add(step);
            result
        })
        .collect()
}

pub struct QtReader<'a> {
    remaining: &'a [u8],
}

impl<'a> QtReader<'a> {
    pub(crate) fn remaining_bytes(&self) -> &'a [u8] {
        self.remaining
    }
    pub(crate) fn skip(&mut self, count: usize) -> io::Result<()> {
        self.take(count).map(|_| ())
    }
    pub fn remaining(&self) -> usize {
        self.remaining.len()
    }

    pub fn read_u8(&mut self) -> io::Result<u8> {
        Ok(self.take(1)?[0])
    }

    pub fn read_u16(&mut self) -> io::Result<u16> {
        Ok(u16::from_be_bytes(self.take(2)?.try_into().unwrap()))
    }
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { remaining: bytes }
    }

    fn take(&mut self, count: usize) -> io::Result<&'a [u8]> {
        if count > self.remaining.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "truncated Qt stream",
            ));
        }
        let (value, rest) = self.remaining.split_at(count);
        self.remaining = rest;
        Ok(value)
    }

    pub fn read_u32(&mut self) -> io::Result<u32> {
        Ok(u32::from_be_bytes(self.take(4)?.try_into().unwrap()))
    }

    pub fn read_i64(&mut self) -> io::Result<i64> {
        Ok(i64::from_be_bytes(self.take(8)?.try_into().unwrap()))
    }

    pub fn read_f32(&mut self) -> io::Result<f32> {
        Ok(f32::from_be_bytes(self.take(4)?.try_into().unwrap()))
    }

    pub fn read_f64(&mut self) -> io::Result<f64> {
        Ok(f64::from_be_bytes(self.take(8)?.try_into().unwrap()))
    }

    pub fn read_bytes(&mut self) -> io::Result<Option<&'a [u8]>> {
        match self.read_u32()? {
            u32::MAX => Ok(None),
            length => self.take(length as usize).map(Some),
        }
    }

    /// QString lengths count bytes, not UTF-16 units; null differs from empty.
    pub fn read_string(&mut self) -> io::Result<Option<String>> {
        let Some(raw) = self.read_bytes()? else {
            return Ok(None);
        };
        if raw.len() % 2 != 0 {
            return Err(invalid("odd QString byte length"));
        }
        let units: Vec<_> = raw
            .chunks_exact(2)
            .map(|b| u16::from_be_bytes([b[0], b[1]]))
            .collect();
        String::from_utf16(&units)
            .map(Some)
            .map_err(|_| invalid("invalid QString UTF-16"))
    }
}

#[derive(Default)]
pub struct QtWriter(Vec<u8>);

impl QtWriter {
    pub fn write_u16(&mut self, value: u16) {
        self.0.extend(value.to_be_bytes());
    }
    pub fn write_u8(&mut self, value: u8) {
        self.0.push(value);
    }
    pub fn write_u32(&mut self, value: u32) {
        self.0.extend(value.to_be_bytes());
    }
    pub fn write_bytes(&mut self, value: Option<&[u8]>) -> io::Result<()> {
        match value {
            None => self.write_u32(u32::MAX),
            Some(bytes) => {
                let size = u32::try_from(bytes.len()).map_err(|_| invalid("Qt field too large"))?;
                if size == u32::MAX {
                    return Err(invalid("Qt field uses reserved null length"));
                }
                self.write_u32(size);
                self.0.extend(bytes);
            }
        }
        Ok(())
    }
    pub fn write_string(&mut self, value: Option<&str>) -> io::Result<()> {
        let bytes = value.map(|text| {
            text.encode_utf16()
                .flat_map(u16::to_be_bytes)
                .collect::<Vec<_>>()
        });
        self.write_bytes(bytes.as_deref())
    }
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_length_vectors() {
        for (size, wire) in [
            (0, vec![0]),
            (63, vec![63]),
            (64, vec![0x40, 0x40]),
            (16383, vec![0x7f, 0xff]),
            (16384, vec![0x80, 0x40, 0]),
            (4194303, vec![0xbf, 0xff, 0xff]),
            (4194304, vec![0xc0, 0x40, 0, 0]),
            (MAX_WIRE_SIZE, vec![0xff; 4]),
        ] {
            assert_eq!(encode_size(size).unwrap(), wire);
            assert_eq!(read_size(&mut wire.as_slice()).unwrap(), size);
        }
        assert!(encode_size(MAX_WIRE_SIZE + 1).is_err());
    }

    #[test]
    fn rejects_truncation_and_oversized_frames() {
        assert!(read_size(&mut [0xc0, 1].as_slice()).is_err());
        assert!(read_frame(&mut [5].as_slice(), 4).is_err());
        assert!(read_frame(&mut [2, 1].as_slice(), 4).is_err());
        let mut bytes = Vec::new();
        write_frame(&mut bytes, b"abc").unwrap();
        write_frame(&mut bytes, b"def").unwrap();
        let mut input = bytes.as_slice();
        assert_eq!(read_frame(&mut input, 3).unwrap(), b"abc");
        assert_eq!(read_frame(&mut input, 3).unwrap(), b"def");
    }

    #[test]
    fn qt_null_empty_surrogates_and_malformed_strings() {
        assert_eq!(QtReader::new(&[0xff; 4]).read_string().unwrap(), None);
        assert_eq!(
            QtReader::new(&[0; 4]).read_string().unwrap(),
            Some(String::new())
        );
        assert_eq!(
            QtReader::new(&[0, 0, 0, 4, 0xd8, 0x3d, 0xde, 0x80])
                .read_string()
                .unwrap(),
            Some("🚀".into())
        );
        assert!(QtReader::new(&[0, 0, 0, 1, 0]).read_string().is_err());
        assert!(QtReader::new(&[0, 0, 0, 2, 0xd8, 0]).read_string().is_err());
        assert!(QtReader::new(&[0, 0, 0, 4, 0]).read_string().is_err());
    }

    #[test]
    fn reference_obfuscation_vector() {
        assert_eq!(net_crypt_key(0x101), 0x6f32);
        assert_eq!(net_crypt_key(0x91cf), 0x91cf);
        assert_eq!(au_crypt(b"test", 0x6f32), [0x7b, 0x74, 0x60, 0x61]);
        assert_eq!(au_crypt(&au_crypt(b"test", 0x6f32), 0x6f32), b"test");
        assert_eq!(au_crypt(b"test", 0), b"test");
    }
}
