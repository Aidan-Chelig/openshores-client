use crate::model::*;

fn source(uv: [f32; 2], name: &[u8]) -> (Mesh, NativeBodyMergeGeometry) {
    let mesh = Mesh {
        vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        geometry_order: vec![GeometryChunk::Vertices, GeometryChunk::MappingCoordinates],
        faces: vec![Face {
            vertices: [0, 1, 2],
            flags: 0,
            extension: Some([0.; 9]),
            nt_extension: None,
        }],
        uvs: Some(vec![uv; 3]),
        vertex_groups: vec![VertexGroup {
            name: Some(b"BottomSkin".to_vec()),
            members: Some(vec![
                GroupVertex {
                    position: [0.; 3],
                    weight: 0.,
                },
                GroupVertex {
                    position: [99.; 3],
                    weight: 1.,
                },
            ]),
            field_order: vec![VertexGroupChunk::Name, VertexGroupChunk::Members],
        }],
        ..Default::default()
    };
    let definition = NativeBodyModelGeometry {
        fields_00: [1.; 6],
        fields_40: [0.; 3],
        fields_58: [1.; 5],
    };
    let mut metadata = NativeBodyFaceMetadata::default();
    metadata
        .materials
        .insert(name.iter().copied().map(u16::from).collect(), vec![0]);
    let geometry = mesh
        .body_merge_geometry(ModelVariant::N, &definition, [0; 3], metadata)
        .unwrap();
    (mesh, geometry)
}

#[test]
fn upload_uses_payload_uvs_and_zero_weight_bottom_membership() {
    let (mesh, mut source) = source([0.125, 0.25], b"Skin");
    source.object.field_369 = true;
    source.mapping_coordinates[0] = [99.; 2]; // array value is not normal-body UV authority
    let result = source
        .textured_baked_preview_geometry(&mesh, ModelVariant::N)
        .unwrap();
    assert_eq!(result.geometry.uvs.unwrap(), vec![[0.125, 0.25]; 3]);
    assert_eq!(result.top_face, [0., 1., 1.]);
    assert_eq!(
        result.material_memberships,
        vec![vec![
            b"Skin".iter().copied().map(u16::from).collect::<Vec<_>>()
        ]]
    );
    assert_eq!(
        source.object.vertices[0].payload.words_00_through_24[8],
        0.125_f32.to_bits()
    );
    assert!(
        source
            .untextured_baked_preview_geometry(&mesh, ModelVariant::N)
            .unwrap()
            .uvs
            .is_none()
    );
}

#[test]
fn head_and_torso_merge_uv_remaps_and_material_partitions_are_retained() {
    for head in [true, false] {
        let (mesh, mut destination) = source([0.125, 0.25], b"A");
        let (_, mut appended) = source([0.5, 0.75], b"B");
        if head {
            destination
                .append_head_geometry_stages(&mut appended, ModelVariant::N, 0.)
                .unwrap();
        } else {
            destination
                .append_torso_geometry_stages(&mut appended, ModelVariant::N, 0.)
                .unwrap();
        }
        let result = destination
            .textured_baked_preview_geometry(&mesh, ModelVariant::N)
            .unwrap();
        let uv = result.geometry.uvs.as_ref().unwrap();
        for face in &result.geometry.faces[..1] {
            assert_eq!(
                face.vertices.map(|i| uv[usize::from(i)]),
                [[0.125, 0.25]; 3]
            );
        }
        for face in &result.geometry.faces[1..] {
            assert_eq!(face.vertices.map(|i| uv[usize::from(i)]), [[0.5, 0.75]; 3]);
        }
        // Head's material B records face COUNT=2, not new ordinal1; torso does
        // not transfer source material membership at all. Neither is repaired.
        assert_eq!(result.material_memberships[1], Vec::<Vec<u16>>::new());
        let partitions = result.partitions();
        assert_eq!(partitions.len(), 2);
        assert_eq!(partitions[&Vec::<Vec<u16>>::new()], [1]);
    }
}

#[test]
fn unsupported_mapping_and_malformed_payload_fail_explicitly() {
    let (mesh, mut geometry) = source([0., 0.], b"A");
    assert!(
        geometry
            .textured_baked_preview_geometry(&mesh, ModelVariant::N)
            .is_err()
    );
    geometry.object.field_369 = true;
    assert!(
        geometry
            .textured_baked_preview_geometry(&mesh, ModelVariant::X)
            .is_err()
    );
    geometry.object.vertices[0].payload.words_00_through_24[8] = f32::NAN.to_bits();
    assert!(
        geometry
            .textured_baked_preview_geometry(&mesh, ModelVariant::N)
            .is_err()
    );
    geometry.object.vertices[0].payload.words_00_through_24[8] = 0;
    geometry.mapping_coordinates.clear();
    assert!(
        geometry
            .textured_baked_preview_geometry(&mesh, ModelVariant::N)
            .is_err()
    );
}
