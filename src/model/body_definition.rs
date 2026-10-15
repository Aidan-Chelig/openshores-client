//! Geometry projection of Au3dsGLSLModelDefinition, not its colors/cache/bones.
use super::NativeBodyDeformation;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativeBodyModelGeometry {
    /// Constructor's first six doubles, stored at definition00..28. The cache
    /// forwards only00/08 to object148/140; the others are not guessed here.
    pub fields_00: [f64; 6],
    pub fields_40: [f64; 3],
    /// Constructor's five doubles following AuPoint, definition58..78.
    pub fields_58: [f64; 5],
}

impl NativeBodyModelGeometry {
    /// CreateBodyModel/GenerateBodyDeviceObjects' established geometry writes.
    /// Existing object control points remain intact. Other writes (group48,
    /// object148, colors, bone averaging/device objects) are outside this API.
    pub fn apply_to_deformation(&self, object: &mut NativeBodyDeformation) {
        object.base.field_140 = self.fields_00[1];
        object.base.fields_1c0 = self.fields_40.map(|value| value as f32);
        object.field_178 = self.fields_58[0];
        object.field_170 = self.fields_58[1];
        object.field_188 = self.fields_58[2];
        object.field_180 = self.fields_58[3];
        object.field_190 = self.fields_58[4];
    }
}

#[cfg(test)]
mod tests;
