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
mod worker;
use worker::worker;

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
    Message(crate::chat::Outgoing),
    Choice(crate::chat::Reply),
    Hail,
    StoryControl(crate::chat::StoryControl),
    Pickup(u32),
    Disconnect,
}
pub enum Event {
    StoryControlSent(crate::chat::StoryControl),
    ChoiceSent(crate::chat::Reply),
    IncomingChat(crate::chat::Incoming),
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

#[cfg(test)]
mod tests;
