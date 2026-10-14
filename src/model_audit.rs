//! Structural diagnostics, not a verdict about original rendering or collision.
use crate::{model::Model, model_preview};
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
};

const MAX_WORK: usize = 3_000_000;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct MeshAudit {
    pub unused_vertices: usize,
    pub repeated_index_faces: usize,
    pub zero_area_faces: usize,
    /// Distinct triangle incidence on raw index edges; no positional welding.
    pub boundary_edges: usize,
    pub nonmanifold_edges: usize,
    pub unassigned_faces: usize,
    pub multiply_assigned_faces: usize,
    pub missing_material_groups: usize,
    pub ambiguous_material_groups: usize,
    pub uv_count: Option<usize>,
    /// Presence only: extension floats still have no assigned shading semantics.
    pub extension_faces: usize,
}

pub fn audit(model: &Model) -> io::Result<Vec<MeshAudit>> {
    model_preview::bounds(model)?;
    let mut names = BTreeMap::<&[u8], usize>::new();
    let mut work = 0usize;
    let mut charge = |amount: usize| -> io::Result<()> {
        work = work
            .checked_add(amount)
            .filter(|work| *work <= MAX_WORK)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "model audit exceeds work budget",
                )
            })?;
        Ok(())
    };
    for material in &model.materials {
        charge(1)?;
        if let Some(name) = &material.name {
            *names.entry(name).or_default() += 1;
        }
    }
    let mut results = Vec::new();
    for mesh in &model.meshes {
        charge(mesh.vertices.len() + mesh.faces.len())?;
        let mut report = MeshAudit {
            uv_count: mesh.uvs.as_ref().map(Vec::len),
            ..Default::default()
        };
        let mut used = vec![false; mesh.vertices.len()];
        let mut incidence = BTreeMap::<(u16, u16), usize>::new();
        let mut assigned = vec![0usize; mesh.faces.len()];
        for face in &mesh.faces {
            charge(3)?;
            let [a, b, c] = face.vertices;
            for index in face.vertices {
                used[index as usize] = true;
            }
            if a == b || b == c || a == c {
                report.repeated_index_faces += 1;
            }
            let points = face
                .vertices
                .map(|index| mesh.vertices[index as usize].map(f64::from));
            let ab = std::array::from_fn::<_, 3, _>(|axis| points[1][axis] - points[0][axis]);
            let ac = std::array::from_fn::<_, 3, _>(|axis| points[2][axis] - points[0][axis]);
            let cross = [
                ab[1] * ac[2] - ab[2] * ac[1],
                ab[2] * ac[0] - ab[0] * ac[2],
                ab[0] * ac[1] - ab[1] * ac[0],
            ];
            if cross == [0.0; 3] {
                report.zero_area_faces += 1;
            }
            let edges: BTreeSet<_> = [(a, b), (b, c), (c, a)]
                .into_iter()
                .filter(|(a, b)| a != b)
                .map(|(a, b)| (a.min(b), a.max(b)))
                .collect();
            for edge in edges {
                *incidence.entry(edge).or_default() += 1;
            }
            report.extension_faces += usize::from(face.extension.is_some());
        }
        for group in &mesh.materials {
            charge(1 + group.faces.len())?;
            match names.get(group.name.as_slice()).copied().unwrap_or(0) {
                0 => report.missing_material_groups += 1,
                1 => {}
                _ => report.ambiguous_material_groups += 1,
            }
            for face in &group.faces {
                let count = assigned.get_mut(*face as usize).ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "audit material references invalid face",
                    )
                })?;
                *count += 1;
            }
        }
        report.unused_vertices = used.iter().filter(|used| !**used).count();
        report.boundary_edges = incidence.values().filter(|count| **count == 1).count();
        report.nonmanifold_edges = incidence.values().filter(|count| **count > 2).count();
        report.unassigned_faces = assigned.iter().filter(|count| **count == 0).count();
        report.multiply_assigned_faces = assigned.iter().filter(|count| **count > 1).count();
        results.push(report);
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Face, FaceMaterial, Material, Mesh};

    fn face(vertices: [u16; 3]) -> Face {
        Face {
            vertices,
            flags: 0,
            extension: None,
        }
    }
    fn mesh() -> Mesh {
        Mesh {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [8.0, 8.0, 8.0],
            ],
            faces: vec![
                face([0, 1, 2]),
                face([0, 1, 3]),
                face([0, 1, 2]),
                face([0, 0, 1]),
            ],
            ..Default::default()
        }
    }

    #[test]
    fn topology_and_ambiguous_bindings_remain_visible() {
        let mut mesh = mesh();
        mesh.materials = vec![
            FaceMaterial {
                name: b"duplicate".to_vec(),
                faces: vec![0, 1],
            },
            FaceMaterial {
                name: b"missing".to_vec(),
                faces: vec![0],
            },
        ];
        let model = Model {
            meshes: vec![mesh],
            materials: vec![
                Material {
                    name: Some(b"duplicate".to_vec()),
                    ..Default::default()
                },
                Material {
                    name: Some(b"duplicate".to_vec()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let report = &audit(&model).unwrap()[0];
        assert_eq!(report.unused_vertices, 1);
        assert_eq!(report.repeated_index_faces, 1);
        assert_eq!(report.zero_area_faces, 1);
        assert_eq!(report.nonmanifold_edges, 1);
        assert_eq!(report.boundary_edges, 2);
        assert_eq!(report.unassigned_faces, 2);
        assert_eq!(report.multiply_assigned_faces, 1);
        assert_eq!(report.missing_material_groups, 1);
        assert_eq!(report.ambiguous_material_groups, 1);
    }

    #[test]
    fn invalid_material_indices_fail_without_mutation() {
        let mut mesh = mesh();
        mesh.materials.push(FaceMaterial {
            name: vec![],
            faces: vec![99],
        });
        let model = Model {
            meshes: vec![mesh],
            ..Default::default()
        };
        assert!(audit(&model).is_err());
        assert_eq!(model.meshes[0].materials[0].faces, vec![99]);
        assert!(audit(&Model::default()).unwrap().is_empty());
    }

    #[test]
    fn assignment_budget_is_checked_before_iteration() {
        let mut mesh = mesh();
        mesh.materials.push(FaceMaterial {
            name: vec![],
            faces: vec![0; MAX_WORK],
        });
        let model = Model {
            meshes: vec![mesh],
            ..Default::default()
        };
        assert!(
            audit(&model)
                .unwrap_err()
                .to_string()
                .contains("work budget")
        );
    }
}
