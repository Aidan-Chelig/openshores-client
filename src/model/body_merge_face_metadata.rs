//! Head-merge object110 material map and final object120 smoothing writes.
use super::ModelVariant;
use std::{collections::BTreeMap, io};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NativeBodyFaceMetadata {
    /// Actual QString UTF-16 code units, not guessed decoding of raw names.
    pub materials: BTreeMap<Vec<u16>, Vec<u16>>,
    pub smoothing: Vec<u32>,
}

impl NativeBodyFaceMetadata {
    /// Material membership stage run after each source face has been appended.
    /// Native records the resulting destination face COUNT narrowed to u16,
    /// not count-1. Source face ordinal likewise wraps to u16. Duplicate source
    /// entries are a boolean contains test, not multiple destination appends.
    pub fn append_head_material_membership(
        &mut self,
        source: &Self,
        destination_faces_before: usize,
        appended_faces: usize,
    ) -> io::Result<()> {
        if destination_faces_before
            .checked_add(appended_faces)
            .is_none_or(|count| count > i32::MAX as usize)
        {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "native head face count exceeds signed-int capacity",
            ));
        }
        for ordinal in 0..appended_faces {
            let count = (destination_faces_before + ordinal + 1) as u16;
            for (name, members) in &source.materials {
                if members.contains(&(ordinal as u16)) {
                    self.materials.entry(name.clone()).or_default().push(count);
                }
            }
        }
        Ok(())
    }

    /// Final smoothing stage, AFTER connection-normal processing. N/NT leave
    /// the existing list untouched; Standard/X clear and append one per face.
    pub fn finish_head_smoothing(
        &mut self,
        variant: ModelVariant,
        final_faces: usize,
    ) -> io::Result<()> {
        if final_faces > i32::MAX as usize {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "native head smoothing count exceeds signed-int capacity",
            ));
        }
        if !matches!(variant, ModelVariant::N | ModelVariant::NT) {
            self.smoothing.clear();
            self.smoothing.resize(final_faces, 1);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn head_material_membership_uses_postappend_count_and_contains_semantics() {
        let name = vec![0x0061, 0xd800]; // preserve even an unpaired QString unit
        let mut destination = NativeBodyFaceMetadata {
            materials: BTreeMap::from([(name.clone(), vec![7])]),
            smoothing: vec![99],
        };
        let source = NativeBodyFaceMetadata {
            materials: BTreeMap::from([
                (name.clone(), vec![0, 0, 1]),
                (vec![98], vec![1]),
                (vec![99], vec![9]),
            ]),
            smoothing: vec![42],
        };
        destination
            .append_head_material_membership(&source, 2, 2)
            .unwrap();
        assert_eq!(destination.materials[&name], [7, 3, 4]);
        assert_eq!(destination.materials[&vec![98]], [4]);
        assert!(!destination.materials.contains_key(&vec![99]));
        assert_eq!(destination.smoothing, [99]);
        assert_eq!(source.materials[&name], [0, 0, 1]);
    }

    #[test]
    fn head_material_face_ordinals_and_destination_counts_wrap_like_u16() {
        let source = NativeBodyFaceMetadata {
            materials: BTreeMap::from([(vec![97], vec![0])]),
            ..Default::default()
        };
        let mut destination = NativeBodyFaceMetadata::default();
        destination
            .append_head_material_membership(&source, 65_535, 65_537)
            .unwrap();
        assert_eq!(destination.materials[&vec![97]], [0, 0]);
    }

    #[test]
    fn head_smoothing_is_format_specific_and_capacity_errors_do_not_mutate() {
        for variant in [
            ModelVariant::N,
            ModelVariant::NT,
            ModelVariant::Standard,
            ModelVariant::X,
        ] {
            let mut state = NativeBodyFaceMetadata {
                smoothing: vec![8, 9],
                ..Default::default()
            };
            state.finish_head_smoothing(variant, 3).unwrap();
            assert_eq!(
                state.smoothing,
                if matches!(variant, ModelVariant::N | ModelVariant::NT) {
                    vec![8, 9]
                } else {
                    vec![1; 3]
                }
            );
            let original = state.clone();
            assert!(
                state
                    .append_head_material_membership(&original, usize::MAX, 1)
                    .is_err()
            );
            assert_eq!(state, original);
        }
    }
}
