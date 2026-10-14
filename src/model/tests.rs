use super::*;
fn chunk(id: u16, body: &[u8]) -> Vec<u8> {
    let mut bytes = id.to_le_bytes().to_vec();
    bytes.extend(((body.len() + 6) as u32).to_le_bytes());
    bytes.extend(body);
    bytes
}
fn fixture(variant: ModelVariant, bad_vertex: bool, bad_material: bool) -> Vec<u8> {
    let mut vertices = 3u16.to_le_bytes().to_vec();
    for value in [0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0] {
        vertices.extend(value.to_le_bytes());
    }
    let mut faces = 1u16.to_le_bytes().to_vec();
    for value in [0u16, 1, if bad_vertex { 3 } else { 2 }, 7] {
        faces.extend(value.to_le_bytes());
    }
    if matches!(variant, ModelVariant::N | ModelVariant::NT) {
        for value in 0..9 {
            faces.extend((value as f32).to_le_bytes());
        }
    }
    if variant == ModelVariant::NT {
        for value in 9..19 {
            faces.extend((value as f32).to_le_bytes());
        }
    }
    faces.extend(chunk(0x4150, &42u32.to_le_bytes()));
    let mut assignment = b"mat\0".to_vec();
    assignment.extend(1u16.to_le_bytes());
    assignment.extend(u16::from(bad_material).to_le_bytes());
    faces.extend(chunk(0x4130, &assignment));
    let mut mesh = chunk(0x4110, &vertices);
    mesh.extend(chunk(0x4120, &faces));
    // Deliberately unequal UV/vertex counts are preserved, as in reference assets.
    let mut uv = 1u16.to_le_bytes().to_vec();
    uv.extend(0.25f32.to_le_bytes());
    uv.extend(0.75f32.to_le_bytes());
    mesh.extend(chunk(0x4140, &uv));
    let mut object = b"triangle\0".to_vec();
    object.extend(chunk(0x4100, &mesh));
    let mut editor = chunk(0x4000, &object);
    let mut material = chunk(0xa000, b"mat\0");
    material.extend(chunk(0xa200, &chunk(0xa300, b"texture.png\0")));
    editor.extend(chunk(0xafff, &material));
    let mut main = chunk(0x0002, &3u32.to_le_bytes());
    main.extend(chunk(0x3d3d, &editor));
    main.extend(chunk(0xbeef, b"opaque"));
    chunk(0x4d4d, &main)
}

#[test]
fn four_variants_preserve_raw_geometry_and_opaque_bytes() {
    for variant in [
        ModelVariant::Standard,
        ModelVariant::N,
        ModelVariant::NT,
        ModelVariant::X,
    ] {
        let bytes = fixture(variant, false, false);
        let model = Model::decode(&bytes, variant).unwrap();
        assert_eq!(model.meshes[0].vertices[1], [1.0, 0.0, 0.0]);
        assert_eq!(model.meshes[0].faces[0].vertices, [0, 1, 2]);
        assert_eq!(
            model.meshes[0].faces[0].extension.is_some(),
            matches!(variant, ModelVariant::N | ModelVariant::NT)
        );
        assert_eq!(
            model.meshes[0].faces[0].nt_extension.is_some(),
            variant == ModelVariant::NT
        );
        if let Some(values) = model.meshes[0].faces[0].nt_extension {
            assert_eq!(values, std::array::from_fn(|index| (index + 9) as f32));
        }
        if let Some(values) = model.meshes[0].faces[0].extension {
            assert_eq!(values[8], 8.0);
        }
        assert_eq!(model.meshes[0].smoothing, Some(vec![42]));
        assert_eq!(model.meshes[0].uvs, Some(vec![[0.25, 0.75]]));
        assert_eq!(model.materials[0].textures, vec![b"texture.png".to_vec()]);
        let opaque = model
            .opaque
            .iter()
            .find(|chunk| chunk.id == 0xbeef)
            .unwrap();
        assert_eq!(
            &model.source[opaque.range.start + 6..opaque.range.end],
            b"opaque"
        );
        for end in 0..bytes.len() {
            assert!(Model::decode(&bytes[..end], variant).is_err());
        }
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(Model::decode(&trailing, variant).is_err());
    }
}

#[test]
fn indices_variants_and_nonfinite_values_are_rejected() {
    assert!(
        Model::decode(
            &fixture(ModelVariant::Standard, true, false),
            ModelVariant::Standard
        )
        .is_err()
    );
    assert!(
        Model::decode(
            &fixture(ModelVariant::Standard, false, true),
            ModelVariant::Standard
        )
        .is_err()
    );
    assert!(
        Model::decode(
            &fixture(ModelVariant::Standard, false, false),
            ModelVariant::N
        )
        .is_err()
    );
    assert!(
        Model::decode(
            &fixture(ModelVariant::N, false, false),
            ModelVariant::Standard
        )
        .is_err()
    );
    let bytes = chunk(
        0x4d4d,
        &chunk(0x3d3d, &chunk(0x0100, &f32::NAN.to_le_bytes())),
    );
    assert!(Model::decode(&bytes, ModelVariant::Standard).is_err());
    assert!(ModelVariant::from_name("module.SoH").is_err());
}

