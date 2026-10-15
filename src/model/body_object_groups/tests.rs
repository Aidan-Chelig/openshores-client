use super::*;
use crate::model::{
    GroupVertex, ModelVariant, NativeBodyObjectScale, VertexGroup, VertexGroupChunk,
};

fn raw(name: &[u8], point: [f32; 3]) -> VertexGroup {
    VertexGroup {
        name: Some(name.to_vec()),
        members: Some(vec![GroupVertex {
            position: point,
            weight: 1.,
        }]),
        field_order: vec![VertexGroupChunk::Name, VertexGroupChunk::Members],
    }
}

#[test]
fn every_control_point_destination_and_the_midpoint_retain_exception_match_native() {
    for (kind, destination) in [
        (3, 0x1cc),
        (4, 0x200),
        (10, 0x234),
        (11, 0x304),
        (12, 0x268),
        (13, 0x29c),
        (14, 0x2d0),
        (19, 0x3ac),
        (20, 0x3e0),
    ] {
        let mut group =
            NativeBodyVertexGroup::from_group(&raw(b"ConnectPt", [1., 2., 3.])).unwrap();
        // Exercise all concrete loader cases, including kinds12..14 which the
        // current raw-name classifier's earlier ScalePt substring shadows.
        group.kind_34 = kind;
        let mut state = NativeBodyObjectGroups::default();
        state.apply_group(group);
        assert_eq!(state.control_point(destination), Some([1., 2., 3.]));
        assert_eq!(state.retained_groups().len(), usize::from(kind == 4));
        assert!(!state.sets_field_3a9());
    }
}

#[test]
fn all_default_cases_retain_in_order_but_only_kind_one_sets_flag() {
    let mut state = NativeBodyObjectGroups::default();
    let kinds = [0, 1, 2, 5, 6, 7, 8, 9, 15, 18, 21, 37, u32::MAX];
    for kind in kinds {
        let mut group = NativeBodyVertexGroup::from_group(&VertexGroup::default()).unwrap();
        group.kind_34 = kind;
        let mut isolated = NativeBodyObjectGroups::default();
        isolated.apply_group(group.clone());
        assert_eq!(isolated.sets_field_3a9(), kind == 1);
        state.apply_group(group);
        assert_eq!(state.sets_field_3a9(), kind != 0);
    }
    assert_eq!(
        state
            .retained_groups()
            .iter()
            .map(|g| g.kind_34)
            .collect::<Vec<_>>(),
        kinds
    );
    assert_eq!(state.control_point(0x1cc), None);
}

#[test]
fn repeated_fields_overwrite_even_with_zero_points_and_missing_writes_stay_absent() {
    let mesh = Mesh {
        vertex_groups: vec![
            raw(b"ConnectPt", [1., 2., 3.]),
            raw(b"MidPt", [9.; 3]),
            raw(b"ConnectPt", [0.; 3]),
        ],
        ..Default::default()
    };
    let state = NativeBodyObjectGroups::from_mesh(&mesh).unwrap();
    assert_eq!(state.control_point(0x1cc), Some([0.; 3]));
    // One-record MidPt does not initialize its point, but still copies zero
    // into object200 and appends the group under the actual native cases.
    assert_eq!(state.control_point(0x200), Some([0.; 3]));
    assert_eq!(state.retained_groups().len(), 1);
    assert_eq!(state.control_point(0x234), None);
    assert_eq!(mesh.vertex_groups.len(), 3);
}

#[test]
fn later_malformed_groups_preserve_earlier_state_without_publishing_success() {
    let mesh = Mesh {
        vertex_groups: vec![
            raw(b"ConnectPt", [1., 2., 3.]),
            VertexGroup {
                name: Some(b"Hand".to_vec()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    let mut state = NativeBodyObjectGroups::default();
    assert_eq!(
        state.apply_mesh_groups(&mesh).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(state.control_point(0x1cc), Some([1., 2., 3.]));
    assert!(state.retained_groups().is_empty());
    assert!(!state.sets_field_3a9());
}

#[test]
fn point_only_mesh_uses_extracted_connection_but_retained_groups_are_unsupported() {
    let mut mesh = Mesh {
        vertices: vec![[1., 2., 3.]],
        vertex_groups: vec![
            raw(b"ConnectPt", [0.25, 0.5, 0.75]),
            raw(b"HandWingPt", [9.; 3]),
        ],
        ..Default::default()
    };
    let definition = NativeBodyObjectScale {
        field_140: 2.,
        fields_1c0: [0.; 3],
        fields_1f4: [199.; 3],
    };
    let points = mesh
        .body_unweighted_positions(ModelVariant::N, 1., &definition)
        .unwrap();
    assert_eq!(points, [[1.5, 3., 4.5]]);
    assert_eq!(definition.fields_1f4, [199.; 3]);
    mesh.vertex_groups.push(raw(b"Hand", [0.; 3]));
    assert_eq!(
        mesh.body_unweighted_positions(ModelVariant::N, 1., &definition)
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
}
