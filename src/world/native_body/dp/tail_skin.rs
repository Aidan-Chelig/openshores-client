//! DpTail::CreateSkin geometry requests, excluding material/cache/device effects.
use super::*;
use crate::model::NativeBodyModelGeometry;
use std::io;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeDpTailSkinGeometry {
    pub resource: String,
    pub geometry: NativeBodyModelGeometry,
    pub bones: [usize; 3],
}

impl NativeDpPartState {
    /// Requires this part's OneTimeCalcs attachment and actual owner fields.
    /// None is the native disabled-tail case, not missing/failed geometry.
    pub fn tail_skin_geometry(
        &self,
        owner_dna: &NativeDhDna,
        fields_4120: [f64; 4],
    ) -> io::Result<Option<NativeDpTailSkinGeometry>> {
        if self.allocation.kind != NativeDpPartKind::Tail {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "native tail skin requested for non-tail part",
            ));
        }
        let raw = &self.allocation.dna.0;
        if raw[14] & 0x30 == 0 {
            return Ok(None);
        }
        let segments = raw[14] >> 6;
        let stem = if segments == 0 {
            "Tip".to_owned()
        } else {
            format!("Tail{}", segments * 2 - 1)
        };
        let length = f32::from(raw[19] >> 3) * 0.0625 + 0.5;
        let width = f32::from(raw[20] & 31) * 0.0625 * 0.25 * length;
        let third = length / 3_f32;
        Ok(Some(NativeDpTailSkinGeometry {
            resource: format!(":data/{}{}{}.3dsN", raw[16] & 7, stem, (raw[14] >> 4) & 3),
            geometry: NativeBodyModelGeometry {
                // XMM1 is loaded from the1.0 constant before construction;
                // decompiler's literal0 does not describe this float ABI.
                fields_00: [
                    1.,
                    f64::from(native_dp_size_factor(owner_dna) / 0.84_f32),
                    fields_4120[0],
                    fields_4120[1],
                    fields_4120[2],
                    fields_4120[3],
                ],
                fields_40: self.connect_30,
                // Tail divisions happen in float BEFORE widening, unlike the
                // analogous arm/leg factor's double division.
                fields_58: [
                    1.,
                    1.,
                    f64::from(length / 1.5_f32),
                    f64::from((width + third) / 0.875_f32),
                    1.,
                ],
            },
            bones: [17; 3],
        }))
    }
}

impl NativeDpInitialization {
    pub fn tail_skin_geometry_request(
        &self,
        part_index: usize,
        fields_4120: [f64; 4],
    ) -> io::Result<Option<NativeDpTailSkinGeometry>> {
        if !self.parts_calculated() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "native tail skin requires completed part calculations",
            ));
        }
        self.parts
            .get(part_index)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "native part index out of range",
                )
            })?
            .tail_skin_geometry(&self.dna, fields_4120)
    }
}

#[cfg(test)]
mod tests;
