//! Reusable mounted CPU body generation, independent of the viewer/Bevy.
use super::*;
use crate::{
    asset_mount::AssetMount,
    model::{
        Mesh, Model, ModelVariant, NativeBodyFaceMetadata, NativeBodyMergeGeometry,
        NativeBodyModelGeometry, NativeBodySurfacePreview,
    },
    world::native_dna::NativeDhDna,
};
use std::io;

#[derive(Debug)]
pub struct NativeDpAvatarSource {
    pub resource: String,
    /// Original decoded resource; never deformed or pose-transformed in place.
    pub mesh: Mesh,
}

#[derive(Debug)]
pub struct NativeDpAvatarSurface {
    pub part_index: usize,
    pub sources: Vec<NativeDpAvatarSource>,
    /// Native merged or unmerged state with untouched weight payloads.
    pub native: NativeBodyMergeGeometry,
    /// Projected/deformed MODEL-space upload inputs, not yet bone-skinned.
    pub surface: NativeBodySurfacePreview,
}

#[derive(Debug)]
pub struct NativeDpAvatarGeometry {
    pub dna: NativeDhDna,
    pub field_4068: i32,
    pub field_406c: i32,
    pub body: NativeDpInitialization,
    pub standing: NativeDpStandingPose,
    pub skin: NativeDpSkinAtlas,
    pub skin_offsets: [f64; 4],
    pub surfaces: Vec<NativeDpAvatarSurface>,
}

#[derive(Clone, Copy)]
enum Merge {
    Head,
    Torso,
    Separate,
}
type Piece = (String, NativeBodyModelGeometry, [usize; 3]);

impl NativeDpAvatarGeometry {
    /// Generate normal body skin plus its verified Stand skeleton from explicit
    /// DNA/owner appearance. Bone skinning and smoothing remain separate stages;
    /// never reapply object deformation/projection when consuming these inputs.
    pub fn generate(
        mount: &AssetMount,
        dna: &NativeDhDna,
        field_4068: i32,
        field_406c: i32,
    ) -> io::Result<Self> {
        if field_4068 != 0 || field_406c != 0 {
            return Err(unsupported(
                "assembled normal-skin generator does not support armor/helmet appearance fields yet",
            ));
        }
        let mut hands = NativeDpHandCache::new(mount);
        let mut feet = NativeDpFootCache::new(mount);
        let mut body = NativeDpInitialization::begin(dna);
        body.finish_part_calculations(field_4068, &mut hands)?;
        let offsets = native_dp_skin_offsets(dna);
        let plans = body.skin_geometry_plans(field_4068, field_406c, offsets)?;
        let mut surfaces = Vec::new();
        for planned in plans {
            let index = planned.part_index;
            match planned.plan {
                NativeDpSkinPlan::Arm(pieces) => {
                    for p in pieces {
                        surfaces.push(build_surface(
                            mount,
                            index,
                            vec![(p.resource, p.geometry, p.bones)],
                            Merge::Separate,
                            0.,
                        )?);
                    }
                }
                NativeDpSkinPlan::Leg(pieces) => {
                    for p in pieces {
                        surfaces.push(build_surface(
                            mount,
                            index,
                            vec![(p.resource, p.geometry, p.bones)],
                            Merge::Separate,
                            0.,
                        )?);
                    }
                }
                NativeDpSkinPlan::Tail(Some(p)) => surfaces.push(build_surface(
                    mount,
                    index,
                    vec![(p.resource, p.geometry, p.bones)],
                    Merge::Separate,
                    0.,
                )?),
                NativeDpSkinPlan::Head(plan) => {
                    if plan.eye_reference.is_some() {
                        return Err(unsupported(
                            "separate PA eye appearance is not supported by normal skin",
                        ));
                    }
                    let tolerance = plan
                        .fallback
                        .first()
                        .and_then(|p| p.merge_tolerance)
                        .ok_or_else(|| {
                            unsupported("native head fallback lacks a merge tolerance")
                        })?;
                    let pieces = plan
                        .fallback
                        .into_iter()
                        .map(|p| (p.resource, p.geometry, p.bones))
                        .collect();
                    surfaces.push(build_surface(mount, index, pieces, Merge::Head, tolerance)?);
                    if let Some(p) = plan.neck {
                        surfaces.push(build_surface(
                            mount,
                            index,
                            vec![(p.resource, p.geometry, p.bones)],
                            Merge::Separate,
                            0.,
                        )?);
                    }
                }
                NativeDpSkinPlan::Torso(Some(plan)) => {
                    if plan.fallback.iter().any(|p| p.crest) {
                        return Err(unsupported("native torso crest merge is not implemented"));
                    }
                    let pieces = plan
                        .fallback
                        .into_iter()
                        .map(|p| (p.resource, p.geometry, p.bones))
                        .collect();
                    surfaces.push(build_surface(
                        mount,
                        index,
                        pieces,
                        Merge::Torso,
                        plan.merge_tolerance,
                    )?);
                }
                NativeDpSkinPlan::Tail(None) | NativeDpSkinPlan::Torso(None) => {}
            }
        }
        let standing = body.pose_neutral_standing(field_4068, &mut hands, &mut feet)?;
        Ok(Self {
            dna: dna.clone(),
            field_4068,
            field_406c,
            body,
            standing,
            skin: NativeDpSkinAtlas::from_dna(dna),
            skin_offsets: offsets,
            surfaces,
        })
    }
}

