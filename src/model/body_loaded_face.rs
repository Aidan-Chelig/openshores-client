//! Stored Au3dsGLSLFace fields, distinct from the serialized 3DS record.
use super::{Face, ModelVariant, NativeHeadVertexMapping, invalid};
use std::io;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeBodyLoadedFace {
    pub vertices: [u16; 3],
    pub flags_06: u16,
    pub fields_08: [f64; 9],
    pub fields_50: [f64; 9],
    pub field_98: f32,
    pub fields_9c: [u32; 3],
}

impl NativeBodyLoadedFace {
    /// LoadFaceListChunk's assigned fields. N/NT's first nine serialized floats
    /// widen into50..90; NT's following nine widen into08..48 and its last
    /// float goes to98. Other variants initialize both arrays to zero.
    pub fn from_serialized(face: &Face, variant: ModelVariant) -> io::Result<Self> {
        let extended = matches!(variant, ModelVariant::N | ModelVariant::NT);
        if face.extension.is_some() != extended
            || face.nt_extension.is_some() != (variant == ModelVariant::NT)
        {
            return Err(invalid(
                "serialized face extension does not match native model type",
            ));
        }
        let mut result = Self {
            vertices: face.vertices,
            flags_06: face.flags,
            fields_08: [0.; 9],
            fields_50: [0.; 9],
            field_98: 1.,
            fields_9c: [0; 3],
        };
        if let Some(values) = face.extension {
            result.fields_50 = values.map(f64::from);
        }
        if let Some(values) = face.nt_extension {
            result.fields_08 = std::array::from_fn(|i| f64::from(values[i]));
            result.field_98 = values[9];
        }
        Ok(result)
    }
}

impl NativeHeadVertexMapping {
    /// Fresh head-merge faces: retain only per-corner50..90, replace vertices,
    /// zero flags and08..48, and initialize98=1,9c..a4=0. Native narrows mapped
    /// int indices to u16 without a degeneracy check. This does not perform
    /// the separate connection-normal, face-material or smoothing-map updates.
    pub fn head_appended_faces(
        &self,
        faces: &[NativeBodyLoadedFace],
    ) -> io::Result<Vec<NativeBodyLoadedFace>> {
        faces
            .iter()
            .map(|face| {
                let mut vertices = [0; 3];
                for (destination, source) in vertices.iter_mut().zip(face.vertices) {
                    *destination = *self
                        .source_to_destination
                        .get(usize::from(source))
                        .ok_or_else(|| invalid("head face refers to unmapped source vertex"))?
                        as u16;
                }
                Ok(NativeBodyLoadedFace {
                    vertices,
                    flags_06: 0,
                    fields_08: [0.; 9],
                    fields_50: face.fields_50,
                    field_98: 1.,
                    fields_9c: [0; 3],
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_loaded_face_extension_fields_have_distinct_storage() {
        let mut raw = Face {
            vertices: [1, 2, 3],
            flags: 0xabcd,
            extension: Some([0.25; 9]),
            nt_extension: Some([0.75; 10]),
        };
        let face = NativeBodyLoadedFace::from_serialized(&raw, ModelVariant::NT).unwrap();
        assert_eq!(face.fields_50, [0.25; 9]);
        assert_eq!(face.fields_08, [0.75; 9]);
        assert_eq!(face.field_98, 0.75);
        assert_eq!(face.flags_06, 0xabcd);
        assert_eq!(face.fields_9c, [0; 3]);
        assert!(NativeBodyLoadedFace::from_serialized(&raw, ModelVariant::N).is_err());
        raw.nt_extension = None;
        let face = NativeBodyLoadedFace::from_serialized(&raw, ModelVariant::N).unwrap();
        assert_eq!(face.fields_08, [0.; 9]);
        assert_eq!(face.field_98, 1.);
        raw.extension = None;
        for variant in [ModelVariant::Standard, ModelVariant::X] {
            assert_eq!(
                NativeBodyLoadedFace::from_serialized(&raw, variant)
                    .unwrap()
                    .fields_50,
                [0.; 9]
            );
        }
    }

    #[test]
    fn head_face_remapping_resets_metadata_preserves_corner_fields_and_keeps_degenerates() {
        let raw = Face {
            vertices: [0, 1, 2],
            flags: u16::MAX,
            extension: Some([0.5; 9]),
            nt_extension: Some([9.; 10]),
        };
        let input = NativeBodyLoadedFace::from_serialized(&raw, ModelVariant::NT).unwrap();
        let mapping = NativeHeadVertexMapping {
            source_to_destination: vec![65_537, 1, 2],
            ..Default::default()
        };
        let output = mapping
            .head_appended_faces(std::slice::from_ref(&input))
            .unwrap();
        assert_eq!(output[0].vertices, [1, 1, 2]);
        assert_eq!(output[0].fields_50, input.fields_50);
        assert_eq!(output[0].flags_06, 0);
        assert_eq!(output[0].fields_08, [0.; 9]);
        assert_eq!(output[0].field_98, 1.);
        assert_eq!(input.flags_06, u16::MAX);
        assert!(
            NativeHeadVertexMapping::default()
                .head_appended_faces(&[input])
                .is_err()
        );
    }
}
