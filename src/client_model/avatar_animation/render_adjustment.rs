//! Admitted DaCreature::BodyPoseAdjustTransform branches, after skeletal posing.
//! Native render matrix post-rotation is applied here in local draw coordinates;
//! the authoritative actor root and immutable prepared surfaces stay intact.
use super::{AvatarAnimationFrame, NativeDpBoneTransform};

impl AvatarAnimationFrame {
    pub(super) fn adjust_render_pose(&mut self, general: u32) {
        let angle = match general {
            0x13 => f64::from_bits(0x3ff921fb54442d18),
            0x14 => f64::from_bits(0xbff921fb54442d18),
            0x15 => f64::from_bits(0x400921fb54442d18),
            _ => return,
        };
        let mut rotation = NativeDpBoneTransform::identity();
        rotation.rotate_y(angle);
        self.eye = rotation.project(self.eye);
        for surface in &mut self.surfaces {
            for corner in &mut surface.corners {
                let [x, y, z, w] = corner.position;
                // Pure rotation: weighted XYZ and W remain homogeneous. Never
                // divide by W here or alter native raw influence/material data.
                let [x, y, z] = rotation.project([x, y, z].map(f64::from));
                corner.position = [x as f32, y as f32, z as f32, w];
                corner.normal = rotation
                    .project(corner.normal.map(f64::from))
                    .map(|v| v as f32);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::native_body::{NativeDpAvatarCorner, NativeDpPosedAvatarSurface};

    fn frame() -> AvatarAnimationFrame {
        AvatarAnimationFrame {
            surfaces: vec![NativeDpPosedAvatarSurface {
                surface_index: 7,
                part_index: 3,
                corners: vec![NativeDpAvatarCorner {
                    source_vertex: 19,
                    position: [4., 6., 8., 2.],
                    normal: [1., 0., 0.],
                    uv: [0.25, 0.75],
                    top_face: 0.4,
                }],
                material_memberships: vec![vec![vec![2, 5]]],
            }],
            eye: [3., 4., 5.],
        }
    }

    #[test]
    fn body_rotations_keep_weighted_geometry_normals_and_eye_coherent() {
        for (code, point, normal, eye) in [
            (0x13, [8., 6., -4., 2.], [0., 0., -1.], [5., 4., -3.]),
            (0x14, [-8., 6., 4., 2.], [0., 0., 1.], [-5., 4., 3.]),
            (0x15, [-4., 6., -8., 2.], [-1., 0., 0.], [-3., 4., -5.]),
        ] {
            let mut result = frame();
            result.adjust_render_pose(code);
            let surface = &result.surfaces[0];
            let corner = &surface.corners[0];
            for (actual, expected) in corner.position.into_iter().zip(point) {
                assert!((actual - expected).abs() < 1e-6);
            }
            for (actual, expected) in corner.normal.into_iter().zip(normal) {
                assert!((actual - expected).abs() < 1e-6);
            }
            for (actual, expected) in result.eye.into_iter().zip(eye) {
                assert!((actual - expected).abs() < 1e-12);
            }
            assert_eq!(corner.position[3].to_bits(), 2_f32.to_bits());
            assert_eq!(corner.uv, [0.25, 0.75]);
            assert_eq!(corner.top_face, 0.4);
            assert_eq!(corner.source_vertex, 19);
            assert_eq!((surface.surface_index, surface.part_index), (7, 3));
            assert_eq!(surface.material_memberships, vec![vec![vec![2, 5]]]);
        }
    }

    #[test]
    fn other_general_poses_leave_every_draw_attribute_exact() {
        for code in [
            0x10, 0x11, 0x12, 0x18, 0x19, 0x1b, 0x1c, 0x20, 0x24, 0x25, 0x26, 0x35,
        ] {
            let mut result = frame();
            let original = frame();
            result.adjust_render_pose(code);
            assert_eq!(result.eye, original.eye);
            assert_eq!(result.surfaces[0].corners, original.surfaces[0].corners);
        }
    }
}
