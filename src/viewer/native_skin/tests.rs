use super::*;
use crate::model::{Face, NativeBodyMeshGeometry, NativeBodyVertexWeights};
use std::io::Write;

fn png() -> Vec<u8> {
    fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
        out.extend((data.len() as u32).to_be_bytes());
        out.extend(kind);
        out.extend(data);
        let mut crc = !0_u32;
        for byte in kind.iter().chain(data) {
            crc ^= u32::from(*byte);
            for _ in 0..8 {
                crc = (crc >> 1) ^ (0xedb88320_u32.wrapping_mul(crc & 1));
            }
        }
        out.extend((!crc).to_be_bytes());
    }
    let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut header = Vec::new();
    header.extend(1_u32.to_be_bytes());
    header.extend(2_u32.to_be_bytes());
    header.extend([8, 6, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &header);
    let mut zip = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    zip.write_all(&[0, 255, 0, 0, 128, 0, 0, 0, 255, 255])
        .unwrap();
    chunk(&mut out, b"IDAT", &zip.finish().unwrap());
    chunk(&mut out, b"IEND", &[]);
    out
}

#[test]
fn png_decoding_flips_rows_retains_alpha_and_sets_sampler() {
    let image = decode_atlas(&png()).unwrap();
    assert_eq!((image.width(), image.height()), (1, 2));
    assert_eq!(image.data.unwrap(), [0, 0, 255, 255, 255, 0, 0, 128]);
    assert_eq!(
        image.texture_descriptor.format,
        TextureFormat::Rgba8UnormSrgb
    );
    let ImageSampler::Descriptor(sampler) = image.sampler else {
        panic!("missing sampler")
    };
    assert_eq!(sampler.mag_filter, ImageFilterMode::Linear);
    assert_eq!(sampler.min_filter, ImageFilterMode::Linear);
    assert_eq!(sampler.address_mode_u, ImageAddressMode::Repeat);
}

#[test]
fn png_errors_are_explicit_and_dimensions_are_bounded_before_decode() {
    let png = png();
    for length in [0, 8, 16, 23, 35, png.len() - 4] {
        assert!(decode_atlas(&png[..length]).is_err());
    }
    let mut oversized = png.clone();
    oversized[16..20].copy_from_slice(&4097_u32.to_be_bytes());
    assert!(
        decode_atlas(&oversized)
            .unwrap_err()
            .to_string()
            .contains("dimensions")
    );
    let mut corrupt = png;
    corrupt[40] ^= 0xff;
    assert!(decode_atlas(&corrupt).is_err());
    assert!(decode_atlas(b"not a PNG").is_err());
}

fn surface() -> NativeBodySurfacePreview {
    NativeBodySurfacePreview {
        geometry: NativeBodyMeshGeometry {
            positions: vec![
                [0., 0., 0.],
                [1., 0., 0.],
                [0., 1., 0.],
                [0., 0., 0.],
                [0., 0., 1.],
                [1., 0., 0.],
            ],
            weights: vec![NativeBodyVertexWeights::default(); 6],
            faces: vec![[0, 1, 2], [3, 4, 5]]
                .into_iter()
                .map(|vertices| Face {
                    vertices,
                    flags: 0,
                    extension: None,
                    nt_extension: None,
                })
                .collect(),
            uvs: Some(vec![
                [0., 0.],
                [0.1, 0.],
                [0., 0.1],
                [0.5, 0.5],
                [0.5, 0.6],
                [0.6, 0.5],
            ]),
        },
        top_face: vec![1., 1., 1., 0., 0., 0.],
        material_memberships: vec![vec![vec![65]], vec![vec![66]]],
    }
}

#[test]
fn uv_seams_and_material_splits_preserve_global_angle_smoothing() {
    use bevy::mesh::VertexAttributeValues::Float32x3;
    let surface = surface();
    let positions = surface
        .geometry
        .positions
        .iter()
        .map(|p| p.map(|v| v as f32))
        .collect();
    let display = DisplayMesh {
        positions,
        triangles: vec![[0, 1, 2], [3, 4, 5]],
        colors: None,
    };
    let smooth = render_meshes(&surface, &display, 91.).unwrap();
    let hard = render_meshes(&surface, &display, 89.).unwrap();
    assert_eq!(smooth.len(), 2);
    let get = |mesh: &Mesh| match mesh.attribute(Mesh::ATTRIBUTE_NORMAL).unwrap() {
        Float32x3(v) => v[0],
        _ => panic!(),
    };
    assert_eq!(get(&smooth[0]), get(&smooth[1]));
    assert_ne!(get(&hard[0]), get(&hard[1]));
    for (part, mesh) in smooth.iter().enumerate() {
        assert!(mesh.indices().is_none()); // every triangle corner has independent attributes
        let Some(bevy::mesh::VertexAttributeValues::Float32x2(uv)) =
            mesh.attribute(Mesh::ATTRIBUTE_UV_0)
        else {
            panic!()
        };
        assert_eq!(
            uv,
            &surface.geometry.uvs.as_ref().unwrap()[part * 3..part * 3 + 3]
        );
        let Some(bevy::mesh::VertexAttributeValues::Float32x4(control)) =
            mesh.attribute(Mesh::ATTRIBUTE_COLOR)
        else {
            panic!()
        };
        assert!(
            control
                .iter()
                .all(|c| c[3] == if part == 0 { 1. } else { 0. })
        );
    }
    assert_eq!(surface.geometry.positions[0], surface.geometry.positions[3]);
    assert_ne!(
        surface.geometry.uvs.as_ref().unwrap()[0],
        surface.geometry.uvs.as_ref().unwrap()[3]
    );
}

