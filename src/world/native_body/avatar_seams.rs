//! Optional modern render-only repairs. Never mutate mounted/native geometry.
use super::{
    NativeDpAvatarGeometry, NativeDpPartJoints, NativeDpPosedAvatarSurface,
    NativeDpPreparedAvatarSurface,
};
use crate::model::Face;
use crate::model::NativeBodySurfacePreview;
use std::collections::{BTreeMap, BTreeSet};
use std::io;

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn delta(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}
fn normal([a, b, c]: [[f64; 3]; 3]) -> [f64; 3] {
    let u = delta(b, a);
    let v = delta(c, a);
    let n = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    let length = dot(n, n).sqrt();
    if length > 0. {
        n.map(|x| x / length)
    } else {
        [0.; 3]
    }
}

/// A connected, consistently oriented end-cap patch and its directed perimeter.
/// Indices refer to the original surface, including its UV/payload duplicates.
#[derive(Debug)]
pub struct NativeAvatarEndCap {
    pub faces: BTreeSet<usize>,
    pub perimeter: Vec<usize>,
}

/// Experimental modern cap selection, anchored to the actual native distal joint.
/// Reject ambiguous/non-manifold boundaries instead of fabricating a cuff.
pub fn native_avatar_end_cap(
    surface: &NativeBodySurfacePreview,
    joint: [f64; 3],
) -> io::Result<NativeAvatarEndCap> {
    let g = &surface.geometry;
    if g.positions.is_empty()
        || g.positions
            .iter()
            .flatten()
            .any(|v| !v.is_finite() || !(*v as f32).is_finite())
        || joint.iter().any(|v| !v.is_finite())
    {
        return Err(invalid(
            "end cap requires finite, nonempty model-space inputs",
        ));
    }
    let mut positions = BTreeMap::new();
    let ids: Vec<_> = g
        .positions
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let key = p.map(|x| {
                let x = x as f32;
                if x == 0. { 0 } else { x.to_bits() }
            });
            *positions.entry(key).or_insert(i)
        })
        .collect();
    let mut edges: BTreeMap<(usize, usize), Vec<usize>> = BTreeMap::new();
    let mut triangles = Vec::new();
    let mut normals = Vec::new();
    let mut seed = None;
    let mut nearest = f64::INFINITY;
    let center = std::array::from_fn(|i| {
        g.positions.iter().map(|p| p[i]).sum::<f64>() / g.positions.len() as f64
    });
    let axis = delta(joint, center);
    let length = dot(axis, axis).sqrt();
    if !length.is_finite() || length == 0. {
        return Err(invalid("end cap has no joint direction"));
    }
    let axis = axis.map(|v| v / length);
    for (fi, face) in g.faces.iter().enumerate() {
        let v = face.vertices.map(usize::from);
        if v.iter().any(|i| *i >= ids.len()) {
            return Err(invalid("end cap face index out of range"));
        }
        let t = v.map(|i| g.positions[i]);
        let n = normal(t);
        let center = std::array::from_fn(|i| (t[0][i] + t[1][i] + t[2][i]) / 3.);
        let d = delta(center, joint);
        if dot(n, axis) > 0.8 && dot(d, d) < nearest {
            seed = Some(fi);
            nearest = dot(d, d);
        }
        triangles.push(v);
        normals.push(n);
        for (a, b) in [(v[0], v[1]), (v[1], v[2]), (v[2], v[0])] {
            let (a, b) = (ids[a], ids[b]);
            if a != b {
                edges.entry((a.min(b), a.max(b))).or_default().push(fi);
            }
        }
    }
    if edges.values().any(|f| f.len() != 2) {
        return Err(invalid("cap selection requires a closed manifold surface"));
    }
    let seed = seed.ok_or_else(|| invalid("end cap has no nondegenerate faces"))?;
    let mut faces = BTreeSet::new();
    let mut pending = vec![seed];
    while let Some(fi) = pending.pop() {
        if dot(normals[fi], normals[seed]) <= 0.5 || !faces.insert(fi) {
            continue;
        }
        let v = triangles[fi];
        for (a, b) in [(v[0], v[1]), (v[1], v[2]), (v[2], v[0])] {
            let (a, b) = (ids[a], ids[b]);
            if a != b {
                pending.extend(&edges[&(a.min(b), a.max(b))]);
            }
        }
    }
    let mut boundary = BTreeMap::new();
    for &fi in &faces {
        let v = triangles[fi];
        for (a, b) in [(v[0], v[1]), (v[1], v[2]), (v[2], v[0])] {
            let (ca, cb) = (ids[a], ids[b]);
            if ca == cb {
                continue;
            }
            if edges[&(ca.min(cb), ca.max(cb))]
                .iter()
                .filter(|f| faces.contains(f))
                .count()
                == 1
                && boundary.insert(ca, (cb, a)).is_some()
            {
                return Err(invalid("end cap has a branching perimeter"));
            }
        }
    }
    let start = *boundary
        .keys()
        .next()
        .ok_or_else(|| invalid("end cap has no perimeter"))?;
    let mut cursor = start;
    let mut perimeter = Vec::new();
    loop {
        let (next, original) = boundary
            .remove(&cursor)
            .ok_or_else(|| invalid("end cap perimeter is not a cycle"))?;
        perimeter.push(original);
        cursor = next;
        if cursor == start {
            break;
        }
    }
    if !boundary.is_empty() || perimeter.len() < 3 {
        return Err(invalid("end cap has multiple or degenerate perimeters"));
    }
    Ok(NativeAvatarEndCap { faces, perimeter })
}

