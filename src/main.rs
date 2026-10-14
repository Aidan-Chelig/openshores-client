use openshores_client::login::{self, LoginReply};
use std::io::{IsTerminal, Read};
use std::net::SocketAddr;
use std::time::Duration;
use std::{env, fs, io, path::Path};
use zeroize::Zeroizing;

fn password_from_stdin(reader: &mut impl Read) -> io::Result<Zeroizing<String>> {
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

fn login_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
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
    let mut options = args[1..].iter();
    while let Some(arg) = options.next() {
        if arg == "--password-stdin" && !from_stdin {
            from_stdin = true;
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
    if avatar_id.is_some() && !address.ip().is_loopback() {
        return Err("scene probes are restricted to local development servers".into());
    }
    if avatar_id == Some(0) {
        return Err("--avatar must be nonzero".into());
    }
    if avatar_id.is_none() && (scene_port.is_some() || chat_port.is_some() || capture.is_some()) {
        return Err("--scene-port, --chat-port, and --capture require --avatar".into());
    }
    if scene_port == Some(0) || chat_port == Some(0) {
        return Err("ports must be nonzero".into());
    }
    // Check the username before requesting a password or opening a connection.
    login::encode_login(username, &[0; 20])?;
    let password = if from_stdin {
        if io::stdin().is_terminal() {
            return Err("use the hidden password prompt for terminal input".into());
        }
        password_from_stdin(&mut io::stdin().lock())?
    } else {
        Zeroizing::new(rpassword::prompt_password("Password: ")?)
    };
    let digest = Zeroizing::new(login::wire_password(&password));
    drop(password);
    match login::exchange(address, username, &digest, Duration::from_secs(10))? {
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
                let world = openshores_client::local_scene::probe(
                    SocketAddr::new(address.ip(), scene_port.unwrap_or(16759)),
                    id,
                    Duration::from_secs(30),
                    chat_port,
                )?;
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

fn inventory(path: &Path, count: &mut usize, bytes: &mut u64) -> io::Result<()> {
    let mut entries = fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            continue;
        }
        if kind.is_dir() {
            if matches!(
                entry.file_name().to_str(),
                Some(".git" | "__pycache__" | ".venv")
            ) {
                continue;
            }
            inventory(&entry.path(), count, bytes)?;
        } else if kind.is_file() {
            *count += 1;
            *bytes += entry.metadata()?.len();
        }
    }
    Ok(())
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("login") => login_command(&args[1..])?,
        Some("inspect-gd") if args.len() <= 2 => {
            let path = args
                .get(1)
                .map(String::as_str)
                .unwrap_or("reference/OpenShores-server/GD");
            let rows = openshores_client::assets::construction_processes(&fs::read(path)?)?;
            println!("{} construction processes from {path}", rows.len());
            for row in rows {
                println!(
                    "{:3}  {:30} industry={:3} terrain={:3} radius={:3}",
                    row.id, row.name, row.industry, row.terrain, row.radius
                );
            }
        }
        Some("inspect-reference") if args.len() <= 2 => {
            let root = Path::new(args.get(1).map(String::as_str).unwrap_or("reference"));
            let mut entries = fs::read_dir(root)?.collect::<Result<Vec<_>, _>>()?;
            entries.sort_by_key(|entry| entry.file_name());
            for entry in entries {
                if !entry.file_type()?.is_dir() {
                    continue;
                }
                let (mut count, mut bytes) = (0, 0);
                inventory(&entry.path(), &mut count, &mut bytes)?;
                println!(
                    "{}: {count} files, {bytes} bytes",
                    entry.file_name().to_string_lossy()
                );
            }
        }
        None | Some("--help" | "-h") => println!(
            "OpenShores Rust client foundation\n\nUsage: cargo run -- login USERNAME [IP:PORT] [--password-stdin]\n       cargo run -- login USERNAME [IP:PORT] --avatar ID [--scene-port PORT] [--chat-port PORT] [--capture NEW_FILE] [--password-stdin]\n       cargo run -- inspect-reference [directory]\n       cargo run -- inspect-gd [GD file]\n\nLogin defaults to 127.0.0.1:16757 and prompts for a hidden password.\nAvatar scene probes only use loopback endpoints and stop after the initial manifest.\nInventory excludes symlinks, .git, __pycache__, and .venv."
        ),
        _ => return Err("unknown command; use --help".into()),
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