fn build_surface(
    mount: &AssetMount,
    part_index: usize,
    pieces: Vec<Piece>,
    merge: Merge,
    tolerance: f64,
) -> io::Result<NativeDpAvatarSurface> {
    let mut destination: Option<NativeBodyMergeGeometry> = None;
    let mut sources = Vec::new();
    for (resource, definition, bones) in pieces {
        let model = Model::decode(&mount.read_resource(&resource)?, ModelVariant::N)?;
        if model.meshes.len() != 1 {
            return Err(unsupported(&format!(
                "native avatar requires one object in {resource}"
            )));
        }
        let mesh = model.meshes.into_iter().next().unwrap();
        let metadata = metadata(&mesh)?;
        let mut source = mesh.body_merge_geometry(
            ModelVariant::N,
            &definition,
            bones.map(|i| i as i32),
            metadata,
        )?;
        if let Some(dest) = destination.as_mut() {
            match merge {
                Merge::Head => {
                    dest.append_head_geometry_stages(&mut source, ModelVariant::N, tolerance)?;
                }
                Merge::Torso => {
                    dest.append_torso_geometry_stages(&mut source, ModelVariant::N, tolerance)?;
                }
                Merge::Separate => {
                    return Err(unsupported(
                        "separate native part unexpectedly contains merge pieces",
                    ));
                }
            }
        } else {
            destination = Some(source);
        }
        sources.push(NativeDpAvatarSource { resource, mesh });
    }
    let native =
        destination.ok_or_else(|| unsupported("native skin plan has no source geometry"))?;
    let original = &sources[0].mesh;
    let surface = if native.object.field_369 {
        native.textured_baked_preview_geometry(original, ModelVariant::N)?
    } else {
        native.textured_unmerged_preview_geometry(original, ModelVariant::N)?
    };
    Ok(NativeDpAvatarSurface {
        part_index,
        sources,
        native,
        surface,
    })
}

fn metadata(mesh: &Mesh) -> io::Result<NativeBodyFaceMetadata> {
    let mut metadata = NativeBodyFaceMetadata {
        smoothing: mesh.smoothing.clone().unwrap_or_default(),
        ..Default::default()
    };
    for material in &mesh.materials {
        if !material.name.is_ascii()
            || metadata
                .materials
                .insert(
                    material.name.iter().map(|b| u16::from(*b)).collect(),
                    material.faces.clone(),
                )
                .is_some()
        {
            return Err(unsupported(
                "native avatar requires unique ASCII material names",
            ));
        }
    }
    Ok(metadata)
}

fn unsupported(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, message)
}
