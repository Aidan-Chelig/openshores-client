//! Explicit fixture-DNA diagnostic, not the production Person model generator.
use openshores_client::{
    asset_mount::AssetMount,
    model::{Mesh, Model, ModelVariant, NativeBodyFaceMetadata, NativeBodyMergeGeometry},
    world::{
        native_body::{
            NativeDpAvatarGeometry, NativeDpHandCache, NativeDpInitialization, NativeDpSkinPlan,
            native_dp_skin_offsets,
        },
        native_dna::NativeDhDna,
    },
};
use std::{io, path::Path};

pub(super) fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() < 2 || !matches!(args[1].as_str(), "head" | "torso" | "body") {
        return Err(
            "usage: view-native-body INSTALLATION head|torso|body [--fixture base|many] [--view oblique|front|side|back] [--native-seams] [--smooth-angle DEGREES] [--untextured] [--screenshot NEW_PNG] [--unlit] [--smoke-frames N]".into(),
        );
    }
    let (angle, options) = smoothing_options(&args[2..])?;
    let mut untextured = false;
    let mut screenshot = None;
    let mut flags = Vec::new();
    let mut fixture = None;
    let mut view = None;
    let mut native_seams = false;
    let mut options = options.iter();
    while let Some(option) = options.next() {
        match option.as_str() {
            "--native-seams" if !native_seams => native_seams = true,
            "--fixture" if fixture.is_none() => {
                fixture = Some(
                    options
                        .next()
                        .ok_or("--fixture requires base or many")?
                        .as_str(),
                )
            }
            "--view" if view.is_none() => {
                view = Some(
                    options
                        .next()
                        .ok_or("--view requires oblique, front, side or back")?
                        .as_str(),
                )
            }
            "--untextured" if !untextured => untextured = true,
            "--screenshot" if screenshot.is_none() => {
                screenshot = Some(std::path::PathBuf::from(
                    options
                        .next()
                        .ok_or("--screenshot requires a new PNG path")?,
                ))
            }
            "--untextured" | "--screenshot" | "--fixture" | "--view" | "--native-seams" => {
                return Err("duplicate preview option".into());
            }
            _ => flags.push(option.clone()),
        }
    }
    let (frames, unlit) = super::terrain::viewer_options(&flags)?;
    if args[1] == "body" {
        use openshores_client::viewer::{
            NativeAvatarView, NativeSkinPreviewOptions, run_textured_avatar,
        };
        let dna = match fixture.unwrap_or("base") {
            "base" => fixture_dna(),
            "many" => {
                let mut dna = fixture_dna();
                dna.0[1] = 2;
                dna.0[12] = 8;
                dna.0[14] = 0x91;
                dna.0[20] = 0x80;
                dna.0[5] = 2;
                dna.0[13] = 0x60;
                dna.0[15] = 0x40;
                dna
            }
            _ => return Err("--fixture must be base or many".into()),
        };
        let view = match view.unwrap_or("oblique") {
            "oblique" => NativeAvatarView::Oblique,
            "front" => NativeAvatarView::Front,
            "side" => NativeAvatarView::Side,
            "back" => NativeAvatarView::Back,
            _ => return Err("--view must be oblique, front, side or back".into()),
        };
        let mount = AssetMount::open(&args[0])?;
        validate_screenshot(&mount, screenshot.as_deref())?;
        let avatar = NativeDpAvatarGeometry::generate(&mount, &dna, 0, 0)?;
        let atlas = if untextured {
            None
        } else {
            Some(avatar.skin.read_skin(&mount)?)
        };
        run_textured_avatar(
            &avatar,
            atlas.as_deref(),
            NativeSkinPreviewOptions {
                smoke_frames: frames,
                unlit,
                angle_degrees: angle,
                screenshot,
            },
            view,
            !native_seams,
        )?;
        return Ok(());
    }
    if fixture.is_some() || view.is_some() || native_seams {
        return Err("--fixture, --view and --native-seams are body-only options".into());
    }
    if !untextured {
        let mount = AssetMount::open(&args[0])?;
        validate_screenshot(&mount, screenshot.as_deref())?;
        let (native, seed) = build_native(Path::new(&args[0]), args[1] == "head")?;
        let surface = native.textured_baked_preview_geometry(&seed, ModelVariant::N)?;
        let dna = fixture_dna();
        let atlas = openshores_client::world::native_body::NativeDpSkinAtlas::from_dna(&dna);
        println!(
            "Offline merged {}: {} vertices, {} faces, {}. Fixture DNA; unposed; smoothing {angle} degrees.",
            args[1],
            surface.geometry.positions.len(),
            surface.geometry.faces.len(),
            atlas.skin_path()
        );
        openshores_client::viewer::run_textured_body(
            &surface,
            &atlas.read_skin(&mount)?,
            native_dp_skin_offsets(&dna),
            openshores_client::viewer::NativeSkinPreviewOptions {
                smoke_frames: frames,
                unlit,
                angle_degrees: angle,
                screenshot,
            },
        )?;
        return Ok(());
    }
    if screenshot.is_some() {
        return Err("--screenshot currently requires textured mode".into());
    }
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

