//! Mounted atlas decoding and render-only surface splits. No native asset is embedded.
use super::*;
use crate::model::{Mesh as SourceMesh, NativeBodySurfacePreview};
use bevy::{
    asset::embedded_asset,
    image::{
        CompressedImageFormats, ImageAddressMode, ImageFilterMode, ImageSampler,
        ImageSamplerDescriptor, ImageType,
    },
    light::NotShadowCaster,
    pbr::{ExtendedMaterial, MaterialExtension},
    render::{
        render_resource::{AsBindGroup, TextureFormat},
        view::screenshot::{Screenshot, ScreenshotCaptured},
    },
    shader::ShaderRef,
};
use std::path::PathBuf;
mod avatar;
pub use avatar::{NativeAvatarView, run_textured_avatar};

pub struct NativeSkinPreviewOptions {
    pub smoke_frames: Option<u32>,
    pub unlit: bool,
    pub angle_degrees: f64,
    /// Explicit new local file only; nothing is written into the installation.
    pub screenshot: Option<PathBuf>,
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
struct SkinExtension {
    #[texture(100)]
    #[sampler(101)]
    atlas: Handle<Image>,
    #[uniform(102)]
    offsets: Vec4,
}
type SkinMaterial = ExtendedMaterial<StandardMaterial, SkinExtension>;
impl MaterialExtension for SkinExtension {
    fn vertex_shader() -> ShaderRef {
        "embedded://openshores_client/viewer/native_skin_vertex.wgsl".into()
    }
    fn fragment_shader() -> ShaderRef {
        "embedded://openshores_client/viewer/native_skin.wgsl".into()
    }
}

#[derive(Resource)]
pub(super) struct SkinPreview {
    meshes: Vec<Mesh>,
    image: Option<Image>,
    offsets: Vec4,
    screenshot: Option<PathBuf>,
    frames: u32,
    pub(super) view: Option<NativeAvatarView>,
}

pub(super) struct SkinPlugin;
impl Plugin for SkinPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "native_skin.wgsl");
        embedded_asset!(app, "native_skin_vertex.wgsl");
        app.add_plugins(MaterialPlugin::<SkinMaterial>::default())
            .add_systems(Startup, setup.run_if(resource_exists::<SkinPreview>))
            .add_systems(
                Update,
                (lighting, capture).run_if(resource_exists::<SkinPreview>),
            );
    }
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

