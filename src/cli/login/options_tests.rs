use super::*;

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

#[test]
fn pure_parser_preserves_defaults_and_all_connection_options() {
    let input = args(&["rust-local-test"]);
    let defaults = parse(&input).unwrap();
    assert_eq!(defaults.username, "rust-local-test");
    assert_eq!(defaults.address, "127.0.0.1:16757".parse().unwrap());
    assert!(!defaults.from_stdin);
    assert!(defaults.avatar_id.is_none() && defaults.watch_seconds.is_none());
    let input = args(&[
        "rust-local-test",
        "--watch-seconds",
        "3",
        "--avatar",
        "0x20",
        "127.0.0.1:17000",
        "--capture",
        "scene.bin",
        "--password-stdin",
        "--scene-port",
        "17001",
        "--chat-port",
        "17002",
    ]);
    let options = parse(&input).unwrap();
    assert_eq!(options.avatar_id, Some(32));
    assert_eq!(options.watch_seconds, Some(3));
    assert_eq!(options.scene_port, Some(17001));
    assert_eq!(options.chat_port, Some(17002));
    assert_eq!(options.address, "127.0.0.1:17000".parse().unwrap());
    assert_eq!(options.capture.map(String::as_str), Some("scene.bin"));
    assert!(options.from_stdin);
}

#[test]
fn command_values_survive_option_extraction_without_io() {
    let input = args(&[
        "rust-local-test",
        "--avatar",
        "32",
        "--watch-seconds",
        "3",
        "--look-at",
        "0x29",
        "--move-gear",
        "5",
        "0",
        "1",
    ]);
    let options = parse(&input).unwrap();
    assert_eq!(options.look_at, Some(SceneCommand::LookAt { target: 41 }));
    assert_eq!(
        options.move_gear,
        Some(SceneCommand::MoveGear {
            source_slot: 5,
            source_subindex: 0,
            destination_slot: 1
        })
    );
    let input = args(&[
        "rust-local-test",
        "--avatar",
        "32",
        "--watch-seconds",
        "3",
        "--position-report",
        "1",
        "2",
        "3",
        "0.1",
        "0.2",
        "0.3",
    ]);
    assert_eq!(
        parse(&input).unwrap().position_report,
        Some(SceneCommand::PositionReport {
            projected: [1., 2., 3.],
            rotation: [0.1, 0.2, 0.3]
        })
    );
    let input = args(&[
        "rust-local-test",
        "--avatar",
        "32",
        "--watch-seconds",
        "3",
        "--pose-report",
        "00010203040506070809",
    ]);
    assert_eq!(
        parse(&input).unwrap().pose_report,
        Some(SceneCommand::PoseReport {
            poses: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
        })
    );
}

#[test]
fn missing_duplicate_and_out_of_range_options_fail_in_pure_validation() {
    for values in [
        vec![],
        vec!["rust-local-test", "--avatar"],
        vec!["rust-local-test", "--watch-seconds", "61"],
        vec!["rust-local-test", "--password-stdin", "--password-stdin"],
        vec!["rust-local-test", "--avatar", "32", "--scene-port", "0"],
        vec![
            "rust-local-test",
            "--avatar",
            "32",
            "--watch-seconds",
            "3",
            "--drop-gear",
            "0",
            "0",
        ],
    ] {
        assert!(parse(&args(&values)).is_err());
    }
}
