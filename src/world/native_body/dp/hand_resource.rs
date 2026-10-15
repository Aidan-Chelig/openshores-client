//! Verified HandLength resource/cache-key selection, not its geometry arithmetic.
use super::{NativeDhDna, NativeDpModelDna};
use crate::{
    asset_mount::AssetMount,
    model::{Model, ModelVariant},
};
use std::io;

pub(super) fn uses_pa(field_4068: i32) -> bool {
    matches!(field_4068, 0x41 | 0x50 | 0x99 | 0x9b)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeDpHandResource {
    resource: String,
    key: NativeDpModelDna,
}

impl NativeDpHandResource {
    /// Caller supplies actual body4068 and HandLength's boolean, not inferred
    /// species/pose defaults. That boolean changes the PA key, NOT the filename.
    pub fn new(dna: &NativeDhDna, field_4068: i32, mode: bool) -> Self {
        let pa = uses_pa(field_4068);
        let mut key = NativeDpModelDna::from_dhdna(dna);
        if pa && !mode {
            key.force_field_bc();
        }
        let resource = if pa {
            ":data/PAHandRight.3dsN".to_owned()
        } else {
            format!(":data/{}HandRight{}.3dsN", dna.0[16] & 7, dna.0[11] >> 5)
        };
        Self { resource, key }
    }

    pub fn resource(&self) -> &str {
        &self.resource
    }
    pub fn model_dna(&self) -> &NativeDpModelDna {
        &self.key
    }

    /// User installation only. No basename search, alternate model, built-in
    /// geometry, extraction into the repository or native library execution.
    /// Returns raw decoded geometry; it does not claim native HandLength.
    pub fn read_model(&self, mount: &AssetMount) -> io::Result<Model> {
        Model::decode(&mount.read_resource(&self.resource)?, ModelVariant::N)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_runtime_field_controls_pa_selection_and_key_override() {
        let mut dna = NativeDhDna([255; 24]);
        dna.0[16] = 250;
        dna.0[11] = 0xa7;
        for raw in [-1, 0, 0x41, 0x50, 0x99, 0x9b, 0x141, 0x199, i32::MAX] {
            for mode in [false, true] {
                let request = NativeDpHandResource::new(&dna, raw, mode);
                let pa = matches!(raw, 0x41 | 0x50 | 0x99 | 0x9b);
                assert_eq!(
                    request.resource(),
                    if pa {
                        ":data/PAHandRight.3dsN"
                    } else {
                        ":data/2HandRight5.3dsN"
                    }
                );
                let mut fields = *NativeDpModelDna::from_dhdna(&dna).fields();
                if pa && !mode {
                    fields[47] = 1;
                }
                assert_eq!(request.model_dna().fields(), &fields);
                assert_eq!(request.model_dna().fields()[45], 7);
            }
        }
    }
}
