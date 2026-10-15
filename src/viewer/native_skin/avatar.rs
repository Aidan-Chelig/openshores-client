//! Full-body renderer adapter; all native deformation/pose work stays on CPU.
use super::*;
use crate::world::native_body::{NativeDpAvatarGeometry, NativeDpPosedAvatarSurface};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NativeAvatarView {
    #[default]
    Oblique,
    Front,
    Side,
    Back,
}
impl NativeAvatarView {
    pub(in crate::viewer) fn orbit(self) -> (f32, f32) {
        let yaw = match self {
            Self::Oblique => 0.75,
            Self::Front => std::f32::consts::FRAC_PI_2,
            Self::Side => 0.,
            Self::Back => -std::f32::consts::FRAC_PI_2,
        };
        (yaw, if self == Self::Oblique { 0.35 } else { 0. })
    }
}

/// Render-only affine framing of actual drawn corners, never helper vertices.
/// Native positions, normals, weights and original surfaces remain unchanged.
fn render_avatar(
    surfaces: &[NativeDpPosedAvatarSurface],
) -> io::Result<(DisplayGeometry, Vec<Mesh>)> {
    let mut low = [f64::INFINITY; 3];
    let mut high = [f64::NEG_INFINITY; 3];
    for c in surfaces.iter().flat_map(|s| &s.corners) {
        if c.position.iter().chain(&c.normal).any(|v| !v.is_finite()) || c.position[3] <= 0. {
            return Err(invalid(
                "avatar renderer requires finite corners with positive homogeneous W",
            ));
        }
        for axis in 0..3 {
            let value = f64::from(c.position[axis]) / f64::from(c.position[3]);
            low[axis] = low[axis].min(value);
            high[axis] = high[axis].max(value);
        }
    }
    let span = (0..3).map(|a| high[a] - low[a]).fold(0_f64, f64::max);
    if !span.is_finite() || span <= 0. {
        return Err(invalid("avatar has no finite drawable extent"));
    }
    let center: [f64; 3] = std::array::from_fn(|a| (low[a] + high[a]) * 0.5);
    let mut display = Vec::new();
    let mut meshes = Vec::new();
    for surface in surfaces {
        if surface.corners.len() != surface.material_memberships.len() * 3 {
            return Err(invalid("posed avatar corner/material count mismatch"));
        }
        let positions: Vec<[f32; 3]> = surface
            .corners
            .iter()
            .map(|c| {
                std::array::from_fn(|a| {
                    ((f64::from(c.position[a]) / f64::from(c.position[3]) - center[a])
                        * (2. / span)) as f32
                })
            })
            .collect();
        if positions.iter().flatten().any(|v| !v.is_finite()) {
            return Err(invalid("avatar framing is nonfinite"));
        }
        let mut partitions = BTreeMap::new();
        for (face, names) in surface.material_memberships.iter().enumerate() {
            partitions.entry(names).or_insert_with(Vec::new).push(face);
        }
        for faces in partitions.values() {
            let mut points = Vec::new();
            let mut normals = Vec::new();
            let mut uvs = Vec::new();
            let mut controls = Vec::new();
            for &face in faces {
                for (point, c) in positions[face * 3..face * 3 + 3]
                    .iter()
                    .zip(&surface.corners[face * 3..face * 3 + 3])
                {
                    if c.uv.iter().any(|v| !v.is_finite())
                        || !c.top_face.is_finite()
                        || !(0.0..=1.).contains(&c.top_face)
                    {
                        return Err(invalid("invalid posed avatar texture control"));
                    }
                    points.push(*point);
                    normals.push(c.normal);
                    uvs.push(c.uv);
                    controls.push([c.position[3], 1., 1., c.top_face]);
                }
            }
            meshes.push(
                Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
                )
                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, points)
                .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
                .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
                .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, controls),
            );
        }
        display.push(DisplayMesh {
            triangles: (0..surface.corners.len() / 3)
                .map(|f| [(f * 3) as u32, (f * 3 + 1) as u32, (f * 3 + 2) as u32])
                .collect(),
            positions,
            colors: None,
        });
    }
    Ok((
        DisplayGeometry {
            meshes: display,
            crease_degrees: None,
            smooth: false,
            terrain_probe: None,
        },
        meshes,
    ))
}

