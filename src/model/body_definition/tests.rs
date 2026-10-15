use super::*;
use crate::model::{Mesh, ModelVariant, NativeBodyObjectScale};

fn object() -> NativeBodyDeformation {
    NativeBodyDeformation {
        base: NativeBodyObjectScale {
            field_140: -99.,
            fields_1c0: [-99.; 3],
            fields_1f4: [1., 2., 3.],
        },
        field_170: -99.,
        field_178: -99.,
        field_180: -99.,
        field_188: -99.,
        field_190: -99.,
        fields_228: [4., 5., 6.],
        fields_2c4: [7., 8., 9.],
        fields_25c: [10., 11., 12.],
    }
}

#[test]
fn constructor_geometry_projects_to_actual_generator_offsets_and_preserves_metadata() {
    let geometry = NativeBodyModelGeometry {
        fields_00: [f64::NAN, 2., f64::NAN, f64::NAN, f64::NAN, f64::NAN],
        fields_40: [10., 20., 30.],
        fields_58: [3., 4., 5., 6., 7.],
    };
    let mut actual = object();
    geometry.apply_to_deformation(&mut actual);
    assert_eq!(
        actual,
        NativeBodyDeformation {
            base: NativeBodyObjectScale {
                field_140: 2.,
                fields_1c0: [10., 20., 30.],
                ..object().base
            },
            field_178: 3.,
            field_170: 4.,
            field_188: 5.,
            field_180: 6.,
            field_190: 7.,
            ..object()
        }
    );
}

#[test]
fn translation_narrows_like_native_without_rejecting_overflow_or_signed_zero() {
    let geometry = NativeBodyModelGeometry {
        fields_00: [0., -0., 0., 0., 0., 0.],
        fields_40: [1. + f64::EPSILON, -0., f64::MAX],
        fields_58: [1.; 5],
    };
    let mut actual = object();
    geometry.apply_to_deformation(&mut actual);
    assert_eq!(actual.base.field_140.to_bits(), (-0_f64).to_bits());
    assert_eq!(
        actual.base.fields_1c0.map(f32::to_bits),
        [1_f32.to_bits(), (-0_f32).to_bits(), f32::INFINITY.to_bits()]
    );
    assert_eq!(
        actual.base.apply_unweighted_vertex([f64::NAN; 3]),
        [1., 0., f64::INFINITY]
    );
}

#[test]
fn actual_definition_fields_feed_loaded_mesh_deformation_not_display_scale() {
    let geometry = NativeBodyModelGeometry {
        fields_00: [123., 2., 456., 789., 321., 654.],
        fields_40: [10., 20., 30.],
        fields_58: [3., 4., 5., 6., 7.],
    };
    let mut actual = object();
    geometry.apply_to_deformation(&mut actual);
    let mesh = Mesh {
        vertices: vec![[4., 5., 10.]],
        ..Default::default()
    };
    assert_eq!(
        mesh.body_deformed_positions(ModelVariant::N, 1., &actual, [0.; 3], [0.; 3])
            .unwrap(),
        [[16., 26., 44.]]
    );
}
