//! Modern proximal sleeves, separate from native geometry and bone placement.
use super::avatar_contact::{Triangle, has_area, meshes_touch, point_triangle_distance};
use super::avatar_seams::bridge_surfaces;
use super::{
    NativeAvatarEndCap, NativeDpAvatarGeometry, NativeDpBoneTransforms, NativeDpPartKind,
    NativeDpPosedAvatarSurface, NativeDpPreparedAvatarSurface, native_avatar_end_cap,
};
use crate::model::NativeBodySurfacePreview;
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
};

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeAvatarVertexSourceRange {
    pub surface_index: usize,
    pub start: usize,
    pub count: usize,
}

/// An explicit modern render batch containing the torso and previously detached
/// limbs. Ranges retain every original surface/part's vertex provenance.
#[derive(Debug)]
pub struct NativeAvatarRootSeamRepair {
    pub root_part_index: usize,
    pub torso_surface_index: usize,
    pub consumed_render_surface_indices: Vec<usize>,
    pub vertex_sources: Vec<NativeAvatarVertexSourceRange>,
    pub first_vertex_count: usize,
    pub removed_faces: [usize; 2],
    pub bridge_faces: usize,
    pub surface: NativeBodySurfacePreview,
}

fn posed_triangles(
    surface: &NativeBodySurfacePreview,
    bones: &NativeDpBoneTransforms,
) -> io::Result<Vec<Triangle>> {
    let g = &surface.geometry;
    if g.positions.len() != g.weights.len() {
        return Err(invalid("proximal seam attribute count mismatch"));
    }
    let points = g
        .positions
        .iter()
        .zip(&g.weights)
        .map(|(p, w)| {
            let v = bones
                .skin_shader_vertex(p.map(|v| v as f32), [0., 0., 1.], w)?
                .position;
            if !v[3].is_finite() || v[3] <= 0. {
                return Err(invalid("proximal seam requires positive homogeneous W"));
            }
            Ok(std::array::from_fn(|i| f64::from(v[i]) / f64::from(v[3])))
        })
        .collect::<io::Result<Vec<[f64; 3]>>>()?;
    g.faces
        .iter()
        .map(|f| {
            if f.vertices.iter().any(|i| usize::from(*i) >= points.len()) {
                return Err(invalid("proximal seam face index out of range"));
            }
            Ok(f.vertices.map(|i| points[usize::from(i)]))
        })
        .collect()
}

// Select an actual torso face near the native joint, not a fabricated point or
// offset. Require ordinary two-sided edge incidence so existing native defects
// cannot be enlarged by removing a boundary/nonmanifold face.
fn torso_patch(
    surface: &NativeBodySurfacePreview,
    joint: [f64; 3],
    torso_face_count: usize,
) -> io::Result<NativeAvatarEndCap> {
    let g = &surface.geometry;
    let mut canonical = BTreeMap::new();
    let ids: Vec<_> = g
        .positions
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let key = p.map(|v| {
                let v = v as f32;
                if v == 0. { 0 } else { v.to_bits() }
            });
            *canonical.entry(key).or_insert(i)
        })
        .collect();
    let mut edges = BTreeMap::new();
    for face in &g.faces {
        let v = face.vertices.map(|i| ids[usize::from(i)]);
        for (a, b) in [(v[0], v[1]), (v[1], v[2]), (v[2], v[0])] {
            *edges.entry((a.min(b), a.max(b))).or_insert(0usize) += 1;
        }
    }
    let face = g
        .faces
        .iter()
        .enumerate()
        .take(torso_face_count)
        .filter(|(_, f)| {
            let v = f.vertices.map(|i| ids[usize::from(i)]);
            has_area(
                f.vertices
                    .map(|i| g.positions[usize::from(i)].map(|v| f64::from(v as f32))),
            ) && v[0] != v[1]
                && v[1] != v[2]
                && v[0] != v[2]
                && [(v[0], v[1]), (v[1], v[2]), (v[2], v[0])]
                    .into_iter()
                    .all(|(a, b)| edges[&(a.min(b), a.max(b))] == 2)
        })
        .min_by(|(_, a), (_, b)| {
            let distance = |f: &crate::model::Face| {
                point_triangle_distance(joint, f.vertices.map(|i| g.positions[usize::from(i)]))
            };
            distance(a).total_cmp(&distance(b))
        })
        .map(|(i, _)| i)
        .ok_or_else(|| invalid("no ordinary torso patch for proximal seam"))?;
    Ok(NativeAvatarEndCap {
        faces: BTreeSet::from([face]),
        perimeter: g.faces[face].vertices.map(usize::from).to_vec(),
    })
}

