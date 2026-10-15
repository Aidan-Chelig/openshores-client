//! Head-merge N/NT connection corner collection and final field50 updates.
use super::{
    ModelVariant, NativeBodyLoadedFace, NativeBodyStoredVertex, NativeBodyVertexPointMap,
    NativeHeadVertexMapping, invalid,
};
use std::io;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NativeHeadConnectionNormals {
    pub points: NativeBodyVertexPointMap,
    /// Native merge returns false on missing/invalid secondary connection
    /// mappings, but continues constructing faces. Not a Rust decode failure.
    pub fully_connected: bool,
}

impl NativeHeadVertexMapping {
    pub fn collect_head_connection_normals(
        &self,
        variant: ModelVariant,
        source_faces: &[NativeBodyLoadedFace],
        vertices: &[NativeBodyStoredVertex],
    ) -> io::Result<NativeHeadConnectionNormals> {
        let mut result = NativeHeadConnectionNormals {
            fully_connected: true,
            ..Default::default()
        };
        if !matches!(variant, ModelVariant::N | ModelVariant::NT) {
            return Ok(result);
        }
        for face in source_faces {
            for (corner, source_index) in face.vertices.into_iter().enumerate() {
                let mapped = *self
                    .source_to_destination
                    .get(usize::from(source_index))
                    .ok_or_else(|| invalid("head normal source index is unmapped"))?;
                if !self.existing_to_appended.contains_key(&mapped) {
                    continue;
                }
                let Some(vertex) = self
                    .appended_to_connection
                    .get(&mapped)
                    .and_then(|index| vertices.get(*index))
                else {
                    result.fully_connected = false;
                    continue;
                };
                result.points.insert_first(
                    vertex,
                    std::array::from_fn(|axis| face.fields_50[corner * 3 + axis]),
                )?;
            }
        }
        Ok(result)
    }

    /// All destination faces, including previously existing faces, participate.
    /// Only corners whose indices exist in the primary remap are looked up.
    pub fn apply_head_connection_normals(
        &self,
        variant: ModelVariant,
        normals: &NativeHeadConnectionNormals,
        vertices: &[NativeBodyStoredVertex],
        faces: &mut [NativeBodyLoadedFace],
    ) -> io::Result<usize> {
        if !matches!(variant, ModelVariant::N | ModelVariant::NT) {
            return Ok(0);
        }
        let mut changed = 0;
        for face in faces {
            for (corner, index) in face.vertices.into_iter().map(usize::from).enumerate() {
                if !self.existing_to_appended.contains_key(&index) {
                    continue;
                }
                let vertex = vertices
                    .get(index)
                    .ok_or_else(|| invalid("head normal destination index is missing"))?;
                if let Some(point) = normals.points.get(vertex)? {
                    face.fields_50[corner * 3..corner * 3 + 3].copy_from_slice(&point);
                    changed += 1;
                }
            }
        }
        Ok(changed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    #[test]
    fn connection_normals_keep_first_corner_and_update_old_and_new_faces() {
        let vertices = vec![
            NativeBodyStoredVertex {
                position: [0.; 3],
                payload: Default::default()
            };
            3
        ];
        let mapping = NativeHeadVertexMapping {
            source_to_destination: vec![1, 2],
            existing_to_appended: BTreeMap::from([(0, 1), (1, 1), (2, 2)]),
            appended_to_connection: BTreeMap::from([(1, 0), (2, 0)]),
        };
        let source = NativeBodyLoadedFace {
            vertices: [0, 1, 0],
            flags_06: 7,
            fields_08: [9.; 9],
            fields_50: [1., 2., 3., 4., 5., 6., 7., 8., 9.],
            field_98: 0.5,
            fields_9c: [10; 3],
        };
        let normals = mapping
            .collect_head_connection_normals(
                ModelVariant::N,
                std::slice::from_ref(&source),
                &vertices,
            )
            .unwrap();
        assert!(normals.fully_connected);
        let mut faces = vec![source.clone(), source.clone()];
        faces[0].vertices = [0, 0, 0];
        faces[1].vertices = [1, 2, 1];
        assert_eq!(
            mapping
                .apply_head_connection_normals(ModelVariant::N, &normals, &vertices, &mut faces)
                .unwrap(),
            6
        );
        for face in faces {
            assert_eq!(face.fields_50, [1., 2., 3., 1., 2., 3., 1., 2., 3.]);
            assert_eq!(face.fields_08, source.fields_08);
        }
        let mut invalid = mapping;
        invalid.appended_to_connection.clear();
        assert!(
            !invalid
                .collect_head_connection_normals(
                    ModelVariant::NT,
                    std::slice::from_ref(&source),
                    &vertices
                )
                .unwrap()
                .fully_connected
        );
        assert!(
            invalid
                .collect_head_connection_normals(ModelVariant::Standard, &[source], &[])
                .unwrap()
                .fully_connected
        );
    }
}
