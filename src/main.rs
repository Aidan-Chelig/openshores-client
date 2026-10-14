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
    let mut watch_seconds = None;
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
                let session = openshores_client::local_scene::Session::open(
                    SocketAddr::new(address.ip(), scene_port.unwrap_or(16759)),
                    id,
                    Duration::from_secs(30),
                    chat_port,
                )?;
                let (world, state) = if let Some(seconds) = watch_seconds {
                    let watched = session.watch(Duration::from_secs(seconds))?;
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

fn print_world(world: &openshores_client::world::WorldState) -> io::Result<()> {
    let mut counts = std::collections::BTreeMap::new();
    for entity in world.entities.values() {
        *counts.entry(entity.kind).or_insert(0usize) += 1;
    }
    println!(
        "Decoded {} entities; {} visible in manifest.",
        world.entities.len(),
        world.visible_ids.len()
    );
    println!("Entity kinds: {counts:?}");
    println!("Missing manifest IDs: {:?}", world.missing_manifest_ids());
    println!("Unresolved parents: {:?}", world.unresolved_parents());
    println!("Opaque packet counts: {:?}", world.opaque_packet_counts);
    if let Some(id) = world.active_unit {
        let chain = world.ancestry(id)?;
        let labels: Vec<_> = chain
            .iter()
            .map(|id| format!("{:?}({id:#010x})", world.entities[id].kind))
            .collect();
        println!("Player ancestry: {}", labels.join(" -> "));
    }
    Ok(())
}

#[cfg(any(feature = "viewer", test))]
fn viewer_options(args: &[String]) -> Result<(Option<u32>, bool), Box<dyn std::error::Error>> {
    let mut frames = None;
    let mut unlit = false;
    let mut options = args.iter();
    while let Some(option) = options.next() {
        match option.as_str() {
            "--unlit" if !unlit => unlit = true,
            "--smoke-frames" if frames.is_none() => {
                let count: u32 = options
                    .next()
                    .ok_or("--smoke-frames requires a count")?
                    .parse()?;
                if !(1..=3600).contains(&count) {
                    return Err("smoke frame limit must be 1..=3600".into());
                }
                frames = Some(count);
            }
            _ => return Err(format!("unknown or duplicate viewer option: {option}").into()),
        }
    }
    Ok((frames, unlit))
}

#[cfg(any(feature = "viewer", test))]
fn terrain_demo_options(
    args: &[String],
) -> Result<(u32, Option<u32>, bool), Box<dyn std::error::Error>> {
    let (bands, flags) = match args.first().filter(|arg| !arg.starts_with('-')) {
        Some(value) => (
            value
                .parse::<u32>()
                .map_err(|_| "latitude bands must be an integer in 3..=128")?,
            &args[1..],
        ),
        None => (32, args),
    };
    if !(3..=128).contains(&bands) {
        return Err("latitude bands must be in 3..=128".into());
    }
    let (frames, unlit) = viewer_options(flags)?;
    Ok((bands, frames, unlit))
}

fn print_terrain_statistics(mesh: &openshores_client::terrain_mesh::TerrainMesh) {
    let stats = mesh.statistics();
    println!(
        "Diagnostic base terrain: {} latitude bands, {} longitude sectors; {} vertices, {} triangles.",
        stats.latitude_bands, stats.longitude_sectors, stats.vertices, stats.triangles
    );
    println!(
        "Base radius: {:.12}; sampled altitude MSL: {:.12} .. {:.12} (not global extrema).",
        stats.base_radius, stats.minimum_altitude_msl, stats.maximum_altitude_msl
    );
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("login") => login_command(&args[1..])?,
        Some("inspect-terrain-demo") if args.len() <= 2 => {
            let bands = args
                .get(1)
                .map(|value| value.parse::<u32>())
                .transpose()
                .map_err(|_| "latitude bands must be an integer in 3..=128")?
                .unwrap_or(32);
            let mesh = openshores_client::terrain_mesh::TerrainMesh::generate(
                openshores_client::terrain_mesh::demonstration_surface(),
                bands,
            )?;
            print_terrain_statistics(&mesh);
        }
        Some("inspect-terrain-demo") => {
            return Err("usage: inspect-terrain-demo [LATITUDE_BANDS]".into());
        }
        Some("view-terrain-demo") => {
            #[cfg(feature = "viewer")]
            {
                let (bands, frames, unlit) = terrain_demo_options(&args[1..])?;
                let surface = openshores_client::terrain_mesh::demonstration_surface();
                let mesh = openshores_client::terrain_mesh::TerrainMesh::generate(surface, bands)?;
                println!(
                    "Offline synthetic terrain demo ({bands} bands, {} vertices, {} triangles); no scene capture or server required.",
                    mesh.positions().len(),
                    mesh.triangles().len()
                );
                print_terrain_statistics(&mesh);
                openshores_client::viewer::run_terrain(&mesh, frames, unlit)?;
            }
            #[cfg(not(feature = "viewer"))]
            return Err("enable --features viewer to run view-terrain-demo".into());
        }
        Some("view-terrain") => {
            #[cfg(feature = "viewer")]
            {
                if args.len() < 4 {
                    return Err("usage: view-terrain CAPTURE_FILE GLOBE_ID LATITUDE_BANDS [--unlit] [--smoke-frames N]".into());
                }
                let (frames, unlit) = viewer_options(&args[4..])?;
                let bands = args[3].parse::<u32>()?;
                let id = if let Some(hex) = args[2].strip_prefix("0x") {
                    u32::from_str_radix(hex, 16)?
                } else {
                    args[2].parse::<u32>()?
                };
                let world = openshores_client::world::read_capture(&mut io::BufReader::new(
                    fs::File::open(&args[1])?,
                ))?;
                let surface = world
                    .entities
                    .get(&id)
                    .and_then(|entity| entity.globe_surface)
                    .ok_or("selected atom has no decoded globe surface")?;
                let mesh = openshores_client::terrain_mesh::TerrainMesh::generate(surface, bands)?;
                print_terrain_statistics(&mesh);
                openshores_client::viewer::run_terrain(&mesh, frames, unlit)?;
            }
            #[cfg(not(feature = "viewer"))]
            return Err(
                "the offline viewer is optional; enable --features viewer for view-terrain".into(),
            );
        }
        Some("view-model") => {
            #[cfg(feature = "viewer")]
            {
                if args.len() < 2 {
                    return Err("usage: view-model MODEL_FILE [--unlit] [--smoke-frames N]".into());
                }
                let (frames, unlit) = viewer_options(&args[2..])?;
                let mut data = Vec::new();
                fs::File::open(&args[1])?
                    .take(64 * 1024 * 1024 + 1)
                    .read_to_end(&mut data)?;
                let variant = openshores_client::model::ModelVariant::from_name(&args[1])?;
                let model = openshores_client::model::Model::decode(&data, variant)?;
                openshores_client::viewer::run(&model, frames, unlit)?;
            }
            #[cfg(not(feature = "viewer"))]
            return Err("the offline viewer is optional; run cargo run --features viewer -- view-model MODEL_FILE".into());
        }
        Some("audit-model") if args.len() == 2 => {
            let mut data = Vec::new();
            fs::File::open(&args[1])?
                .take(64 * 1024 * 1024 + 1)
                .read_to_end(&mut data)?;
            let variant = openshores_client::model::ModelVariant::from_name(&args[1])?;
            let model = openshores_client::model::Model::decode(&data, variant)?;
            let reports = openshores_client::model_audit::audit(&model)?;
            for (index, report) in reports.iter().enumerate() {
                println!(
                    "{index}: unused={}; repeated={}; zero_area={}; boundary={}; nonmanifold={}; unassigned={}; overlapping={}; missing_material={}; ambiguous_material={}; uvs={:?}; extensions={}",
                    report.unused_vertices,
                    report.repeated_index_faces,
                    report.zero_area_faces,
                    report.boundary_edges,
                    report.nonmanifold_edges,
                    report.unassigned_faces,
                    report.multiply_assigned_faces,
                    report.missing_material_groups,
                    report.ambiguous_material_groups,
                    report.uv_count,
                    report.extension_faces
                );
            }
            println!(
                "Audited {} meshes. Raw-index topology and exact material names only; no welding, repair, default materials or rendering assumptions.",
                reports.len()
            );
        }
        Some("export-model" | "preview-model") if args.len() == 3 => {
            let mut data = Vec::new();
            fs::File::open(&args[1])?
                .take(64 * 1024 * 1024 + 1)
                .read_to_end(&mut data)?;
            let variant = openshores_client::model::ModelVariant::from_name(&args[1])?;
            let model = openshores_client::model::Model::decode(&data, variant)?;
            let mut output = io::BufWriter::new(
                fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&args[2])?,
            );
            if args[0] == "export-model" {
                openshores_client::model_preview::write_obj(&model, &mut output)?;
            } else {
                openshores_client::model_preview::write_svg(&model, &mut output)?;
            }
            use std::io::Write;
            output.flush()?;
            println!(
                "Wrote raw geometry debugging export to {:?}. Original transforms, materials, normals, UVs and animation are omitted.",
                args[2]
            );
        }
        Some("verify-models") if args.len() == 3 => {
            let mut data = Vec::new();
            fs::File::open(&args[1])?
                .take(128 * 1024 * 1024 + 1)
                .read_to_end(&mut data)?;
            let resources =
                openshores_client::qt_resources::QtResources::open(&data, args[2].parse()?)?;
            let (mut count, mut meshes, mut vertices, mut faces) = (0usize, 0usize, 0usize, 0usize);
            let mut rejected = 0usize;
            for (index, entry) in resources.entries().iter().enumerate() {
                let Ok(variant) = openshores_client::model::ModelVariant::from_name(&entry.path)
                else {
                    continue;
                };
                let payload = resources.read_entry(index)?;
                let model = match openshores_client::model::Model::decode(&payload, variant) {
                    Ok(model) => model,
                    Err(error) => {
                        eprintln!("Rejected {:?}: {error}", entry.path);
                        rejected += 1;
                        continue;
                    }
                };
                let vertex_count: usize = model.meshes.iter().map(|mesh| mesh.vertices.len()).sum();
                let face_count: usize = model.meshes.iter().map(|mesh| mesh.faces.len()).sum();
                println!(
                    "{index}: {:?}; meshes={}; vertices={vertex_count}; faces={face_count}; geometry={}; materials={}",
                    entry.path,
                    model.meshes.len(),
                    model.geometry_digest(),
                    model.material_digest()
                );
                count += 1;
                meshes += model.meshes.len();
                vertices += vertex_count;
                faces += face_count;
            }
            println!(
                "Verified {count} models; {meshes} meshes; {vertices} vertices; {faces} faces. Coordinates and matrices remain unconverted; opaque chunks remain retained."
            );
            if rejected != 0 {
                return Err(format!(
                    "{rejected} model resources rejected; verification is incomplete"
                )
                .into());
            }
        }
        Some("inspect-model") if args.len() == 2 => {
            let mut data = Vec::new();
            fs::File::open(&args[1])?
                .take(64 * 1024 * 1024 + 1)
                .read_to_end(&mut data)?;
            let variant = openshores_client::model::ModelVariant::from_name(&args[1])?;
            let model = openshores_client::model::Model::decode(&data, variant)?;
            println!(
                "Model: {variant:?}; {} meshes; {} materials; version={:?}; master_scale={:?}; geometry={}",
                model.meshes.len(),
                model.materials.len(),
                model.version,
                model.master_scale,
                model.geometry_digest()
            );
            for mesh in &model.meshes {
                println!(
                    "{:?}: {} vertices; {} faces; UV count={:?}; matrix={:?}; smoothing={}; material groups={}",
                    String::from_utf8_lossy(&mesh.name),
                    mesh.vertices.len(),
                    mesh.faces.len(),
                    mesh.uvs.as_ref().map(Vec::len),
                    mesh.matrix,
                    mesh.smoothing.is_some(),
                    mesh.materials.len()
                );
            }
            for material in &model.materials {
                println!(
                    "Material {:?}: colors={:?}; amounts={:?}; two_sided={}; shading={:?}; textures={:?}",
                    material
                        .name
                        .as_ref()
                        .map(|name| String::from_utf8_lossy(name)),
                    material.colors,
                    material.amounts,
                    material.two_sided,
                    material.shading,
                    material
                        .textures
                        .iter()
                        .map(|name| String::from_utf8_lossy(name))
                        .collect::<Vec<_>>()
                );
            }
            println!(
                "Raw coordinate bounds: {:?}",
                openshores_client::model_preview::bounds(&model)?
            );
            println!(
                "Chunk counts: {:?}; {} opaque chunks retained.",
                model.chunks,
                model.opaque.len()
            );
        }
        Some("inspect-qt" | "verify-qt" | "extract-qt") => {
            let extract = args[0] == "extract-qt";
            if args.len() != if extract { 6 } else { 3 } {
                return Err("usage: inspect-qt/verify-qt DLL VERSION; extract-qt DLL VERSION POOL_INDEX ENTRY_INDEX NEW_OUTPUT".into());
            }
            let mut data = Vec::new();
            fs::File::open(&args[1])?
                .take(128 * 1024 * 1024 + 1)
                .read_to_end(&mut data)?;
            let resources =
                openshores_client::qt_resources::QtResources::open(&data, args[2].parse()?)?;
            if extract {
                let pool: usize = args[3].parse()?;
                let index: usize = args[4].parse()?;
                let entry = resources
                    .entries()
                    .get(index)
                    .ok_or("resource index out of range")?;
                if entry.pool != pool {
                    return Err("entry does not belong to the requested pool".into());
                }
                let payload = resources.read_entry(index)?;
                use std::io::Write;
                let mut output = fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&args[5])?;
                output.write_all(&payload)?;
                println!("Extracted {:?}: {} bytes.", entry.path, payload.len());
            } else {
                println!(
                    "Qt: {} pools; {} entries.",
                    resources.pool_count,
                    resources.entries().len()
                );
                let mut total = 0usize;
                for (index, entry) in resources.entries().iter().enumerate() {
                    if args[0] == "verify-qt" {
                        total = total
                            .checked_add(resources.read_entry(index)?.len())
                            .ok_or("Qt total overflow")?;
                        if total > 512 * 1024 * 1024 {
                            return Err("Qt verification exceeds total output budget".into());
                        }
                    } else {
                        println!(
                            "{index}: pool={} {:?}; locale={}/{}; flags={}; stored={}; raw={}; offset={}",
                            entry.pool,
                            entry.path,
                            entry.country,
                            entry.language,
                            entry.flags,
                            entry.stored_bytes,
                            entry.original_bytes,
                            entry.payload_range.start
                        );
                    }
                }
                if args[0] == "verify-qt" {
                    println!(
                        "Verified {} entries; {total} raw bytes.",
                        resources.entries().len()
                    );
                }
            }
        }
        Some("inspect-har" | "verify-har") if args.len() == 2 => {
            let mut data = Vec::new();
            fs::File::open(&args[1])?
                .take(64 * 1024 * 1024 + 1)
                .read_to_end(&mut data)?;
            let archive = openshores_client::har::HarArchive::open(&data)?;
            println!(
                "HAR: {} entries; index at {}.",
                archive.entries().len(),
                archive.index_offset
            );
            let mut total = 0usize;
            for entry in archive.entries() {
                if args[0] == "verify-har" {
                    let payload = archive.read_entry(&entry.name)?;
                    total += payload.len();
                } else {
                    println!(
                        "{:?}: {} compressed / {} raw bytes; offset={}",
                        entry.name, entry.compressed_bytes, entry.original_bytes, entry.offset
                    );
                }
            }
            if args[0] == "verify-har" {
                println!(
                    "Verified {} entries; {total} decompressed bytes.",
                    archive.entries().len()
                );
            }
        }
        Some("extract-har") if args.len() == 4 => {
            let mut data = Vec::new();
            fs::File::open(&args[1])?
                .take(64 * 1024 * 1024 + 1)
                .read_to_end(&mut data)?;
            let archive = openshores_client::har::HarArchive::open(&data)?;
            let payload = archive.read_entry(&args[2])?;
            use std::io::Write;
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&args[3])?;
            file.write_all(&payload)?;
            println!(
                "Extracted {:?}: {} bytes to {:?}.",
                args[2],
                payload.len(),
                args[3]
            );
        }
        Some("export-terrain-demo") if args.len() == 3 => {
            let bands = args[1]
                .parse::<u32>()
                .map_err(|_| "latitude bands must be an integer in 3..=128")?;
            let mesh = openshores_client::terrain_mesh::TerrainMesh::generate(
                openshores_client::terrain_mesh::demonstration_surface(),
                bands,
            )?;
            let mut output = io::BufWriter::new(
                fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&args[2])?,
            );
            mesh.write_obj(&mut output)?;
            use std::io::Write;
            output.flush()?;
            print_terrain_statistics(&mesh);
            println!(
                "Exported synthetic base terrain: {} vertices, {} triangles to {:?}.",
                mesh.positions().len(),
                mesh.triangles().len(),
                args[2]
            );
        }
        Some("export-terrain-demo") => {
            return Err("usage: export-terrain-demo LATITUDE_BANDS NEW_OBJ_FILE".into());
        }
        Some("export-terrain") if args.len() == 5 => {
            let bands = args[3].parse::<u32>()?;
            let world = openshores_client::world::read_capture(&mut io::BufReader::new(
                fs::File::open(&args[1])?,
            ))?;
            let id = if let Some(hex) = args[2].strip_prefix("0x") {
                u32::from_str_radix(hex, 16)?
            } else {
                args[2].parse::<u32>()?
            };
            let surface = world
                .entities
                .get(&id)
                .and_then(|entity| entity.globe_surface)
                .ok_or("selected atom has no decoded globe surface")?;
            let mesh = openshores_client::terrain_mesh::TerrainMesh::generate(surface, bands)?;
            let mut output = io::BufWriter::new(
                fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&args[4])?,
            );
            mesh.write_obj(&mut output)?;
            use std::io::Write;
            output.flush()?;
            print_terrain_statistics(&mesh);
            println!(
                "Exported base terrain: {} vertices, {} triangles to {:?}.",
                mesh.positions().len(),
                mesh.triangles().len(),
                args[4]
            );
        }
        Some("export-terrain") => {
            return Err(
                "usage: export-terrain CAPTURE_FILE GLOBE_ID LATITUDE_BANDS NEW_OBJ_FILE".into(),
            );
        }
        Some("sample-terrain") if args.len() == 5 => {
            let world = openshores_client::world::read_capture(&mut io::BufReader::new(
                fs::File::open(&args[1])?,
            ))?;
            let id = if let Some(hex) = args[2].strip_prefix("0x") {
                u32::from_str_radix(hex, 16)?
            } else {
                args[2].parse::<u32>()?
            };
            let surface = world
                .entities
                .get(&id)
                .and_then(|entity| entity.globe_surface)
                .ok_or("selected atom has no decoded globe surface")?;
            let lat = args[3].parse::<f64>()?.to_radians();
            let lon = args[4].parse::<f64>()?.to_radians();
            println!(
                "Globe: {id:#010x}; size={}; atmosphere={}/{}; water={}",
                surface.size, surface.atmosphere_type, surface.atmosphere_density, surface.water
            );
            println!("Terrain parameters: {:?}", surface.terrain.values());
            println!(
                "Base altitude MSL: {:.12}",
                surface.terrain.altitude_msl(surface.size, lat, lon)?
            );
            println!(
                "Surface position: {:?}",
                surface.terrain.surface_position(surface.size, lat, lon)?
            );
        }
        Some("inspect-scene") if args.len() == 2 => {
            let world = openshores_client::world::read_capture(&mut io::BufReader::new(
                fs::File::open(&args[1])?,
            ))?;
            print_world(&world)?;
            for entity in world.entities.values() {
                println!(
                    "{:#010x} {:?} name={:?} parent={:?} transform={:?} globe_surface={:?} person={:?} geology={:?}",
                    entity.id,
                    entity.kind,
                    entity.name,
                    entity.parent,
                    entity.transform,
                    entity.globe_surface,
                    entity.person,
                    entity.geology.as_ref().map(|geo| (
                        geo.features.len(),
                        geo.declared_count,
                        geo.unknown_kind
                    ))
                );
            }
        }
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
        None | Some("--help" | "-h") => {
            println!(
                "Local receive-only scene session:\n       cargo run -- login USERNAME --avatar ID --watch-seconds 10 [--capture NEW_FILE]\n       Duration: 1..60 seconds after bootstrap; loopback only\n"
            );
            println!(
                "Offline terrain statistics:\n       cargo run --offline -- inspect-terrain-demo [LATITUDE_BANDS]\n"
            );
            println!(
                "No-capture offline demo:\n       cargo run --offline --features viewer -- view-terrain-demo [LATITUDE_BANDS] [--unlit] [--smoke-frames N]\n       cargo run --offline -- export-terrain-demo LATITUDE_BANDS NEW_OBJ_FILE\n"
            );
            println!(
                "Offline terrain viewer:\n       cargo run --features viewer -- view-terrain CAPTURE_FILE GLOBE_ID LATITUDE_BANDS [--unlit] [--smoke-frames N]\n"
            );
            println!(
                "Offline base-terrain export:\n       cargo run -- export-terrain CAPTURE_FILE GLOBE_ID LATITUDE_BANDS NEW_OBJ_FILE\n       LATITUDE_BANDS: 3..128; original LOD/geology/water omitted\n"
            );
            println!(
                "Optional offline Bevy preview:\n       cargo run --features viewer -- view-model MODEL_FILE [--unlit] [--smoke-frames 1..3600]\n"
            );
            println!(
                "Raw-coordinate model commands:\n       cargo run -- inspect-model MODEL_FILE\n       cargo run -- audit-model MODEL_FILE\n       cargo run -- verify-models DLL VERSION\n       cargo run -- export-model MODEL_FILE NEW_OBJ_FILE\n       cargo run -- preview-model MODEL_FILE NEW_SVG_FILE\n"
            );
            println!(
                "Qt resource commands (version must be established from registration code):\n       cargo run -- inspect-qt DLL VERSION\n       cargo run -- verify-qt DLL VERSION\n       cargo run -- extract-qt DLL VERSION POOL_INDEX ENTRY_INDEX NEW_OUTPUT_FILE\n"
            );
            println!(
                "OpenShores Rust client foundation\n\nUsage: cargo run -- login USERNAME [IP:PORT] [--password-stdin]\n       cargo run -- login USERNAME [IP:PORT] --avatar ID [--scene-port PORT] [--chat-port PORT] [--capture NEW_FILE] [--password-stdin]\n       cargo run -- inspect-reference [directory]\n       cargo run -- inspect-gd [GD file]\n       cargo run -- inspect-scene CAPTURE_FILE\n       cargo run -- sample-terrain CAPTURE_FILE GLOBE_ID LAT_DEGREES LON_DEGREES\n       cargo run -- inspect-har HAR_FILE\n       cargo run -- verify-har HAR_FILE\n       cargo run -- extract-har HAR_FILE ENTRY_NAME NEW_OUTPUT_FILE\n\nLogin defaults to 127.0.0.1:16757 and prompts for a hidden password.\nAvatar scene probes only use loopback endpoints and stop after the initial manifest.\nInventory excludes symlinks, .git, __pycache__, and .venv."
            );
        }
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
}
