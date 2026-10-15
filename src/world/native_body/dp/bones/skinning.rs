//! CPU counterpart of the mounted Body.v.glsl bone loop. Primary bone matrices
//! are narrowed to f32 by RenderSkin before GPU upload. This is NOT AuMatrix4x4
//! Project: shader matrix multiplication does not divide by W or skip columns.
use super::NativeDpBoneTransforms;
use crate::model::NativeBodyVertexWeights;
use std::io;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativeDpSkinnedVertex {
    /// Preserve the weighted homogeneous result, including non-unit W. Do not
    /// silently normalize weights or discard W before the final view projection.
    pub position: [f32; 4],
    pub normal: [f32; 3],
}

fn unsupported(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, message)
}

impl NativeDpBoneTransforms {
    /// `position` is the triangle builder's projected/deformed model coordinate,
    /// NOT a stored pre-projection vertex. `normal` is its model-space corner
    /// normal. Bone matrices already contain native part-composition writes;
    /// adding a generic parent hierarchy here would transform the vertex twice.
    ///
    /// Reproduce the shader's four ordered influences, mat4 position / mat3
    /// normal multiplication and final normal normalization. CPU/GPU FMA and
    /// normalize rounding may differ; this does not claim bit-identical shaders.
    /// Undefined/nonfinite shader inputs fail explicitly instead of inventing
    /// a bone, weight, normal or collapsed position. The raw payload is untouched.
    pub fn skin_shader_vertex(
        &self,
        position: [f32; 3],
        normal: [f32; 3],
        payload: &NativeBodyVertexWeights,
    ) -> io::Result<NativeDpSkinnedVertex> {
        if position.iter().chain(&normal).any(|v| !v.is_finite()) {
            return Err(unsupported("nonfinite model-space skinning input"));
        }
        let mut point = [0_f32; 4];
        let mut direction = [0_f32; 3];
        let mut influenced = false;
        for (bone, weight) in payload.bones().into_iter().zip(payload.weights()) {
            // Shader tests weight first. Unused indices, including -1, need not
            // reference a bone; negative zero likewise bypasses the influence.
            if weight == 0. {
                continue;
            }
            if !weight.is_finite() {
                return Err(unsupported("nonfinite native bone weight"));
            }
            let bone = usize::try_from(bone)
                .ok()
                .and_then(|i| self.slots.get(i))
                .ok_or_else(|| unsupported("active native bone index is outside 0..61"))?;
            let matrix = bone.matrix_00.map(|v| v as f32);
            if matrix.iter().any(|v| !v.is_finite()) {
                return Err(unsupported(
                    "active bone matrix cannot be uploaded as finite f32",
                ));
            }
            for row in 0..4 {
                let value = ((matrix[row] * position[0] + matrix[4 + row] * position[1])
                    + matrix[8 + row] * position[2])
                    + matrix[12 + row];
                point[row] += weight * value;
            }
            for row in 0..3 {
                let value = (matrix[row] * normal[0] + matrix[4 + row] * normal[1])
                    + matrix[8 + row] * normal[2];
                direction[row] += weight * value;
            }
            influenced = true;
        }
        let length = ((direction[0] * direction[0] + direction[1] * direction[1])
            + direction[2] * direction[2])
            .sqrt();
        if !influenced
            || point.iter().any(|v| !v.is_finite())
            || point[3] == 0.
            || !length.is_finite()
            || length == 0.
        {
            return Err(unsupported(
                "native skinning has no finite drawable position/normal",
            ));
        }
        Ok(NativeDpSkinnedVertex {
            position: point,
            normal: direction.map(|v| v / length),
        })
    }
}

#[cfg(test)]
mod tests;
