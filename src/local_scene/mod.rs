//! Bounded, loopback-only scene sessions for the supplied development server.
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
/// Opening/watching sends no gameplay requests; explicit typed commands are
/// separate. No inferred heartbeat replies are sent.
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

#[derive(Clone, Copy, Debug)]
pub struct ScheduledCommand {
    pub after: Duration,
    pub command: crate::commands::SceneCommand,
}

#[derive(Clone, Copy, Debug)]
pub struct ScheduledChatCommand {
    pub after: Duration,
    pub command: crate::commands::ChatCommand,
}

pub enum SceneRead {
    Packet(Vec<u8>),
    Idle,
    Closed,
}

#[derive(Clone, Copy)]
enum OutboundCommand {
    Scene(crate::commands::SceneCommand),
    Chat(crate::commands::ChatCommand),
}

struct QueuedCommand {
    after: Duration,
    command: OutboundCommand,
}

impl Session {
    /// Incremental bounded receive for a UI/network worker. Never treats a
    /// partial frame as idle or EOF success. Captures only incoming packets.
    pub fn receive_update(&mut self, timeout: Duration) -> io::Result<SceneRead> {
        if timeout.is_zero() || timeout > Duration::from_secs(1) {
            return Err(invalid(
                "receive timeout must be positive and at most one second",
            ));
        }
        let deadline = Instant::now() + timeout;
        self.world.deadline = deadline;
        self.world.continuation_deadline = deadline.checked_add(Duration::from_millis(100));
        let before = self.world.received;
        let payload = match read_frame(&mut self.world, 16 * 1024 * 1024) {
            Ok(packet) => packet,
            Err(error) if self.world.received == before => {
                return match error.kind() {
                    io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock => Ok(SceneRead::Idle),
                    io::ErrorKind::UnexpectedEof => Ok(SceneRead::Closed),
                    _ => Err(error),
                };
            }
            Err(error) => return Err(error),
        };
        if self.bootstrap.packet_counts.values().sum::<usize>() >= 20_000 {
            return Err(invalid("scene session exceeds packet budget"));
        }
        let bytes = self
            .bootstrap
            .bytes_received
            .checked_add(payload.len())
            .filter(|bytes| *bytes <= 128 * 1024 * 1024)
            .ok_or_else(|| invalid("scene session exceeds byte budget"))?;
        let opcode = *payload
            .first()
            .ok_or_else(|| invalid("empty scene packet"))?;
        if opcode == 0x2a && scene::decode_active_unit(&payload)? != self.bootstrap.avatar_id {
            return Err(invalid("scene session changed the selected avatar"));
        }
        self.bootstrap.bytes_received = bytes;
        *self.bootstrap.packet_counts.entry(opcode).or_default() += 1;
        self.bootstrap.packets.push(payload.clone());
        Ok(SceneRead::Packet(payload))
    }
    /// Inventory drop requests belong to the authenticated chat channel.
    /// A completed write is not an acknowledgement or optimistic state change.
    pub fn send_chat_command(
        &mut self,
        command: crate::commands::ChatCommand,
        timeout: Duration,
    ) -> io::Result<()> {
        let payload = command.encode()?;
        if timeout.is_zero() || timeout > Duration::from_secs(10) {
            return Err(invalid(
                "command write timeout must be positive and at most 10 seconds",
            ));
        }
        let channel = self
            ._chat
            .as_mut()
            .ok_or_else(|| invalid("authenticated chat channel is unavailable"))?;
        channel.deadline = Instant::now()
            .checked_add(timeout)
            .ok_or_else(|| invalid("invalid command deadline"))?;
        write_frame(channel, &payload)
    }
    /// Explicit commands use the authenticated world channel, never redirect/chat.
    /// Success means transport write completed, not server acceptance.
    pub fn send_command(
        &mut self,
        command: crate::commands::SceneCommand,
        timeout: Duration,
    ) -> io::Result<()> {
        let payload = command.encode()?;
        if timeout.is_zero() || timeout > Duration::from_secs(10) {
            return Err(invalid(
                "command write timeout must be positive and at most 10 seconds",
            ));
        }
        self.world.deadline = Instant::now()
            .checked_add(timeout)
            .ok_or_else(|| invalid("invalid command deadline"))?;
        self.world.continuation_deadline = None;
        write_frame(&mut self.world, &payload)
    }
    /// Receive and apply updates for a bounded duration, preserving the complete
    /// capture. Timeout/EOF mid-frame is an error, never a successful idle stop.
    pub fn watch(self, duration: Duration) -> io::Result<WatchedScene> {
        self.watch_with_commands(duration, &[])
    }

