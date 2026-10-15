//! Offline CPU generation from mounted resources, not a renderer smoke test.
use openshores_client::{
    asset_mount::AssetMount,
    model::{ModelVariant, crease_corner_normals},
    world::{native_body::NativeDpAvatarGeometry, native_dna::NativeDhDna},
};

#[test]
#[ignore = "requires OPENSHORES_HAZERON_INSTALLATION; no server, original binary or GPU"]
fn mounted_complete_native_avatar_inputs_cover_two_anatomies() {
    let mount =
        AssetMount::open(std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").unwrap()).unwrap();
    for second in [false, true] {
        let mut dna = NativeDhDna([0; 24]);
        dna.0[1] = if second { 2 } else { 1 };
        dna.0[12] = if second { 8 } else { 4 };
        dna.0[14] = if second { 0x91 } else { 0x51 };
        dna.0[20] = if second { 0x80 } else { 0x60 };
        if second {
            dna.0[5] = 2;
            dna.0[13] = 0x60;
            dna.0[15] = 0x40;
        }
        let avatar = NativeDpAvatarGeometry::generate(&mount, &dna, 0, 0).unwrap();
        assert_eq!(avatar.dna, dna);
        assert_eq!(avatar.standing.pose.fields_3c, [0x24; 10]);
        // Native two-head planning emits one shared/bifurcated Neck22 object,
        // not a separately duplicated neck mesh for each head.
        assert_eq!(avatar.surfaces.len(), if second { 21 } else { 12 });
        let mut faces = 0;
        for generated in &avatar.surfaces {
            let surface = &generated.surface;
            let g = &surface.geometry;
            assert_eq!(g.positions.len(), g.weights.len());
            assert_eq!(g.uvs.as_ref().unwrap().len(), g.positions.len());
            assert_eq!(surface.top_face.len(), g.positions.len());
            assert_eq!(surface.material_memberships.len(), g.faces.len());
            assert!(g.positions.iter().flatten().all(|v| v.is_finite()));
            assert!(!g.faces.is_empty());
            let mut seen = vec![false; g.positions.len()];
            for face in &g.faces {
                for index in face.vertices {
                    let index = usize::from(index);
                    assert!(index < g.positions.len());
                    if !seen[index] {
                        let result = avatar
                            .body
                            .bones()
                            .skin_shader_vertex(
                                g.positions[index].map(|v| v as f32),
                                [0., 1., 0.],
                                &g.weights[index],
                            )
                            .unwrap_or_else(|e| {
                                panic!("{} vertex{index}: {e}", generated.sources[0].resource)
                            });
                        assert!(result.position.iter().all(|v| v.is_finite()));
                        seen[index] = true;
                    }
                }
            }
            if !generated.native.object.field_369 {
                assert_eq!(generated.sources.len(), 1);
                let source = &generated.sources[0].mesh;
                assert_eq!(
                    g.positions,
                    source
                        .body_deformed_positions(
                            ModelVariant::N,
                            1.,
                            &generated.native.deformation,
                            generated.native.scale_points[0],
                            generated.native.scale_points[1]
                        )
                        .unwrap()
                );
            }
            faces += g.faces.len();
            println!(
                "fixture{} part{} {}: {} vertices, {} faces",
                u8::from(second),
                generated.part_index,
                generated.sources[0].resource,
                g.positions.len(),
                g.faces.len()
            );
        }
        assert!(faces > 2000);
        for index in 0..avatar.body.parts().len() {
            assert!(avatar.surfaces.iter().any(|s| s.part_index == index));
        }
        // The actual native wrist/ankle pivots must be shared by adjacent
        // posed bones; never fix a geometry gap with a made-up joint offset.
        for (part_index, part) in avatar.body.parts().iter().enumerate() {
            use openshores_client::world::native_body::NativeDpPartJoints;
            let joint = match part.joints {
                NativeDpPartJoints::Arm { field_70, .. } => field_70,
                NativeDpPartJoints::Leg { field_68, .. } => field_68,
                _ => continue,
            };
            let indices = part.limb_bones().unwrap();
            let proximal = avatar.body.bones().slots[indices[1]].project(joint);
            let distal = avatar.body.bones().slots[indices[2]].project(joint);
            for axis in 0..3 {
                assert!((proximal[axis] - distal[axis]).abs() < 1e-10);
            }
            let surfaces: Vec<_> = avatar
                .surfaces
                .iter()
                .filter(|s| s.part_index == part_index)
                .collect();
            assert_eq!(surfaces.len(), 2);
            for (axis, expected) in joint.into_iter().enumerate() {
                assert_eq!(
                    surfaces[1].native.deformation.base.fields_1c0[axis],
                    expected as f32
                );
            }
        }
        let posed = avatar.posed_surfaces(60.).unwrap();
        assert_eq!(posed.len(), avatar.surfaces.len());
        for (surface_index, (posed, source)) in posed.iter().zip(&avatar.surfaces).enumerate() {
            let g = &source.surface.geometry;
            assert_eq!(posed.surface_index, surface_index);
            assert_eq!(posed.part_index, source.part_index);
            assert_eq!(
                posed.material_memberships,
                source.surface.material_memberships
            );
            assert_eq!(posed.corners.len(), g.faces.len() * 3);
            let positions: Vec<_> = g.positions.iter().map(|p| p.map(|v| v as f32)).collect();
            let triangles: Vec<_> = g.faces.iter().map(|f| f.vertices.map(u32::from)).collect();
            let normals = crease_corner_normals(&positions, &triangles, 60.).unwrap();
            for (i, corner) in posed.corners.iter().enumerate() {
                let index = usize::from(g.faces[i / 3].vertices[i % 3]);
                assert_eq!(corner.source_vertex, index);
                let expected = avatar
                    .body
                    .bones()
                    .skin_shader_vertex(positions[index], normals[i], &g.weights[index])
                    .unwrap();
                assert_eq!(corner.position, expected.position);
                assert_eq!(corner.normal, expected.normal);
                assert_eq!(corner.uv, g.uvs.as_ref().unwrap()[index]);
                assert_eq!(corner.top_face, source.surface.top_face[index]);
                let length = corner.normal.into_iter().map(|v| v * v).sum::<f32>().sqrt();
                assert!((length - 1.).abs() < 1e-5);
            }
        }
        assert!(avatar.posed_surfaces(f64::NAN).is_err());
        for part_index in 0..avatar.body.parts().len() {
            let surfaces: Vec<_> = posed
                .iter()
                .filter(|s| s.part_index == part_index)
                .collect();
            if surfaces.len() != 2 {
                continue;
            }
            let points =
                |surface: &openshores_client::world::native_body::NativeDpPosedAvatarSurface| {
                    surface
                        .corners
                        .iter()
                        .map(|c| {
                            (
                                c.source_vertex,
                                std::array::from_fn::<_, 3, _>(|i| {
                                    f64::from(c.position[i]) / f64::from(c.position[3])
                                }),
                            )
                        })
                        .collect::<std::collections::BTreeMap<_, _>>()
                };
            let first = points(surfaces[0]);
            let second_surface = points(surfaces[1]);
            let mut nearest = f64::INFINITY;
            for a in first.values() {
                for b in second_surface.values() {
                    let distance = (0..3).map(|i| (a[i] - b[i]).powi(2)).sum::<f64>().sqrt();
                    nearest = nearest.min(distance);
                }
            }
            println!(
                "fixture{} part{part_index}: closest surface vertex separation {nearest}",
                u8::from(second)
            );
        }
    }
}
