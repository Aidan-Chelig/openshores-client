//! Composition of reconstructed head object-geometry stages, not model cache.
use super::{
    ModelVariant, NativeBodyDeformation, NativeBodyFaceMetadata, NativeBodyLoadedFace,
    NativeBodyMergeBake, NativeHeadVertexMapping,
};
use std::io;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeBodyMergeGeometry {
    pub object: NativeBodyMergeBake,
    pub faces: Vec<NativeBodyLoadedFace>,
    pub metadata: NativeBodyFaceMetadata,
    pub deformation: NativeBodyDeformation,
    pub scale_points: [[f32; 3]; 2],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeHeadGeometryReport {
    pub vertices: NativeHeadVertexMapping,
    pub fully_connected: bool,
    pub updated_corners: usize,
}

impl NativeBodyMergeGeometry {
    /// Destination object must already be seeded by the instance owner. This
    /// composes only the reconstructed first-object geometry/face stages; it
    /// neither copies model instances nor claims material-object/cache ownership.
    /// Earlier stage writes can survive a later Unsupported/error result.
    pub fn append_head_geometry_stages(
        &mut self,
        source: &mut Self,
        variant: ModelVariant,
        tolerance: f64,
    ) -> io::Result<NativeHeadGeometryReport> {
        self.object.bake_once(
            &self.deformation,
            self.scale_points[0],
            self.scale_points[1],
        )?;
        // Native copies the destination object's connection vertex at1cc into
        // source1cc before source deformation. Only its xyz is consumed here.
        source.deformation.base.fields_1f4 = self.deformation.base.fields_1f4;
        let vertices = self.object.append_head_vertices(
            &mut source.object,
            &source.deformation,
            source.scale_points,
            tolerance,
        )?;
        let normals = vertices.collect_head_connection_normals(
            variant,
            &source.faces,
            &self.object.vertices,
        )?;
        let before = self.faces.len();
        self.faces
            .extend(vertices.head_appended_faces(&source.faces)?);
        self.metadata.append_head_material_membership(
            &source.metadata,
            before,
            source.faces.len(),
        )?;
        let updated_corners = vertices.apply_head_connection_normals(
            variant,
            &normals,
            &self.object.vertices,
            &mut self.faces,
        )?;
        self.metadata
            .finish_head_smoothing(variant, self.faces.len())?;
        Ok(NativeHeadGeometryReport {
            vertices,
            fully_connected: normals.fully_connected,
            updated_corners,
        })
    }
}

#[cfg(test)]
mod tests;
