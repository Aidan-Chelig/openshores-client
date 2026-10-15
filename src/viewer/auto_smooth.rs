//! Preview-only crease normals. Exact coincident positions join edge adjacency,
//! including duplicated seam vertices; no epsilon welding or source mutation.
use super::*;

pub(super) fn mesh(positions: &[[f32; 3]], triangles: &[[u32; 3]], degrees: f64) -> Mesh {
    let generated = crate::model::crease_corner_normals(positions, triangles, degrees)
        .expect("viewer validates geometry and smoothing angle before mesh construction");
    let expanded: Vec<_> = triangles
        .iter()
        .flatten()
        .map(|i| positions[*i as usize])
        .collect();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, expanded)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, generated)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn normals(mesh: &Mesh) -> &Vec<[f32; 3]> {
        let Some(bevy::mesh::VertexAttributeValues::Float32x3(values)) =
            mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
        else {
            panic!("missing normals")
        };
        values
    }

    #[test]
    fn threshold_splits_sharp_edges_and_smooths_duplicate_seams() {
        let points = [
            [0., 0., 0.],
            [1., 0., 0.],
            [0., 1., 0.],
            [0., 0., 0.],
            [1., 0., 0.],
            [0., 0., 1.],
        ];
        let faces = [[0, 1, 2], [3, 5, 4]];
        let hard = mesh(&points, &faces, 89.);
        assert_eq!(normals(&hard)[0], [0., 0., 1.]);
        assert_eq!(normals(&hard)[3], [0., 1., 0.]);
        let smooth = mesh(&points, &faces, 91.);
        assert_eq!(normals(&smooth)[0], normals(&smooth)[3]);
        assert!((normals(&smooth)[0][1] - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6);
        assert_eq!(normals(&smooth)[2], [0., 0., 1.]);
        // Exactly equal to the threshold is hard; zero degrees means flat.
        assert_eq!(normals(&mesh(&points, &faces, 90.))[0], [0., 0., 1.]);
        assert_eq!(normals(&mesh(&points, &faces, 0.))[0], [0., 0., 1.]);
    }

    #[test]
    fn area_weighting_and_disconnected_fans_and_degeneracy() {
        let points = [
            [0., 0., 0.],
            [2., 0., 0.],
            [0., 2., 0.],
            [0., 0., 1.],
            [-1., 0., 0.],
            [0., 0., -1.],
        ];
        let result = mesh(&points, &[[0, 1, 2], [0, 3, 1], [0, 4, 5], [0, 0, 0]], 100.);
        let n = normals(&result);
        assert!((n[0][2] / n[0][1] - 2.).abs() < 1e-6);
        assert_eq!(n[6], [0., -1., 0.]);
        assert_eq!(n[9], [0., 0., 1.]);
        assert!(n.iter().flatten().all(|v| v.is_finite()));
    }

    #[test]
    fn non_manifold_edges_stay_split_and_empty_geometry_is_safe() {
        let points = [
            [0., 0., 0.],
            [1., 0., 0.],
            [0., 1., 0.],
            [0., 0., 1.],
            [0., -1., 0.],
        ];
        let result = mesh(&points, &[[0, 1, 2], [0, 3, 1], [0, 1, 4]], 180.);
        assert_eq!(normals(&result)[0], [0., 0., 1.]);
        assert_eq!(normals(&result)[3], [0., 1., 0.]);
        assert_eq!(normals(&result)[6], [0., 0., -1.]);
        assert!(normals(&mesh(&[], &[], 60.)).is_empty());
        for angle in [f64::NAN, f64::INFINITY, -1., 181.] {
            assert!(run_auto_smooth(&Model::default(), None, false, angle).is_err());
        }
    }
}
