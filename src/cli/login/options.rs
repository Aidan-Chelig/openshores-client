use super::*;
use openshores_client::commands::{ChatCommand, SceneCommand};

#[cfg(test)]
#[path = "options_tests.rs"]
mod tests;

pub(super) struct LoginOptions<'a> {
    pub(super) username: &'a str,
    pub(super) address: SocketAddr,
    pub(super) from_stdin: bool,
    pub(super) avatar_id: Option<u32>,
    pub(super) scene_port: Option<u16>,
    pub(super) chat_port: Option<u16>,
    pub(super) capture: Option<&'a String>,
    pub(super) watch_seconds: Option<u64>,
    pub(super) look_at: Option<SceneCommand>,
    pub(super) move_gear: Option<SceneCommand>,
    pub(super) drop_gear: Option<ChatCommand>,
    pub(super) use_gear: Option<ChatCommand>,
    pub(super) pickup_target: Option<u32>,
    pub(super) position_report: Option<SceneCommand>,
    pub(super) pose_report: Option<SceneCommand>,
}

pub(super) fn parse(args: &[String]) -> Result<LoginOptions<'_>, Box<dyn std::error::Error>> {
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
    Ok(LoginOptions {
        username: username.as_str(),
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
    })
}