    /// Bounded same-channel commands interleaved with incoming state updates.
    pub fn watch_with_commands(
        self,
        duration: Duration,
        commands: &[ScheduledCommand],
    ) -> io::Result<WatchedScene> {
        if commands.len() > 16 {
            return Err(invalid("scheduled command budget exceeded"));
        }
        let queue: Vec<_> = commands
            .iter()
            .map(|scheduled| QueuedCommand {
                after: scheduled.after,
                command: OutboundCommand::Scene(scheduled.command),
            })
            .collect();
        self.watch_scheduled(duration, &queue)
    }

    /// Same-channel chat writes while receiving the corresponding scene updates.
    /// A delay is a test profile, not a server-readiness acknowledgement.
    pub fn watch_with_chat_commands(
        self,
        duration: Duration,
        commands: &[ScheduledChatCommand],
    ) -> io::Result<WatchedScene> {
        if commands.len() > 16 {
            return Err(invalid("scheduled command budget exceeded"));
        }
        let queue: Vec<_> = commands
            .iter()
            .map(|scheduled| QueuedCommand {
                after: scheduled.after,
                command: OutboundCommand::Chat(scheduled.command),
            })
            .collect();
        self.watch_scheduled(duration, &queue)
    }

    fn watch_scheduled(
        mut self,
        duration: Duration,
        commands: &[QueuedCommand],
    ) -> io::Result<WatchedScene> {
        if duration.is_zero() || duration > Duration::from_secs(60) {
            return Err(invalid(
                "scene watch duration must be positive and at most 60 seconds",
            ));
        }
        if commands.len() > 16 {
            return Err(invalid("scheduled command budget exceeded"));
        }
        let mut previous = Duration::ZERO;
        for scheduled in commands {
            match scheduled.command {
                OutboundCommand::Scene(command) => {
                    command.encode()?;
                }
                OutboundCommand::Chat(command) => {
                    command.encode()?;
                    if self._chat.is_none() {
                        return Err(invalid("authenticated chat channel is unavailable"));
                    }
                }
            }
            if scheduled.after < previous || scheduled.after >= duration {
                return Err(invalid(
                    "scheduled commands must be ordered and precede watch end",
                ));
            }
            previous = scheduled.after;
        }
        let started = Instant::now();
        let deadline = started
            .checked_add(duration)
            .ok_or_else(|| invalid("invalid watch deadline"))?;
        let mut state = crate::world::WorldState::default();
        for packet in &self.bootstrap.packets {
            state.ingest(packet)?;
        }
        let mut updates_received = 0;
        let mut next_command = 0;
        let end = 'watch: loop {
            if Instant::now() >= deadline {
                break WatchEnd::DurationElapsed;
            }
            while next_command < commands.len() && started.elapsed() >= commands[next_command].after
            {
                let Some(left) = deadline
                    .checked_duration_since(Instant::now())
                    .filter(|left| !left.is_zero())
                else {
                    break 'watch WatchEnd::DurationElapsed;
                };
                let timeout = left.min(Duration::from_secs(1));
                match commands[next_command].command {
                    OutboundCommand::Scene(command) => self.send_command(command, timeout)?,
                    OutboundCommand::Chat(command) => self.send_chat_command(command, timeout)?,
                }
                next_command += 1;
            }
            let wake = commands
                .get(next_command)
                .map(|command| started + command.after)
                .unwrap_or(deadline)
                .min(deadline);
            self.world.deadline = wake;
            // A frame begun before the watch boundary gets one fixed, bounded
            // completion grace. Slow peers cannot renew this per byte.
            self.world.continuation_deadline = wake.checked_add(Duration::from_millis(100));
            let before = self.world.received;
            let payload = match read_frame(&mut self.world, 16 * 1024 * 1024) {
                Ok(payload) => payload,
                Err(error) if self.world.received == before => match error.kind() {
                    io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock => {
                        if Instant::now() < deadline {
                            continue;
                        }
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
mod tests;
