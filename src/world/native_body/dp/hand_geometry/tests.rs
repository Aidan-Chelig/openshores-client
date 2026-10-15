use super::*;
use crate::model::{Face, GroupVertex, Mesh};

fn group(name: &[u8], values: &[[f32; 3]]) -> VertexGroup {
    VertexGroup {
        name: Some(name.to_vec()),
        members: Some(
            values
                .iter()
                .map(|p| GroupVertex {
                    position: *p,
                    weight: 0.0,
                })
                .collect(),
        ),
        field_order: vec![VertexGroupChunk::Name, VertexGroupChunk::Members],
    }
}

#[test]
fn point_flags_preserve_priority_last_wire_record_and_fast_path() {
    for name in [b"Hand".as_slice(), b"RightHand", b"HandAnchor"] {
        assert_eq!(point(&group(name, &[[9., 8., 7.]])).unwrap(), (2, [0.; 3]));
        assert_eq!(
            point(&group(name, &[[9., 8., 7.], [1., 2., 3.]])).unwrap(),
            (2, [1., 2., 3.])
        );
    }
    assert_eq!(
        point(&group(b"HandConnectPt", &[[9., 8., 7.]])).unwrap(),
        (1, [9., 8., 7.])
    );
    for name in [
        b"HandScalePt".as_slice(),
        b"HandWingPt",
        b"TorsoHand",
        b"HandMidPt",
        b"BottomSkinHand",
    ] {
        assert_eq!(
            point(&group(name, &[[1.; 3], [2.; 3]])).unwrap(),
            (0, [0.; 3])
        );
    }
    let mut reverse = group(b"Hand", &[[1.; 3], [2.; 3]]);
    reverse.field_order.reverse();
    assert_eq!(point(&reverse).unwrap(), (2, [0.; 3]));
}

#[test]
fn bounds_use_native_exclusion_and_first_retained_object_metadata() {
    let make = |name: &[u8], vertices: Vec<[f32; 3]>| Mesh {
        name: name.to_vec(),
        vertices,
        faces: vec![Face {
            vertices: [0, 1, 2],
            flags: 0,
            extension: Some([0.; 9]),
            nt_extension: None,
        }],
        ..Default::default()
    };
    let mut mesh = make(b"HandMesh", vec![[1., 2., 3.], [4., 8., 6.], [2., 3., -1.]]);
    mesh.vertex_groups = vec![
        group(b"ConnectPt", &[[1., 2., 3.]]),
        group(b"Hand", &[[9.; 3], [4., 5., 6.]]),
    ];
    let model = Model {
        meshes: vec![mesh, make(b"-helper", vec![[100.; 3]; 3])],
        ..Default::default()
    };
    let g = NativeDpHandGeometry::from_model(&model).unwrap();
    assert_eq!(g.minimum, [1., 2., -1.]);
    assert_eq!(g.maximum, [4., 8., 6.]);
    assert_eq!(g.connect, [1., 2., 3.]);
    assert_eq!(g.pivot, [4., 5., 6.]);
}

#[test]
fn arithmetic_has_finite_values_and_positive_length_for_known_extents() {
    let geometry = NativeDpHandGeometry {
        minimum: [-1., -2., -3.],
        maximum: [4., 8., 6.],
        connect: [0., 1., 2.],
        pivot: [0.5, 0.75, 1.25],
    };
    for (raw, bits) in [
        (0, [0, 0x3fbbc0ec1335cb58, 0x3fd3d2f1c4942388]),
        (0x55, [0, 0x400c98bf0aa80000, 0x40068a0bdb000000]),
        (0xaa, [0, 0x4021a0194b8e79e1, 0x402661a6374f55b0]),
        (255, [0, 0x404c88ec526be000, 0x4040b9460cac0000]),
    ] {
        let offset = geometry.offset_for_dna(&NativeDhDna([raw; 24]));
        // Independently evaluated scalar SSE sequence with Python struct
        // narrowing at every f32 operation; no native process was executed.
        assert_eq!(offset.map(f64::to_bits), bits);
        assert_eq!(offset[0].to_bits(), 0);
        assert!(offset[1] > 0. && offset[2] > 0.);
        assert!(offset.into_iter().all(f64::is_finite));
    }
}
