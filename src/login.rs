//! Login wire format from protocol/login.py and protocol/login_reply.py.
use crate::protocol::{
    QtReader, QtWriter, VERSION, au_crypt, net_crypt_key, read_frame, write_frame,
};
use sha1::{Digest, Sha1};
use std::{
    io,
    net::{SocketAddr, TcpStream},
    time::Duration,
};
use zeroize::Zeroizing;

/// The supplied server's accounts.py::_wire_password hashes UTF-8 verbatim.
pub fn wire_password(typed: &str) -> [u8; 20] {
    Sha1::digest(typed.as_bytes()).into()
}

#[derive(Debug, PartialEq, Eq)]
pub struct Avatar {
    pub id: u32,
    pub name: Option<String>,
    pub dna: [u8; 24],
    pub sex: u8,
    pub left_handed: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LoginReply {
    Accepted {
        status: u32,
        active_unit: u32,
        slots: Box<[Avatar; 6]>,
        trailer: u8,
    },
    Rejected {
        reason: u8,
        detail: u32,
    },
    Redirect {
        host: String,
        port: u16,
    },
    VersionMismatch,
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

/// `password_hash` is the SHA-1 digest of the typed password's UTF-8 bytes.
/// Username bytes match the supplied server's Latin-1-first decoder.
pub fn encode_login(username: &str, password_hash: &[u8; 20]) -> io::Result<Vec<u8>> {
    let name = username
        .chars()
        .map(|c| {
            u8::try_from(u32::from(c)).map_err(|_| invalid("server expects a Latin-1 username"))
        })
        .collect::<io::Result<Vec<_>>>()?;
    if name.is_empty() || name.contains(&0) {
        return Err(invalid("username must be nonempty and contain no NUL"));
    }
    let mut out = QtWriter::default();
    out.write_u8(3);
    out.write_string(Some(VERSION))?;
    out.write_bytes(Some(&au_crypt(&name, net_crypt_key(0x101))))?;
    out.write_bytes(Some(password_hash))?;
    Ok(out.into_bytes())
}

pub fn decode_reply(bytes: &[u8]) -> io::Result<LoginReply> {
    let mut input = QtReader::new(bytes);
    let reply = match input.read_u8()? {
        3 => {
            let status = input.read_u32()?;
            let active_unit = input.read_u32()?;
            let mut slots = Vec::with_capacity(6);
            for _ in 0..6 {
                let id = input.read_u32()?;
                let name = input.read_string()?;
                let dna = input
                    .read_bytes()?
                    .ok_or_else(|| invalid("null avatar DNA"))?
                    .try_into()
                    .map_err(|_| invalid("avatar DNA must contain 24 bytes"))?;
                let sex = input.read_u8()?;
                let left_handed = input.read_u8()? != 0;
                slots.push(Avatar {
                    id,
                    name,
                    dna,
                    sex,
                    left_handed,
                });
            }
            LoginReply::Accepted {
                status,
                active_unit,
                slots: Box::new(slots.try_into().unwrap()),
                trailer: input.read_u8()?,
            }
        }
        0xb => LoginReply::Rejected {
            reason: input.read_u8()?,
            detail: input.read_u32()?,
        },
        2 => LoginReply::Redirect {
            host: input
                .read_string()?
                .ok_or_else(|| invalid("null redirect host"))?,
            port: input.read_u16()?,
        },
        9 => LoginReply::VersionMismatch,
        _ => return Err(invalid("unrecognized login reply opcode")),
    };
    if input.remaining() != 0 {
        return Err(invalid("trailing bytes in login reply"));
    }
    Ok(reply)
}

/// One exchange with the explicitly selected endpoint; redirects are returned
/// to the caller. All socket operations have a caller-selected timeout.
pub fn exchange(
    address: SocketAddr,
    username: &str,
    password_hash: &[u8; 20],
    timeout: Duration,
) -> io::Result<LoginReply> {
    let request = Zeroizing::new(encode_login(username, password_hash)?);
    let mut socket = TcpStream::connect_timeout(&address, timeout)?;
    socket.set_read_timeout(Some(timeout))?;
    socket.set_write_timeout(Some(timeout))?;
    write_frame(&mut socket, &request)?;
    decode_reply(&read_frame(&mut socket, 1024 * 1024)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{net::TcpListener, thread};

    fn hex(text: &str) -> Vec<u8> {
        text.trim()
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect()
    }

    #[test]
    fn python_generated_roster_fixture() {
        let bytes = hex(include_str!("../tests/fixtures/login-ok.hex"));
        let LoginReply::Accepted {
            status,
            active_unit,
            slots,
            trailer,
        } = decode_reply(&bytes).unwrap()
        else {
            panic!("expected roster")
        };
        assert_eq!((status, active_unit, trailer), (1, 42, 5));
        assert_eq!(slots[0].name.as_deref(), Some("Explorer 🚀"));
        assert_eq!(slots[0].dna, std::array::from_fn(|i| i as u8));
        assert!(slots[0].left_handed);
        assert!(
            slots[1..]
                .iter()
                .all(|slot| slot.id == 0 && slot.dna == [0; 24])
        );
        for end in 0..bytes.len() {
            assert!(decode_reply(&bytes[..end]).is_err());
        }
        let mut extended = bytes;
        extended.push(0);
        assert!(decode_reply(&extended).is_err());
    }

    #[test]
    fn python_generated_failure_and_redirect() {
        assert_eq!(
            decode_reply(&hex("0b0200000000")).unwrap(),
            LoginReply::Rejected {
                reason: 2,
                detail: 0
            }
        );
        assert_eq!(
            decode_reply(&hex("0200000012003100320037002e0030002e0030002e00314177")).unwrap(),
            LoginReply::Redirect {
                host: "127.0.0.1".into(),
                port: 16759
            }
        );
        assert_eq!(decode_reply(&[9]).unwrap(), LoginReply::VersionMismatch);
        assert!(encode_login("🚀", &[0; 20]).is_err());
    }

    #[test]
    fn fragmented_loopback_exchange() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            use std::io::Write;
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let request = read_frame(&mut socket, 1024).unwrap();
            let mut input = QtReader::new(&request);
            assert_eq!(input.read_u8().unwrap(), 3);
            assert_eq!(input.read_string().unwrap().as_deref(), Some(VERSION));
            assert_eq!(
                au_crypt(input.read_bytes().unwrap().unwrap(), net_crypt_key(0x101)),
                b"Explorer"
            );
            assert_eq!(input.read_bytes().unwrap().unwrap(), &[0xab; 20]);
            assert_eq!(input.remaining(), 0);
            for byte in hex("060b0200000000") {
                socket.write_all(&[byte]).unwrap();
            }
        });
        let reply = exchange(address, "Explorer", &[0xab; 20], Duration::from_secs(5)).unwrap();
        server.join().unwrap();
        assert_eq!(
            reply,
            LoginReply::Rejected {
                reason: 2,
                detail: 0
            }
        );
    }
}
