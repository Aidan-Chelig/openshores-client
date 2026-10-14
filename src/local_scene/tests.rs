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
fn story_controls_use_chat_without_touching_scene_state() {
    let (mut session, _server, _initial, mut chat) = watch_fixture();
    for control in [
        crate::chat::StoryControl::Close,
        crate::chat::StoryControl::Hint,
        crate::chat::StoryControl::Resend,
    ] {
        session
            .send_story_control(control, Duration::from_millis(20))
            .unwrap();
        assert_eq!(read_frame(&mut chat, 100).unwrap(), control.encode());
    }
    assert!(session.bootstrap.packets.is_empty());
}

#[test]
fn choice_reply_is_written_only_to_the_authenticated_chat_channel() {
    let (mut session, _server, _initial, mut chat) = watch_fixture();
    let reply = crate::chat::Reply {
        instance: 1,
        index: 3,
        offered: 700,
    };
    session
        .send_choice(reply, Duration::from_millis(20))
        .unwrap();
    assert_eq!(read_frame(&mut chat, 100).unwrap(), reply.encode().unwrap());
    assert!(session.bootstrap.packets.is_empty());
}

#[test]
fn chat_receive_never_enters_scene_capture_and_partial_frames_fail() {
    let (mut session, _server, _initial, mut chat) = watch_fixture();
    assert!(matches!(
        session.receive_chat(Duration::from_millis(2)).unwrap(),
        SceneRead::Idle
    ));
    write_frame(&mut chat, &[0x0a, 0, 0, 0, 0]).unwrap();
    assert!(matches!(
        session.receive_chat(Duration::from_millis(2)).unwrap(),
        SceneRead::Packet(_)
    ));
    assert!(session.bootstrap.packets.is_empty());
    chat.write_all(&[5, 0x0a]).unwrap();
    assert!(session.receive_chat(Duration::from_millis(2)).is_err());
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
fn commands_are_framed_on_world_channel_with_no_optimistic_state() {
    use crate::commands::SceneCommand;
    let (mut session, mut world, mut initial, mut chat) = watch_fixture();
    let command = SceneCommand::MoveGear {
        source_slot: 5,
        source_subindex: 0,
        destination_slot: 1,
    };
    session
        .send_command(command, Duration::from_secs(1))
        .unwrap();
    assert_eq!(
        read_frame(&mut world, 100).unwrap(),
        command.encode().unwrap()
    );
    assert!(session.bootstrap.packets.is_empty());
    assert!(session.send_command(command, Duration::ZERO).is_err());
    initial.set_nonblocking(true).unwrap();
    chat.set_nonblocking(true).unwrap();
    assert_eq!(
        initial.read(&mut [0]).unwrap_err().kind(),
        io::ErrorKind::WouldBlock
    );
    assert_eq!(
        chat.read(&mut [0]).unwrap_err().kind(),
        io::ErrorKind::WouldBlock
    );
}

#[test]
fn inventory_commands_route_only_to_authenticated_chat() {
    let (mut session, mut world, mut initial, mut chat) = watch_fixture();
    let command = crate::commands::ChatCommand::DropGear {
        source_slot: 5,
        source_subindex: 0,
    };
    session
        .send_chat_command(command, Duration::from_secs(1))
        .unwrap();
    assert_eq!(
        read_frame(&mut chat, 100).unwrap(),
        command.encode().unwrap()
    );
    let use_command = crate::commands::ChatCommand::UseGear {
        source_slot: 5,
        source_subindex: 3,
    };
    session
        .send_chat_command(use_command, Duration::from_secs(1))
        .unwrap();
    assert_eq!(
        read_frame(&mut chat, 100).unwrap(),
        use_command.encode().unwrap()
    );
    assert!(session.bootstrap.packets.is_empty());
    world.set_nonblocking(true).unwrap();
    initial.set_nonblocking(true).unwrap();
    assert_eq!(
        world.read(&mut [0]).unwrap_err().kind(),
        io::ErrorKind::WouldBlock
    );
    assert_eq!(
        initial.read(&mut [0]).unwrap_err().kind(),
        io::ErrorKind::WouldBlock
    );
    assert!(session.send_chat_command(command, Duration::ZERO).is_err());
    session._chat = None;
    assert!(
        session
            .send_chat_command(command, Duration::from_secs(1))
            .is_err()
    );
}

#[test]
fn incremental_receive_distinguishes_idle_closed_and_partial_frames() {
    let (mut session, mut peer, _initial, _chat) = watch_fixture();
    assert!(matches!(
        session.receive_update(Duration::from_millis(1)).unwrap(),
        SceneRead::Idle
    ));
    write_frame(&mut peer, &[0xfe, 9]).unwrap();
    assert!(
        matches!(session.receive_update(Duration::from_millis(20)).unwrap(), SceneRead::Packet(packet) if packet == [0xfe, 9])
    );
    assert_eq!(session.bootstrap.packets, vec![vec![0xfe, 9]]);
    peer.shutdown(std::net::Shutdown::Write).unwrap();
    assert!(matches!(
        session.receive_update(Duration::from_millis(20)).unwrap(),
        SceneRead::Closed
    ));
    let (mut session, mut peer, _initial, _chat) = watch_fixture();
    peer.write_all(&[2, 0xfe]).unwrap();
    assert!(session.receive_update(Duration::from_millis(1)).is_err());
    assert!(session.bootstrap.packets.is_empty());
}

#[test]
fn incremental_receive_rejects_changed_avatar_and_counts_moved_bootstrap_packets() {
    let (mut session, mut peer, _initial, _chat) = watch_fixture();
    write_frame(&mut peer, &[0x2a, 0, 0, 0, 43]).unwrap();
    assert!(session.receive_update(Duration::from_millis(20)).is_err());
    assert!(session.bootstrap.packets.is_empty());
    let (mut session, mut peer, _initial, _chat) = watch_fixture();
    session.bootstrap.packet_counts.insert(0xfe, 20_000);
    write_frame(&mut peer, &[0xfe]).unwrap();
    assert!(session.receive_update(Duration::from_millis(20)).is_err());
    assert!(session.receive_update(Duration::ZERO).is_err());
    assert!(session.receive_update(Duration::from_secs(2)).is_err());
}

#[test]
fn scheduled_chat_requests_receive_scene_updates_without_cross_channel_writes() {
    let (session, mut world, mut initial, mut chat) = watch_fixture();
    let command = crate::commands::ChatCommand::UseGear {
        source_slot: 5,
        source_subindex: 3,
    };
    let server = std::thread::spawn(move || {
        chat.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
        write_frame(&mut world, &[0xfe, 1]).unwrap();
        assert_eq!(
            read_frame(&mut chat, 100).unwrap(),
            command.encode().unwrap()
        );
        write_frame(&mut world, &[0xfe, 2]).unwrap();
        world.shutdown(std::net::Shutdown::Write).unwrap();
        assert_eq!(
            read_frame(&mut world, 100).unwrap_err().kind(),
            io::ErrorKind::UnexpectedEof
        );
        assert_eq!(
            read_frame(&mut initial, 100).unwrap_err().kind(),
            io::ErrorKind::UnexpectedEof
        );
    });
    let watched = session
        .watch_with_chat_commands(
            Duration::from_millis(200),
            &[ScheduledChatCommand {
                after: Duration::from_millis(40),
                command,
            }],
        )
        .unwrap();
    server.join().unwrap();
    assert_eq!(watched.capture.packets, vec![vec![0xfe, 1], vec![0xfe, 2]]);
    assert_eq!(watched.end, WatchEnd::ServerClosed);
}

#[test]
fn invalid_chat_schedules_and_missing_channel_fail_before_any_write() {
    let command = crate::commands::ChatCommand::UseGear {
        source_slot: 5,
        source_subindex: 3,
    };
    for schedule in [
        vec![
            ScheduledChatCommand {
                after: Duration::ZERO,
                command
            };
            17
        ],
        vec![ScheduledChatCommand {
            after: Duration::from_secs(1),
            command,
        }],
        vec![
            ScheduledChatCommand {
                after: Duration::from_millis(50),
                command,
            },
            ScheduledChatCommand {
                after: Duration::ZERO,
                command,
            },
        ],
        vec![
            ScheduledChatCommand {
                after: Duration::ZERO,
                command,
            },
            ScheduledChatCommand {
                after: Duration::from_millis(40),
                command: crate::commands::ChatCommand::UseGear {
                    source_slot: 0,
                    source_subindex: 0,
                },
            },
        ],
    ] {
        let (session, _world, _initial, mut chat) = watch_fixture();
        assert!(
            session
                .watch_with_chat_commands(Duration::from_millis(100), &schedule)
                .is_err()
        );
        assert_eq!(chat.read(&mut [0]).unwrap(), 0);
    }
    let (mut session, _world, _initial, mut chat) = watch_fixture();
    session._chat = None;
    assert!(
        session
            .watch_with_chat_commands(
                Duration::from_millis(100),
                &[ScheduledChatCommand {
                    after: Duration::ZERO,
                    command
                }]
            )
            .is_err()
    );
    assert_eq!(chat.read(&mut [0]).unwrap(), 0);
}

#[test]
fn scheduled_commands_interleave_with_inbound_frames() {
    use crate::commands::SceneCommand;
    let (session, mut peer, _initial, _chat) = watch_fixture();
    let commands = [
        ScheduledCommand {
            after: Duration::ZERO,
            command: SceneCommand::TargetPin { target: 42 },
        },
        ScheduledCommand {
            after: Duration::from_millis(40),
            command: SceneCommand::TargetPin { target: 0 },
        },
    ];
    let server = std::thread::spawn(move || {
        peer.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
        assert_eq!(
            read_frame(&mut peer, 100).unwrap(),
            commands[0].command.encode().unwrap()
        );
        write_frame(&mut peer, &[0xfe, 7]).unwrap();
        assert_eq!(
            read_frame(&mut peer, 100).unwrap(),
            commands[1].command.encode().unwrap()
        );
        write_frame(&mut peer, &[0xfe, 8]).unwrap();
        peer.shutdown(std::net::Shutdown::Write).unwrap();
    });
    let result = session
        .watch_with_commands(Duration::from_millis(200), &commands)
        .unwrap();
    server.join().unwrap();
    assert_eq!(result.end, WatchEnd::ServerClosed);
    assert_eq!(result.capture.packets, vec![vec![0xfe, 7], vec![0xfe, 8]]);
    assert_eq!(result.updates_received, 2);
}

#[test]
fn invalid_schedules_fail_before_writing_commands() {
    let pin = crate::commands::SceneCommand::TargetPin { target: 42 };
    for commands in [
        vec![ScheduledCommand {
            after: Duration::from_secs(1),
            command: pin,
        }],
        vec![
            ScheduledCommand {
                after: Duration::from_millis(50),
                command: pin,
            },
            ScheduledCommand {
                after: Duration::ZERO,
                command: pin,
            },
        ],
        vec![
            ScheduledCommand {
                after: Duration::ZERO,
                command: pin
            };
            17
        ],
    ] {
        let (session, mut peer, _initial, _chat) = watch_fixture();
        assert!(
            session
                .watch_with_commands(Duration::from_millis(100), &commands)
                .is_err()
        );
        assert_eq!(peer.read(&mut [0]).unwrap(), 0);
    }
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
