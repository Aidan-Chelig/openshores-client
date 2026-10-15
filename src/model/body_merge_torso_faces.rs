//! Normal/non-crest torso faces and connection corners. Not the crest branch.
use super::{
    ModelVariant, NativeBodyLoadedFace, NativeBodyStoredVertex, NativeHeadConnectionNormals,
    NativeHeadVertexMapping, NativeTorsoVertexMapping,
};
use std::io;

/// Shared native point-map/result shape. For the normal torso branch (bool=false)
/// invalid secondary mappings have the same sticky-false behavior as head.
pub type NativeTorsoConnectionNormals = NativeHeadConnectionNormals;

impl NativeTorsoVertexMapping {
    // Face loops consume these tables by membership/key/value, not their
    // different vertex-stage construction semantics. Keep the actual torso
    // tables intact while reusing the verified identical face arithmetic.
    fn face_stage_mapping(&self) -> NativeHeadVertexMapping {
        NativeHeadVertexMapping {
            source_to_destination: self.source_to_destination.clone(),
            existing_to_appended: self.primary.clone(),
            appended_to_connection: self.secondary.clone(),
        }
    }

    /// Torso builds the same fresh face fields and u16 index narrowing as head;
    /// it does NOT run head's material-membership loop after each append.
    pub fn torso_appended_faces(
        &self,
        faces: &[NativeBodyLoadedFace],
    ) -> io::Result<Vec<NativeBodyLoadedFace>> {
        self.face_stage_mapping().head_appended_faces(faces)
    }

    /// Only normal torso merging reaches this loop. Crest merging takes an
    /// earlier distinct instance/object append branch, not this geometry path.
    pub fn collect_torso_connection_normals(
        &self,
        variant: ModelVariant,
        source_faces: &[NativeBodyLoadedFace],
        vertices: &[NativeBodyStoredVertex],
    ) -> io::Result<NativeTorsoConnectionNormals> {
        self.face_stage_mapping()
            .collect_head_connection_normals(variant, source_faces, vertices)
    }

    pub fn apply_torso_connection_normals(
        &self,
        variant: ModelVariant,
        normals: &NativeTorsoConnectionNormals,
        vertices: &[NativeBodyStoredVertex],
        faces: &mut [NativeBodyLoadedFace],
    ) -> io::Result<usize> {
        self.face_stage_mapping()
            .apply_head_connection_normals(variant, normals, vertices, faces)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    #[test]
    fn torso_face_stage_uses_actual_torso_tables_and_preserves_metadata_boundaries() {
        let mapping = NativeTorsoVertexMapping {
            source_to_destination: vec![1],
            primary: BTreeMap::from([(0, 0), (1, 1)]),
            secondary: BTreeMap::from([(1, 0)]),
            ..Default::default()
        };
        let vertices = vec![
            NativeBodyStoredVertex {
                position: [0.; 3],
                payload: Default::default()
            };
            2
        ];
        let face = NativeBodyLoadedFace {
            vertices: [0; 3],
            flags_06: 7,
            fields_08: [9.; 9],
            fields_50: [2.; 9],
            field_98: 0.5,
            fields_9c: [8; 3],
        };
        let mut faces = vec![face.clone()];
        faces.extend(
            mapping
                .torso_appended_faces(std::slice::from_ref(&face))
                .unwrap(),
        );
        assert_eq!(faces[1].vertices, [1; 3]);
        assert_eq!(faces[1].flags_06, 0);
        assert_eq!(faces[1].fields_08, [0.; 9]);
        assert_eq!(faces[1].field_98, 1.);
        let normals = mapping
            .collect_torso_connection_normals(ModelVariant::N, &[face], &vertices)
            .unwrap();
        assert!(normals.fully_connected);
        assert_eq!(
            mapping
                .apply_torso_connection_normals(ModelVariant::N, &normals, &vertices, &mut faces)
                .unwrap(),
            6
        );
        assert!(faces.iter().all(|f| f.fields_50 == [2.; 9]));
        assert_eq!(faces[0].flags_06, 7);
        let mut invalid = mapping;
        invalid.secondary.clear();
        assert!(
            !invalid
                .collect_torso_connection_normals(ModelVariant::NT, &faces[0..1], &vertices)
                .unwrap()
                .fully_connected
        );
        assert!(
            invalid
                .collect_torso_connection_normals(ModelVariant::X, &faces, &[])
                .unwrap()
                .fully_connected
        );
    }
}
