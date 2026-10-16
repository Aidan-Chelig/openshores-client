//! Local globe-child model placement. No fabricated gravity or ancestor rotation.
use crate::atom_position::PackedPosition;
use bevy::prelude::*;

/// Source-client locomotion frame: +X right, +Y forward, +Z radial up.
/// Yaw is the UI's clockwise/rightward heading, not a global Z Euler angle.
pub(super) fn tangent_frame(position: [f64; 3], yaw: f32) -> Option<Mat3> {
    let point = bevy::math::DVec3::from_array(position);
    if !point.is_finite() || !yaw.is_finite() {
        return None;
    }
    let radial = point.normalize_or_zero();
    let up = if radial.length_squared() > 0. {
        radial.as_vec3()
    } else {
        Vec3::Z
    };
    let basis = if up.dot(Vec3::Z).abs() > 0.95 {
        Vec3::Y
    } else {
        Vec3::Z
    };
    let right = basis.cross(up).normalize();
    let north = up.cross(right);
    let forward = north * yaw.cos() + right * yaw.sin();
    Some(Mat3::from_cols(forward.cross(up), forward, up))
}

/// Encode our source-profile predicted frame in the existing Rx*Ry*Rz path.
pub(super) fn heading_rotation(position: [f64; 3], yaw: f32) -> Option<[f64; 3]> {
    let frame = tangent_frame(position, yaw)?;
    let (x, y, z) = Quat::from_mat3(&frame).normalize().to_euler(EulerRot::XYZ);
    Some([x, y, z].map(f64::from))
}

pub(super) fn received_heading(position: [f64; 3], rotation: [f64; 3]) -> Option<f32> {
    let frame = tangent_frame(position, 0.)?;
    let rotation = local_model([0.; 3], rotation.map(|v| v as f32), 1.)?.rotation;
    let forward = rotation * Vec3::Y;
    let x = forward.dot(frame.x_axis);
    let y = forward.dot(frame.y_axis);
    (x * x + y * y > 1e-10).then(|| x.atan2(y))
}

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
    fn reported_heading_roundtrips_native_axes_at_poles_and_off_axis() {
        for position in [
            [0., 0., 100.],
            [0., 0., -100.],
            [100., 0., 0.],
            [30., -40., -50.],
        ] {
            for yaw in [-3.1, -1.57, -0.3, 0., 0.7, 1.57, 3.1] {
                let frame = tangent_frame(position, yaw).unwrap();
                let angles = heading_rotation(position, yaw).unwrap();
                let placement = local_model(position, angles.map(|v| v as f32), 0.001).unwrap();
                for (axis, expected) in [
                    (Vec3::X, frame.x_axis),
                    (Vec3::Y, frame.y_axis),
                    (Vec3::Z, frame.z_axis),
                ] {
                    assert!(
                        (placement.rotation * axis - expected).length() < 2e-5,
                        "{position:?} {yaw}"
                    );
                }
                let recovered = received_heading(position, angles).unwrap();
                assert!((recovered - yaw).sin().abs() < 2e-5);
                assert!(frame.determinant() > 0.99999);
            }
        }
    }

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
