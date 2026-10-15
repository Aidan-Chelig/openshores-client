//! Torso CreateSkin geometry plan: combined cache key then ordered fallback.
use super::*;
use crate::model::NativeBodyModelGeometry;
use std::io;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeDpTorsoSkinPiece {
    pub resource: String,
    pub geometry: NativeBodyModelGeometry,
    pub bones: [usize; 3],
    pub crest: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct NativeDpTorsoSkinPlan {
    /// Native combined-model key, NOT a raw .3dsN resource filename.
    pub combined_key: String,
    pub geometry: NativeBodyModelGeometry,
    pub bones: [usize; 3],
    /// Used only if native combined-model creation fails. Not a substitute
    /// for implementing cache lookup, MergeTorsoWithInstance or publication.
    pub fallback: Vec<NativeDpTorsoSkinPiece>,
    pub merge_tolerance: f64,
}

impl NativeDpPartState {
    pub fn torso_skin_plan(
        &self,
        owner: &NativeDhDna,
        dimensions: &NativeDpDimensions,
        field_4068: i32,
        fields_4120: [f64; 4],
    ) -> io::Result<Option<NativeDpTorsoSkinPlan>> {
        if self.allocation.kind != NativeDpPartKind::Torso {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "native torso skin requested for non-torso part",
            ));
        }
        let raw = &self.allocation.dna.0;
        let count = raw[20] >> 5;
        if count == 0 {
            return Ok(None);
        }
        let step = dimensions.fields_4080_through_40b8()[10];
        let length = f32::from(raw[19] >> 3) * 0.0625 + 0.5;
        let width = f64::from(
            (f32::from(raw[20] & 31) * 0.0625 * 0.25 * length + length / 3_f32) / 0.875_f32,
        );
        let uniform = f64::from(native_dp_size_factor(owner) / 0.84_f32);
        let mut geometry = NativeBodyModelGeometry {
            fields_00: [
                f64::from(step),
                uniform,
                fields_4120[0],
                fields_4120[1],
                fields_4120[2],
                fields_4120[3],
            ],
            fields_40: [
                0.,
                f64::from((native_dh_waist_vertex(&self.allocation.dna) as i32 - 1) as f32 * step),
                0.,
            ],
            fields_58: [f64::from(length / 1.5_f32), width, 1., 1., 1.],
        };
        let pa = hand_resource::uses_pa(field_4068);
        let prefix = if pa {
            ":data/PA".to_owned()
        } else {
            format!(":data/{}", owner.0[16] & 7)
        };
        let crest = !pa && raw[18] & 0x30 != 0;
        let crest_shape = (raw[4] >> 2) & 3;
        let crest_variant = (raw[18] >> 4) & 3;
        let mut key = prefix.clone();
        for index in 0..u32::from(count) {
            let code = native_dp_torso_segment(owner, index);
            if code != 0 {
                key.push_str(&code.to_string());
                if crest {
                    key.push_str(&format!("TC{crest_shape}{crest_variant}"));
                }
            }
        }
        let base = geometry;
        let mut fallback = Vec::new();
        let (mut previous, mut current) = (0, 0);
        for index in 0..u32::from(count) {
            let code = native_dp_torso_segment(owner, index);
            if code == 0 {
                continue;
            }
            let next = current + usize::from(index != u32::from(count) - 1);
            let bones = [previous, current, next];
            fallback.push(NativeDpTorsoSkinPiece {
                resource: format!("{prefix}{}.3dsN", stem(code)),
                geometry,
                bones,
                crest: false,
            });
            if crest {
                fallback.push(NativeDpTorsoSkinPiece {
                    resource: format!("{prefix}TorsoCrest{crest_shape}{crest_variant}.3dsN"),
                    geometry,
                    bones,
                    crest: true,
                });
            }
            geometry.fields_40[1] -= f64::from(step);
            previous = current;
            current += 1;
        }
        Ok(Some(NativeDpTorsoSkinPlan {
            combined_key: key,
            geometry: base,
            bones: [0, 0, 1],
            fallback,
            merge_tolerance: (width * 0.05) * uniform,
        }))
    }
}

fn stem(code: u32) -> &'static str {
    match code {
        1 | 3 | 6 | 39 => "NoLimbClosedToButt",
        2 | 8 | 10 | 12 | 15 | 17 => "TopLimbToButt",
        4 | 31 => "NoLimbToArm",
        5 => "NoLimbClosedToNoLimb",
        7 | 16 => "TopLimbToLeg",
        9 => "NoLimbClosedToArmClosed",
        11 | 18 => "TopToTop",
        13 => "TopLimbToArm",
        14 => "TopLimbToNoLimb",
        19 | 21 | 24 | 26 => "ArmToButt",
        20 | 27 => "ArmToArmClosed",
        22 => "ArmToArm",
        23 => "ArmToNoLimb",
        25 => "ArmToLeg",
        28 | 30 | 33 | 35 => "NoLimbToButt",
        32 => "NoLimbToNoLimb",
        34 => "NoLimbToLeg",
        37 | 44 | 46 | 48 | 51 | 53 => "LegToButt",
        41 => "LegClosedToNoLimb",
        42 => "LegClosedToButt",
        43 => "LegClosedToLeg",
        50 => "LegToNoLimb",
        52 => "LegToLeg",
        _ => "ModelMissing", // Literal native placeholder, not a fallback asset.
    }
}

impl NativeDpInitialization {
    pub fn torso_skin_geometry_plan(
        &self,
        index: usize,
        field_4068: i32,
        fields_4120: [f64; 4],
    ) -> io::Result<Option<NativeDpTorsoSkinPlan>> {
        if !self.parts_calculated() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "native torso skin requires completed part calculations",
            ));
        }
        self.parts
            .get(index)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "native part index out of range",
                )
            })?
            .torso_skin_plan(&self.dna, &self.dimensions, field_4068, fields_4120)
    }
}

#[cfg(test)]
mod tests;
