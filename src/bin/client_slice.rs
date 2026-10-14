//! Graphical connected slice; the original CLI remains the default binary.
use openshores_client::client_runtime::LocalEndpoint;
use std::{
    io::{self, BufRead},
    net::SocketAddr,
};
use zeroize::Zeroizing;

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut address: SocketAddr = "127.0.0.1:16757".parse()?;
    let mut scene_port = 16759;
    let mut chat_port = 16758;
    let mut smoke_frames = None;
    let mut endpoint_given = false;
    let mut local_smoke = None;
    let mut auto_login = None;
    let mut screenshot = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--auto-login" if auto_login.is_none() => {
                let username = args
                    .next()
                    .ok_or("--auto-login requires USERNAME AVATAR_ID")?;
                let avatar = args
                    .next()
                    .ok_or("--auto-login requires USERNAME AVATAR_ID")?
                    .parse::<u32>()?;
                auto_login = Some((username, avatar));
            }
            "--local-smoke" if local_smoke.is_none() => {
                let username = args
                    .next()
                    .ok_or("--local-smoke requires USERNAME AVATAR_ID")?;
                let avatar = args
                    .next()
                    .ok_or("--local-smoke requires USERNAME AVATAR_ID")?
                    .parse::<u32>()?;
                local_smoke = Some((username, avatar));
            }
            "--screenshot" if screenshot.is_none() => {
                screenshot = Some(std::path::PathBuf::from(
                    args.next().ok_or("missing screenshot path")?,
                ))
            }
            "--scene-port" => scene_port = args.next().ok_or("missing scene port")?.parse()?,
            "--chat-port" => chat_port = args.next().ok_or("missing chat port")?.parse()?,
            "--smoke-frames" if smoke_frames.is_none() => {
                smoke_frames = Some(args.next().ok_or("missing smoke frame limit")?.parse()?)
            }
            "--help" | "-h" => {
                println!(
                    "OpenShores Rust local client slice\nUsage: cargo run --offline --features client --bin openshores-client-slice -- [IP:PORT] [--scene-port PORT] [--chat-port PORT] [--smoke-frames 1..3600]\nLoopback only. Login and avatar selection occur in the window.\n--auto-login USERNAME AVATAR_ID: synthetic rust-local-* account only; reads password from stdin.\n--local-smoke USERNAME AVATAR_ID [--screenshot NEW_PATH]: disposable integration actions; reads password from stdin."
                );
                return Ok(());
            }
            _ if !arg.starts_with('-') && !endpoint_given => {
                address = arg.parse()?;
                endpoint_given = true;
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "unknown or duplicate client option",
                )
                .into());
            }
        }
    }
    let endpoint = LocalEndpoint {
        login: address,
        scene_port,
        chat_port,
    }
    .validate()?;
    if auto_login.is_some() && (local_smoke.is_some() || screenshot.is_some()) {
        return Err("--auto-login cannot combine with --local-smoke or --screenshot".into());
    }
    if let Some((username, avatar_id)) = local_smoke {
        if smoke_frames.is_some() {
            return Err("--local-smoke cannot combine with --smoke-frames".into());
        }
        let password = read_password(io::stdin().lock())?;
        openshores_client::client_ui::run_local_smoke(
            endpoint,
            openshores_client::client_ui::LocalSmoke {
                conversation: std::env::var_os("OPENSHORES_TEST_CONVERSATION").is_some(),
                username,
                password,
                avatar_id,
                screenshot,
            },
        )?;
    } else if let Some((username, avatar_id)) = auto_login {
        let password = read_password(io::stdin().lock())?;
        openshores_client::client_ui::run_auto_login(
            endpoint,
            openshores_client::client_ui::LocalAutoLogin {
                username,
                password,
                avatar_id,
            },
            smoke_frames,
        )?;
    } else {
        if screenshot.is_some() {
            return Err("--screenshot requires --local-smoke".into());
        }
        openshores_client::client_ui::run(endpoint, smoke_frames)?;
    }
    Ok(())
}

fn read_password(reader: impl BufRead) -> io::Result<Zeroizing<String>> {
    let mut password = Zeroizing::new(String::new());
    reader.take(1027).read_line(&mut password)?;
    if password.ends_with('\n') {
        password.pop();
        if password.ends_with('\r') {
            password.pop();
        }
    }
    if password.is_empty() || password.len() > 1024 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "password must be 1..=1024 bytes",
        ));
    }
    Ok(password)
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
    fn bounded_password_preserves_spaces_and_strips_line_endings() {
        for input in [" synthetic \n", " synthetic \r\n", " synthetic "] {
            assert_eq!(
                read_password(input.as_bytes()).unwrap().as_str(),
                " synthetic "
            );
        }
        assert!(read_password(&b"\n"[..]).is_err());
        assert!(read_password("x".repeat(1025).as_bytes()).is_err());
        assert_eq!(
            read_password(format!("{}\r\n", "x".repeat(1024)).as_bytes())
                .unwrap()
                .len(),
            1024
        );
    }
}
