//! Render-corner expansion after model-space smoothing and native bone skinning.
use super::{NativeDpAvatarGeometry, NativeDpBoneTransforms};
use crate::model::{NativeBodySurfacePreview, crease_corner_normals};
use std::io;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeDpAvatarCorner {
    /// Face-major source vertex, retained for payload/UV/normal diagnostics.
    /// With seam repair, indexes the derived combined surface; its repair
    /// report identifies the two sources and their vertex-index split.
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
    pub part_index: usize,
    /// Face order matches the input render surface, independent of material
    /// splits. The explicit repair path uses its derived combined surface.
    pub corners: Vec<NativeDpAvatarCorner>,
    pub material_memberships: Vec<Vec<Vec<u16>>>,
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
        // Validate even if a caller supplies an empty public surface list.
        crease_corner_normals(&[], &[], angle_degrees)?;
        self.surfaces
            .iter()
            .enumerate()
            .map(|(surface_index, source)| {
                Ok(NativeDpPosedAvatarSurface {
                    surface_index,
                    part_index: source.part_index,
                    corners: skin_corners(&source.surface, self.body.bones(), angle_degrees)?,
                    material_memberships: source.surface.material_memberships.clone(),
                })
            })
            .collect()
    }
}

pub(super) fn skin_corners(
    surface: &NativeBodySurfacePreview,
    bones: &NativeDpBoneTransforms,
    angle: f64,
) -> io::Result<Vec<NativeDpAvatarCorner>> {
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
    triangles
        .iter()
        .flatten()
        .enumerate()
        .map(|(corner, index)| {
            let index = *index as usize;
            let result =
                bones.skin_shader_vertex(positions[index], normals[corner], &g.weights[index])?;
            Ok(NativeDpAvatarCorner {
                source_vertex: index,
                position: result.position,
                normal: result.normal,
                uv: uv[index],
                top_face: surface.top_face[index],
            })
        })
        .collect()
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
