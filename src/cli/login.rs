use super::*;

pub(super) fn password_from_stdin(reader: &mut impl Read) -> io::Result<Zeroizing<String>> {
    let mut bytes = Zeroizing::new(Vec::new());
    reader.take(4097).read_to_end(&mut bytes)?;
    if bytes.len() > 4096 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "password input exceeds 4096 bytes",
        ));
    }
    if bytes.last() == Some(&b'\n') {
        bytes.pop();
        if bytes.last() == Some(&b'\r') {
            bytes.pop();
        }
    }
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "password input must be UTF-8"))?;
    if text.contains(['\r', '\n']) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "password input must contain one line",
        ));
    }
    Ok(Zeroizing::new(text.to_owned()))
}

pub(super) fn login_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let username = args
        .first()
        .filter(|arg| !arg.starts_with('-'))
        .ok_or("usage: login USERNAME [IP:PORT] [--password-stdin]")?;
    let mut address: SocketAddr = "127.0.0.1:16757".parse()?;
    let mut address_given = false;
    let mut from_stdin = false;
    let mut avatar_id = None;
    let mut scene_port = None;
    let mut chat_port = None;
    let mut capture = None;
    let mut watch_seconds = None;
    let mut look_at = None;
    let mut move_gear = None;
    let mut drop_gear = None;
    let mut use_gear = None;
    let mut pickup_target = None;
    let mut position_report = None;
    let mut pose_report = None;
    let mut options = args[1..].iter();
    while let Some(arg) = options.next() {
        if arg == "--password-stdin" && !from_stdin {
            from_stdin = true;
        } else if arg == "--watch-seconds" && watch_seconds.is_none() {
            let seconds = options
                .next()
                .ok_or("--watch-seconds requires 1..=60")?
                .parse::<u64>()?;
            if !(1..=60).contains(&seconds) {
                return Err("--watch-seconds must be in 1..=60".into());
            }
            watch_seconds = Some(seconds);
        } else if arg == "--pose-report" && pose_report.is_none() {
            let text = options
                .next()
                .ok_or("--pose-report requires exactly 20 hexadecimal digits")?;
            if text.len() != 20 || !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return Err("--pose-report requires exactly 20 hexadecimal digits".into());
            }
            let mut poses = [0; 10];
            for (index, pose) in poses.iter_mut().enumerate() {
                *pose = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16)?;
            }
            pose_report = Some(openshores_client::commands::SceneCommand::PoseReport { poses });
        } else if arg == "--position-report" && position_report.is_none() {
            let mut values = [0.0; 6];
            for value in &mut values {
                *value = options
                    .next()
                    .ok_or("--position-report requires X Y Z RX RY RZ")?
                    .parse::<f64>()?;
            }
            let command = openshores_client::commands::SceneCommand::PositionReport {
                projected: [values[0], values[1], values[2]],
                rotation: [values[3], values[4], values[5]],
            };
            command.encode()?;
            position_report = Some(command);
        } else if arg == "--look-at" && look_at.is_none() {
            let value = options
                .next()
                .ok_or("--look-at requires a numeric ID (zero clears)")?;
            let target = if let Some(hex) = value.strip_prefix("0x") {
                u32::from_str_radix(hex, 16)?
            } else {
                value.parse::<u32>()?
            };
            look_at = Some(openshores_client::commands::SceneCommand::LookAt { target });
        } else if arg == "--pickup-target" && pickup_target.is_none() {
            let value = options
                .next()
                .ok_or("--pickup-target requires a nonzero numeric item ID")?;
            let target = if let Some(hex) = value.strip_prefix("0x") {
                u32::from_str_radix(hex, 16)?
            } else {
                value.parse::<u32>()?
            };
            if target == 0 {
                return Err("--pickup-target requires a nonzero item ID".into());
            }
            pickup_target = Some(target);
        } else if (arg == "--drop-gear" && drop_gear.is_none())
            || (arg == "--use-gear" && use_gear.is_none())
        {
            let source_slot = options
                .next()
                .ok_or("--drop-gear/--use-gear requires SRC_SLOT SRC_SUB")?
                .parse::<u8>()?;
            let source_subindex = options
                .next()
                .ok_or("--drop-gear/--use-gear requires SRC_SLOT SRC_SUB")?
                .parse::<u8>()?;
            let command = if arg == "--use-gear" {
                openshores_client::commands::ChatCommand::UseGear {
                    source_slot,
                    source_subindex,
                }
            } else {
                openshores_client::commands::ChatCommand::DropGear {
                    source_slot,
                    source_subindex,
                }
            };
            command.encode()?;
            if arg == "--use-gear" {
                use_gear = Some(command);
            } else {
                drop_gear = Some(command);
            }
        } else if arg == "--move-gear" && move_gear.is_none() {
            let mut fields = [0u8; 3];
            for field in &mut fields {
                *field = options
                    .next()
                    .ok_or("--move-gear requires SRC_SLOT SRC_SUB DST_SLOT")?
                    .parse()?;
            }
            let command = openshores_client::commands::SceneCommand::MoveGear {
                source_slot: fields[0],
                source_subindex: fields[1],
                destination_slot: fields[2],
            };
            command.encode()?;
            move_gear = Some(command);
        } else if arg == "--avatar" && avatar_id.is_none() {
            let value = options.next().ok_or("--avatar requires a numeric ID")?;
            avatar_id = Some(if let Some(hex) = value.strip_prefix("0x") {
                u32::from_str_radix(hex, 16)?
            } else {
                value.parse::<u32>()?
            });
        } else if arg == "--capture" && capture.is_none() {
            capture = Some(options.next().ok_or("--capture requires a new file path")?);
        } else if arg == "--scene-port" && scene_port.is_none() {
            scene_port = Some(
                options
                    .next()
                    .ok_or("--scene-port requires a port")?
                    .parse::<u16>()?,
            );
        } else if arg == "--chat-port" && chat_port.is_none() {
            chat_port = Some(
                options
                    .next()
                    .ok_or("--chat-port requires a port")?
                    .parse::<u16>()?,
            );
        } else if !arg.starts_with('-') && !address_given {
            address = arg
                .parse()
                .map_err(|_| "login endpoint must be IP:PORT (IPv6: [IP]:PORT)")?;
            address_given = true;
        } else {
            return Err("usage: login USERNAME [IP:PORT] [--password-stdin]".into());
        }
    }
    if use_gear.is_some()
        && (drop_gear.is_some()
            || move_gear.is_some()
            || pickup_target.is_some()
            || position_report.is_some()
            || pose_report.is_some())
    {
        return Err(
            "--use-gear cannot be combined with drop/move/pickup/position/pose commands".into(),
        );
    }
    if use_gear.is_some() && watch_seconds.is_none_or(|seconds| seconds < 3) {
        return Err("--use-gear requires --watch-seconds 3..=60".into());
    }
    if position_report.is_some()
        && (pickup_target.is_some() || drop_gear.is_some() || move_gear.is_some())
    {
        return Err("--position-report cannot be combined with pickup or gear move/drop".into());
    }
    if pose_report.is_some()
        && (position_report.is_some()
            || pickup_target.is_some()
            || drop_gear.is_some()
            || move_gear.is_some())
    {
        return Err("--pose-report cannot be combined with position/pickup/gear commands".into());
    }
    if pickup_target.is_some() && (drop_gear.is_some() || move_gear.is_some()) {
        return Err("--pickup-target cannot be combined with gear move/drop".into());
    }
    if pickup_target.is_some() && watch_seconds.is_none_or(|seconds| seconds < 3) {
        return Err("--pickup-target requires --watch-seconds 3..=60".into());
    }
    if drop_gear.is_some() && move_gear.is_some() {
        return Err("--drop-gear and --move-gear cannot be combined: cross-channel execution order is not guaranteed".into());
    }
    if (look_at.is_some()
        || move_gear.is_some()
        || drop_gear.is_some()
        || use_gear.is_some()
        || pickup_target.is_some()
        || position_report.is_some()
        || pose_report.is_some())
        && (avatar_id.is_none() || watch_seconds.is_none())
    {
        return Err("gameplay commands require --avatar and --watch-seconds".into());
    }
    if avatar_id.is_some() && !address.ip().is_loopback() {
        return Err("scene probes are restricted to local development servers".into());
    }
    if avatar_id == Some(0) {
        return Err("--avatar must be nonzero".into());
    }
    if avatar_id.is_none()
        && (scene_port.is_some()
            || chat_port.is_some()
            || capture.is_some()
            || watch_seconds.is_some())
    {
        return Err(
            "--scene-port, --chat-port, --capture, and --watch-seconds require --avatar".into(),
        );
    }
    if scene_port == Some(0) || chat_port == Some(0) {
        return Err("ports must be nonzero".into());
    }
    // Check the username before requesting a password or opening a connection.
    wire_login::encode_login(username, &[0; 20])?;
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
