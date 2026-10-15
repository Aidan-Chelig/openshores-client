use super::*;
use crate::model::{GroupVertex, NativeBodyVertexGroup, VertexGroup, VertexGroupChunk};

fn group(name: &[u8], pivot: [f32; 3]) -> NativeBodyVertexGroup {
    NativeBodyVertexGroup::from_group(&VertexGroup {
        name: Some(name.to_vec()),
        members: Some(vec![
            GroupVertex {
                position: [1., 2., 3.],
                weight: 0.,
            },
            GroupVertex {
                position: pivot,
                weight: 0.5,
            },
        ]),
        field_order: vec![VertexGroupChunk::Name, VertexGroupChunk::Members],
    })
    .unwrap()
}

#[test]
fn membership_uses_narrowed_position_not_weight_and_never_matches_absent_keys() {
    let mut groups = NativeBodyObjectGroups::default();
    groups.apply_group(group(b"Torso", [4., 5., 6.]));
    let selected = groups
        .select_vertex([1. + f64::EPSILON, 2., 3.], [0.; 3], [0.; 3])
        .unwrap();
    assert!(selected.any_match);
    assert_eq!(selected.anatomical[0], Some([4., 5., 6.]));
    assert_eq!(
        groups.select_vertex([9.; 3], [0.; 3], [0.; 3]).unwrap(),
        NativeBodyVertexSelection::default()
    );
    assert_eq!(
        groups
            .select_vertex([f64::NAN; 3], [0.; 3], [0.; 3])
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
}

#[test]
fn group_order_last_pivot_wins_and_each_group_uses_first_anatomical_flag() {
    let mut groups = NativeBodyObjectGroups::default();
    groups.apply_group(group(b"Torso", [4.; 3]));
    let mut multiple = group(b"Torso", [5.; 3]);
    multiple.flags_50 = [true; 6];
    groups.apply_group(multiple);
    groups.apply_group(group(b"Foot", [6.; 3]));
    groups.apply_group(group(b"Neck", [7.; 3]));
    groups.apply_group(group(b"Tail", [8.; 3]));
    groups.apply_group(group(b"Head", [9.; 3]));
    let selected = groups
        .select_vertex([1., 2., 3.], [0.; 3], [0.; 3])
        .unwrap();
    assert_eq!(
        selected.anatomical,
        [
            Some([5.; 3]),
            None,
            Some([6.; 3]),
            Some([7.; 3]),
            Some([8.; 3]),
            Some([9.; 3])
        ]
    );
}

#[test]
fn wings_are_case_sensitive_and_hand_wing_is_sticky_across_later_hand_groups() {
    let mut groups = NativeBodyObjectGroups::default();
    groups.apply_group(group(b"Handwing", [4.; 3]));
    let first = groups
        .select_vertex([1., 2., 3.], [0.; 3], [0.; 3])
        .unwrap();
    assert!(!first.wing && !first.hand_wing);
    groups.apply_group(group(b"HandWing", [5.; 3]));
    groups.apply_group(group(b"Hand", [6.; 3]));
    let selected = groups
        .select_vertex([1., 2., 3.], [0.; 3], [0.; 3])
        .unwrap();
    assert!(selected.wing && selected.hand_wing);
    assert_eq!(selected.anatomical[1], Some([6.; 3]));
}

#[test]
fn top_bottom_overrides_and_special_kind_precedence_match_native() {
    let mut groups = NativeBodyObjectGroups::default();
    groups.apply_group(group(b"Top", [4.; 3]));
    groups.apply_group(group(b"Bottom", [5.; 3]));
    let mut special = group(b"TorsoWing", [6.; 3]);
    special.kind_34 = 4;
    groups.apply_group(special);
    let selected = groups
        .select_vertex([1., 2., 3.], [-0.; 3], [1., 0., 0.])
        .unwrap();
    assert_eq!(selected.top, Some([4.; 3]));
    assert_eq!(selected.bottom, Some([1., 0., 0.]));
    assert!(selected.midpoint && selected.wing);
    assert_eq!(selected.anatomical, [None; 6]);
    let selected = groups
        .select_vertex([1., 2., 3.], [f32::NAN, 0., 0.], [0.; 3])
        .unwrap();
    assert!(selected.top.unwrap()[0].is_nan());
    assert_eq!(selected.bottom, Some([5.; 3]));
}

#[test]
fn unclassified_members_still_select_weighted_path_and_extracted_groups_do_not() {
    let mut groups = NativeBodyObjectGroups::default();
    groups.apply_group(group(b"ConnectPt", [4.; 3]));
    assert!(
        !groups
            .select_vertex([1., 2., 3.], [0.; 3], [0.; 3])
            .unwrap()
            .any_match
    );
    groups.apply_group(group(b"Unknown", [5.; 3]));
    let selected = groups
        .select_vertex([1., 2., 3.], [0.; 3], [0.; 3])
        .unwrap();
    assert!(selected.any_match);
    assert_eq!(selected.anatomical, [None; 6]);
}
#[test]
fn initialized_top_scale_is_retained_without_top_membership() {
    let groups = NativeBodyObjectGroups::default();
    let selection = groups
        .select_vertex([1.; 3], [1., 2., 3.], [0.; 3])
        .unwrap();
    assert_eq!(selection.top, None);
    assert_eq!(selection.top_scale_point, [1., 2., 3.]);
    assert!(!selection.any_match);
}
