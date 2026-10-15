//! Mounted foot selection and shared FootLength/FootHeight native cache.
use super::{NativeDhDna, NativeDpFootGeometry, NativeDpModelDna};
use crate::{
    asset_mount::AssetMount,
    model::{Model, ModelVariant},
};
use std::{collections::BTreeMap, io};

pub fn native_dh_has_ankle_bend(dna: &NativeDhDna) -> bool {
    dna.0[10] & 0xa0 != 0x80
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeDpFootResource {
    resource: String,
    key: NativeDpModelDna,
}

impl NativeDpFootResource {
    pub fn new(dna: &NativeDhDna, field_4068: i32, mode: bool) -> Self {
        let pa = super::hand_resource::uses_pa(field_4068);
        let mut key = NativeDpModelDna::from_dhdna(dna);
        if pa && !mode {
            key.force_field_bc();
        }
        let resource = if pa {
            format!(
                ":data/PAFootRight{}.3dsN",
                u8::from(native_dh_has_ankle_bend(dna))
            )
        } else {
            format!(":data/{}FootRight{}.3dsN", dna.0[16] & 7, dna.0[10] >> 5)
        };
        Self { resource, key }
    }
    pub fn resource(&self) -> &str {
        &self.resource
    }
    pub fn model_dna(&self) -> &NativeDpModelDna {
        &self.key
    }
    pub fn read_model(&self, mount: &AssetMount) -> io::Result<Model> {
        Model::decode(&mount.read_resource(&self.resource)?, ModelVariant::N)
    }
}

pub struct NativeDpFootCache<'a> {
    mount: &'a AssetMount,
    geometry: BTreeMap<String, NativeDpFootGeometry>,
    offsets: BTreeMap<NativeDpModelDna, [f64; 3]>,
}

impl<'a> NativeDpFootCache<'a> {
    pub fn new(mount: &'a AssetMount) -> Self {
        Self {
            mount,
            geometry: BTreeMap::new(),
            offsets: BTreeMap::new(),
        }
    }
    fn offset(&mut self, dna: &NativeDhDna, field_4068: i32, mode: bool) -> io::Result<[f64; 3]> {
        let request = NativeDpFootResource::new(dna, field_4068, mode);
        if let Some(offset) = self.offsets.get(request.model_dna())
            && offset.iter().any(|v| *v != 0.)
        {
            return Ok(*offset);
        }
        if self.offsets.len() >= 20_000 && !self.offsets.contains_key(request.model_dna()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "native foot-offset Rust cache budget",
            ));
        }
        if !self.geometry.contains_key(request.resource()) {
            let geometry = NativeDpFootGeometry::from_model(&request.read_model(self.mount)?)?;
            self.geometry
                .insert(request.resource().to_owned(), geometry);
        }
        let offset = self.geometry[request.resource()].offset_for_dna(dna);
        self.offsets.insert(request.model_dna().clone(), offset);
        Ok(offset)
    }
    pub fn foot_length(
        &mut self,
        dna: &NativeDhDna,
        field_4068: i32,
        mode: bool,
    ) -> io::Result<f32> {
        Ok(self.offset(dna, field_4068, mode)?[1] as f32)
    }
    /// FootHeight always uses mode=false, including its PA cache-key override.
    pub fn foot_height(&mut self, dna: &NativeDhDna, field_4068: i32) -> io::Result<f32> {
        Ok(self.offset(dna, field_4068, false)?[2] as f32)
    }
}

#[cfg(test)]
mod tests;