/// `None` atlas selects an explicit white untextured diagnostic with identical
/// posed corners/normals, not a recomputed generic model-viewer mesh.
/// `repair_seams` opts into modern derived cuffs; false retains native surfaces.
pub fn run_textured_avatar(
    avatar: &NativeDpAvatarGeometry,
    atlas_png: Option<&[u8]>,
    options: NativeSkinPreviewOptions,
    view: NativeAvatarView,
    repair_seams: bool,
) -> io::Result<()> {
    if let Some(path) = &options.screenshot
        && (path.symlink_metadata().is_ok() || options.smoke_frames.is_some_and(|n| n < 900))
    {
        return Err(invalid(
            "screenshot requires a new path and at least 900 smoke frames",
        ));
    }
    if avatar
        .skin_offsets
        .iter()
        .any(|v| !v.is_finite() || !(*v as f32).is_finite())
    {
        return Err(invalid("nonfinite avatar skin offsets"));
    }
    let surfaces = if repair_seams {
        avatar.posed_surfaces_with_seam_repair(options.angle_degrees)?
    } else {
        avatar.posed_surfaces(options.angle_degrees)?
    };
    let (display, meshes) = render_avatar(&surfaces)?;
    let image = if let Some(bytes) = atlas_png {
        decode_atlas(bytes)?
    } else {
        Image::new_fill(
            bevy::render::render_resource::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            bevy::render::render_resource::TextureDimension::D2,
            &[255, 255, 255, 255],
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        )
    };
    println!(
        "Assembled avatar: {} render surfaces, {} render partitions; CPU-smoothed/skinned normals, retained homogeneous W, source XYZ unchanged; {:?} view; seam repair {}.",
        surfaces.len(),
        meshes.len(),
        view,
        if repair_seams {
            "enabled (modern)"
        } else {
            "disabled (native)"
        }
    );
    launch_with_skin(
        display,
        options.smoke_frames,
        options.unlit,
        Some(SkinPreview {
            meshes,
            image: Some(image),
            offsets: Vec4::from_array(avatar.skin_offsets.map(|v| v as f32)),
            screenshot: options.screenshot,
            frames: 0,
            view: Some(view),
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::native_body::NativeDpAvatarCorner;
    use bevy::mesh::VertexAttributeValues;

    fn posed() -> NativeDpPosedAvatarSurface {
        NativeDpPosedAvatarSurface {
            surface_index: 0,
            part_index: 2,
            corners: [[0., 0., 0., 1.], [2., 0., 0., 2.], [0., 1., 0., 1.]]
                .into_iter()
                .enumerate()
                .map(|(i, position)| NativeDpAvatarCorner {
                    source_vertex: i,
                    position,
                    normal: [1., 0., 0.],
                    uv: [i as f32, 0.25],
                    top_face: i as f32 * 0.5,
                })
                .collect(),
            material_memberships: vec![vec![vec![65]]],
        }
    }

    #[test]
    fn adapter_frames_drawn_corners_preserves_normals_uvs_controls_and_homogeneous_w() {
        let source = posed();
        let before = source.corners.clone();
        let (display, meshes) = render_avatar(&[source]).unwrap();
        assert_eq!(
            display.meshes[0].positions,
            [[-1., -1., 0.], [1., -1., 0.], [-1., 1., 0.]]
        );
        assert_eq!(meshes.len(), 1);
        assert!(meshes[0].indices().is_none());
        let Some(VertexAttributeValues::Float32x3(normals)) =
            meshes[0].attribute(Mesh::ATTRIBUTE_NORMAL)
        else {
            panic!()
        };
        assert_eq!(normals, &vec![[1., 0., 0.]; 3]); // not triangle face normals
        let Some(VertexAttributeValues::Float32x2(uvs)) = meshes[0].attribute(Mesh::ATTRIBUTE_UV_0)
        else {
            panic!()
        };
        assert_eq!(uvs, &vec![[0., 0.25], [1., 0.25], [2., 0.25]]);
        let Some(VertexAttributeValues::Float32x4(controls)) =
            meshes[0].attribute(Mesh::ATTRIBUTE_COLOR)
        else {
            panic!()
        };
        assert_eq!(
            controls,
            &vec![[1., 1., 1., 0.], [2., 1., 1., 0.5], [1., 1., 1., 1.]]
        );
        // Restore W after affine display framing, then arbitrary view/projection.
        let affine = Mat4::from_scale_rotation_translation(
            Vec3::splat(2.),
            Quat::IDENTITY,
            Vec3::new(-1., -1., 0.),
        );
        let projection = Mat4::perspective_rh(0.8, 1.4, 0.1, 100.)
            * Mat4::look_at_rh(Vec3::new(3., 4., 5.), Vec3::ZERO, Vec3::Z);
        for (i, c) in before.iter().enumerate() {
            let clip_from_upload = projection
                * Vec4::from((Vec3::from_array(display.meshes[0].positions[i]), 1.))
                * controls[i][0];
            let native_clip = projection * affine * Vec4::from_array(c.position);
            assert!((clip_from_upload - native_clip).abs().max_element() < 1e-5);
        }
        assert!(
            include_str!("../native_skin_vertex.wgsl")
                .contains("position_world_to_clip(out.world_position.xyz) * input.color.r")
        );
    }

    #[test]
    fn renderer_rejects_invalid_homogeneous_and_inconsistent_inputs() {
        assert!(render_avatar(&[]).is_err());
        for w in [0., -1., f32::NAN, f32::INFINITY] {
            let mut p = posed();
            p.corners[0].position[3] = w;
            assert!(render_avatar(&[p]).is_err());
        }
        let mut p = posed();
        p.material_memberships.clear();
        assert!(render_avatar(&[p]).is_err());
        let mut p = posed();
        p.corners[0].uv[0] = f32::NAN;
        assert!(render_avatar(&[p]).is_err());
    }
}
