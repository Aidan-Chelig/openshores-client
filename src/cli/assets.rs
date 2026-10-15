//! User-provided asset/resource commands; original files are never executed.
use std::fs;
use std::io::Read;

pub(super) fn run(args: &[String]) -> Result<bool, Box<dyn std::error::Error>> {
    match args.first().map(String::as_str) {
        Some("inspect-mounted-har") if (3..=4).contains(&args.len()) => {
            let mount = openshores_client::asset_mount::AssetMount::open(&args[1])?;
            let bytes = mount.read_resource(&args[2])?;
            let archive = openshores_client::har::HarArchive::open_sequential(&bytes)?;
            if let Some(name) = args.get(3) {
                println!("{}", std::str::from_utf8(&archive.read_entry(name)?)?);
            } else {
                for entry in archive.entries() {
                    println!("{}", entry.name);
                }
            }
        }
        Some("inspect-mounted-model") if args.len() == 3 => {
            let mount = openshores_client::asset_mount::AssetMount::open(&args[1])?;
            let bytes = mount.read_resource(&args[2])?;
            let variant = openshores_client::model::ModelVariant::from_name(&args[2])?;
            let model = openshores_client::model::Model::decode(&bytes, variant)?;
            println!(
                "Mounted model {:?}: {} meshes, {} vertices, {} faces",
                args[2],
                model.meshes.len(),
                model
                    .meshes
                    .iter()
                    .map(|mesh| mesh.vertices.len())
                    .sum::<usize>(),
                model
                    .meshes
                    .iter()
                    .map(|mesh| mesh.faces.len())
                    .sum::<usize>()
            );
        }
        Some("prepare-assets") if args.len() == 3 => {
            let prepared = openshores_client::asset_mount::cache::prepare(&args[1], &args[2])?;
            println!(
                "Prepared {} assets; {} geometry conversions; {} opaque models; cache {}",
                prepared.mount.entries().len(),
                prepared.converted_models,
                prepared.opaque_models,
                prepared.cache.display()
            );
        }
        Some("inspect-mount" | "verify-mount") if args.len() == 2 => {
            let mount = openshores_client::asset_mount::AssetMount::open(&args[1])?;
            println!(
                "Mounted {} assets from {}",
                mount.entries().len(),
                mount.root().display()
            );
            for (key, entry) in mount.entries() {
                if args[0] == "verify-mount" {
                    mount.read(key)?;
                }
                println!("{key}\t{}", entry.bytes);
            }
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

        _ => return Ok(false),
    }
    Ok(true)
}
