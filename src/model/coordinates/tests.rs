use super::*;

const IDENTITY: [f32; 12] = [1., 0., 0., 0., 1., 0., 0., 0., 1., 0., 0., 0.];

fn mesh(matrix: [f32; 12], order: Vec<GeometryChunk>) -> Mesh {
    Mesh {
        vertices: vec![[24., 48., 72.], [0., 0., 0.]],
        matrix: Some(matrix),
        geometry_order: order,
        ..Default::default()
    }
}

#[test]
fn matrix_loading_round_trip_does_not_transform_source_coordinates_twice() {
    for matrix in [
        [0., 1., 0., -1., 0., 0., 0., 0., 1., 12., 24., 36.],
        [2., 0., 0., 1., 4., 0., 0., 0., 8., 12., 24., 36.],
    ] {
        for order in [
            vec![GeometryChunk::Vertices, GeometryChunk::Matrix],
            vec![GeometryChunk::Matrix, GeometryChunk::Vertices],
        ] {
            let mesh = mesh(matrix, order);
            for variant in [ModelVariant::Standard, ModelVariant::N, ModelVariant::NT] {
                assert_eq!(
                    mesh.loaded_positions(variant, 1.).unwrap(),
                    mesh.vertices
                        .iter()
                        .map(|point| point.map(f64::from))
                        .collect::<Vec<_>>()
                );
                assert_eq!(
                    mesh.loaded_positions(variant, 2.).unwrap(),
                    vec![[48., 96., 144.], [0., 0., 0.]]
                );
            }
        }
    }
}

#[test]
fn x_conversion_obeys_chunk_order_instead_of_rescaling_all_models() {
    let mut translated = IDENTITY;
    translated[9..].copy_from_slice(&[12., 24., 36.]);
    let before = mesh(
        translated,
        vec![GeometryChunk::Vertices, GeometryChunk::Matrix],
    );
    let after = mesh(
        translated,
        vec![GeometryChunk::Matrix, GeometryChunk::Vertices],
    );
    assert_eq!(
        before.loaded_positions(ModelVariant::X, 1.).unwrap(),
        vec![[2., 4., 6.], [0., 0., 0.]]
    );
    assert_eq!(
        after.loaded_positions(ModelVariant::X, 1.).unwrap(),
        vec![[24., 48., 72.], [0., 0., 0.]]
    );
    assert_eq!(before.vertices, vec![[24., 48., 72.], [0., 0., 0.]]);
    let no_matrix = Mesh {
        vertices: before.vertices.clone(),
        ..Default::default()
    };
    assert_eq!(
        no_matrix.loaded_positions(ModelVariant::X, 1.).unwrap(),
        after.loaded_positions(ModelVariant::X, 1.).unwrap()
    );
}

#[test]
fn x_scaling_widens_before_multiplication_and_rounds_only_at_storage() {
    let value = (0..1000)
        .map(|offset| f32::from_bits(0x3f000000 + offset))
        .find(|value| {
            ((f64::from(*value) * (1. / 12.)) as f32).to_bits() != (*value * (1. / 12.)).to_bits()
        })
        .unwrap();
    let mesh = Mesh {
        vertices: vec![[value; 3]],
        matrix: Some(IDENTITY),
        geometry_order: vec![GeometryChunk::Vertices, GeometryChunk::Matrix],
        ..Default::default()
    };
    assert_eq!(
        mesh.loaded_positions(ModelVariant::X, 1.).unwrap(),
        vec![[f64::from((f64::from(value) * (1. / 12.)) as f32); 3]]
    );
}

#[test]
fn missing_order_singular_matrices_and_nonfinite_results_are_explicit_errors() {
    let mut mesh = mesh(IDENTITY, vec![]);
    assert!(mesh.loaded_positions(ModelVariant::N, 1.).is_err());
    mesh.geometry_order = vec![GeometryChunk::Vertices, GeometryChunk::Matrix];
    mesh.matrix = Some([0.; 12]);
    assert!(mesh.loaded_positions(ModelVariant::N, 1.).is_err());
    mesh.matrix = Some(IDENTITY);
    for scale in [0., -1., f64::INFINITY, f64::NAN, f64::MAX] {
        assert!(mesh.loaded_positions(ModelVariant::N, scale).is_err());
    }
    mesh.vertices[0][0] = f32::NAN;
    assert!(mesh.loaded_positions(ModelVariant::N, 1.).is_err());
}

#[test]
fn decoder_preserves_coordinate_chunk_order_without_changing_raw_fingerprint() {
    fn chunk(id: u16, body: &[u8]) -> Vec<u8> {
        [
            id.to_le_bytes().to_vec(),
            ((body.len() + 6) as u32).to_le_bytes().to_vec(),
            body.to_vec(),
        ]
        .concat()
    }
    let vertices = chunk(
        0x4110,
        &[
            1u16.to_le_bytes().to_vec(),
            [24f32, 48., 72.]
                .into_iter()
                .flat_map(f32::to_le_bytes)
                .collect(),
        ]
        .concat(),
    );
    let matrix = chunk(
        0x4160,
        &IDENTITY
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect::<Vec<_>>(),
    );
    let decode = |chunks: Vec<u8>| {
        let object = [b"mesh\0".to_vec(), chunk(0x4100, &chunks)].concat();
        crate::model::Model::decode(
            &chunk(0x4d4d, &chunk(0x3d3d, &chunk(0x4000, &object))),
            ModelVariant::X,
        )
        .unwrap()
    };
    let first = decode([vertices.clone(), matrix.clone()].concat());
    let second = decode([matrix, vertices].concat());
    assert_eq!(first.geometry_digest(), second.geometry_digest()); // Raw parity serialization is unchanged.
    assert_eq!(
        first.meshes[0]
            .loaded_positions(ModelVariant::X, 1.)
            .unwrap(),
        vec![[2., 4., 6.]]
    );
    assert_eq!(
        second.meshes[0]
            .loaded_positions(ModelVariant::X, 1.)
            .unwrap(),
        vec![[24., 48., 72.]]
    );
}
