//! Explicit fixture-DNA diagnostic, not the production Person model generator.
use openshores_client::{
    asset_mount::AssetMount,
    model::{Mesh, Model, ModelVariant, NativeBodyFaceMetadata, NativeBodyMergeGeometry},
    world::{
        native_body::{NativeDpHandCache, NativeDpInitialization, NativeDpSkinPlan},
        native_dna::NativeDhDna,
    },
};
use std::{io, path::Path};

pub(super) fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() < 2 || !matches!(args[1].as_str(), "head" | "torso") {
        return Err(
            "usage: view-native-body INSTALLATION head|torso [--smooth-angle DEGREES] [--unlit] [--smoke-frames N]".into(),
        );
    }
    let (angle, options) = smoothing_options(&args[2..])?;
    let (frames, unlit) = super::terrain::viewer_options(&options)?;
    let model = build(Path::new(&args[0]), args[1] == "head")?;
    println!(
        "Offline merged {}: {} vertices, {} faces. Fixture DNA; untextured and unposed. Drag to orbit, wheel to zoom, L lighting, W wire, R reset, Esc quit.",
        args[1],
        model.meshes[0].vertices.len(),
        model.meshes[0].faces.len()
    );
    println!(
        "Preview auto smoothing: {angle} degrees (0 = flat). Source normal records are unchanged."
    );
    openshores_client::viewer::run_auto_smooth(&model, frames, unlit, angle)?;
    Ok(())
}

fn smoothing_options(args: &[String]) -> Result<(f64, Vec<String>), Box<dyn std::error::Error>> {
    let mut angle = None;
    let mut remaining = Vec::new();
    let mut options = args.iter();
    while let Some(option) = options.next() {
        if option == "--smooth-angle" {
            if angle.is_some() {
                return Err("duplicate --smooth-angle".into());
            }
            let value: f64 = options
                .next()
                .ok_or("--smooth-angle requires degrees")?
                .parse()?;
            if !value.is_finite() || !(0.0..=180.0).contains(&value) {
                return Err("smoothing angle must be finite and in 0..=180 degrees".into());
            }
            angle = Some(value);
        } else {
            remaining.push(option.clone());
        }
    }
    Ok((angle.unwrap_or(60.0), remaining))
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, message)
}

fn build(installation: &Path, head: bool) -> io::Result<Model> {
    let mount = AssetMount::open(installation)?;
    let mut dna = NativeDhDna([0; 24]);
    dna.0[1] = 1;
    dna.0[12] = 4;
    dna.0[14] = 0x51;
    dna.0[20] = 0x60;
    let mut body = NativeDpInitialization::begin(&dna);
    body.finish_part_calculations(0, &mut NativeDpHandCache::new(&mount))?;
    let mut pieces = Vec::new();
    for planned in body.skin_geometry_plans(0, 0, [1.; 4])? {
        match planned.plan {
            NativeDpSkinPlan::Head(plan) if head => {
                for piece in plan.fallback {
                    pieces.push((
                        piece.resource,
                        piece.geometry,
                        piece.bones,
                        piece
                            .merge_tolerance
                            .ok_or_else(|| invalid("missing head merge tolerance"))?,
                    ));
                }
            }
            NativeDpSkinPlan::Torso(Some(plan)) if !head => {
                for piece in plan.fallback {
                    if piece.crest {
                        return Err(invalid("crest merge is not implemented"));
                    }
                    pieces.push((
                        piece.resource,
                        piece.geometry,
                        piece.bones,
                        plan.merge_tolerance,
                    ));
                }
            }
            _ => {}
        }
    }
    let mut destination: Option<NativeBodyMergeGeometry> = None;
    let mut projection_source = None;
    for (resource, definition, bones, tolerance) in pieces {
        let model = Model::decode(&mount.read_resource(&resource)?, ModelVariant::N)?;
        if model.meshes.len() != 1 {
            return Err(invalid("preview requires exactly one mesh per resource"));
        }
        let mesh = &model.meshes[0];
        let mut metadata = NativeBodyFaceMetadata {
            smoothing: mesh.smoothing.clone().unwrap_or_default(),
            ..Default::default()
        };
        for material in &mesh.materials {
            if !material.name.is_ascii()
                || metadata
                    .materials
                    .insert(
                        material.name.iter().map(|byte| u16::from(*byte)).collect(),
                        material.faces.clone(),
                    )
                    .is_some()
            {
                return Err(invalid("preview requires unique ASCII material names"));
            }
        }
        let mut source = mesh.body_merge_geometry(
            ModelVariant::N,
            &definition,
            bones.map(|bone| bone as i32),
            metadata,
        )?;
        if let Some(destination) = destination.as_mut() {
            if head {
                destination.append_head_geometry_stages(&mut source, ModelVariant::N, tolerance)?;
            } else {
                destination.append_torso_geometry_stages(
                    &mut source,
                    ModelVariant::N,
                    tolerance,
                )?;
            }
        } else {
            destination = Some(source);
            projection_source = model.meshes.into_iter().next();
        }
    }
    let geometry = destination
        .ok_or_else(|| invalid("no preview pieces"))?
        .untextured_baked_preview_geometry(
            &projection_source.ok_or_else(|| invalid("no projection seed"))?,
            ModelVariant::N,
        )?;
    // The inspector centers/scales for viewing only; native mesh state is untouched.
    Ok(Model {
        meshes: vec![Mesh {
            vertices: geometry
                .positions
                .iter()
                .map(|point| point.map(|v| v as f32))
                .collect(),
            faces: geometry.faces,
            ..Default::default()
        }],
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoothing_options_validate_angles_and_preserve_viewer_flags() {
        let parse = |args: &[&str]| {
            super::smoothing_options(&args.iter().map(|s| s.to_string()).collect::<Vec<_>>())
        };
        assert_eq!(parse(&[]).unwrap().0, 60.);
        assert_eq!(
            parse(&["--smooth-angle", "0", "--unlit"]).unwrap(),
            (0., vec!["--unlit".to_string()])
        );
        assert_eq!(parse(&["--smooth-angle", "180"]).unwrap().0, 180.);
        for args in [
            vec!["--smooth-angle"],
            vec!["--smooth-angle", "NaN"],
            vec!["--smooth-angle", "inf"],
            vec!["--smooth-angle", "-1"],
            vec!["--smooth-angle", "181"],
            vec!["--smooth-angle", "30", "--smooth-angle", "60"],
        ] {
            assert!(parse(&args).is_err());
        }
    }
    #[test]
    #[ignore = "requires OPENSHORES_HAZERON_INSTALLATION; offline, no window"]
    fn mounted_visual_preview_geometry() {
        let path = std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").unwrap();
        for (head, vertices, faces) in [(true, 902, 1228), (false, 639, 996)] {
            let model = super::build(std::path::Path::new(&path), head).unwrap();
            assert_eq!(model.meshes[0].vertices.len(), vertices);
            assert_eq!(model.meshes[0].faces.len(), faces);
            assert!(
                model.meshes[0]
                    .vertices
                    .iter()
                    .flatten()
                    .all(|v| v.is_finite())
            );
        }
    }
}