fn validate_screenshot(mount: &AssetMount, path: Option<&Path>) -> io::Result<()> {
    if let Some(path) = path {
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        if parent.canonicalize()?.starts_with(mount.root()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "screenshot must be outside the Hazeron installation",
            ));
        }
    }
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

fn fixture_dna() -> NativeDhDna {
    let mut dna = NativeDhDna([0; 24]);
    dna.0[1] = 1;
    dna.0[12] = 4;
    dna.0[14] = 0x51;
    dna.0[20] = 0x60;
    dna
}

fn build_native(installation: &Path, head: bool) -> io::Result<(NativeBodyMergeGeometry, Mesh)> {
    let mount = AssetMount::open(installation)?;
    let dna = fixture_dna();
    let mut body = NativeDpInitialization::begin(&dna);
    body.finish_part_calculations(0, &mut NativeDpHandCache::new(&mount))?;
    let mut pieces = Vec::new();
    for planned in body.skin_geometry_plans(0, 0, native_dp_skin_offsets(&dna))? {
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
    Ok((
        destination.ok_or_else(|| invalid("no preview pieces"))?,
        projection_source.ok_or_else(|| invalid("no projection seed"))?,
    ))
}

fn build(installation: &Path, head: bool) -> io::Result<Model> {
    let (destination, source) = build_native(installation, head)?;
    let geometry = destination.untextured_baked_preview_geometry(&source, ModelVariant::N)?;
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
    fn body_options_reject_invalid_or_duplicate_inputs_before_mounting() {
        for flags in [
            vec!["--fixture", "unknown"],
            vec!["--fixture"],
            vec!["--view", "unknown"],
            vec!["--view"],
            vec!["--fixture", "base", "--fixture", "many"],
            vec!["--native-seams", "--native-seams"],
            vec!["--view", "front", "--view", "back"],
        ] {
            let args = [vec!["absent-installation", "body"], flags].concat();
            let err = super::run(&args.iter().map(|s| s.to_string()).collect::<Vec<_>>())
                .unwrap_err()
                .to_string();
            assert!(
                err.contains("--fixture") || err.contains("--view") || err.contains("duplicate"),
                "{err}"
            );
        }
        for part in ["head", "torso"] {
            let args = ["absent-installation", part, "--view", "front"].map(String::from);
            assert!(
                super::run(&args)
                    .unwrap_err()
                    .to_string()
                    .contains("body-only")
            );
        }
    }
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
            let (native, seed) = super::build_native(std::path::Path::new(&path), head).unwrap();
            let surface = native
                .textured_baked_preview_geometry(&seed, super::ModelVariant::N)
                .unwrap();
            let uvs = surface.geometry.uvs.as_ref().unwrap();
            assert_eq!(uvs.len(), vertices);
            assert!(uvs.iter().flatten().all(|v| v.is_finite()));
            assert_eq!(surface.top_face.len(), vertices);
            assert!(surface.top_face.iter().all(|v| *v == 0. || *v == 1.));
            assert_eq!(
                surface.partitions().values().map(Vec::len).sum::<usize>(),
                faces
            );
            println!(
                "{}: {} UVs, {} bottom vertices, {} material partitions",
                if head { "head" } else { "torso" },
                uvs.len(),
                surface.top_face.iter().filter(|v| **v == 0.).count(),
                surface.partitions().len()
            );
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