/// Derived rendering data, not a native source surface. Vertex indices below
/// `first_vertex_count` belong to the first source; the rest to the second.
#[derive(Debug)]
pub struct NativeAvatarSeamRepair {
    pub source_surfaces: [usize; 2],
    pub first_vertex_count: usize,
    pub removed_faces: [usize; 2],
    pub bridge_faces: usize,
    pub surface: NativeBodySurfacePreview,
}

impl NativeDpAvatarGeometry {
    /// Explicit opt-in modern geometry repair. Source geometry, payloads, pose,
    /// and the fidelity-reference `posed_surfaces` path remain untouched.
    pub fn seam_repairs(&self) -> io::Result<Vec<NativeAvatarSeamRepair>> {
        let mut result = Vec::new();
        for (part_index, part) in self.body.parts().iter().enumerate() {
            let joint = match part.joints {
                NativeDpPartJoints::Arm { field_70, .. } => field_70,
                NativeDpPartJoints::Leg { field_68, .. } => field_68,
                _ => continue,
            };
            let sources: Vec<_> = self
                .surfaces
                .iter()
                .enumerate()
                .filter(|(_, s)| s.part_index == part_index)
                .collect();
            let [(a, first), (b, second)] = sources.as_slice() else {
                return Err(invalid("seam repair requires exactly two limb surfaces"));
            };
            result.push(repair_pair(
                &first.surface,
                &second.surface,
                joint,
                [*a, *b],
            )?);
        }
        Ok(result)
    }

    pub fn posed_surfaces_with_seam_repair(
        &self,
        angle: f64,
    ) -> io::Result<Vec<NativeDpPosedAvatarSurface>> {
        self.prepare_surfaces_with_seam_repair(angle)?
            .iter()
            .map(|surface| surface.skin(self.body.bones()))
            .collect()
    }

    pub fn prepare_surfaces_with_seam_repair(
        &self,
        angle: f64,
    ) -> io::Result<Vec<NativeDpPreparedAvatarSurface>> {
        let repairs = self.seam_repairs()?;
        let mut result = Vec::new();
        for (i, source) in self.surfaces.iter().enumerate() {
            if repairs.iter().any(|r| r.source_surfaces[1] == i) {
                continue;
            }
            let surface = repairs
                .iter()
                .find(|r| r.source_surfaces[0] == i)
                .map_or(&source.surface, |r| &r.surface);
            result.push(NativeDpPreparedAvatarSurface::from_surface(
                i,
                source.part_index,
                surface,
                angle,
            )?);
        }
        // Also validate the smoothing angle for an empty avatar.
        crate::model::crease_corner_normals(&[], &[], angle)?;
        Ok(result)
    }
}

fn repair_pair(
    a: &NativeBodySurfacePreview,
    b: &NativeBodySurfacePreview,
    joint: [f64; 3],
    source_surfaces: [usize; 2],
) -> io::Result<NativeAvatarSeamRepair> {
    let ca = native_avatar_end_cap(a, joint)?;
    let cb = native_avatar_end_cap(b, joint)?;
    bridge_surfaces(a, b, ca, cb, source_surfaces, true)
}

