//! Leg/Foot CreateSkin geometry projection; no cache/material/device effects.
use super::*;
use crate::model::NativeBodyModelGeometry;
use std::io;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeDpLegSkinGeometry {
    pub resource: String,
    pub geometry: NativeBodyModelGeometry,
    pub bones: [usize; 3],
}

impl NativeDpPartState {
    /// Requires this part's OneTimeCalcs connection and actual owner fields.
    pub fn leg_skin_geometry(
        &self,
        owner_dna: &NativeDhDna,
        dimensions: &NativeDpDimensions,
        field_4068: i32,
        fields_4120: [f64; 4],
    ) -> io::Result<Vec<NativeDpLegSkinGeometry>> {
        if !matches!(
            self.allocation.kind,
            NativeDpPartKind::LeftLegAndFoot | NativeDpPartKind::RightLegAndFoot
        ) {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "native leg skin requested for non-leg part",
            ));
        }
        let bones = self.limb_bones()?;
        let right = bones[0] < 59 && (0xfbefbfffffffffff_u64 >> bones[0]) & 1 == 0;
        let side = if right { "Right" } else { "Left" };
        let raw = &self.allocation.dna.0;
        let height = f64::from(f32::from(raw[15] >> 3) * 0.0625 + 0.5) / 1.5;
        let width = f64::from((f32::from(raw[16] >> 3) * 0.0625 + 1.) * 0.25);
        let leg_height = f64::from(f32::from(raw[2] >> 3) * 0.0625 + 0.5) / 1.5;
        let leg_width = f64::from((f32::from(raw[3] >> 3) * 0.0625 + 1.) * 0.25);
        let foot = (f32::from(raw[10] & 31) * 0.0625 + 0.5) * 0.25;
        let pa = hand_resource::uses_pa(field_4068);
        let prefix = if pa {
            ":data/PA".to_owned()
        } else {
            format!(":data/{}", owner_dna.0[16] & 7)
        };
        let geometry = NativeBodyModelGeometry {
            // XMM1 contains1. Decompiler literal0/int4120 are incorrect ABI
            // inference; disassembly forwards4120 via MOVSD, no conversion.
            fields_00: [
                1.,
                f64::from(native_dp_size_factor(owner_dna) / 0.84_f32),
                fields_4120[0],
                fields_4120[1],
                fields_4120[2],
                fields_4120[3],
            ],
            fields_40: self.connect_30,
            fields_58: [
                height,
                (width + width) * height,
                leg_height,
                (leg_width + leg_width) * leg_height,
                1.,
            ],
        };
        let resource = if pa {
            format!("{prefix}Leg{side}.3dsN")
        } else {
            format!("{prefix}Leg{side}{}.3dsN", (raw[12] >> 4) & 3)
        };
        let mut requests = vec![NativeDpLegSkinGeometry {
            resource,
            geometry,
            bones,
        }];
        let mut geometry = geometry;
        let d = dimensions.fields_4080_through_40b8();
        geometry.fields_40[1] = (f64::from(d[7]) + geometry.fields_40[1]) + f64::from(d[0]);
        geometry.fields_58[2] = f64::from(foot + foot);
        let selector = if pa {
            u8::from(native_dh_has_ankle_bend(&self.allocation.dna))
        } else {
            raw[10] >> 5
        };
        requests.push(NativeDpLegSkinGeometry {
            resource: format!("{prefix}Foot{side}{selector}.3dsN"),
            geometry,
            bones,
        });
        Ok(requests)
    }
}

impl NativeDpInitialization {
    pub fn leg_skin_geometry_requests(
        &self,
        part_index: usize,
        field_4068: i32,
        fields_4120: [f64; 4],
    ) -> io::Result<Vec<NativeDpLegSkinGeometry>> {
        if !self.parts_calculated() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "native leg skin requires completed part calculations",
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
            .leg_skin_geometry(&self.dna, &self.dimensions, field_4068, fields_4120)
    }
}

#[cfg(test)]
mod tests;
