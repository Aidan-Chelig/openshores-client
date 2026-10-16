//! Adult neutral camera point from native Skull EyePosition and head bone.
use super::*;
use std::io;

#[derive(Clone, Debug)]
pub struct NativeDpAvatarEye {
    local: [f64; 3],
    bone: usize,
}
impl NativeDpAvatarEye {
    pub fn location(&self, bones: &NativeDpBoneTransforms) -> io::Result<[f64; 3]> {
        let point = bones.slots[self.bone].project(self.local);
        if point.iter().any(|v| !v.is_finite()) {
            return Err(invalid("nonfinite native camera point"));
        }
        Ok(point)
    }
}

impl NativeDpAvatarGeometry {
    /// DpBody::CameraLocation/CurrentCameraLoc for the represented neutral pose.
    /// field3a is an explicit native head-selection flag, not guessed from sex.
    pub fn standing_camera_location(&self, field_3a: bool) -> io::Result<[f64; 3]> {
        if self.standing.pose.fields_3c != [0x24; 10] || self.standing.pose.field_6c {
            return Err(invalid("camera location requires neutral Stand"));
        }
        self.prepare_eye(field_3a)?.location(self.body.bones())
    }

    /// Retain authored camera metadata once; sample it through current head bones.
    pub fn prepare_eye(&self, field_3a: bool) -> io::Result<NativeDpAvatarEye> {
        let selected = usize::from(self.dna.0[5] & 2 != 0 && !field_3a);
        let (index, part) = self
            .body
            .parts()
            .iter()
            .enumerate()
            .filter(|(_, part)| part.allocation.kind == NativeDpPartKind::HeadAndNeck)
            .nth(selected)
            .ok_or_else(|| invalid("native camera head is missing"))?;
        let NativeDpPartJoints::Head { field_50: head } = part.joints else {
            return Err(invalid("native camera head joint is missing"));
        };
        let surface = self
            .surfaces
            .iter()
            .find(|surface| {
                surface.part_index == index
                    && surface
                        .sources
                        .iter()
                        .any(|source| source.resource.contains("Skull"))
            })
            .ok_or_else(|| invalid("native camera Skull source is missing"))?;
        // Camera metadata is published when the original Skull instance is
        // created, BEFORE head merge baking transforms retained group points.
        let skull = surface
            .sources
            .iter()
            .find(|source| source.resource.contains("Skull"))
            .unwrap();
        let groups = crate::model::NativeBodyObjectGroups::from_mesh(&skull.mesh)?;
        let eye = groups
            .control_point(0x304)
            .ok_or_else(|| invalid("native Skull lacks authored EyePosition"))?;
        let connection = surface.native.deformation.base.fields_1f4;
        let pivot = groups
            .retained_groups()
            .iter()
            .find(|group| group.flags_50[5])
            .map(|group| group.point_28)
            .unwrap_or([0.; 3]);
        let uniform = surface.native.deformation.base.field_140;
        let width = surface.native.deformation.field_180;
        let offset = skull_camera_offset(eye, connection, pivot, uniform, width);
        let local = current_camera_point(head, offset);
        let bone = if part.allocation.index == 1 { 12 } else { 7 };
        Ok(NativeDpAvatarEye { local, bone })
    }
}

fn current_camera_point(head: [f64; 3], offset: [f64; 3]) -> [f64; 3] {
    // CurrentCameraLoc 0x1412d090b zeroes xmm0; 0x1412d0960 adds
    // that ZERO to X, not body.field4050. Preserve the shipped asymmetry.
    [head[0], head[1] + offset[1], head[2] + offset[2]]
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, message)
}

fn skull_camera_offset(
    eye: [f32; 3],
    connection: [f32; 3],
    pivot: [f32; 3],
    uniform: f64,
    width: f64,
) -> [f64; 3] {
    std::array::from_fn(|axis| {
        let mut e = f64::from(eye[axis]) * uniform;
        let mut c = f64::from(connection[axis]) * uniform;
        if width != 1. {
            let p = f64::from(pivot[axis]);
            e = (e - p) * width + p;
            c = (c - p) * width + p;
        }
        e - c
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn current_camera_preserves_native_ignored_lateral_offset() {
        assert_eq!(
            current_camera_point([1., 2., 3.], [4., 5., 6.]),
            [1., 7., 9.]
        );
    }
    #[test]
    fn eye_offset_uses_independent_products_and_native_width_pivot_order() {
        assert_eq!(
            skull_camera_offset([1., 2., 3.], [0.5, 1., 1.5], [4., 5., 6.], 2., 0.5),
            [0.5, 1., 1.5]
        );
        let actual = skull_camera_offset([1., 2., 3.], [0.5, 1., 1.5], [4., 5., 6.], 2., 1.);
        assert_eq!(actual, [1., 2., 3.]);
    }
}
