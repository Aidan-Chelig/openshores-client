use super::*;

fn definition() -> NativeBodyDeformation {
    NativeBodyDeformation {
        base: NativeBodyObjectScale {
            field_140: 2.,
            fields_1c0: [10., 20., 30.],
            fields_1f4: [1., 2., 3.],
        },
        field_170: 7.,
        field_178: 5.,
        field_180: 3.,
        field_188: 4.,
    }
}
fn selected(index: usize) -> NativeBodyVertexSelection {
    let mut selected = NativeBodyVertexSelection {
        any_match: true,
        ..Default::default()
    };
    selected.anatomical[index] = Some([7., 8., 9.]);
    selected
}

#[test]
fn all_four_branches_transform_vertex_and_connection_before_final_offset() {
    let definition = definition();
    for (index, expected) in [
        (4, [28., 44., 72.]),
        (5, [28., 38., 72.]),
        (3, [28., 44., 72.]),
        (2, [28., 92., 72.]),
    ] {
        assert_eq!(
            definition
                .apply_vertex([4., 5., 10.], &selected(index))
                .unwrap(),
            expected
        );
    }
}

#[test]
fn torso_uses_distinct_radial_and_height_fields_not_limb_scales() {
    let mut definition = definition();
    definition.field_180 = f64::NAN;
    definition.field_188 = f64::NAN;
    assert_eq!(
        definition
            .apply_vertex([4., 5., 10.], &selected(0))
            .unwrap(),
        [52., 50., 128.]
    );
    definition.field_170 = 1.;
    definition.field_178 = 1.;
    assert_eq!(
        definition
            .apply_vertex([4., 5., 10.], &selected(0))
            .unwrap(),
        [16., 26., 44.]
    );
}

#[test]
fn torso_top_wing_correction_runs_after_torso_height_scaling() {
    let mut selection = selected(0);
    selection.top = Some([0., 2., 0.]);
    selection.wing = true;
    // Connection y4 -> (4-8)*5+8=-12 -> (-12-2)*5+2=-68;
    // vertex y10 ->18, final20-(-68)+18=106.
    assert_eq!(
        definition()
            .apply_vertex([4., 5., 10.], &selection)
            .unwrap(),
        [52., 106., 128.]
    );
    selection.wing = false;
    assert_eq!(
        definition()
            .apply_vertex([4., 5., 10.], &selection)
            .unwrap(),
        [52., 50., 128.]
    );
}

