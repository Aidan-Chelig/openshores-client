//! Geometry requests made by DpArmAndHand::CreateSkin, not GL/material creation.
use super::*;
use crate::model::NativeBodyModelGeometry;
use std::io;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeDpArmSkinGeometry {
    pub resource: String,
    pub geometry: NativeBodyModelGeometry,
    pub bones: [usize; 3],
}

impl NativeDpPartState {
    /// Caller supplies actual owner DNA/dimensions/4120..4138 and this part's
    /// OneTimeCalcs connection. No guessed material, model defaults or bounds.
    pub fn arm_skin_geometry(
        &self,
        owner_dna: &NativeDhDna,
        dimensions: &NativeDpDimensions,
        field_4068: i32,
        fields_4120: [f64; 4],
    ) -> io::Result<Vec<NativeDpArmSkinGeometry>> {
        if !matches!(
            self.allocation.kind,
            NativeDpPartKind::LeftArmAndHand | NativeDpPartKind::RightArmAndHand
        ) {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "native arm skin requested for non-arm part",
            ));
        }
        let raw = &self.allocation.dna.0;
        let selector = raw[2] & 7;
        let hand = (f32::from(raw[11] & 31) * 0.0625 + 0.5) * 0.25;
        let torso_height = f64::from(f32::from(raw[21] & 31) * 0.0625 + 0.5) / 1.5;
        let torso_width = f64::from((f32::from(raw[22] & 31) * 0.0625 + 1.) * 0.25);
        let arm_height = f64::from(f32::from(raw[6] >> 3) * 0.0625 + 0.5) / 1.5;
        let arm_width = f64::from((f32::from(raw[8] >> 3) * 0.0625 + 1.) * 0.25);
        let bones = self.limb_bones()?;
        // Native selects Right with this literal mask of the first Bones out
        // reference, not part-index parity or a guessed mirroring transform.
        let right = bones[0] < 41 && (0xfffffefbefffffff_u64 >> bones[0]) & 1 == 0;
        let side = if right { "Right" } else { "Left" };
        let pa = hand_resource::uses_pa(field_4068);
        let prefix = if pa {
            ":data/PA".to_owned()
        } else {
            format!(":data/{}", owner_dna.0[16] & 7)
        };
        let geometry = NativeBodyModelGeometry {
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
                torso_height,
                (torso_width + torso_width) * torso_height,
                arm_height,
                (arm_width + arm_width) * arm_height,
                if selector > 1 {
                    f64::from(hand + hand)
                } else {
                    1.
                },
            ],
        };
        let resource = if selector > 1 {
            format!(":data/Wing{side}{selector}.3dsN")
        } else if pa {
            format!("{prefix}Arm{side}.3dsN")
        } else {
            format!("{prefix}Arm{side}{selector}.3dsN")
        };
        let mut requests = vec![NativeDpArmSkinGeometry {
            resource,
            geometry,
            bones,
        }];
        if selector < 2 {
            let mut geometry = geometry;
            let d = dimensions.fields_4080_through_40b8();
            geometry.fields_40[1] = (f64::from(d[13]) + geometry.fields_40[1]) + f64::from(d[2]);
            // Native local_268 +68 (dStack_200), NOT the final78 scalar.
            geometry.fields_58[2] = f64::from(hand + hand);
            let resource = if pa {
                format!("{prefix}Hand{side}.3dsN")
            } else {
                format!("{prefix}Hand{side}{}.3dsN", raw[11] >> 5)
            };
            requests.push(NativeDpArmSkinGeometry {
                resource,
                geometry,
                bones,
            });
        }
        Ok(requests)
    }
}

impl NativeDpInitialization {
    pub fn arm_skin_geometry_requests(
        &self,
        part_index: usize,
        field_4068: i32,
        fields_4120: [f64; 4],
    ) -> io::Result<Vec<NativeDpArmSkinGeometry>> {
        if !self.parts_calculated() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "native arm skin requires completed part calculations",
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
            .arm_skin_geometry(&self.dna, &self.dimensions, field_4068, fields_4120)
    }
}

#[cfg(test)]
mod tests;