pub(super) fn bridge_surfaces(
    a: &NativeBodySurfacePreview,
    b: &NativeBodySurfacePreview,
    ca: NativeAvatarEndCap,
    cb: NativeAvatarEndCap,
    source_surfaces: [usize; 2],
    require_closed: bool,
) -> io::Result<NativeAvatarSeamRepair> {
    for source in [a, b] {
        let count = source.geometry.positions.len();
        if source.geometry.weights.len() != count
            || source.top_face.len() != count
            || source
                .geometry
                .uvs
                .as_ref()
                .is_none_or(|uv| uv.len() != count)
            || source.material_memberships.len() != source.geometry.faces.len()
        {
            return Err(invalid("seam source attribute count mismatch"));
        }
    }
    let offset = a.geometry.positions.len();
    if offset + b.geometry.positions.len() > usize::from(u16::MAX) + 1 {
        return Err(invalid("seam repair exceeds native face index capacity"));
    }
    let mut geometry = a.geometry.clone();
    geometry.positions.extend_from_slice(&b.geometry.positions);
    geometry.weights.extend_from_slice(&b.geometry.weights);
    let uv = geometry
        .uvs
        .as_mut()
        .ok_or_else(|| invalid("seam source lacks UVs"))?;
    uv.extend_from_slice(
        b.geometry
            .uvs
            .as_ref()
            .ok_or_else(|| invalid("seam source lacks UVs"))?,
    );
    geometry.faces.clear();
    let mut memberships = Vec::new();
    for (source, cap, offset) in [(a, &ca, 0), (b, &cb, offset)] {
        if source.material_memberships.len() != source.geometry.faces.len() {
            return Err(invalid("seam source material count mismatch"));
        }
        for (i, face) in source.geometry.faces.iter().enumerate() {
            if cap.faces.contains(&i) {
                continue;
            }
            let mut face = face.clone();
            face.vertices = face.vertices.map(|v| (usize::from(v) + offset) as u16);
            geometry.faces.push(face);
            memberships.push(source.material_memberships[i].clone());
        }
    }
    let mut ra = ca.perimeter.clone();
    let mut rb: Vec<_> = cb.perimeter.iter().rev().map(|i| i + offset).collect();
    let distance = |i: usize, j: usize| {
        let d = delta(geometry.positions[i], geometry.positions[j]);
        dot(d, d)
    };
    let (mut best, mut start) = (f64::INFINITY, (0, 0));
    for (i, &ai) in ra.iter().enumerate() {
        for (j, &bj) in rb.iter().enumerate() {
            let d = distance(ai, bj);
            if d < best {
                best = d;
                start = (i, j);
            }
        }
    }
    ra.rotate_left(start.0);
    rb.rotate_left(start.1);
    let (n, m) = (ra.len(), rb.len());
    // Minimum-cost monotone zipper accommodates unequal cuff tessellation.
    let mut costs = vec![f64::INFINITY; (n + 1) * (m + 1)];
    let mut from_a = vec![false; costs.len()];
    costs[0] = 0.;
    for i in 0..=n {
        for j in 0..=m {
            if i == 0 && j == 0 {
                continue;
            }
            // Keep perimeter progress within half a turn. Otherwise a cheap
            // path can wrap one complete ring at a fixed vertex on the other,
            // using the same geometric cross-edge four times.
            if (i as f64 / n as f64 - j as f64 / m as f64).abs() >= 0.5 {
                continue;
            }
            let index = i * (m + 1) + j;
            let edge = distance(ra[i % n], rb[j % m]);
            let ac = if i > 0 {
                costs[(i - 1) * (m + 1) + j]
            } else {
                f64::INFINITY
            };
            let bc = if j > 0 {
                costs[i * (m + 1) + j - 1]
            } else {
                f64::INFINITY
            };
            from_a[index] = ac <= bc;
            costs[index] = edge + ac.min(bc);
        }
    }
    if !costs[n * (m + 1) + m].is_finite() {
        return Err(invalid("no valid bounded-progress cuff zipper"));
    }
    let bridge_start = geometry.faces.len();
    let (mut i, mut j) = (n, m);
    while i > 0 || j > 0 {
        let advance_a = from_a[i * (m + 1) + j];
        let vertices = if advance_a {
            let t = [ra[(i - 1) % n], ra[i % n], rb[j % m]];
            i -= 1;
            t
        } else {
            let t = [ra[i % n], rb[j % m], rb[(j - 1) % m]];
            j -= 1;
            t
        };
        geometry.faces.push(Face {
            vertices: vertices.map(|v| v as u16),
            flags: 0,
            extension: None,
            nt_extension: None,
        });
        let source = if advance_a { a } else { b };
        let cap = if advance_a { &ca } else { &cb };
        memberships.push(source.material_memberships[*cap.faces.first().unwrap()].clone());
    }
    let mut top_face = a.top_face.clone();
    top_face.extend_from_slice(&b.top_face);
    // Reject unsupported cap/zipper combinations at the API boundary, rather
    // than relying on mounted-fixture tests to catch malformed derived meshes.
    let mut canonical = BTreeMap::new();
    let ids: Vec<_> = geometry
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
    for (face_index, face) in geometry.faces.iter().enumerate() {
        let t = face
            .vertices
            .map(|i| geometry.positions[usize::from(i)].map(|v| f64::from(v as f32)));
        if (require_closed || face_index >= bridge_start) && dot(normal(t), normal(t)) == 0. {
            return Err(invalid("seam repair has a degenerate render triangle"));
        }
        let v = face.vertices.map(|i| ids[usize::from(i)]);
        for (a, b) in [(v[0], v[1]), (v[1], v[2]), (v[2], v[0])] {
            let entry = edges.entry((a.min(b), a.max(b))).or_insert((0usize, 0i32));
            entry.0 += 1;
            entry.1 += if a < b { 1 } else { -1 };
        }
    }
    let mut original_edges = BTreeMap::new();
    if !require_closed {
        for (source, offset) in [(a, 0), (b, offset)] {
            for face in &source.geometry.faces {
                let v = face.vertices.map(|i| ids[usize::from(i) + offset]);
                for (a, b) in [(v[0], v[1]), (v[1], v[2]), (v[2], v[0])] {
                    let entry = original_edges
                        .entry((a.min(b), a.max(b)))
                        .or_insert((0usize, 0i32));
                    entry.0 += 1;
                    entry.1 += if a < b { 1 } else { -1 };
                }
            }
        }
        original_edges.retain(|_, v| *v != (2, 0));
    }
    if edges
        .iter()
        .any(|(key, v)| *v != (2, 0) && original_edges.get(key) != Some(v))
        || original_edges
            .iter()
            .any(|(key, v)| edges.get(key) != Some(v))
    {
        return Err(invalid("seam repair is not closed and consistently wound"));
    }
    Ok(NativeAvatarSeamRepair {
        source_surfaces,
        first_vertex_count: offset,
        removed_faces: [ca.faces.len(), cb.faces.len()],
        bridge_faces: n + m,
        surface: NativeBodySurfacePreview {
            geometry,
            top_face,
            material_memberships: memberships,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{NativeBodyMeshGeometry, NativeBodyVertexWeights};

    fn cube(z: f64) -> NativeBodySurfacePreview {
        let positions = [
            [0., 0., 0.],
            [1., 0., 0.],
            [1., 1., 0.],
            [0., 1., 0.],
            [0., 0., 1.],
            [1., 0., 1.],
            [1., 1., 1.],
            [0., 1., 1.],
        ]
        .map(|[x, y, v]| [x, y, v + z])
        .to_vec();
        let faces = [
            [0, 2, 1],
            [0, 3, 2],
            [4, 5, 6],
            [4, 6, 7],
            [0, 1, 5],
            [0, 5, 4],
            [1, 2, 6],
            [1, 6, 5],
            [2, 3, 7],
            [2, 7, 6],
            [3, 0, 4],
            [3, 4, 7],
        ]
        .into_iter()
        .map(|vertices| Face {
            vertices,
            flags: 0,
            extension: None,
            nt_extension: None,
        })
        .collect();
        NativeBodySurfacePreview {
            geometry: NativeBodyMeshGeometry {
                positions,
                faces,
                weights: vec![NativeBodyVertexWeights::default(); 8],
                uvs: Some(vec![[0.25, 0.75]; 8]),
            },
            top_face: vec![1.; 8],
            material_memberships: vec![vec![vec![65]]; 12],
        }
    }

    #[test]
    fn facing_closed_caps_are_replaced_without_moving_or_reweighting_sources() {
        let (a, b) = (cube(0.), cube(2.));
        let joint = [0.5, 0.5, 1.5];
        let cap = native_avatar_end_cap(&a, joint).unwrap();
        assert_eq!(cap.faces.len(), 2);
        assert_eq!(cap.perimeter.len(), 4);
        let repaired = repair_pair(&a, &b, joint, [3, 4]).unwrap();
        assert_eq!(repaired.removed_faces, [2, 2]);
        assert_eq!(repaired.bridge_faces, 8);
        assert_eq!(repaired.surface.geometry.faces.len(), 28);
        assert_eq!(
            &repaired.surface.geometry.positions[..8],
            a.geometry.positions
        );
        assert_eq!(
            &repaired.surface.geometry.positions[8..],
            b.geometry.positions
        );
        assert_eq!(a.geometry.faces.len(), 12);
        assert_eq!(b.geometry.faces.len(), 12);
        assert_eq!(
            repaired.surface.geometry.weights,
            vec![NativeBodyVertexWeights::default(); 16]
        );
        assert_eq!(repaired.surface.geometry.uvs, Some(vec![[0.25, 0.75]; 16]));
    }

    #[test]
    fn open_or_nonmanifold_sources_are_rejected() {
        let mut a = cube(0.);
        a.geometry.faces.pop();
        assert!(native_avatar_end_cap(&a, [0.5, 0.5, 1.5]).is_err());
        a.geometry.faces.push(a.geometry.faces[0].clone());
        assert!(native_avatar_end_cap(&a, [0.5, 0.5, 1.5]).is_err());
    }
}
