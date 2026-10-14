use openshores_client::login::{self as wire_login, LoginReply};
use std::io::{IsTerminal, Read};
use std::net::SocketAddr;
use std::time::Duration;
use std::{env, fs, io, path::Path};
use zeroize::Zeroizing;

mod assets;
mod terrain;
#[cfg(test)]
use terrain::{terrain_demo_options, viewer_options};
mod login;
mod models;
mod native_capture;
mod native_item;
mod native_item_replay;
mod native_prefix;
mod positions;
use login::login_command;
#[cfg(test)]
use login::password_from_stdin;

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
    println!(
        "Commodity catalog: {} server-provided rows.",
        world.commodities.len()
    );
    println!("Missing manifest IDs: {:?}", world.missing_manifest_ids());
    println!("Unresolved parents: {:?}", world.unresolved_parents());
    println!("Opaque packet counts: {:?}", world.opaque_packet_counts);
    if let Some(id) = world.active_unit {
        if let Some(gear) = world
            .entities
            .get(&id)
            .and_then(|entity| entity.person.as_ref())
            .and_then(|person| person.gear.as_ref())
        {
            let labels: Vec<_> = gear
                .slots
                .iter()
                .filter_map(|slot| {
                    slot.item.as_ref().map(|item| {
                        let cid = item.common.commodity_id;
                        (
                            slot.slot_type,
                            slot.subindex,
                            cid,
                            world
                                .commodities
                                .get(i32::from(cid))
                                .and_then(|row| row.name.as_deref()),
                        )
                    })
                })
                .collect();
            println!("Player gear labels (latest received batch): {labels:?}");
        }
        let chain = world.ancestry(id)?;
        let labels: Vec<_> = chain
            .iter()
            .map(|id| format!("{:?}({id:#010x})", world.entities[id].kind))
            .collect();
        println!("Player ancestry: {}", labels.join(" -> "));
    }
    Ok(())
}

pub(super) fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().skip(1).collect();
    if models::run(&args)? {
        return Ok(());
    }
    if positions::run(&args)? {
        return Ok(());
    }
    if native_prefix::run(&args)? {
        return Ok(());
    }
    if native_capture::run(&args)? {
        return Ok(());
    }
    if native_item::run(&args)? {
        return Ok(());
    }
    if native_item_replay::run(&args)? {
        return Ok(());
    }
    if assets::run(&args)? || terrain::run(&args)? {
        return Ok(());
    }
    match args.first().map(String::as_str) {
        Some("inspect-chat") if args.len() == 2 => {
            let data = fs::read(&args[1])?;
            println!("{:?}", openshores_client::chat::decode_incoming(&data)?);
        }
        Some("login") => login_command(&args[1..])?,
        Some("inspect-commodities") if args.len() == 2 => {
            let world = openshores_client::world::read_capture(&mut io::BufReader::new(
                fs::File::open(&args[1])?,
            ))?;
            println!(
                "Commodity catalog: {} server-provided rows.",
                world.commodities.len()
            );
            for row in world.commodities.rows() {
                println!(
                    "{}: name={:?} description={:?} bitmap={:?} model={:?} flags={:#x} unparsed={}B",
                    row.id,
                    row.name,
                    row.description,
                    row.bitmap,
                    row.model,
                    row.flags,
                    row.unparsed_suffix.len()
                );
            }
        }
        Some("inspect-scene") if args.len() == 2 => {
            let world = openshores_client::world::read_capture(&mut io::BufReader::new(
                fs::File::open(&args[1])?,
            ))?;
            print_world(&world)?;
            for entity in world.entities.values() {
                println!(
                    "{:#010x} {:?} name={:?} parent={:?} transform={:?} globe_surface={:?} person={:?} item={:?} geology={:?}",
                    entity.id,
                    entity.kind,
                    entity.name,
                    entity.parent,
                    entity.transform,
                    entity.globe_surface,
                    entity.person,
                    entity.item,
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
                "Offline native item readers (not a scene packet):\n       cargo run -- inspect-native-item-common HEX_COMMON_RECORD\n       cargo run -- inspect-native-item-value HEX_TYPE_AND_RECORD\n"
            );
            println!(
                "Offline native capture replay (prefix-only or established items):\n       cargo run -- inspect-native-capture CAPTURE_FILE\n       cargo run -- inspect-native-item-capture CAPTURE_FILE\n       cargo run -- replay-native-items CAPTURE_FILE\n"
            );
            println!(
                "Offline patched native atom prefix (Qt 4.0; other opcodes stay opaque):\n       cargo run -- inspect-native-atom HEX_UNFRAMED_PAYLOAD\n"
            );
            println!(
                "Offline native ID/base prefix (after opcode; unknown layers retained):\n       cargo run -- inspect-native-base type-widths|single|double HEX_FIELDS_AFTER_OPCODE\n"
            );
            println!(
                "Offline native packed-field inspection (explicit Qt encoding; not a scene frame):\n       cargo run -- inspect-atom-position type-widths|single|double HEX_RECORD\n       cargo run -- dump-atom-matrix type-widths|single|double HEX_RECORD\n       cargo run -- dump-atom-rotation-vectors type-widths|single|double HEX_RECORD\n"
            );
            println!(
                "Top-level local item-use test (requires --avatar and --watch-seconds 3..60):\n       --use-gear SRC_SLOT SRC_SUB\n       One request after 1 second; slots 1..9, subindices 0..15; cannot combine with other gameplay actions except look-at\n"
            );
            println!(
                "Server-provided commodity labels/resources:\n       cargo run -- inspect-commodities CAPTURE_FILE\n"
            );
            println!(
                "Raw local pose-report test (requires --avatar and --watch-seconds):\n       --pose-report HEX20\n       Ten raw pose bytes; auxiliary trigger fields zero; cannot combine with position/pickup/move/drop\n"
            );
            println!(
                "Raw local position-report test (requires --avatar and --watch-seconds):\n       --position-report X Y Z RX RY RZ\n       Finite raw wire coordinates/rotation, not walking controls; cannot combine with pickup/move/drop\n"
            );
            println!(
                "Local reference-server pickup test profile:\n       --pickup-target NUMERIC_ITEM_ID\n       Requires --avatar and --watch-seconds 3..60; cannot combine with move/drop\n"
            );
            println!(
                "Top-level local inventory drop (requires --avatar and --watch-seconds):\n       --drop-gear SRC_SLOT SRC_SUB\n       Slot range: 1..9; subindex: 0..15; cannot combine with --move-gear\n"
            );
            println!(
                "Explicit local gameplay commands (require --avatar and --watch-seconds):\n       --look-at NUMERIC_ID\n       --move-gear SRC_SLOT SRC_SUB DST_SLOT\n       Slot/subindex range: 0..15; destination subindex is server-assigned\n"
            );
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

#[cfg(test)]
mod tests;