#[test]
fn malformed_chunks_names_and_duplicate_empty_lists() {
    for body in [&[1u8][..], &[1, 0, 5, 0, 0, 0], &[1, 0, 255, 255, 255, 255]] {
        assert!(Model::decode(&chunk(0x4d4d, body), ModelVariant::Standard).is_err());
    }
    let unnamed = chunk(0x4d4d, &chunk(0x3d3d, &chunk(0x4000, b"no terminator")));
    assert!(Model::decode(&unnamed, ModelVariant::Standard).is_err());
    let mut mesh = chunk(0x4110, &[0, 0]);
    mesh.extend(chunk(0x4110, &[0, 0]));
    let mut object = b"empty\0".to_vec();
    object.extend(chunk(0x4100, &mesh));
    let duplicate = chunk(0x4d4d, &chunk(0x3d3d, &chunk(0x4000, &object)));
    assert!(Model::decode(&duplicate, ModelVariant::Standard).is_err());
    let too_many = chunk(0x4d4d, &chunk(0xbeef, &[]).repeat(MAX_CHUNKS));
    assert!(Model::decode(&too_many, ModelVariant::Standard).is_err());
}

fn material_fixture(body: &[u8]) -> Vec<u8> {
    chunk(0x4d4d, &chunk(0x3d3d, &chunk(0xafff, body)))
}

#[test]
fn material_encodings_and_source_order_are_preserved() {
    let mut rgb = chunk(0x11, &[10, 20, 30]);
    rgb.extend(chunk(0x12, &[40, 50, 60]));
    // Unsupported linear-float colors stay opaque, matching the search boundary.
    rgb.extend(chunk(0x13, &[0; 12]));
    let mut material = chunk(0xa020, &rgb);
    let floats: Vec<_> = [0.25f32, 0.5, 0.75]
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect();
    material.extend(chunk(0xa030, &chunk(0x10, &floats)));
    material.extend(chunk(0xa040, &chunk(0x30, &75u16.to_le_bytes())));
    material.extend(chunk(0xa050, &chunk(0x30, &0.375f32.to_le_bytes())));
    material.extend(chunk(0xa081, &[]));
    material.extend(chunk(0xa100, &3u16.to_le_bytes()));
    let model = Model::decode(&material_fixture(&material), ModelVariant::N).unwrap();
    let decoded = &model.materials[0];
    assert_eq!(
        decoded.colors,
        vec![
            ColorRecord {
                property: 0xa020,
                value: ColorValue::Rgb8([10, 20, 30])
            },
            ColorRecord {
                property: 0xa020,
                value: ColorValue::LinearRgb8([40, 50, 60])
            },
            ColorRecord {
                property: 0xa030,
                value: ColorValue::RgbFloat([0.25, 0.5, 0.75])
            },
        ]
    );
    assert_eq!(
        decoded.amounts,
        vec![
            AmountRecord {
                property: 0xa040,
                value: AmountValue::Word(75)
            },
            AmountRecord {
                property: 0xa050,
                value: AmountValue::Float(0.375)
            },
        ]
    );
    assert!(decoded.two_sided);
    assert_eq!(decoded.shading, Some(3));
    assert!(model.opaque.iter().any(|chunk| chunk.id == 0x13));
    assert!(Model::decode(&material_fixture(&material), ModelVariant::X).is_err());
    assert_ne!(model.geometry_digest(), model.material_digest());
}

#[test]
fn malformed_material_values_are_rejected() {
    for material in [
        chunk(0xa020, &chunk(0x11, &[1, 2])),
        chunk(0xa020, &chunk(0x12, &[1, 2, 3, 4])),
        chunk(0xa020, &chunk(0x10, &f32::INFINITY.to_le_bytes().repeat(3))),
        chunk(0xa040, &chunk(0x30, &[0, 0, 0])),
        chunk(0xa050, &chunk(0x30, &f32::NAN.to_le_bytes())),
        chunk(0xa081, &[0]),
        chunk(0xa100, &[0]),
        [chunk(0xa100, &[0, 0]), chunk(0xa100, &[0, 0])].concat(),
    ] {
        assert!(Model::decode(&material_fixture(&material), ModelVariant::N).is_err());
    }
}

#[test]
fn nt_tail_is_not_misread_as_child_chunks_and_is_part_of_the_digest() {
    assert_eq!(
        ModelVariant::from_name("model.3dsNT").unwrap(),
        ModelVariant::NT
    );
    let bytes = fixture(ModelVariant::NT, false, false);
    for variant in [ModelVariant::Standard, ModelVariant::N, ModelVariant::X] {
        assert!(Model::decode(&bytes, variant).is_err());
    }
    let model = Model::decode(&bytes, ModelVariant::NT).unwrap();
    let original_digest = model.geometry_digest();
    let mut changed = bytes.clone();
    let tail: Vec<_> = (9..19)
        .flat_map(|value| (value as f32).to_le_bytes())
        .collect();
    let offset = changed
        .windows(tail.len())
        .position(|bytes| bytes == tail)
        .unwrap();
    changed[offset..offset + 4].copy_from_slice(&19.0f32.to_le_bytes());
    let changed_model = Model::decode(&changed, ModelVariant::NT).unwrap();
    assert_ne!(original_digest, changed_model.geometry_digest());
    assert_eq!(model.material_digest(), changed_model.material_digest());
    changed[offset..offset + 4].copy_from_slice(&f32::NAN.to_le_bytes());
    assert!(Model::decode(&changed, ModelVariant::NT).is_err());
}
