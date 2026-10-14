//! Offline terrain commands and optional viewer argument validation.
use std::{fs, io};

#[cfg(any(feature = "viewer", test))]
pub(super) fn viewer_options(
    args: &[String],
) -> Result<(Option<u32>, bool), Box<dyn std::error::Error>> {
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
pub(super) fn terrain_demo_options(
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

pub(super) fn run(args: &[String]) -> Result<bool, Box<dyn std::error::Error>> {
    match args.first().map(String::as_str) {
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

        _ => return Ok(false),
    }
    Ok(true)
}
