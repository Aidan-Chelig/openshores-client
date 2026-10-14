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
    received: u64,
    continuation_deadline: Option<Instant>,
}

fn time_left(deadline: Instant) -> io::Result<Duration> {
    deadline
        .checked_duration_since(Instant::now())
        .filter(|d| !d.is_zero())
        .ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "scene I/O deadline exceeded"))
}

impl Read for DeadlineSocket {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.socket
            .set_read_timeout(Some(time_left(self.deadline)?))?;
        let count = self.socket.read(buffer)?;
        self.received += count as u64;
        if count > 0
            && let Some(deadline) = self.continuation_deadline.take()
        {
            self.deadline = deadline;
        }
        Ok(count)
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
    Ok(DeadlineSocket {
        socket,
        deadline,
        received: 0,
        continuation_deadline: None,
    })
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
    Ok(Session::open(address, avatar_id, timeout, chat_port_override)?.bootstrap)
}

/// Owns all three authenticated scene channels beyond the initial manifest.
/// No gameplay requests or inferred heartbeat replies are sent.
pub struct Session {
    pub bootstrap: Bootstrap,
    world: DeadlineSocket,
    _initial: DeadlineSocket,
    _chat: Option<DeadlineSocket>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WatchEnd {
    DurationElapsed,
    ServerClosed,
}

pub struct WatchedScene {
    pub capture: Bootstrap,
    pub state: crate::world::WorldState,
    pub updates_received: usize,
    pub end: WatchEnd,
}

impl Session {
    /// Receive and apply updates for a bounded duration, preserving the complete
    /// capture. Timeout/EOF mid-frame is an error, never a successful idle stop.
    pub fn watch(mut self, duration: Duration) -> io::Result<WatchedScene> {
        if duration.is_zero() || duration > Duration::from_secs(60) {
            return Err(invalid(
                "scene watch duration must be positive and at most 60 seconds",
            ));
        }
        let deadline = Instant::now()
            .checked_add(duration)
            .ok_or_else(|| invalid("invalid watch deadline"))?;
        let mut state = crate::world::WorldState::default();
        for packet in &self.bootstrap.packets {
            state.ingest(packet)?;
        }
        let mut updates_received = 0;
        let end = loop {
            self.world.deadline = deadline;
            // A frame begun before the watch boundary gets one fixed, bounded
            // completion grace. Slow peers cannot renew this per byte.
            self.world.continuation_deadline = deadline.checked_add(Duration::from_millis(100));
            let before = self.world.received;
            let payload = match read_frame(&mut self.world, 16 * 1024 * 1024) {
                Ok(payload) => payload,
                Err(error) if self.world.received == before => match error.kind() {
                    io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock => {
                        break WatchEnd::DurationElapsed;
                    }
                    io::ErrorKind::UnexpectedEof => break WatchEnd::ServerClosed,
                    _ => return Err(error),
                },
                Err(error) => return Err(error),
            };
            if self.bootstrap.packets.len() >= 20_000 {
                return Err(invalid("scene session exceeds packet budget"));
            }
            let bytes = self
                .bootstrap
                .bytes_received
                .checked_add(payload.len())
                .filter(|n| *n <= 128 * 1024 * 1024)
                .ok_or_else(|| invalid("scene session exceeds byte budget"))?;
            let opcode = *payload
                .first()
                .ok_or_else(|| invalid("empty scene packet"))?;
            if opcode == 0x2a && scene::decode_active_unit(&payload)? != self.bootstrap.avatar_id {
                return Err(invalid("scene session changed the selected avatar"));
            }
            state.ingest(&payload)?;
            self.bootstrap.bytes_received = bytes;
            *self.bootstrap.packet_counts.entry(opcode).or_insert(0) += 1;
            self.bootstrap.packets.push(payload);
            updates_received += 1;
        };
        Ok(WatchedScene {
            capture: self.bootstrap,
            state,
            updates_received,
            end,
        })
    }

