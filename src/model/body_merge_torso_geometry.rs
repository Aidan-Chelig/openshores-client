//! Composed normal torso geometry stages; excludes crest/model-instance paths.
use super::{
    ModelVariant, NativeBodyMergeGeometry, NativeTorsoVertexMapping, NativeTorsoVertexOptions,
};
use std::io;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeTorsoGeometryReport {
    pub vertices: NativeTorsoVertexMapping,
    pub fully_connected: bool,
    pub updated_corners: usize,
    pub group_map_updates: usize,
}

impl NativeBodyMergeGeometry {
    /// Existing seeded destination, non-crest branch only. Unlike head this
    /// does not inherit the source connection point or append face-material
    /// membership. Source/destination instances and materials remain external.
    /// Earlier writes survive a later stage error; no publication is implied.
    pub fn append_torso_geometry_stages(
        &mut self,
        source: &mut Self,
        variant: ModelVariant,
        tolerance: f64,
    ) -> io::Result<NativeTorsoGeometryReport> {
        let last_bottom_group = self.object.bake_torso_once(
            &self.deformation,
            self.scale_points[0],
            self.scale_points[1],
        )?;
        let vertices = self.object.append_torso_vertices(
            &mut source.object,
            NativeTorsoVertexOptions {
                deformation: source.deformation,
                scale_points: source.scale_points,
                tolerance,
                source_mapping_count: source.mapping_coordinates.len(),
                last_bottom_group,
            },
        )?;
        self.mapping_coordinates
            .extend_from_slice(&vertices.appended_mapping_coordinates);
        let normals = vertices.collect_torso_connection_normals(
            variant,
            &source.faces,
            &self.object.vertices,
        )?;
        self.faces
            .extend(vertices.torso_appended_faces(&source.faces)?);
        let updated_corners = vertices.apply_torso_connection_normals(
            variant,
            &normals,
            &self.object.vertices,
            &mut self.faces,
        )?;
        let group_map_updates = self.transfer_torso_group_maps(source)?;
        // Identical verified format rule, after torso's distinct group stage.
        self.metadata
            .finish_head_smoothing(variant, self.faces.len())?;
        Ok(NativeTorsoGeometryReport {
            vertices,
            fully_connected: normals.fully_connected,
            updated_corners,
            group_map_updates,
        })
    }
}
