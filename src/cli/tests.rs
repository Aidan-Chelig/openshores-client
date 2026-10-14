use super::*;

#[test]
fn invalid_gameplay_arguments_fail_before_password_or_network() {
    for flags in [
        vec!["--use-gear", "5", "3"],
        vec![
            "--avatar",
            "42",
            "--watch-seconds",
            "1",
            "--use-gear",
            "5",
            "3",
        ],
        vec![
            "--avatar",
            "42",
            "--watch-seconds",
            "3",
            "--use-gear",
            "0",
            "3",
        ],
        vec![
            "--avatar",
            "42",
            "--watch-seconds",
            "3",
            "--use-gear",
            "5",
            "16",
        ],
        vec![
            "--avatar",
            "42",
            "--watch-seconds",
            "3",
            "--use-gear",
            "5",
            "3",
            "--drop-gear",
            "5",
            "0",
        ],
        vec!["--pose-report", "25252525252525252525"],
        vec![
            "--avatar",
            "42",
            "--watch-seconds",
            "3",
            "--pose-report",
            "25",
        ],
        vec![
            "--avatar",
            "42",
            "--watch-seconds",
            "3",
            "--pose-report",
            "gg252525252525252525",
        ],
        vec![
            "--avatar",
            "42",
            "--watch-seconds",
            "3",
            "--pose-report",
            "25252525252525252525",
            "--move-gear",
            "5",
            "0",
            "1",
        ],
        vec!["--position-report", "0", "0", "1001", "0", "0", "0"],
        vec![
            "--avatar",
            "42",
            "--watch-seconds",
            "3",
            "--position-report",
            "NaN",
            "0",
            "0",
            "0",
            "0",
            "0",
        ],
        vec![
            "--avatar",
            "42",
            "--watch-seconds",
            "3",
            "--position-report",
            "0",
            "0",
        ],
        vec![
            "--avatar",
            "42",
            "--watch-seconds",
            "3",
            "--position-report",
            "0",
            "0",
            "1001",
            "0",
            "0",
            "0",
            "--drop-gear",
            "5",
            "0",
        ],
        vec!["--pickup-target", "42"],
        vec![
            "--avatar",
            "42",
            "--watch-seconds",
            "1",
            "--pickup-target",
            "42",
        ],
        vec![
            "--avatar",
            "42",
            "--watch-seconds",
            "3",
            "--pickup-target",
            "0",
        ],
        vec![
            "--avatar",
            "42",
            "--watch-seconds",
            "3",
            "--pickup-target",
            "42",
            "--drop-gear",
            "5",
            "0",
        ],
    ] {
        let args: Vec<String> = std::iter::once("Explorer")
            .chain(flags)
            .map(str::to_owned)
            .collect();
        assert!(login_command(&args).is_err());
    }
    for flags in [
        vec!["--drop-gear", "5", "0"],
        vec![
            "--avatar",
            "42",
            "--watch-seconds",
            "1",
            "--drop-gear",
            "0",
            "0",
        ],
        vec![
            "--avatar",
            "42",
            "--watch-seconds",
            "1",
            "--drop-gear",
            "5",
            "16",
        ],
        vec![
            "--avatar",
            "42",
            "--watch-seconds",
            "1",
            "--drop-gear",
            "5",
            "0",
            "--move-gear",
            "5",
            "0",
            "1",
        ],
    ] {
        let args: Vec<String> = std::iter::once("Explorer")
            .chain(flags)
            .map(str::to_owned)
            .collect();
        assert!(login_command(&args).is_err());
    }
    for flags in [
        vec!["--move-gear", "5", "0", "1"],
        vec!["--look-at", "42"],
        vec![
            "--avatar",
            "42",
            "--watch-seconds",
            "1",
            "--move-gear",
            "16",
            "0",
            "1",
        ],
        vec![
            "--avatar",
            "42",
            "--watch-seconds",
            "1",
            "--move-gear",
            "5",
            "0",
        ],
        vec!["--avatar", "42", "--look-at", "0"],
        vec![
            "--avatar",
            "42",
            "--watch-seconds",
            "1",
            "--look-at",
            "not-an-id",
        ],
    ] {
        let args: Vec<String> = std::iter::once("Explorer")
            .chain(flags)
            .map(str::to_owned)
            .collect();
        assert!(login_command(&args).is_err());
    }
}

#[test]
fn invalid_watch_arguments_fail_before_password_or_network() {
    for flags in [
        vec!["--watch-seconds", "0"],
        vec!["--watch-seconds", "61"],
        vec!["--watch-seconds"],
        vec!["--watch-seconds", "1"],
        vec![
            "--avatar",
            "42",
            "203.0.113.1:16757",
            "--watch-seconds",
            "1",
        ],
    ] {
        let args: Vec<String> = std::iter::once("Explorer")
            .chain(flags)
            .map(str::to_owned)
            .collect();
        assert!(login_command(&args).is_err());
    }
}

#[test]
fn terrain_demo_resolution_is_optional_and_bounded() {
    let args = |values: &[&str]| values.iter().map(|v| v.to_string()).collect::<Vec<_>>();
    assert_eq!(terrain_demo_options(&[]).unwrap(), (32, None, false));
    assert_eq!(
        terrain_demo_options(&args(&["128", "--unlit", "--smoke-frames", "90"])).unwrap(),
        (128, Some(90), true)
    );
    assert_eq!(
        terrain_demo_options(&args(&["--unlit"])).unwrap(),
        (32, None, true)
    );
    for values in [
        vec!["0"],
        vec!["2"],
        vec!["129"],
        vec!["BANDS"],
        vec!["32", "64"],
        vec!["--unlit", "64"],
    ] {
        assert!(terrain_demo_options(&args(&values)).is_err());
    }
}

#[test]
fn viewer_flags_are_bounded_and_order_independent() {
    let args = |values: &[&str]| {
        values
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
    };
    assert_eq!(viewer_options(&args(&[])).unwrap(), (None, false));
    assert_eq!(
        viewer_options(&args(&["--unlit", "--smoke-frames", "90"])).unwrap(),
        (Some(90), true)
    );
    assert_eq!(
        viewer_options(&args(&["--smoke-frames", "90", "--unlit"])).unwrap(),
        (Some(90), true)
    );
    for values in [
        vec!["--unlit", "--unlit"],
        vec!["--smoke-frames"],
        vec!["--smoke-frames", "0"],
        vec!["--smoke-frames", "3601"],
        vec!["--surprise"],
        vec!["--smoke-frames", "1", "--smoke-frames", "2"],
    ] {
        assert!(viewer_options(&args(&values)).is_err());
    }
}

#[test]
fn stdin_password_preserves_spaces_and_unicode() {
    assert_eq!(
        &**password_from_stdin(&mut "  café 🚀  \r\n".as_bytes()).unwrap(),
        "  café 🚀  "
    );
    assert_eq!(&**password_from_stdin(&mut b"".as_slice()).unwrap(), "");
    assert!(password_from_stdin(&mut b"first\nsecond\n".as_slice()).is_err());
    assert!(password_from_stdin(&mut [0xff].as_slice()).is_err());
    assert!(password_from_stdin(&mut vec![b'x'; 4097].as_slice()).is_err());
}
