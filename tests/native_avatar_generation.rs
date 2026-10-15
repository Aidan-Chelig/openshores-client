//! Offline CPU generation from mounted resources, not a renderer smoke test.
use openshores_client::{
    asset_mount::AssetMount,
    model::ModelVariant,
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
    }
}