#[test]
fn torso_remains_lower_priority_than_foot_and_unsupported_hand() {
    let mut selection = selected(0);
    selection.anatomical[2] = Some([7., 8., 9.]);
    assert_eq!(
        definition()
            .apply_vertex([4., 5., 10.], &selection)
            .unwrap(),
        [28., 92., 72.]
    );
    selection.anatomical[1] = Some([7., 8., 9.]);
    assert_eq!(
        definition()
            .apply_vertex([4., 5., 10.], &selection)
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
}

#[test]
fn native_priority_and_unsupported_hand_cannot_fall_through() {
    let mut selection = selected(2);
    selection.anatomical[1] = Some([0.; 3]);
    assert_eq!(
        definition()
            .apply_vertex([4., 5., 10.], &selection)
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
    selection.anatomical[3] = Some([7., 8., 9.]);
    assert_eq!(
        definition()
            .apply_vertex([4., 5., 10.], &selection)
            .unwrap(),
        [28., 44., 72.]
    );
    selection.anatomical[5] = Some([7., 8., 9.]);
    assert_eq!(
        definition()
            .apply_vertex([4., 5., 10.], &selection)
            .unwrap(),
        [28., 38., 72.]
    );
    selection.anatomical[4] = Some([7., 8., 9.]);
    assert_eq!(
        definition()
            .apply_vertex([4., 5., 10.], &selection)
            .unwrap(),
        [28., 44., 72.]
    );
}

#[test]
fn top_and_wing_are_both_required_for_extra_connection_only_correction() {
    let mut selection = selected(5);
    selection.top = Some([0., 2., 0.]);
    let baseline = definition()
        .apply_vertex([4., 5., 10.], &selection)
        .unwrap();
    assert_eq!(baseline, [28., 38., 72.]);
    selection.wing = true;
    assert_eq!(
        definition()
            .apply_vertex([4., 5., 10.], &selection)
            .unwrap(),
        [28., 62., 72.]
    );
    selection.top = None;
    assert_eq!(
        definition()
            .apply_vertex([4., 5., 10.], &selection)
            .unwrap(),
        baseline
    );
}

#[test]
fn zero_scale_skips_nan_input_but_not_later_branch_arithmetic() {
    let mut definition = definition();
    definition.base.field_140 = -0.;
    definition.base.fields_1f4 = [f32::NAN; 3];
    assert_eq!(
        definition
            .apply_vertex([f64::NAN; 3], &selected(5))
            .unwrap(),
        [10., 20., 30.]
    );
    definition.field_180 = f64::NAN;
    assert!(
        definition
            .apply_vertex([1.; 3], &selected(5))
            .unwrap()
            .iter()
            .all(|v| v.is_nan())
    );
}

#[test]
fn no_match_uses_existing_exact_unweighted_branch_but_unknown_match_is_error() {
    let definition = definition();
    let point = [4., 5., 10.];
    assert_eq!(
        definition
            .apply_vertex(point, &NativeBodyVertexSelection::default())
            .unwrap(),
        definition.base.apply_unweighted_vertex(point)
    );
    assert_eq!(
        definition
            .apply_vertex(
                point,
                &NativeBodyVertexSelection {
                    any_match: true,
                    ..Default::default()
                }
            )
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
}

#[test]
fn taken_branch_retains_cancellation_on_unscaled_axes() {
    let mut definition = definition();
    definition.base.field_140 = 1.;
    definition.base.fields_1f4 = [0.; 3];
    definition.base.fields_1c0 = [0.; 3];
    definition.field_188 = 1.;
    let mut selection = selected(3);
    selection.anatomical[3] = Some([0., 1e30, 0.]);
    assert_eq!(
        definition.apply_vertex([1.; 3], &selection).unwrap(),
        [3., 0., 3.]
    );
}

#[test]
fn mesh_pipeline_selects_actual_members_and_rejects_unsupported_vertices() {
    use crate::model::{GroupVertex, Mesh, ModelVariant, VertexGroup, VertexGroupChunk};
    let mut mesh = Mesh {
        vertices: vec![[4., 5., 10.], [1., 2., 3.]],
        vertex_groups: vec![VertexGroup {
            name: Some(b"Head".to_vec()),
            members: Some(vec![
                GroupVertex {
                    position: [4., 5., 10.],
                    weight: 0.,
                },
                GroupVertex {
                    position: [7., 8., 9.],
                    weight: 1.,
                },
            ]),
            field_order: vec![VertexGroupChunk::Name, VertexGroupChunk::Members],
        }],
        ..Default::default()
    };
    let definition = definition();
    assert_eq!(
        mesh.body_deformed_positions(ModelVariant::N, 1., &definition, [0.; 3], [0.; 3])
            .unwrap(),
        [[28., 38., 72.], [10., 20., 30.]]
    );
    mesh.vertex_groups[0].name = Some(b"Torso".to_vec());
    assert_eq!(
        mesh.body_deformed_positions(ModelVariant::N, 1., &definition, [0.; 3], [0.; 3])
            .unwrap(),
        [[52., 50., 128.], [10., 20., 30.]]
    );
    mesh.vertex_groups[0].name = Some(b"Hand".to_vec());
    assert_eq!(
        mesh.body_deformed_positions(ModelVariant::N, 1., &definition, [0.; 3], [0.; 3])
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
    assert_eq!(mesh.vertices, [[4., 5., 10.], [1., 2., 3.]]);
}
