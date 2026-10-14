//! Local globe-child model placement. No fabricated gravity or ancestor rotation.
use crate::atom_position::PackedPosition;
use bevy::prelude::*;

pub(super) fn local_model(position: [f64; 3], rotation: [f32; 3], scale: f64) -> Option<Transform> {
    if !scale.is_finite() || scale <= 0.0 {
        return None;
    }
    let matrix = PackedPosition {
        flags: 0,
        parent: None,
        position,
        rotation,
    }
    .local_matrix()
    .ok()?;
    let translation = Vec3::from_array(position.map(|value| (value * scale) as f32));
    if !translation.is_finite() {
        return None;
    }
    let basis = Mat3::from_cols(
        Vec3::new(
            matrix[0][0] as f32,
            matrix[0][1] as f32,
            matrix[0][2] as f32,
        ),
        Vec3::new(
            matrix[1][0] as f32,
            matrix[1][1] as f32,
            matrix[1][2] as f32,
        ),
        Vec3::new(
            matrix[2][0] as f32,
            matrix[2][1] as f32,
            matrix[2][2] as f32,
        ),
    );
    Some(
        Transform::from_translation(translation).with_rotation(Quat::from_mat3(&basis).normalize()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bevy_rotation_preserves_native_columns_and_scaled_translation() {
        for rotation in [[0.0; 3], [0.3, -0.7, 1.1], [std::f32::consts::PI, 0.0, 0.7]] {
            let position = [30.0, -20.0, -100.0];
            let transform = local_model(position, rotation, 0.001).unwrap();
            let matrix = PackedPosition {
                flags: 0,
                parent: None,
                position,
                rotation,
            }
            .local_matrix()
            .unwrap();
            for (axis, column) in [Vec3::X, Vec3::Y, Vec3::Z].into_iter().zip(matrix) {
                let expected = Vec3::new(column[0] as f32, column[1] as f32, column[2] as f32);
                assert!((transform.rotation * axis - expected).length() < 1e-6);
            }
            assert!((transform.translation - Vec3::new(0.03, -0.02, -0.1)).length() < 1e-7);
            assert_eq!(transform.scale, Vec3::ONE);
        }
    }

    #[test]
    fn invalid_or_unrepresentable_placement_is_rejected() {
        for scale in [0.0, -1.0, f64::INFINITY, f64::NAN] {
            assert!(local_model([0.0; 3], [0.0; 3], scale).is_none());
        }
        assert!(local_model([f64::MAX; 3], [0.0; 3], 1.0).is_none());
        assert!(local_model([0.0; 3], [f32::NAN; 3], 1.0).is_none());
    }
}
