use super::*;

pub(super) fn worker(
    endpoint: LocalEndpoint,
    incoming: Receiver<Request>,
    outgoing: SyncSender<Event>,
    stopped: Arc<AtomicBool>,
) {
    let mut avatars = BTreeSet::new();
    let mut avatar_names = std::collections::BTreeMap::new();
    let mut choices = std::collections::VecDeque::new();
    let mut story_active = false;
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
                    avatar_names.clear();
                    let hash = Zeroizing::new(login::wire_password(&password));
                    match login::exchange(endpoint.login, &username, &hash, Duration::from_secs(5))?
                    {
                        LoginReply::Accepted { slots, .. } => {
                            avatar_names.extend(
                                slots
                                    .iter()
                                    .filter(|slot| slot.id != 0)
                                    .map(|slot| (slot.id, slot.name.clone().unwrap_or_default())),
                            );
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
                Request::Message(message) => {
                    session
                        .as_mut()
                        .ok_or_else(|| io::Error::other("not connected"))?
                        .send_message(&message, Duration::from_secs(1))?;
                    Ok(Some(Event::Status(
                        "Chat written; awaiting server broadcast".into(),
                    )))
                }
                Request::Choice(reply) => {
                    if !choices.contains(&reply) {
                        return Err(io::Error::other("choice was not offered to this avatar"));
                    }
                    session
                        .as_mut()
                        .ok_or_else(|| io::Error::other("not connected"))?
                        .send_choice(reply, Duration::from_secs(1))?;
                    choices.retain(|choice: &crate::chat::Reply| {
                        choice.instance != reply.instance || choice.offered != reply.offered
                    });
                    Ok(Some(Event::ChoiceSent(reply)))
                }
                Request::StoryControl(control) => {
                    if !story_active {
                        return Err(io::Error::other("no story offered to this avatar"));
                    }
                    session
                        .as_mut()
                        .ok_or_else(|| io::Error::other("not connected"))?
                        .send_story_control(control, Duration::from_secs(1))?;
                    if control == crate::chat::StoryControl::Close {
                        story_active = false;
                        choices.retain(|choice: &crate::chat::Reply| choice.instance == 1);
                    }
                    Ok(Some(Event::StoryControlSent(control)))
                }
                Request::Hail => {
                    let active = session
                        .as_mut()
                        .ok_or_else(|| io::Error::other("not connected"))?;
                    let name = avatar_names
                        .get(&active.bootstrap.avatar_id)
                        .ok_or_else(|| {
                            io::Error::other("avatar missing from authenticated roster")
                        })?;
                    active.send_hail(name, Duration::from_secs(1))?;
                    Ok(Some(Event::Status(
                        "Voice hail written; awaiting nearby native reply".into(),
                    )))
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
                    story_active = false;
                    choices.clear();
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
                match active.receive_chat(Duration::from_millis(1))? {
                    SceneRead::Packet(packet) => {
                        let incoming = crate::chat::decode_incoming(&packet)?;
                        if let crate::chat::Incoming::Story(story) = &incoming
                            && story.header.target == active.bootstrap.avatar_id
                        {
                            story_active = true;
                        }
                        if let crate::chat::Incoming::Choice(choice) = &incoming
                            && choice.offered_to == active.bootstrap.avatar_id
                            && choice.header.target == active.bootstrap.avatar_id
                        {
                            if choice.narrated {
                                story_active = true;
                            }
                            choices.retain(|old| *old != choice.reply());
                            choices.push_back(choice.reply());
                            if choices.len() > 32 {
                                choices.pop_front();
                            }
                        }
                        outgoing
                            .send(Event::IncomingChat(incoming))
                            .map_err(|_| io::Error::other("client event receiver closed"))?;
                    }
                    SceneRead::Closed => return Ok(SceneRead::Closed),
                    SceneRead::Idle => {}
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
                    choices.clear();
                    story_active = false;
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