impl NativeDpAvatarGeometry {
    /// Extend distal repairs only where a whole posed limb remains separated
    /// from the torso. Contact is an exact render-geometry SAT decision, not an
    /// epsilon, a DNA fixture special case, or a guessed part translation.
    pub fn root_seam_repairs(&self) -> io::Result<Vec<NativeAvatarRootSeamRepair>> {
        let distal = self.seam_repairs()?;
        let torso_index = self
            .surfaces
            .iter()
            .position(|s| {
                self.body.parts()[s.part_index].allocation.kind == NativeDpPartKind::Torso
            })
            .ok_or_else(|| invalid("proximal seam repair requires a torso"))?;
        let mut body = self.surfaces[torso_index].surface.clone();
        let mut ranges = vec![NativeAvatarVertexSourceRange {
            surface_index: torso_index,
            start: 0,
            count: body.geometry.positions.len(),
        }];
        let mut consumed = Vec::new();
        let mut result = Vec::new();
        let body_triangles = posed_triangles(&body, self.body.bones())?;
        let mut torso_face_count = body.geometry.faces.len();
        for limb in &distal {
            let limb_index = limb.source_surfaces[0];
            let part_index = self.surfaces[limb_index].part_index;
            if meshes_touch(
                &body_triangles,
                &posed_triangles(&limb.surface, self.body.bones())?,
            ) {
                continue;
            }
            let joint = self.body.parts()[part_index].connect_30;
            let patch = torso_patch(&body, joint, torso_face_count)?;
            let cuff = native_avatar_end_cap(&limb.surface, joint)?;
            let joined = bridge_surfaces(
                &body,
                &limb.surface,
                patch,
                cuff,
                [torso_index, limb_index],
                false,
            )?;
            let start = body.geometry.positions.len();
            ranges.push(NativeAvatarVertexSourceRange {
                surface_index: limb.source_surfaces[0],
                start,
                count: limb.first_vertex_count,
            });
            ranges.push(NativeAvatarVertexSourceRange {
                surface_index: limb.source_surfaces[1],
                start: start + limb.first_vertex_count,
                count: limb.surface.geometry.positions.len() - limb.first_vertex_count,
            });
            consumed.push(limb_index);
            body = joined.surface;
            torso_face_count -= 1;
            result.push(NativeAvatarRootSeamRepair {
                root_part_index: part_index,
                torso_surface_index: torso_index,
                consumed_render_surface_indices: consumed.clone(),
                vertex_sources: ranges.clone(),
                first_vertex_count: joined.first_vertex_count,
                removed_faces: joined.removed_faces,
                bridge_faces: joined.bridge_faces,
                surface: body.clone(),
            });
        }
        Ok(result)
    }

    /// Modern render batches; a repaired torso batch can span several parts.
    /// Its root-repair report preserves the original per-part vertex provenance.
    pub fn posed_surfaces_with_attachment_repair(
        &self,
        angle: f64,
    ) -> io::Result<Vec<NativeDpPosedAvatarSurface>> {
        self.prepare_surfaces_with_attachment_repair(angle)?
            .iter()
            .map(|surface| surface.skin(self.body.bones()))
            .collect()
    }

    /// Choose repair topology once in the current neutral pose, then retain
    /// model-space corners for all subsequent bone samples. Never re-run the
    /// posed contact test mid-stride, which would change topology each frame.
    pub fn prepare_surfaces_with_attachment_repair(
        &self,
        angle: f64,
    ) -> io::Result<Vec<NativeDpPreparedAvatarSurface>> {
        let mut posed = self.prepare_surfaces_with_seam_repair(angle)?;
        let repairs = self.root_seam_repairs()?;
        if let Some(last) = repairs.last() {
            posed.retain(|s| {
                s.surface_index != last.torso_surface_index
                    && !last
                        .consumed_render_surface_indices
                        .contains(&s.surface_index)
            });
            posed.insert(
                0,
                NativeDpPreparedAvatarSurface::from_surface(
                    last.torso_surface_index,
                    self.surfaces[last.torso_surface_index].part_index,
                    &last.surface,
                    angle,
                )?,
            );
        }
        Ok(posed)
    }
}
