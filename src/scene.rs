//! Direction-specific scene bootstrap codecs from the supplied Python server.
//!
//! Opcodes are reused: 0x38 is a client resume hello or a server initialization
//! message; 0x22 is a world redirect or building data depending on phase.
//! Callers must select a decoder using direction and connection state.
use crate::protocol::{QtReader, QtWriter};
use std::io;

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn packet(bytes: &[u8], opcode: u8) -> io::Result<QtReader<'_>> {
    let mut input = QtReader::new(bytes);
    if input.read_u8()? != opcode {
        return Err(invalid("unexpected scene opcode"));
    }
    Ok(input)
}

fn complete(input: &QtReader<'_>) -> io::Result<()> {
    if input.remaining() != 0 {
        return Err(invalid("trailing scene packet bytes"));
    }
    Ok(())
}

/// No guessed platform/session fields: caller supplies values from its session.
pub struct ResumeHello<'a> {
    pub version: &'a str,
    pub session_hi: i32,
    pub session_lo: i32,
    pub avatar_id: u32,
    pub os_kind: u8,
    pub windows_version: u16,
    pub flag40: u8,
    pub extra: u8,
}

impl ResumeHello<'_> {
    pub fn encode(&self) -> io::Result<Vec<u8>> {
        if self.avatar_id == 0 {
            return Err(invalid("resume requires a nonzero avatar ID"));
        }
        let mut out = QtWriter::default();
        out.write_u8(0x38);
        out.write_string(Some(self.version))?;
        out.write_u32(self.session_hi as u32);
        out.write_u32(self.session_lo as u32);
        out.write_u32(self.avatar_id);
        out.write_u8(self.os_kind);
        out.write_u16(self.windows_version);
        out.write_u8(self.flag40);
        out.write_u8(self.extra);
        Ok(out.into_bytes())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct WorldRedirect {
    pub server: Option<String>,
    pub port: u16,
    pub world_state: u8,
    pub account_lo: u32,
    pub account_hi: i32,
}

/// Only valid for a redirect response during the initial world connection.
pub fn decode_world_redirect(bytes: &[u8]) -> io::Result<WorldRedirect> {
    let mut input = packet(bytes, 0x22)?;
    let result = WorldRedirect {
        server: input.read_string()?,
        port: input.read_u16()?,
        world_state: input.read_u8()?,
        account_lo: input.read_u32()?,
        account_hi: input.read_u32()? as i32,
    };
    complete(&input)?;
    Ok(result)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InitSucceeded {
    pub motd: Option<String>,
    pub chat_port: u16,
    pub autime_usec: u64,
}

pub fn decode_init_succeeded(bytes: &[u8]) -> io::Result<InitSucceeded> {
    let mut input = packet(bytes, 0x38)?;
    let motd = input.read_string()?;
    let chat_port = input.read_u16()?;
    let autime_usec = (u64::from(input.read_u32()?) << 32) | u64::from(input.read_u32()?);
    complete(&input)?;
    Ok(InitSucceeded {
        motd,
        chat_port,
        autime_usec,
    })
}

pub fn decode_active_unit(bytes: &[u8]) -> io::Result<u32> {
    let mut input = packet(bytes, 0x2a)?;
    let id = input.read_u32()?;
    complete(&input)?;
    Ok(id)
}

#[derive(Debug, PartialEq, Eq)]
pub enum SceneTime {
    Manifest { time_low: u32, atom_ids: Vec<u32> },
    Heartbeat { time_low: u32 },
}

/// A server 0x18 flag-1 manifest or flag-2 heartbeat, not a client look-at packet.
pub fn decode_scene_time(bytes: &[u8]) -> io::Result<SceneTime> {
    let mut input = packet(bytes, 0x18)?;
    let time_low = input.read_u32()?;
    let result = match input.read_u8()? {
        1 => {
            let count = input.read_u16()? as i16;
            if count < 0 {
                return Err(invalid("negative scene manifest count"));
            }
            if input.remaining() != count as usize * 4 {
                return Err(invalid("scene manifest count does not match payload"));
            }
            let atom_ids = (0..count)
                .map(|_| input.read_u32())
                .collect::<io::Result<_>>()?;
            SceneTime::Manifest { time_low, atom_ids }
        }
        2 => SceneTime::Heartbeat { time_low },
        _ => return Err(invalid("unknown scene time flag")),
    };
    complete(&input)?;
    Ok(result)
}

/// The server parser reads a single u32 after opcode 0x3b.
pub fn encode_select_unit(avatar_id: u32) -> io::Result<Vec<u8>> {
    if avatar_id == 0 {
        return Err(invalid("selection requires a nonzero avatar ID"));
    }
    let mut out = QtWriter::default();
    out.write_u8(0x3b);
    out.write_u32(avatar_id);
    Ok(out.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(text: &str) -> Vec<u8> {
        text.trim()
            .as_bytes()
            .chunks_exact(2)
            .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
            .collect()
    }

    #[test]
    fn python_bootstrap_fixtures() {
        let redirect = hex(include_str!("../tests/fixtures/scene-redirect.hex"));
        assert_eq!(
            decode_world_redirect(&redirect).unwrap(),
            WorldRedirect {
                server: Some("127.0.0.1".into()),
                port: 16759,
                world_state: 2,
                account_lo: 0x89abcdef,
                account_hi: -2
            }
        );
        let init = hex(include_str!("../tests/fixtures/scene-init.hex"));
        assert_eq!(
            decode_init_succeeded(&init).unwrap(),
            InitSucceeded {
                motd: Some("Welcome 🚀".into()),
                chat_port: 16758,
                autime_usec: 0x123456789abcdef0
            }
        );
        for end in 0..redirect.len() {
            assert!(decode_world_redirect(&redirect[..end]).is_err());
        }
        for end in 0..init.len() {
            assert!(decode_init_succeeded(&init[..end]).is_err());
        }
        assert!(decode_init_succeeded(&redirect).is_err());
        assert!(decode_world_redirect(&init).is_err());
    }

    #[test]
    fn manifest_and_heartbeat_are_distinct() {
        assert_eq!(
            decode_scene_time(&hex("181234567802")).unwrap(),
            SceneTime::Heartbeat {
                time_low: 0x12345678
            }
        );
        assert_eq!(
            decode_scene_time(&hex("18123456780100020000002affffffff")).unwrap(),
            SceneTime::Manifest {
                time_low: 0x12345678,
                atom_ids: vec![42, u32::MAX]
            }
        );
        for bad in [
            "181234567801ffff",
            "181234567801000100",
            "181234567803",
            "18123456780200",
        ] {
            assert!(decode_scene_time(&hex(bad)).is_err());
        }
        assert_eq!(decode_active_unit(&hex("2a0000002a")).unwrap(), 42);
        assert_eq!(encode_select_unit(42).unwrap(), hex("3b0000002a"));
        assert!(encode_select_unit(0).is_err());
    }

    #[test]
    fn resume_preserves_field_order_and_signed_bits() {
        let hello = ResumeHello {
            version: "13.110.60",
            session_hi: -2,
            session_lo: 1,
            avatar_id: 42,
            os_kind: 0,
            windows_version: 0,
            flag40: 0x40,
            extra: 0,
        };
        assert_eq!(
            hello.encode().unwrap(),
            hex(include_str!("../tests/fixtures/scene-resume.hex"))
        );
    }
}
