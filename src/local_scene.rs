//! Bounded, loopback-only scene probe for the supplied development server.
use crate::{
    protocol::{QtWriter, VERSION, read_frame, write_frame},
    scene::{self, InitSucceeded, ResumeHello, SceneTime, WorldRedirect},
};
use std::{
    collections::BTreeMap,
    io::{self, Read, Write},
    net::{IpAddr, SocketAddr, TcpStream},
    time::{Duration, Instant},
};

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

struct DeadlineSocket {
    socket: TcpStream,
    deadline: Instant,
}

fn time_left(deadline: Instant) -> io::Result<Duration> {
    deadline
        .checked_duration_since(Instant::now())
        .filter(|d| !d.is_zero())
        .ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "scene bootstrap deadline exceeded"))
}

impl Read for DeadlineSocket {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.socket
            .set_read_timeout(Some(time_left(self.deadline)?))?;
        self.socket.read(buffer)
    }
}

impl Write for DeadlineSocket {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.socket
            .set_write_timeout(Some(time_left(self.deadline)?))?;
        self.socket.write(buffer)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.socket.flush()
    }
}

fn connect(address: SocketAddr, deadline: Instant) -> io::Result<DeadlineSocket> {
    if !address.ip().is_loopback() {
        return Err(invalid("local scene probe only accepts loopback endpoints"));
    }
    let socket = TcpStream::connect_timeout(&address, time_left(deadline)?)?;
    Ok(DeadlineSocket { socket, deadline })
}

fn redirect_address(redirect: &WorldRedirect, fallback: IpAddr) -> io::Result<SocketAddr> {
    let host = match redirect.server.as_deref() {
        None | Some("") => fallback,
        Some("localhost") => "127.0.0.1".parse().unwrap(),
        Some(value) => value
            .parse::<IpAddr>()
            .map_err(|_| invalid("local probe will not resolve a redirect hostname"))?,
    };
    if !host.is_loopback() || redirect.port == 0 {
        return Err(invalid(
            "local probe refused a non-loopback or zero-port redirect",
        ));
    }
    Ok(SocketAddr::new(host, redirect.port))
}

pub struct Bootstrap {
    pub avatar_id: u32,
    pub initialization: InitSucceeded,
    pub manifest: Vec<u32>,
    pub time_low: u32,
    pub packet_counts: BTreeMap<u8, usize>,
    pub bytes_received: usize,
    /// Exact inbound scene payloads; login credentials are never captured.
    pub packets: Vec<Vec<u8>>,
}

impl Bootstrap {
    pub fn write_capture(&self, writer: &mut impl Write) -> io::Result<()> {
        for packet in &self.packets {
            write_frame(writer, packet)?;
        }
        Ok(())
    }
}

/// Synthetic local test profile; platform constants remain unverified against
/// the original client. The reference server parses but does not enforce them.
fn resume(avatar_id: u32, session_hi: i32, session_lo: i32) -> io::Result<Vec<u8>> {
    ResumeHello {
        version: VERSION,
        session_hi,
        session_lo,
        avatar_id,
        os_kind: 0,
        windows_version: 0,
        flag40: 0x40,
        extra: 0,
    }
    .encode()
}

/// Exercise the initial redirect, resume, init ACK, and first scene manifest.
/// Unknown world packets are counted without guessing their payload formats.
pub fn probe(
    address: SocketAddr,
    avatar_id: u32,
    timeout: Duration,
    chat_port_override: Option<u16>,
) -> io::Result<Bootstrap> {
    if avatar_id == 0 {
        return Err(invalid("select a nonzero avatar from the login roster"));
    }
    let deadline = Instant::now()
        .checked_add(timeout)
        .ok_or_else(|| invalid("invalid probe timeout"))?;
    let mut initial = connect(address, deadline)?;
    write_frame(&mut initial, &resume(avatar_id, 0, 0)?)?;
    let redirect = scene::decode_world_redirect(&read_frame(&mut initial, 1024 * 1024)?)?;
    let destination = redirect_address(&redirect, address.ip())?;
    // Retain conn #0 during bootstrap: closing it can prune the server's per-IP
    // session state before the avatar is registered on conn #1.
    let mut world = connect(destination, deadline)?;
    write_frame(
        &mut world,
        &resume(avatar_id, redirect.account_hi, redirect.account_lo as i32)?,
    )?;
    let mut active = false;
    let mut initialization = None;
    let mut chat = None;
    let mut packet_counts = BTreeMap::new();
    let mut bytes_received = 0usize;
    let mut packets = Vec::new();
    for _ in 0..20_000 {
        let payload = read_frame(&mut world, 16 * 1024 * 1024)?;
        bytes_received = bytes_received
            .checked_add(payload.len())
            .ok_or_else(|| invalid("scene byte count overflow"))?;
        if bytes_received > 128 * 1024 * 1024 {
            return Err(invalid("scene bootstrap exceeds byte budget"));
        }
        let opcode = *payload
            .first()
            .ok_or_else(|| invalid("empty scene packet"))?;
        packets.push(payload);
        let payload = packets.last().unwrap();
        *packet_counts.entry(opcode).or_insert(0) += 1;
        match opcode {
            0x2a => {
                if scene::decode_active_unit(payload)? != avatar_id {
                    return Err(invalid("server initialized a different avatar"));
                }
                active = true;
            }
            0x38 => {
                if !active {
                    return Err(invalid(
                        "scene initialization arrived before active-unit confirmation",
                    ));
                }
                let init = scene::decode_init_succeeded(payload)?;
                let mut channel = connect(
                    SocketAddr::new(
                        destination.ip(),
                        chat_port_override.unwrap_or(init.chat_port),
                    ),
                    deadline,
                )?;
                let mut ack = QtWriter::default();
                ack.write_u8(0x0b);
                ack.write_u32(redirect.account_lo);
                ack.write_u32(redirect.account_hi as u32);
                ack.write_u32(avatar_id);
                write_frame(&mut channel, &ack.into_bytes())?;
                chat = Some(channel);
                initialization = Some(init);
            }
            0x18 => {
                if let SceneTime::Manifest { time_low, atom_ids } =
                    scene::decode_scene_time(payload)?
                    && active
                    && let Some(initialization) = initialization.take()
                {
                    if !atom_ids.contains(&avatar_id) {
                        return Err(invalid("scene manifest omits the selected avatar"));
                    }
                    // Keep all channels alive until the first complete manifest.
                    drop(chat);
                    return Ok(Bootstrap {
                        avatar_id,
                        initialization,
                        manifest: atom_ids,
                        time_low,
                        packet_counts,
                        bytes_received,
                        packets,
                    });
                }
            }
            // In world traffic 0x22 is building data, never another redirect.
            _ => {}
        }
    }
    Err(invalid("scene bootstrap exceeds packet budget"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_external_redirects_without_dns_or_connecting() {
        let mut redirect = WorldRedirect {
            server: Some("203.0.113.1".into()),
            port: 16759,
            world_state: 2,
            account_lo: 1,
            account_hi: 0,
        };
        assert!(redirect_address(&redirect, "127.0.0.1".parse().unwrap()).is_err());
        redirect.server = Some("openshores.net".into());
        assert!(redirect_address(&redirect, "127.0.0.1".parse().unwrap()).is_err());
        assert!(
            probe(
                "203.0.113.1:16759".parse().unwrap(),
                42,
                Duration::from_secs(1),
                None
            )
            .is_err()
        );
    }
}
