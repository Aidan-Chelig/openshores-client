//! Render-corner expansion after model-space smoothing and native bone skinning.
use super::{NativeDpAvatarGeometry, NativeDpBoneTransforms};
use crate::model::{NativeBodySurfacePreview, NativeBodyVertexWeights, crease_corner_normals};
use std::io;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeDpAvatarCorner {
    /// Face-major source vertex, retained for payload/UV/normal diagnostics.
    /// With seam repair, indexes the derived combined surface; its distal or
    /// root repair report identifies the original sources and vertex ranges.
    pub source_vertex: usize,
    /// Native homogeneous shader result. No weight normalization or W discard.
    pub position: [f32; 4],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub top_face: f32,
}

#[derive(Debug)]
pub struct NativeDpPosedAvatarSurface {
    pub surface_index: usize,
    /// Primary part. A modern root-repaired batch can contain additional parts;
    /// its explicit root repair report retains their vertex provenance.
    pub part_index: usize,
    /// Face order matches the input render surface, independent of material
    /// splits. The explicit repair path uses its derived combined surface.
    pub corners: Vec<NativeDpAvatarCorner>,
    pub material_memberships: Vec<Vec<Vec<u16>>>,
}

/// Immutable model-space corners, smoothed once before any animated bone pose.
/// UVs, material controls and raw influences survive every animation sample.
#[derive(Clone, Debug)]
pub struct NativeDpPreparedAvatarCorners {
    corners: Vec<NativeDpAvatarCorner>,
    weights: Vec<NativeBodyVertexWeights>,
}

/// Fixed topology/material partitioning for an animated render batch.
#[derive(Clone, Debug)]
pub struct NativeDpPreparedAvatarSurface {
    pub surface_index: usize,
    pub part_index: usize,
    pub corners: NativeDpPreparedAvatarCorners,
    pub material_memberships: Vec<Vec<Vec<u16>>>,
}

impl NativeDpPreparedAvatarSurface {
    pub fn from_surface(
        surface_index: usize,
        part_index: usize,
        surface: &NativeBodySurfacePreview,
        angle: f64,
    ) -> io::Result<Self> {
        Ok(Self {
            surface_index,
            part_index,
            corners: NativeDpPreparedAvatarCorners::from_surface(surface, angle)?,
            material_memberships: surface.material_memberships.clone(),
        })
    }
    pub fn skin(&self, bones: &NativeDpBoneTransforms) -> io::Result<NativeDpPosedAvatarSurface> {
        Ok(NativeDpPosedAvatarSurface {
            surface_index: self.surface_index,
            part_index: self.part_index,
            corners: self.corners.skin(bones)?,
            material_memberships: self.material_memberships.clone(),
        })
    }
}

impl NativeDpPreparedAvatarCorners {
    pub fn skin(&self, bones: &NativeDpBoneTransforms) -> io::Result<Vec<NativeDpAvatarCorner>> {
        self.corners
            .iter()
            .zip(&self.weights)
            .map(|(corner, weights)| {
                let result = bones.skin_shader_vertex(
                    [corner.position[0], corner.position[1], corner.position[2]],
                    corner.normal,
                    weights,
                )?;
                let mut corner = corner.clone();
                corner.position = result.position;
                corner.normal = result.normal;
                Ok(corner)
            })
            .collect()
    }
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

impl NativeDpAvatarGeometry {
    /// Smooth deformed/projected MODEL-space inputs first, then apply each
    /// corner's actual four-influence bone loop exactly once. Never regenerate
    /// crease normals from a bent/posed mesh or split material seams beforehand.
    pub fn posed_surfaces(
        &self,
        angle_degrees: f64,
    ) -> io::Result<Vec<NativeDpPosedAvatarSurface>> {
        self.prepare_surfaces(angle_degrees)?
            .iter()
            .map(|surface| surface.skin(self.body.bones()))
            .collect()
    }

