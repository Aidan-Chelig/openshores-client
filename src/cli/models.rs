//! Offline model diagnostics, separate from connected client commands.
#[cfg(feature = "viewer")]
use super::terrain::viewer_options;
use std::{
    fs,
    io::{self, Read},
};

pub(super) fn run(args: &[String]) -> Result<bool, Box<dyn std::error::Error>> {
    match args.first().map(String::as_str) {
        Some("dump-model-roles") if args.len() == 2 => {
            let mut data = Vec::new();
            fs::File::open(&args[1])?
                .take(64 * 1024 * 1024 + 1)
                .read_to_end(&mut data)?;
            let variant = openshores_client::model::ModelVariant::from_name(&args[1])?;
            let model = openshores_client::model::Model::decode(&data, variant)?;
            let roles: Vec<_> = model
                .mesh_roles(variant)
                .into_iter()
                .map(|role| role.label())
                .collect();
            println!("{roles:?}");
        }
        Some("inspect-model-groups") if args.len() == 2 => {
            let mut data = Vec::new();
            fs::File::open(&args[1])?
                .take(64 * 1024 * 1024 + 1)
                .read_to_end(&mut data)?;
            let variant = openshores_client::model::ModelVariant::from_name(&args[1])?;
            let model = openshores_client::model::Model::decode(&data, variant)?;
            println!("groups={}", model.vertex_group_digest());
        }
        Some("dump-model-coordinates") if (2..=3).contains(&args.len()) => {
            let scale = args
                .get(2)
                .map(|value| value.parse::<f64>())
                .transpose()?
                .unwrap_or(1.0);
            let mut data = Vec::new();
            fs::File::open(&args[1])?
                .take(64 * 1024 * 1024 + 1)
                .read_to_end(&mut data)?;
            let variant = openshores_client::model::ModelVariant::from_name(&args[1])?;
            let model = openshores_client::model::Model::decode(&data, variant)?;
            let points = model
                .meshes
                .iter()
                .map(|mesh| mesh.loaded_positions(variant, scale))
                .collect::<io::Result<Vec<_>>>()?;
            // Numeric arrays are JSON-compatible; all values were validated finite.
            println!("{points:?}");
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
        _ => return Ok(false),
    }
    Ok(true)
}
