//! Fresh local T * Rx * Ry * Rz matrix; no environment or ancestry composition.
use super::PackedPosition;
use std::io;

impl PackedPosition {
    /// Column-major affine matrix for a freshly initialized local position.
    /// Angles are radians, widened from their native float destinations.
    /// This does not reproduce Rx applied to an arbitrary existing matrix,
    /// interpolation, gravity alignment, or an entity's environment transform.
    pub fn local_matrix(&self) -> io::Result<[[f64; 4]; 4]> {
        if self.position.iter().any(|v| !v.is_finite())
            || self.rotation.iter().any(|v| !v.is_finite())
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "non-finite local atom matrix",
            ));
        }
        let [x, y, z] = self.rotation.map(f64::from);
        let (sx, cx) = x.sin_cos();
        let (sy, cy) = y.sin_cos();
        let (sz, cz) = z.sin_cos();
        let c0 = [cy, sx * sy, -cx * sy];
        let c1 = [0.0, cx, sx];
        let c2 = [sy, -sx * cy, cx * cy];
        let right = std::array::from_fn::<_, 3, _>(|i| c0[i] * cz + c1[i] * sz);
        let forward = std::array::from_fn::<_, 3, _>(|i| -c0[i] * sz + c1[i] * cz);
        Ok([
            [right[0], right[1], right[2], 0.0],
            [forward[0], forward[1], forward[2], 0.0],
            [c2[0], c2[1], c2[2], 0.0],
            [self.position[0], self.position[1], self.position[2], 1.0],
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_rotation_is_orthonormal_and_never_rotates_translation() {
        for rotation in [[0.0; 3], [0.3, -0.7, 1.1], [std::f32::consts::PI, 0.0, 0.0]] {
            let record = PackedPosition {
                flags: 0,
                parent: Some(7),
                position: [10.0, -20.0, 30.0],
                rotation,
            };
            let matrix = record.local_matrix().unwrap();
            assert_eq!(matrix[3], [10.0, -20.0, 30.0, 1.0]);
            for a in 0..3 {
                for b in 0..3 {
                    let dot: f64 = (0..3).map(|i| matrix[a][i] * matrix[b][i]).sum();
                    assert!((dot - f64::from(a == b)).abs() < 1e-12);
                }
                assert_eq!(matrix[a][3], 0.0);
            }
        }
    }

    #[test]
    fn positive_z_quarter_turn_moves_local_right_toward_forward() {
        let record = PackedPosition {
            flags: 0x40,
            parent: None,
            position: [0.0; 3],
            rotation: [0.0, 0.0, std::f32::consts::FRAC_PI_2],
        };
        let matrix = record.local_matrix().unwrap();
        assert!(matrix[0][0].abs() < 1e-6 && matrix[0][1] > 0.999999);
        assert!(matrix[1][0] < -0.999999 && matrix[1][1].abs() < 1e-6);
        assert_eq!(matrix[2], [0.0, -0.0, 1.0, 0.0]);
    }
}
