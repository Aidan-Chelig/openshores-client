//! Renderer-independent, loopback-only client worker with bounded queues.
use crate::{
    commands::{ChatCommand, SceneCommand},
    local_scene::{Bootstrap, SceneRead, Session},
    login::{self, Avatar, LoginReply},
};
use std::{
    collections::BTreeSet,
    io,
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender, TryRecvError},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use zeroize::Zeroizing;

#[derive(Clone, Copy)]
pub struct LocalEndpoint {
    pub login: SocketAddr,
    pub scene_port: u16,
    pub chat_port: u16,
}
impl LocalEndpoint {
    pub fn validate(self) -> io::Result<Self> {
        if !self.login.ip().is_loopback()
            || self.login.port() == 0
            || self.scene_port == 0
            || self.chat_port == 0
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "client slice requires nonzero loopback endpoints",
            ));
        }
        Ok(self)
    }
}

pub enum Request {
    Login {
        username: String,
        password: Zeroizing<String>,
    },
    Join(u32),
    Scene(SceneCommand),
    Chat(ChatCommand),
    Pickup(u32),
    Disconnect,
}
pub enum Event {
    Roster(Box<[Avatar; 6]>),
    Bootstrap(Box<Bootstrap>),
    Packet(Vec<u8>),
    SentScene(SceneCommand),
    SentChat(ChatCommand),
    Status(String),
    Error(String),
    Disconnected,
}

pub struct ClientRuntime {
    requests: Option<SyncSender<Request>>,
    events: Option<Receiver<Event>>,
    stopped: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}
