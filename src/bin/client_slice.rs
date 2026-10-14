//! Graphical connected slice; the original CLI remains the default binary.
use openshores_client::client_runtime::LocalEndpoint;
use std::{
    io::{self, BufRead, Read},
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
    let mut screenshot = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
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
                    "OpenShores Rust local client slice\nUsage: cargo run --offline --features client --bin openshores-client-slice -- [IP:PORT] [--scene-port PORT] [--chat-port PORT] [--smoke-frames 1..3600]\nLoopback only. Login and avatar selection occur in the window."
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
    if let Some((username, avatar_id)) = local_smoke {
        if smoke_frames.is_some() {
            return Err("--local-smoke cannot combine with --smoke-frames".into());
        }
        let mut password = Zeroizing::new(String::new());
        io::stdin().lock().take(4098).read_line(&mut password)?;
        if password.len() > 1026 {
            return Err("smoke password exceeds input budget".into());
        }
        if password.ends_with('\n') {
            password.pop();
            if password.ends_with('\r') {
                password.pop();
            }
        }
        openshores_client::client_ui::run_local_smoke(
            endpoint,
            openshores_client::client_ui::LocalSmoke {
                username,
                password,
                avatar_id,
                screenshot,
            },
        )?;
    } else {
        if screenshot.is_some() {
            return Err("--screenshot requires --local-smoke".into());
        }
        openshores_client::client_ui::run(endpoint, smoke_frames)?;
    }
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