/// Bevy's existing PNG decoder, bounded before decode, with the native bottom-up
/// row order. sRGB and repeat wrapping are explicit modern preview choices.
fn decode_atlas(bytes: &[u8]) -> io::Result<Image> {
    if bytes.len() > 32 * 1024 * 1024
        || bytes.get(..8) != Some(b"\x89PNG\r\n\x1a\n")
        || bytes.get(8..16) != Some(b"\0\0\0\rIHDR")
    {
        return Err(invalid("skin atlas must be a bounded PNG with an IHDR"));
    }
    let dimension = |start| -> io::Result<u32> {
        Ok(u32::from_be_bytes(
            bytes
                .get(start..start + 4)
                .ok_or_else(|| invalid("truncated PNG dimensions"))?
                .try_into()
                .unwrap(),
        ))
    };
    let (width, height) = (dimension(16)?, dimension(20)?);
    if width == 0 || height == 0 || width > 4096 || height > 4096 {
        return Err(invalid("skin atlas dimensions must be 1..=4096"));
    }
    // The image decoder can accept a file whose trailing IEND was truncated.
    // Require a complete bounded PNG envelope before trusting decoded pixels.
    let mut cursor = 8_usize;
    let mut has_data = false;
    let mut ended = false;
    while cursor < bytes.len() {
        let length = dimension(cursor)? as usize;
        let end = cursor
            .checked_add(length)
            .and_then(|v| v.checked_add(12))
            .filter(|end| *end <= bytes.len())
            .ok_or_else(|| invalid("truncated PNG chunk"))?;
        let kind = &bytes[cursor + 4..cursor + 8];
        if kind == b"acTL" {
            return Err(invalid("animated skin PNG is unsupported"));
        }
        if kind == b"IHDR" && cursor != 8 {
            return Err(invalid("duplicate PNG IHDR"));
        }
        if kind == b"IDAT" {
            has_data = true;
        }
        if kind == b"IEND" {
            if length != 0 || end != bytes.len() {
                return Err(invalid("invalid PNG end chunk"));
            }
            ended = true;
        }
        cursor = end;
    }
    if !ended || !has_data {
        return Err(invalid("skin PNG is missing data or its end chunk"));
    }
    let mut image = Image::from_buffer(
        bytes,
        ImageType::Extension("png"),
        CompressedImageFormats::NONE,
        true,
        ImageSampler::Default,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .map_err(|error| invalid(&format!("skin PNG decode failed: {error}")))?;
    if image.texture_descriptor.format != TextureFormat::Rgba8UnormSrgb {
        image = image
            .convert(TextureFormat::Rgba8UnormSrgb)
            .ok_or_else(|| invalid("unsupported decoded skin pixel format"))?;
    }
    let data = image
        .data
        .as_mut()
        .ok_or_else(|| invalid("decoded atlas has no pixels"))?;
    let stride = width as usize * 4;
    if image.texture_descriptor.size.width != width
        || image.texture_descriptor.size.height != height
        || data.len() != stride * height as usize
    {
        return Err(invalid("decoded atlas dimensions/pixel count mismatch"));
    }
    for y in 0..height as usize / 2 {
        let other = height as usize - 1 - y;
        for x in 0..stride {
            data.swap(y * stride + x, other * stride + x);
        }
    }
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        ..Default::default()
    });
    Ok(image)
}

/// Compute normals on the WHOLE surface before splitting by UV/material/corner.
/// Render triangles are expanded; neither native indices nor raw weights change.
fn render_meshes(
    surface: &NativeBodySurfacePreview,
    display: &DisplayMesh,
    angle: f64,
) -> io::Result<Vec<Mesh>> {
    let uv = surface
        .geometry
        .uvs
        .as_ref()
        .ok_or_else(|| invalid("textured preview requires UVs"))?;
    let count = display.positions.len();
    if uv.len() != count
        || surface.top_face.len() != count
        || surface.material_memberships.len() != display.triangles.len()
        || uv.iter().flatten().any(|v| !v.is_finite())
        || surface
            .top_face
            .iter()
            .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
        || display
            .triangles
            .iter()
            .flatten()
            .any(|i| *i as usize >= count)
    {
        return Err(invalid("inconsistent textured surface inputs"));
    }
    let smoothed = auto_smooth::mesh(&display.positions, &display.triangles, angle);
    let normals = match smoothed.attribute(Mesh::ATTRIBUTE_NORMAL) {
        Some(bevy::mesh::VertexAttributeValues::Float32x3(values)) => values,
        _ => return Err(invalid("smoothing did not produce corner normals")),
    };
    let mut meshes = Vec::new();
    for faces in surface.partitions().values() {
        let mut positions = Vec::new();
        let mut corner_normals = Vec::new();
        let mut coordinates = Vec::new();
        let mut controls = Vec::new();
        for face in faces {
            for corner in 0..3 {
                let index = display.triangles[*face][corner] as usize;
                positions.push(display.positions[index]);
                corner_normals.push(normals[face * 3 + corner]);
                coordinates.push(uv[index]);
                controls.push([1., 1., 1., surface.top_face[index]]);
            }
        }
        meshes.push(
            Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, corner_normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, coordinates)
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, controls),
        );
    }
    Ok(meshes)
}