    pub fn prepare_surfaces(
        &self,
        angle_degrees: f64,
    ) -> io::Result<Vec<NativeDpPreparedAvatarSurface>> {
        // Validate even if a caller supplies an empty public surface list.
        crease_corner_normals(&[], &[], angle_degrees)?;
        self.surfaces
            .iter()
            .enumerate()
            .map(|(surface_index, source)| {
                NativeDpPreparedAvatarSurface::from_surface(
                    surface_index,
                    source.part_index,
                    &source.surface,
                    angle_degrees,
                )
            })
            .collect()
    }
}

#[cfg(test)]
fn skin_corners(
    surface: &NativeBodySurfacePreview,
    bones: &NativeDpBoneTransforms,
    angle: f64,
) -> io::Result<Vec<NativeDpAvatarCorner>> {
    NativeDpPreparedAvatarCorners::from_surface(surface, angle)?.skin(bones)
}

impl NativeDpPreparedAvatarCorners {
    pub fn from_surface(surface: &NativeBodySurfacePreview, angle: f64) -> io::Result<Self> {
        let g = &surface.geometry;
        let uv = g
            .uvs
            .as_ref()
            .ok_or_else(|| invalid("avatar surface lacks UVs"))?;
        let count = g.positions.len();
        if g.weights.len() != count
            || uv.len() != count
            || surface.top_face.len() != count
            || surface.material_memberships.len() != g.faces.len()
            || uv.iter().flatten().any(|v| !v.is_finite())
            || surface
                .top_face
                .iter()
                .any(|v| !v.is_finite() || !(0.0..=1.).contains(v))
        {
            return Err(invalid("inconsistent avatar surface attributes"));
        }
        let positions: Vec<_> = g.positions.iter().map(|p| p.map(|v| v as f32)).collect();
        let triangles: Vec<_> = g.faces.iter().map(|f| f.vertices.map(u32::from)).collect();
        let normals = crease_corner_normals(&positions, &triangles, angle)?;
        let corners = triangles
            .iter()
            .flatten()
            .enumerate()
            .map(|(corner, index)| {
                let index = *index as usize;
                Ok(NativeDpAvatarCorner {
                    source_vertex: index,
                    position: [
                        positions[index][0],
                        positions[index][1],
                        positions[index][2],
                        1.,
                    ],
                    normal: normals[corner],
                    uv: uv[index],
                    top_face: surface.top_face[index],
                })
            })
            .collect::<io::Result<Vec<_>>>()?;
        let weights = corners
            .iter()
            .map(|corner| g.weights[corner.source_vertex])
            .collect();
        Ok(Self { corners, weights })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Face, NativeBodyMeshGeometry, NativeBodyVertexWeights};

    #[test]
    fn corners_skin_normals_before_material_splitting_and_preserve_w() {
        let mut payload = NativeBodyVertexWeights::default();
        payload.words_00_through_24[4] = 2_f32.to_bits();
        let surface = NativeBodySurfacePreview {
            geometry: NativeBodyMeshGeometry {
                positions: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
                weights: vec![payload; 4],
                faces: [[0, 1, 2], [0, 3, 1]]
                    .into_iter()
                    .map(|vertices| Face {
                        vertices,
                        flags: 0,
                        extension: None,
                        nt_extension: None,
                    })
                    .collect(),
                uvs: Some(vec![[0.1, 0.2], [0.3, 0.4], [0.5, 0.6], [0.7, 0.8]]),
            },
            top_face: vec![0., 1., 0., 1.],
            material_memberships: vec![vec![vec![65]], vec![vec![66]]],
        };
        let mut bones = NativeDpBoneTransforms::initialized();
        // Rotation about X, plus translation. Normals must not see translation.
        bones.slots[0].matrix_00 = [
            1., 0., 0., 0., 0., 0., 1., 0., 0., -1., 0., 0., 10., 20., 30., 1.,
        ];
        let result = skin_corners(&surface, &bones, 91.).unwrap();
        let prepared = NativeDpPreparedAvatarCorners::from_surface(&surface, 91.).unwrap();
        assert_eq!(prepared.skin(&bones).unwrap(), result);
        let rest = prepared
            .skin(&NativeDpBoneTransforms::initialized())
            .unwrap();
        assert_ne!(rest[0].position, result[0].position);
        assert_eq!(rest[0].uv, result[0].uv);
        assert_eq!(rest[0].source_vertex, result[0].source_vertex);
        assert_eq!(
            prepared.skin(&bones).unwrap(),
            result,
            "sampling must not accumulate transforms"
        );
        assert_eq!(result.len(), 6);
        assert_eq!(result[0].position, [20., 40., 60., 2.]);
        assert_eq!(result[1].position, [22., 40., 60., 2.]);
        assert_eq!(result[0].normal, result[3].normal);
        assert!((result[0].normal[1] + std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6);
        assert!((result[0].normal[2] - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6);
        assert_eq!(result[1].uv, [0.3, 0.4]);
        assert_eq!(result[1].top_face, 1.);
        assert_eq!(surface.geometry.weights, vec![payload; 4]);
        assert_eq!(surface.geometry.positions[1], [1., 0., 0.]);
        let hard = skin_corners(&surface, &bones, 90.).unwrap();
        assert_eq!(hard[0].normal, [0., -1., 0.]);
        assert_eq!(hard[3].normal, [0., 0., 1.]);
    }
}