    pub fn open(
        address: SocketAddr,
        avatar_id: u32,
        timeout: Duration,
        chat_port_override: Option<u16>,
    ) -> io::Result<Self> {
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
                        return Ok(Self {
                            world,
                            _initial: initial,
                            _chat: chat,
                            bootstrap: Bootstrap {
                                avatar_id,
                                initialization,
                                manifest: atom_ids,
                                time_low,
                                packet_counts,
                                bytes_received,
                                packets,
                            },
                        });
                    }
                }
                // In world traffic 0x22 is building data, never another redirect.
                _ => {}
            }
        }
        Err(invalid("scene bootstrap exceeds packet budget"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn socket_pair() -> (TcpStream, TcpStream) {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        (client, listener.accept().unwrap().0)
    }

    fn watch_fixture() -> (Session, TcpStream, TcpStream, TcpStream) {
        let (world, server) = socket_pair();
        let (initial, initial_peer) = socket_pair();
        let (chat, chat_peer) = socket_pair();
        let deadline = Instant::now() + Duration::from_secs(1);
        let bootstrap = Bootstrap {
            avatar_id: 42,
            initialization: InitSucceeded {
                motd: None,
                chat_port: 1,
                autime_usec: 0,
            },
            manifest: vec![],
            time_low: 0,
            packet_counts: BTreeMap::new(),
            bytes_received: 0,
            packets: vec![],
        };
        let wrap = |socket| DeadlineSocket {
            socket,
            deadline,
            received: 0,
            continuation_deadline: None,
        };
        (
            Session {
                bootstrap,
                world: wrap(world),
                _initial: wrap(initial),
                _chat: Some(wrap(chat)),
            },
            server,
            initial_peer,
            chat_peer,
        )
    }

    #[test]
    fn watch_applies_updates_captures_and_retains_channels() {
        let (session, mut server, mut initial, mut chat) = watch_fixture();
        let mut player = vec![0x12];
        player.extend(42u32.to_be_bytes());
        player.extend(1000i64.to_be_bytes());
        player.extend([0, 0, 4, 0, 46, 90, 0, 8, 33]);
        // Complete frames delivered in one write; close is a clean session end.
        write_frame(&mut server, &player).unwrap();
        write_frame(&mut server, &[0xfe, 7]).unwrap();
        server.shutdown(std::net::Shutdown::Write).unwrap();
        let result = session.watch(Duration::from_secs(1)).unwrap();
        assert_eq!(result.end, WatchEnd::ServerClosed);
        assert_eq!(result.updates_received, 2);
        assert_eq!(
            result.state.entities[&42].person.as_ref().unwrap().hp,
            Some(46)
        );
        assert_eq!(result.capture.packets, vec![player, vec![0xfe, 7]]);
        assert_eq!(result.state.opaque_packet_counts[&0xfe], 1);
        let mut capture = Vec::new();
        result.capture.write_capture(&mut capture).unwrap();
        let replay = crate::world::read_capture(&mut io::Cursor::new(capture)).unwrap();
        assert_eq!(replay.entities, result.state.entities);
        // Owning session drops every channel on completion.
        let mut byte = [0];
        assert_eq!(initial.read(&mut byte).unwrap(), 0);
        assert_eq!(chat.read(&mut byte).unwrap(), 0);
    }

    #[test]
    fn watch_distinguishes_idle_deadline_from_partial_frames() {
        let (session, _server, _initial, _chat) = watch_fixture();
        assert_eq!(
            session.watch(Duration::from_millis(20)).unwrap().end,
            WatchEnd::DurationElapsed
        );
        for incomplete in [vec![0x40], vec![4, 0x12]] {
            let (session, mut server, _initial, _chat) = watch_fixture();
            server.write_all(&incomplete).unwrap();
            assert!(session.watch(Duration::from_millis(20)).is_err());
        }
        let (session, mut server, _initial, _chat) = watch_fixture();
        server.write_all(&[4, 0x12]).unwrap();
        server.shutdown(std::net::Shutdown::Write).unwrap();
        assert!(session.watch(Duration::from_secs(1)).is_err());
        let (session, _server, _initial, _chat) = watch_fixture();
        assert!(session.watch(Duration::ZERO).is_err());
        let (session, _server, _initial, _chat) = watch_fixture();
        assert!(session.watch(Duration::from_secs(61)).is_err());
    }

    #[test]
    fn frame_started_before_boundary_has_bounded_completion_grace() {
        let (session, mut server, _initial, _chat) = watch_fixture();
        server.write_all(&[2]).unwrap();
        let sender = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(60));
            server.write_all(&[0xfe, 7]).unwrap();
        });
        let result = session.watch(Duration::from_millis(20)).unwrap();
        sender.join().unwrap();
        assert_eq!(result.updates_received, 1);
        assert_eq!(result.end, WatchEnd::DurationElapsed);
        assert_eq!(result.capture.packets, vec![vec![0xfe, 7]]);
    }

    #[test]
    fn watch_rejects_avatar_changes_and_capture_budget_overruns() {
        let (session, mut server, _initial, _chat) = watch_fixture();
        let mut active = vec![0x2a];
        active.extend(43u32.to_be_bytes());
        write_frame(&mut server, &active).unwrap();
        assert!(session.watch(Duration::from_secs(1)).is_err());
        let (mut session, mut server, _initial, _chat) = watch_fixture();
        session.bootstrap.bytes_received = 128 * 1024 * 1024;
        write_frame(&mut server, &[0xfe]).unwrap();
        assert!(session.watch(Duration::from_secs(1)).is_err());
        let (mut session, mut server, _initial, _chat) = watch_fixture();
        session.bootstrap.packets = vec![vec![0xfe]; 20_000];
        write_frame(&mut server, &[0xfe]).unwrap();
        assert!(session.watch(Duration::from_secs(1)).is_err());
    }

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
