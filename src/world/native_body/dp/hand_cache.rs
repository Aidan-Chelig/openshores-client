//! Mounted HandLength dependency, with actual native key and zero sentinel.
use super::{NativeDhDna, NativeDpHandGeometry, NativeDpHandResource, NativeDpModelDna};
use crate::asset_mount::AssetMount;
use std::{collections::BTreeMap, io};

pub struct NativeDpHandCache<'a> {
    mount: &'a AssetMount,
    geometry: BTreeMap<String, NativeDpHandGeometry>,
    offsets: BTreeMap<NativeDpModelDna, [f64; 3]>,
}

impl<'a> NativeDpHandCache<'a> {
    pub fn new(mount: &'a AssetMount) -> Self {
        Self {
            mount,
            geometry: BTreeMap::new(),
            offsets: BTreeMap::new(),
        }
    }

    /// Requires the selected user mount and actual body4068/mode. Unknown or
    /// malformed geometry errors propagate rather than inventing hand length.
    pub fn hand_length(
        &mut self,
        dna: &NativeDhDna,
        field_4068: i32,
        mode: bool,
    ) -> io::Result<f32> {
        let request = NativeDpHandResource::new(dna, field_4068, mode);
        if let Some(offset) = self.offsets.get(request.model_dna())
            && offset.iter().any(|v| *v != 0.0)
        {
            return Ok(offset[1] as f32);
        }
        if self.offsets.len() >= 20_000 && !self.offsets.contains_key(request.model_dna()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "native hand-offset Rust cache budget",
            ));
        }
        if !self.geometry.contains_key(request.resource()) {
            let geometry = NativeDpHandGeometry::from_model(&request.read_model(self.mount)?)?;
            self.geometry
                .insert(request.resource().to_owned(), geometry);
        }
        let offset = self.geometry[request.resource()].offset_for_dna(dna);
        self.offsets.insert(request.model_dna().clone(), offset);
        Ok(offset[1] as f32)
    }
}