pub fn run_textured_body(
    surface: &NativeBodySurfacePreview,
    atlas_png: &[u8],
    offsets: [f64; 4],
    options: NativeSkinPreviewOptions,
) -> io::Result<()> {
    if !options.angle_degrees.is_finite()
        || !(0.0..=180.).contains(&options.angle_degrees)
        || offsets
            .iter()
            .any(|v| !v.is_finite() || !(*v as f32).is_finite())
    {
        return Err(invalid("invalid skin offsets or smoothing angle"));
    }
    if let Some(path) = &options.screenshot
        && (path.symlink_metadata().is_ok() || options.smoke_frames.is_some_and(|n| n < 900))
    {
        return Err(invalid(
            "screenshot requires a new path and at least 900 smoke frames",
        ));
    }
    let model = Model {
        meshes: vec![SourceMesh {
            vertices: surface
                .geometry
                .positions
                .iter()
                .map(|p| p.map(|v| v as f32))
                .collect(),
            faces: surface.geometry.faces.clone(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let display = geometry(&model)?;
    let meshes = render_meshes(surface, &display.meshes[0], options.angle_degrees)?;
    let image = decode_atlas(atlas_png)?;
    println!(
        "Textured skin: {}x{}, {} material partitions; bottom-up RGBA, sRGB preview, linear/repeat sampler, alpha cutoff 0.15; fixture clothes disabled.",
        image.width(),
        image.height(),
        meshes.len()
    );
    launch_with_skin(
        display,
        options.smoke_frames,
        options.unlit,
        Some(SkinPreview {
            meshes,
            image: Some(image),
            offsets: Vec4::from_array(offsets.map(|v| v as f32)),
            screenshot: options.screenshot,
            frames: 0,
            view: None,
        }),
    )
}

fn setup(
    mut commands: Commands,
    mut preview: ResMut<SkinPreview>,
    orbit: Res<Orbit>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SkinMaterial>>,
) {
    let atlas = images.add(
        preview
            .image
            .take()
            .expect("skin image consumed only at startup"),
    );
    let material = materials.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: Color::WHITE,
            unlit: !orbit.lit,
            alpha_mode: AlphaMode::Mask(0.15),
            double_sided: true,
            cull_mode: None,
            perceptual_roughness: 0.55,
            ..Default::default()
        },
        extension: SkinExtension {
            atlas,
            offsets: preview.offsets,
        },
    });
    for mesh in std::mem::take(&mut preview.meshes) {
        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(material.clone()),
            NotShadowCaster,
        ));
    }
}

fn lighting(orbit: Res<Orbit>, mut materials: ResMut<Assets<SkinMaterial>>) {
    // Avoid republishing every material every frame while pipelines warm up.
    let changed: Vec<_> = materials
        .iter()
        .filter(|(_, m)| m.base.unlit == orbit.lit)
        .map(|(id, _)| id)
        .collect();
    for id in changed {
        if let Some(mut material) = materials.get_mut(id) {
            material.base.unlit = !orbit.lit;
        }
    }
}

fn capture(mut commands: Commands, mut preview: ResMut<SkinPreview>) {
    preview.frames += 1;
    // Leave time for cold driver/PBR pipeline compilation before visual evidence.
    if preview.frames == 600
        && let Some(path) = preview.screenshot.take()
    {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_capture(path));
    }
}

fn save_capture(path: PathBuf) -> impl FnMut(On<ScreenshotCaptured>) {
    move |captured| {
        let save = || -> io::Result<()> {
            let image = captured
                .image
                .clone()
                .try_into_dynamic()
                .map_err(io::Error::other)?;
            // Atomic no-clobber check also rejects broken symlink destinations.
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)?;
            image
                .to_rgb8()
                .write_to(
                    &mut file,
                    bevy::image::ImageFormat::Png
                        .as_image_crate_format()
                        .unwrap(),
                )
                .map_err(io::Error::other)
        };
        match save() {
            Ok(()) => info!("Skin preview capture saved to {}", path.display()),
            Err(error) => error!("Skin preview capture failed: {error}"),
        }
    }
}

#[cfg(test)]
mod tests;
