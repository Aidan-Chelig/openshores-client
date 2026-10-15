//! CPU body geometry carrying verified deformation and raw native skin weights.
use super::{Face, Mesh, ModelVariant, NativeBodyModelGeometry, NativeBodyVertexWeights};
use std::io;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeBodyMeshGeometry {
    pub positions: Vec<[f64; 3]>,
    pub weights: Vec<NativeBodyVertexWeights>,
    pub faces: Vec<Face>,
    pub uvs: Option<Vec<[f32; 2]>>,
}

impl Mesh {
    /// Preserve source vertex/face indexing while computing body coordinates and
    /// weights. This is not a native cache merge, bone pose, or renderer-axis map.
    /// Nonfinite native weight results are retained, not silently repaired.
    pub fn body_defined_geometry(
        &self,
        variant: ModelVariant,
        loader_scale: f64,
        geometry: &NativeBodyModelGeometry,
        bones: [i32; 3],
    ) -> io::Result<NativeBodyMeshGeometry> {
        Ok(NativeBodyMeshGeometry {
            positions: self.body_defined_positions(variant, loader_scale, geometry)?,
            weights: self.native_body_vertex_weights(variant, bones)?,
            faces: self.faces.clone(),
            uvs: self.uvs.clone(),
        })
    }
}
