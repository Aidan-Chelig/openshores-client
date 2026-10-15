use super::*;

fn mesh(name: &[u8]) -> Mesh {
    Mesh {
        name: name.to_vec(),
        vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        faces: vec![Face {
            vertices: [0, 1, 2],
            flags: 0,
            extension: None,
            nt_extension: None,
        }],
        ..Default::default()
    }
}
fn group(name: &[u8], count: usize) -> VertexGroup {
    VertexGroup {
        field_order: vec![VertexGroupChunk::Name, VertexGroupChunk::Members],
        name: Some(name.to_vec()),
        members: Some(
            (0..count)
                .map(|index| GroupVertex {
                    position: [index as f32, 0., 0.],
                    weight: 1.,
                })
                .collect(),
        ),
    }
}

#[test]
fn helper_names_use_native_prefix_length_and_case_rules() {
    for variant in [
        ModelVariant::Standard,
        ModelVariant::N,
        ModelVariant::NT,
        ModelVariant::X,
    ] {
        assert_eq!(
            mesh(b"!3 weapon").role(variant),
            MeshRole::Weapon { index: 3 }
        );
        assert_eq!(mesh(b"!x").role(variant), MeshRole::Weapon { index: 0 });
        assert_eq!(
            mesh(b"@2fF command").role(variant),
            MeshRole::Station {
                index: 2,
                flags: 255
            }
        );
        assert_eq!(
            mesh(b"@x+f").role(variant),
            MeshRole::Station {
                index: 0,
                flags: 15
            }
        );
        assert_eq!(
            mesh(b"@2\xa0f").role(variant),
            MeshRole::Station {
                index: 2,
                flags: 15
            }
        );
        assert_eq!(
            mesh(b"@2f\x85").role(variant),
            MeshRole::Station {
                index: 2,
                flags: 15
            }
        );
        assert_eq!(
            mesh(b"tag UpVector helper").role(variant),
            MeshRole::UpVector
        );
        for name in [
            b"!".as_slice(),
            b"@2",
            b"@200",
            b"@2-1",
            b"@2GG",
            b"-floor",
            b"SPt helper",
            b"Upvector",
            b"DataOnly",
            b"x!3",
            b"x@2FF",
        ] {
            assert_eq!(mesh(name).role(variant), MeshRole::Geometry, "{name:?}");
        }
    }
}

#[test]
fn native_object_acceptance_is_separate_from_helper_classification() {
    let mut helper = mesh(b"!3");
    helper.faces.clear();
    assert_eq!(helper.role(ModelVariant::Standard), MeshRole::Rejected);
    assert_eq!(helper.role(ModelVariant::X), MeshRole::Rejected);
    assert_eq!(helper.role(ModelVariant::N), MeshRole::Weapon { index: 3 });
    assert_eq!(helper.role(ModelVariant::NT), MeshRole::Weapon { index: 3 });
    helper.name = b"DataOnly".to_vec();
    assert_eq!(helper.role(ModelVariant::N), MeshRole::Geometry);
    helper.name = b"prefixedUpVector".to_vec();
    assert_eq!(helper.role(ModelVariant::N), MeshRole::Rejected); // No-face exception is exact.
    helper.vertices.clear();
    assert_eq!(helper.role(ModelVariant::N), MeshRole::Rejected);
}

#[test]
fn paired_screens_require_four_unique_positions_and_obey_helper_precedence() {
    let mut screen = mesh(b"console");
    screen.vertex_groups = vec![group(b"Screen1", 4), group(b"Screen2", 4)];
    assert_eq!(screen.role(ModelVariant::N), MeshRole::Screens);
    screen.name = b"@2FF".to_vec();
    assert!(matches!(
        screen.role(ModelVariant::N),
        MeshRole::Station { .. }
    ));
    screen.name = b"!3".to_vec();
    assert_eq!(screen.role(ModelVariant::N), MeshRole::Weapon { index: 3 });
    screen.name = b"UpVector".to_vec();
    assert_eq!(screen.role(ModelVariant::N), MeshRole::Screens);
    screen.vertex_groups.push(group(b"Screen1", 3)); // Later invalid group aborts the screen test.
    assert_eq!(screen.role(ModelVariant::N), MeshRole::UpVector);
    screen.name = b"console".to_vec();
    assert_eq!(screen.role(ModelVariant::N), MeshRole::Geometry);
    screen.vertex_groups = vec![group(b"Screen1", 4), group(b"Screen2", 4)];
    screen.vertex_groups[0].members.as_mut().unwrap()[3].position = [-0., 0., 0.];
    assert_eq!(screen.role(ModelVariant::N), MeshRole::Geometry);
    screen.vertex_groups[0].name = Some(b"screen1".to_vec());
    assert_eq!(screen.role(ModelVariant::N), MeshRole::Geometry);
}

#[test]
fn native_screen_descriptors_persist_across_the_model_object_traversal() {
    let mut first = mesh(b"first screen");
    first.vertex_groups = vec![group(b"Screen1", 4)];
    let mut second = mesh(b"second screen");
    second.vertex_groups = vec![group(b"Screen2", 4)];
    assert_eq!(first.role(ModelVariant::N), MeshRole::Geometry);
    assert_eq!(second.role(ModelVariant::N), MeshRole::Geometry);
    let mut bad = mesh(b"UpVector");
    bad.vertex_groups = vec![group(b"Screen1", 3)];
    let model = Model {
        meshes: vec![
            first,
            second,
            mesh(b"later geometry"),
            bad,
            mesh(b"after failed screen"),
        ],
        ..Default::default()
    };
    assert_eq!(
        model.mesh_roles(ModelVariant::N),
        vec![
            MeshRole::Geometry,
            MeshRole::Screens,
            MeshRole::Screens,
            MeshRole::UpVector,
            MeshRole::Screens
        ]
    );
}
