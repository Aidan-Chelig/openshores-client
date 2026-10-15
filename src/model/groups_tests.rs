use super::*;

fn chunk(id: u16, payload: &[u8]) -> Vec<u8> {
    [
        id.to_le_bytes().to_vec(),
        ((payload.len() + 6) as u32).to_le_bytes().to_vec(),
        payload.to_vec(),
    ]
    .concat()
}
fn fixture(group: &[u8]) -> Vec<u8> {
    let object = [b"mesh\0".to_vec(), chunk(0x4100, &chunk(0x4112, group))].concat();
    chunk(0x4d4d, &chunk(0x3d3d, &chunk(0x4000, &object)))
}

#[test]
fn custom_groups_keep_raw_names_members_signed_zero_and_unknown_children() {
    let name = chunk(0x4113, b"Screen1\0");
    let records: Vec<_> = [1.0f32, 2., -0., 0.25]
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect();
    let members = chunk(0x4114, &[1u16.to_le_bytes().to_vec(), records].concat());
    let group = [name, members, chunk(0xbeef, b"opaque")].concat();
    let bytes = fixture(&group);
    let mut model = Model::decode(&bytes, ModelVariant::N).unwrap();
    let field = &model.meshes[0].vertex_groups[0];
    assert_eq!(field.name.as_deref(), Some(b"Screen1".as_slice()));
    assert_eq!(
        field.field_order,
        [VertexGroupChunk::Name, VertexGroupChunk::Members]
    );
    let member = field.members.as_ref().unwrap()[0];
    assert_eq!(member.position, [1., 2., -0.]);
    assert_eq!(member.position[2].to_bits(), (-0.0f32).to_bits());
    assert_eq!(member.weight, 0.25);
    let raw_digest = model.geometry_digest();
    let digest = model.vertex_group_digest();
    model.meshes[0].vertex_groups[0].members.as_mut().unwrap()[0].weight = 0.5;
    assert_ne!(digest, model.vertex_group_digest());
    assert_eq!(raw_digest, model.geometry_digest());
    assert!(model.opaque.iter().any(|chunk| chunk.id == 0xbeef));
    assert_eq!(model.source, bytes);
    for end in 0..bytes.len() {
        assert!(Model::decode(&bytes[..end], ModelVariant::N).is_err());
    }
}

#[test]
fn vertex_group_field_order_is_retained_without_normalizing_reversed_chunks() {
    let bytes = fixture(&[chunk(0x4114, &0u16.to_le_bytes()), chunk(0x4113, b"Hand\0")].concat());
    let model = Model::decode(&bytes, ModelVariant::N).unwrap();
    assert_eq!(
        model.meshes[0].vertex_groups[0].field_order,
        [VertexGroupChunk::Members, VertexGroupChunk::Name]
    );
    assert_eq!(model.source, bytes);
}

#[test]
fn malformed_and_duplicate_group_fields_are_rejected_including_empty_lists() {
    let name = chunk(0x4113, b"name\0");
    let empty = chunk(0x4114, &0u16.to_le_bytes());
    let valid = Model::decode(
        &fixture(&[name.clone(), empty.clone()].concat()),
        ModelVariant::N,
    )
    .unwrap();
    assert_eq!(valid.meshes[0].vertex_groups[0].members, Some(vec![]));
    for data in [
        [name.clone(), name].concat(),
        [empty.clone(), empty].concat(),
        chunk(0x4113, b"unterminated"),
        chunk(0x4114, &1u16.to_le_bytes()),
        chunk(0x4114, &[0, 0, 0]),
        chunk(
            0x4114,
            &[
                1u16.to_le_bytes().to_vec(),
                f32::NAN.to_le_bytes().repeat(4),
            ]
            .concat(),
        ),
    ] {
        assert!(Model::decode(&fixture(&data), ModelVariant::N).is_err());
    }
}