impl ClientRuntime {
    pub fn start(endpoint: LocalEndpoint) -> io::Result<Self> {
        let endpoint = endpoint.validate()?;
        let (requests, incoming) = mpsc::sync_channel(32);
        let (outgoing, events) = mpsc::sync_channel(32);
        let stopped = Arc::new(AtomicBool::new(false));
        let worker_stop = stopped.clone();
        let worker = thread::Builder::new()
            .name("openshores-local-client".into())
            .spawn(move || worker(endpoint, incoming, outgoing, worker_stop))?;
        Ok(Self {
            requests: Some(requests),
            events: Some(events),
            stopped,
            worker: Some(worker),
        })
    }
    pub fn send(&self, request: Request) -> io::Result<()> {
        self.requests
            .as_ref()
            .ok_or_else(|| io::Error::other("client stopped"))?
            .try_send(request)
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "client request queue full or disconnected",
                )
            })
    }
    pub fn next_event(&self) -> Result<Event, TryRecvError> {
        self.events
            .as_ref()
            .ok_or(TryRecvError::Disconnected)?
            .try_recv()
    }
}
impl Drop for ClientRuntime {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::Release);
        self.requests.take();
        // Unblock a worker awaiting space in the bounded output queue.
        self.events.take();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn worker(
    endpoint: LocalEndpoint,
    incoming: Receiver<Request>,
    outgoing: SyncSender<Event>,
    stopped: Arc<AtomicBool>,
) {
    let mut avatars = BTreeSet::new();
    let mut session: Option<Session> = None;
    let mut pins: Vec<(Instant, u32)> = Vec::new();
    let mut connected_at = None;
    while !stopped.load(Ordering::Acquire) {
        let request = if session.is_some() {
            match incoming.try_recv() {
                Ok(request) => Some(request),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => break,
            }
        } else {
            match incoming.recv_timeout(Duration::from_millis(20)) {
                Ok(request) => Some(request),
                Err(mpsc::RecvTimeoutError::Timeout) => None,
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        };
        if let Some(request) = request {
            let result: io::Result<Option<Event>> = (|| match request {
                Request::Login { username, password } => {
                    if session.is_some() {
                        return Err(io::Error::other("disconnect before logging in again"));
                    }
                    avatars.clear();
                    let hash = Zeroizing::new(login::wire_password(&password));
                    match login::exchange(endpoint.login, &username, &hash, Duration::from_secs(5))?
                    {
                        LoginReply::Accepted { slots, .. } => {
                            avatars.extend(
                                slots.iter().filter(|slot| slot.id != 0).map(|slot| slot.id),
                            );
                            Ok(Some(Event::Roster(slots)))
                        }
                        _ => Err(io::Error::other(
                            "local login rejected or redirected; no automatic redirect followed",
                        )),
                    }
                }
                Request::Join(id) => {
                    if session.is_some() || !avatars.contains(&id) {
                        return Err(io::Error::other(
                            "select an available avatar from the authenticated roster",
                        ));
                    }
                    let opened = Session::open(
                        SocketAddr::new(endpoint.login.ip(), endpoint.scene_port),
                        id,
                        Duration::from_secs(30),
                        Some(endpoint.chat_port),
                    )?;
                    // Keep the full bounded inbound capture in its session owner;
                    // the UI receives an independent bootstrap for decoding.
                    let packets = opened.bootstrap.packets.clone();
                    let bootstrap = Bootstrap {
                        avatar_id: id,
                        initialization: opened.bootstrap.initialization.clone(),
                        manifest: opened.bootstrap.manifest.clone(),
                        time_low: opened.bootstrap.time_low,
                        packet_counts: opened.bootstrap.packet_counts.clone(),
                        bytes_received: opened.bootstrap.bytes_received,
                        packets,
                    };
                    session = Some(opened);
                    connected_at = Some(Instant::now());
                    Ok(Some(Event::Bootstrap(Box::new(bootstrap))))
                }
                Request::Scene(command) => {
                    session
                        .as_mut()
                        .ok_or_else(|| io::Error::other("not connected"))?
                        .send_command(command, Duration::from_secs(1))?;
                    Ok(Some(Event::SentScene(command)))
                }
                Request::Chat(command) => {
                    session
                        .as_mut()
                        .ok_or_else(|| io::Error::other("not connected"))?
                        .send_chat_command(command, Duration::from_secs(1))?;
                    Ok(Some(Event::SentChat(command)))
                }
                Request::Pickup(target) => {
                    if session.is_none() || target == 0 || !pins.is_empty() {
                        return Err(io::Error::other(
                            "pickup requires a nonzero target, connected scene and no pending pickup",
                        ));
                    }
                    let now = Instant::now();
                    pins = [0, 500, 1000, 1500, 2000]
                        .into_iter()
                        .map(|ms| {
                            (
                                now + Duration::from_millis(ms),
                                if ms == 2000 { 0 } else { target },
                            )
                        })
                        .collect();
                    Ok(Some(Event::Status(
                        "Pickup hold requested; waiting for server inventory/manifest".into(),
                    )))
                }
                Request::Disconnect => {
                    session = None;
                    pins.clear();
                    avatars.clear();
                    connected_at = None;
                    Ok(Some(Event::Disconnected))
                }
            })();
            let event = match result {
                Ok(event) => event,
                Err(error) => Some(Event::Error(error.to_string())),
            };
            if let Some(event) = event
                && outgoing.send(event).is_err()
            {
                break;
            }
        }
        if let Some(active) = session.as_mut() {
            let result: io::Result<SceneRead> = (|| {
                if connected_at
                    .is_some_and(|start: Instant| start.elapsed() >= Duration::from_secs(60))
                {
                    return Ok(SceneRead::Closed);
                }
                if let Some((when, target)) = pins.first().copied()
                    && Instant::now() >= when
                {
                    active
                        .send_command(SceneCommand::TargetPin { target }, Duration::from_secs(1))?;
                    pins.remove(0);
                }
                active.receive_update(Duration::from_millis(20))
            })();
            match result {
                Ok(SceneRead::Packet(packet)) => {
                    if outgoing.send(Event::Packet(packet)).is_err() {
                        break;
                    }
                }
                Ok(SceneRead::Idle) => {}
                other => {
                    session = None;
                    pins.clear();
                    avatars.clear();
                    connected_at = None;
                    let event = match other {
                        Err(error) => Event::Error(error.to_string()),
                        _ => Event::Status(
                            "Local slice session ended (60-second bound or server close)".into(),
                        ),
                    };
                    if outgoing.send(event).is_err() || outgoing.send(Event::Disconnected).is_err()
                    {
                        break;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn external_and_zero_endpoints_fail_before_worker_creation() {
        for endpoint in [
            LocalEndpoint {
                login: "192.0.2.1:16757".parse().unwrap(),
                scene_port: 16759,
                chat_port: 16758,
            },
            LocalEndpoint {
                login: "127.0.0.1:0".parse().unwrap(),
                scene_port: 16759,
                chat_port: 16758,
            },
        ] {
            assert!(ClientRuntime::start(endpoint).is_err());
        }
    }
    #[test]
    fn unauthenticated_join_is_rejected_without_scene_connect() {
        let runtime = ClientRuntime::start(LocalEndpoint {
            login: "127.0.0.1:1".parse().unwrap(),
            scene_port: 1,
            chat_port: 1,
        })
        .unwrap();
        runtime.send(Request::Join(42)).unwrap();
        let deadline = Instant::now() + Duration::from_secs(1);
        loop {
            match runtime.next_event() {
                Ok(Event::Error(message)) => {
                    assert!(message.contains("authenticated roster"));
                    break;
                }
                Err(TryRecvError::Empty) => {
                    assert!(Instant::now() < deadline);
                    thread::yield_now();
                }
                _ => panic!("unexpected event"),
            }
        }
    }
}
