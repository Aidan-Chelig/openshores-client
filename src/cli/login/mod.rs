use super::*;
mod credentials;
mod options;
pub(super) use credentials::password_from_stdin;

pub(super) fn login_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let options::LoginOptions {
        username,
        address,
        from_stdin,
        avatar_id,
        scene_port,
        chat_port,
        capture,
        watch_seconds,
        look_at,
        move_gear,
        drop_gear,
        use_gear,
        pickup_target,
        position_report,
        pose_report,
    } = options::parse(args)?;
    let password = if from_stdin {
        if io::stdin().is_terminal() {
            return Err("use the hidden password prompt for terminal input".into());
        }
        password_from_stdin(&mut io::stdin().lock())?
    } else {
        Zeroizing::new(rpassword::prompt_password("Password: ")?)
    };
    let digest = Zeroizing::new(wire_login::wire_password(&password));
    drop(password);
    match wire_login::exchange(address, username, &digest, Duration::from_secs(10))? {
        LoginReply::Accepted {
            status,
            active_unit,
            slots,
            trailer,
        } => {
            println!(
                "Login accepted by {address} (status={status}, active unit={active_unit:#010x}, trailer={trailer})."
            );
            let mut found = false;
            for (index, avatar) in slots
                .iter()
                .enumerate()
                .filter(|(_, avatar)| avatar.id != 0)
            {
                found = true;
                println!(
                    "Slot {}: {:?} ({:#010x}, sex={}, left-handed={})",
                    index + 1,
                    avatar.name.as_deref().unwrap_or(""),
                    avatar.id,
                    avatar.sex,
                    avatar.left_handed
                );
            }
            if !found {
                println!("No avatars in the roster.");
            }
            if let Some(id) = avatar_id {
                if !slots.iter().any(|avatar| avatar.id == id) {
                    return Err("selected avatar is absent from the authenticated roster".into());
                }
                let mut session = openshores_client::local_scene::Session::open(
                    SocketAddr::new(address.ip(), scene_port.unwrap_or(16759)),
                    id,
                    Duration::from_secs(30),
                    chat_port,
                )?;
                for command in [look_at, move_gear].into_iter().flatten() {
                    session.send_command(command, Duration::from_secs(5))?;
                    println!("Sent local scene command: {command:?} (not an acknowledgement).");
                }
                if let Some(command) = drop_gear {
                    session.send_chat_command(command, Duration::from_secs(5))?;
                    println!("Sent local chat command: {command:?} (not an acknowledgement).");
                }
                let (world, state) = if let Some(seconds) = watch_seconds {
                    let watched = if let Some(target) = pickup_target {
                        use openshores_client::{
                            commands::SceneCommand, local_scene::ScheduledCommand,
                        };
                        let schedule: Vec<_> = [0, 500, 1000, 1500, 2000]
                            .into_iter()
                            .map(|milliseconds| ScheduledCommand {
                                after: Duration::from_millis(milliseconds),
                                command: SceneCommand::TargetPin {
                                    target: if milliseconds == 2000 { 0 } else { target },
                                },
                            })
                            .collect();
                        println!(
                            "Local pickup target-pin test profile: {target:#010x}; four pins at 500 ms intervals, then clear (not an acknowledgement)."
                        );
                        session.watch_with_commands(Duration::from_secs(seconds), &schedule)?
                    } else if let Some(command) = position_report.or(pose_report) {
                        use openshores_client::local_scene::ScheduledCommand;
                        let schedule = [
                            ScheduledCommand {
                                after: Duration::ZERO,
                                command,
                            },
                            ScheduledCommand {
                                after: Duration::from_millis(500),
                                command,
                            },
                        ];
                        println!(
                            "Local raw-report test profile: {command:?}; two reports 500 ms apart (not an acknowledgement)."
                        );
                        session.watch_with_commands(Duration::from_secs(seconds), &schedule)?
                    } else if let Some(command) = use_gear {
                        use openshores_client::local_scene::ScheduledChatCommand;
                        let schedule = [ScheduledChatCommand {
                            after: Duration::from_secs(1),
                            command,
                        }];
                        println!(
                            "Local item-use test profile: {command:?}; one request after 1 second (not an acknowledgement)."
                        );
                        session.watch_with_chat_commands(Duration::from_secs(seconds), &schedule)?
                    } else {
                        session.watch(Duration::from_secs(seconds))?
                    };
                    println!(
                        "Local scene watch ended: {:?}; {} post-bootstrap packets applied.",
                        watched.end, watched.updates_received
                    );
                    (watched.capture, watched.state)
                } else {
                    let world = session.bootstrap;
                    let mut state = openshores_client::world::WorldState::default();
                    for packet in &world.packets {
                        state.ingest(packet)?;
                    }
                    (world, state)
                };
                println!(
                    "Scene initialized for {:#010x}: {} manifest atoms, {} bytes received.",
                    world.avatar_id,
                    world.manifest.len(),
                    world.bytes_received
                );
                println!(
                    "Message of the day: {:?}",
                    world.initialization.motd.as_deref().unwrap_or("")
                );
                println!("Scene packet counts: {:?}", world.packet_counts);
                print_world(&state)?;
                if (move_gear.is_some()
                    || drop_gear.is_some()
                    || use_gear.is_some()
                    || pickup_target.is_some())
                    && let Some(gear) = state
                        .entities
                        .get(&id)
                        .and_then(|entity| entity.person.as_ref())
                        .and_then(|person| person.gear.as_ref())
                {
                    let observed: Vec<_> = gear
                        .slots
                        .iter()
                        .map(|slot| {
                            (
                                slot.slot_type,
                                slot.subindex,
                                slot.item
                                    .as_ref()
                                    .map(|item| (item.type_id, item.common.commodity_id)),
                            )
                        })
                        .collect();
                    println!("Observed player gear (latest received batch): {observed:?}");
                }
                if let Some(path) = capture {
                    let mut file = fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(path)?;
                    world.write_capture(&mut file)?;
                    println!(
                        "Saved {} inbound scene packets to {path:?}.",
                        world.packets.len()
                    );
                }
            }
            Ok(())
        }
        LoginReply::Rejected { reason, detail } => {
            let message = match reason {
                1 => "account not found",
                2 => "incorrect password",
                _ => "server rejected login",
            };
            Err(format!("{message} (reason={reason}, detail={detail})").into())
        }
        LoginReply::Redirect { host, port } => Err(format!(
            "server requested redirect to {host:?}:{port}; reconnect explicitly to that endpoint"
        )
        .into()),
        LoginReply::VersionMismatch => Err("server rejected the client protocol version".into()),
    }
}