#[test]
fn malformed_render_inputs_fail_instead_of_panicking() {
    let mut surface = surface();
    let display = DisplayMesh {
        positions: vec![[0.; 3]; 6],
        triangles: vec![[0, 1, 2], [3, 4, 5]],
        colors: None,
    };
    surface.top_face.pop();
    assert!(render_meshes(&surface, &display, 60.).is_err());
    surface.top_face.push(f32::NAN);
    assert!(render_meshes(&surface, &display, 60.).is_err());
}

#[test]
fn setup_attaches_atlas_offsets_mesh_attributes_and_lighting_mode() {
    let source = surface();
    let display = DisplayMesh {
        positions: source
            .geometry
            .positions
            .iter()
            .map(|p| p.map(|v| v as f32))
            .collect(),
        triangles: vec![[0, 1, 2], [3, 4, 5]],
        colors: None,
    };
    let mut app = App::new();
    app.init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<Image>>()
        .init_resource::<Assets<SkinMaterial>>()
        .init_resource::<Orbit>()
        .insert_resource(SkinPreview {
            view: None,
            meshes: render_meshes(&source, &display, 60.).unwrap(),
            image: Some(decode_atlas(&png()).unwrap()),
            offsets: Vec4::new(0.25, 0.75, 0.5, 0.),
            screenshot: None,
            frames: 0,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, lighting);
    app.update();
    let handles: Vec<_> = app
        .world_mut()
        .query::<&MeshMaterial3d<SkinMaterial>>()
        .iter(app.world())
        .map(|m| m.0.clone())
        .collect();
    assert_eq!(handles.len(), 2);
    assert_eq!(handles[0], handles[1]);
    let material = app
        .world()
        .resource::<Assets<SkinMaterial>>()
        .get(&handles[0])
        .unwrap();
    assert_eq!(material.extension.offsets, Vec4::new(0.25, 0.75, 0.5, 0.));
    assert_eq!(material.base.alpha_mode, AlphaMode::Mask(0.15));
    assert!(!material.base.unlit);
    assert_eq!(
        app.world()
            .resource::<Assets<Image>>()
            .get(&material.extension.atlas)
            .unwrap()
            .data
            .as_ref()
            .unwrap(),
        &[0, 0, 255, 255, 255, 0, 0, 128]
    );
    app.world_mut().resource_mut::<Orbit>().lit = false;
    app.update();
    assert!(
        app.world()
            .resource::<Assets<SkinMaterial>>()
            .get(&handles[0])
            .unwrap()
            .base
            .unlit
    );
}

#[test]
#[ignore = "requires OPENSHORES_HAZERON_INSTALLATION; offline atlas decode, no GPU"]
fn mounted_atlas_decode() {
    use crate::{asset_mount::AssetMount, world::native_body::NativeDpSkinAtlas};
    let mount =
        AssetMount::open(std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").unwrap()).unwrap();
    for atlas in [
        NativeDpSkinAtlas::Skin,
        NativeDpSkinAtlas::Exo,
        NativeDpSkinAtlas::Scales,
        NativeDpSkinAtlas::FurFeathers,
    ] {
        let image = decode_atlas(&atlas.read_skin(&mount).unwrap()).unwrap();
        assert_eq!((image.width(), image.height()), (2048, 2048));
    }
}

#[test]
#[ignore = "requires OPENSHORES_AVATAR_SCREENSHOT_DIR with head.png and torso.png GPU captures"]
fn captured_previews_are_nonblank() {
    let directory =
        std::path::PathBuf::from(std::env::var_os("OPENSHORES_AVATAR_SCREENSHOT_DIR").unwrap());
    for part in ["head", "torso"] {
        let bytes = std::fs::read(directory.join(format!("{part}.png"))).unwrap();
        let image = decode_atlas(&bytes).unwrap();
        let pixels = image.data.unwrap();
        let background = &pixels[..4];
        let foreground = pixels
            .chunks_exact(4)
            .filter(|pixel| *pixel != background)
            .count();
        let colors: std::collections::BTreeSet<_> = pixels
            .chunks_exact(4)
            .map(|pixel| <[u8; 4]>::try_from(pixel).unwrap())
            .collect();
        assert!(
            foreground > pixels.len() / 4 / 100,
            "{part}: blank or tiny GPU render"
        );
        assert!(
            colors.len() > 64,
            "{part}: insufficient rendered color variation"
        );
        println!(
            "{part}: {foreground} foreground pixels, {} colors",
            colors.len()
        );
    }
}

#[test]
#[ignore = "requires OPENSHORES_AVATAR_SCREENSHOT_DIR with both body fixtures/four views and untextured captures"]
fn captured_bodies_are_nonblank() {
    let directory =
        std::path::PathBuf::from(std::env::var_os("OPENSHORES_AVATAR_SCREENSHOT_DIR").unwrap());
    let mut names = vec!["body-base-untextured".to_string()];
    for fixture in ["base", "many"] {
        for view in ["oblique", "front", "side", "back"] {
            names.push(format!("body-{fixture}-{view}"));
        }
    }
    for name in names {
        let image =
            decode_atlas(&std::fs::read(directory.join(format!("{name}.png"))).unwrap()).unwrap();
        let pixels = image.data.unwrap();
        let background = &pixels[..4];
        let foreground = pixels.chunks_exact(4).filter(|p| *p != background).count();
        let colors: std::collections::BTreeSet<_> = pixels.chunks_exact(4).collect();
        assert!(
            foreground > pixels.len() / 4 / 100,
            "{name}: blank/tiny render"
        );
        assert!(colors.len() > 64, "{name}: insufficient color variation");
        println!(
            "{name}: {foreground} foreground pixels, {} colors",
            colors.len()
        );
    }
}
