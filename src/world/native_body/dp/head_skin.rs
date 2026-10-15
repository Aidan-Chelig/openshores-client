//! Head/neck CreateSkin geometry plan, not merging/cache/eye publication.
use super::*;
use crate::model::NativeBodyModelGeometry;
use std::io;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeDpHeadSkinPiece {
    pub resource: String,
    pub geometry: NativeBodyModelGeometry,
    pub bones: [usize; 3],
    /// None for the separately published Neck (no MergeHead call).
    pub merge_tolerance: Option<f64>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct NativeDpHeadSkinPlan {
    pub combined_key: String,
    pub geometry: NativeBodyModelGeometry,
    pub bones: [usize; 3],
    /// First Skull is attempted on combined-cache failure. Later pieces are
    /// attempted only after a head instance exists; no success is fabricated.
    pub fallback: Vec<NativeDpHeadSkinPiece>,
    /// PA modes load this separately for body4050..4060, not a merged piece.
    pub eye_reference: Option<String>,
    pub neck: Option<NativeDpHeadSkinPiece>,
}

fn helmet_pa(value: i32) -> bool {
    let index = value.wrapping_sub(0x6a) as u32;
    value == 0x48 || index < 0x33 && (0x5000000000001_u64 >> index) & 1 != 0
}

impl NativeDpPartState {
    pub fn head_skin_plan(
        &self,
        owner: &NativeDhDna,
        dimensions: &NativeDpDimensions,
        field_406c: i32,
        fields_4120: [f64; 4],
    ) -> io::Result<NativeDpHeadSkinPlan> {
        if self.allocation.kind != NativeDpPartKind::HeadAndNeck {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "native head skin requested for non-head part",
            ));
        }
        let NativeDpPartJoints::Head { field_50 } = self.joints else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "native head attachment storage missing",
            ));
        };
        let raw = &self.allocation.dna.0;
        let bone = if self.allocation.index == 1 { 12 } else { 7 };
        let bones = [bone; 3];
        let length = f32::from(raw[19] >> 3) * 0.0625 + 0.5;
        let width = f64::from(
            (f32::from(raw[20] & 31) * 0.0625 * 0.25 * length + length / 3_f32) / 0.875_f32,
        );
        let uniform = f64::from(native_dp_size_factor(owner) / 0.84_f32);
        let geometry = NativeBodyModelGeometry {
            fields_00: [
                1.,
                uniform,
                fields_4120[0],
                fields_4120[1],
                fields_4120[2],
                fields_4120[3],
            ],
            fields_40: field_50,
            fields_58: [1., 1., f64::from(length / 1.5_f32), width, 1.],
        };
        let pa = helmet_pa(field_406c);
        let prefix = if pa {
            ":data/PA".to_owned()
        } else {
            format!(":data/{}", owner.0[16] & 7)
        };
        let skull = raw[16] & 7;
        let skull_prefix = if pa {
            format!("{prefix}{skull}")
        } else {
            prefix.clone()
        };
        let selectors = [
            ("Eye", raw[8] & 7, ((raw[7] >> 2) & 3) + 1),
            ("Ear", raw[6] & 7, ((raw[4] >> 4) & 3) + 1),
            ("Nose", raw[15] & 7, ((raw[14] >> 2) & 3) + 1),
            ("Mouth", raw[13] & 7, (raw[12] >> 6) + 1),
        ];
        let mut key = format!("{skull_prefix}Head");
        for (_, model, count) in selectors {
            key.push_str(&format!("{model}{count}"));
        }
        if raw[12] & 3 != 0 {
            key.push_str(&format!("HS{}{}", raw[7] >> 6, raw[12] & 3));
        }
        if raw[7] & 0x30 != 0 {
            key.push_str(&format!("HC{}{}", (raw[4] >> 2) & 3, (raw[7] >> 4) & 3));
        }
        let tolerance = (width * 0.001) * uniform;
        let mut fallback = vec![NativeDpHeadSkinPiece {
            resource: format!("{skull_prefix}Skull.3dsN"),
            geometry,
            bones,
            merge_tolerance: Some(tolerance),
        }];
        if !pa {
            for ((name, model, count), integrated) in selectors.into_iter().zip([4, 4, 3, 5]) {
                if skull != 5 || model != integrated {
                    fallback.push(NativeDpHeadSkinPiece {
                        resource: format!("{prefix}{name}{model}{count}.3dsN"),
                        geometry,
                        bones,
                        merge_tolerance: Some(tolerance),
                    });
                }
            }
            if raw[12] & 3 != 0 {
                fallback.push(NativeDpHeadSkinPiece {
                    resource: format!("{prefix}Horn{}{}.3dsN", raw[7] >> 6, raw[12] & 3),
                    geometry,
                    bones,
                    merge_tolerance: Some(tolerance),
                });
            }
            if raw[7] & 0x30 != 0 {
                fallback.push(NativeDpHeadSkinPiece {
                    resource: format!(
                        "{prefix}HeadCrest{}{}.3dsN",
                        (raw[4] >> 2) & 3,
                        (raw[7] >> 4) & 3
                    ),
                    geometry,
                    bones,
                    merge_tolerance: Some((uniform * width) * 0.001),
                });
            }
        }
        let count = (raw[14] & 3) + ((raw[5] >> 1) & 1);
        let neck = if count != 0 && (raw[5] & 2 == 0 || self.allocation.index == 1) {
            let mut geometry = geometry;
            geometry.fields_40[1] -=
                f64::from((dimensions.fields_4080_through_40b8()[10] / 2.5_f32) * f32::from(count));
            let end = match count {
                2 => 9,
                3 => 10,
                _ => 8,
            };
            Some(NativeDpHeadSkinPiece {
                resource: format!(
                    "{prefix}Neck{count}{}.3dsN",
                    self.allocation.index.wrapping_add(1) as i32
                ),
                geometry,
                bones: [8, end, bone],
                merge_tolerance: None,
            })
        } else {
            None
        };
        Ok(NativeDpHeadSkinPlan {
            combined_key: key,
            geometry,
            bones,
            fallback,
            eye_reference: pa.then(|| format!(":data/{skull}Skull.3dsN")),
            neck,
        })
    }
}

impl NativeDpInitialization {
    pub fn head_skin_geometry_plan(
        &self,
        index: usize,
        field_406c: i32,
        fields_4120: [f64; 4],
    ) -> io::Result<NativeDpHeadSkinPlan> {
        if !self.parts_calculated() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "native head skin requires completed part calculations",
            ));
        }
        let mut part = self
            .parts
            .get(index)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "native part index out of range",
                )
            })?
            .clone();
        // CreateSkin repeats OneTimeCalcs. A previously posed field50 is not
        // the definition attachment. This read-only request projection uses
        // a recalculated copy, not the live pose or a guessed zero attachment.
        if part.allocation.kind == NativeDpPartKind::HeadAndNeck {
            part.calculate(&self.dimensions, &mut || {
                Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "head calculation must not request HandLength",
                ))
            })?;
        }
        part.head_skin_plan(&self.dna, &self.dimensions, field_406c, fields_4120)
    }
}

#[cfg(test)]
mod tests;
