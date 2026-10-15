//! DpBody::SelectTextures' normal skin/clothes atlas selection only.
//! Does not implement texture binding, clothing composition, or helmet selection.
use crate::{asset_mount::AssetMount, world::native_dna::NativeDhDna};
use std::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeDpSkinAtlas {
    Skin,
    Exo,
    Scales,
    FurFeathers,
}

/// SetupSkinMaterial's atlas offsets (top U/V, bottom U/V), independent of
/// lighting/material QColor fields and the separate clothes wear uniforms.
pub fn native_dp_skin_offsets(dna: &NativeDhDna) -> [f64; 4] {
    let top = dna.0[19] & 7;
    let bottom = dna.0[3] & 7;
    [
        f64::from(top & 3) * 1024. * (1. / 4096.),
        if top < 4 { 0. } else { 0.75 },
        f64::from(bottom & 3) * 1024. * (1. / 4096.),
        if bottom < 4 { 0. } else { 0.75 },
    ]
}

impl NativeDpSkinAtlas {
    /// Native DpBody DNA begins at +0x18; SelectTextures tests byte +0x28.
    pub fn from_dna(dna: &NativeDhDna) -> Self {
        match dna.0[16] & 7 {
            2 | 7 => Self::Skin,   // native first mask 0x84
            0 | 5 => Self::Exo,    // second mask 0x21
            1 | 6 => Self::Scales, // third mask 0x42
            3 | 4 => Self::FurFeathers,
            _ => unreachable!(),
        }
    }

    /// Relative installation path, NOT a Qt model resource or model texture name.
    pub fn skin_path(self) -> &'static str {
        match self {
            Self::Skin => "tex/Skin.png",
            Self::Exo => "tex/Exo.png",
            Self::Scales => "tex/Scales.png",
            Self::FurFeathers => "tex/FurFeathers.png",
        }
    }

    pub fn clothes_path(self) -> &'static str {
        match self {
            Self::Skin => "tex/SkinClothes.png",
            Self::Exo => "tex/ExoClothes.png",
            Self::Scales => "tex/ScalesClothes.png",
            Self::FurFeathers => "tex/FurFeathersClothes.png",
        }
    }

    /// Reuse mount validation/bounds rather than opening paths independently.
    /// Returned bytes remain encoded; this is not successful texture decoding.
    pub fn read_skin(self, mount: &AssetMount) -> io::Result<Vec<u8>> {
        mount
            .read(&format!("loose/{}", self.skin_path()))
            .map_err(|error| {
                io::Error::new(
                    error.kind(),
                    format!("skin atlas {}: {error}", self.skin_path()),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skin_offsets_cover_all_top_bottom_tiles_and_ignore_other_bits() {
        for top in 0..=u8::MAX {
            for bottom in 0..=u8::MAX {
                let mut dna = NativeDhDna([255; 24]);
                dna.0[19] = top;
                dna.0[3] = bottom;
                assert_eq!(
                    native_dp_skin_offsets(&dna),
                    [
                        f64::from(top % 4) / 4.,
                        if top & 4 == 0 { 0. } else { 0.75 },
                        f64::from(bottom % 4) / 4.,
                        if bottom & 4 == 0 { 0. } else { 0.75 },
                    ]
                );
            }
        }
    }

    #[test]
    fn all_dna_byte_values_follow_native_ordered_masks() {
        let expected = [
            NativeDpSkinAtlas::Exo,
            NativeDpSkinAtlas::Scales,
            NativeDpSkinAtlas::Skin,
            NativeDpSkinAtlas::FurFeathers,
            NativeDpSkinAtlas::FurFeathers,
            NativeDpSkinAtlas::Exo,
            NativeDpSkinAtlas::Scales,
            NativeDpSkinAtlas::Skin,
        ];
        for byte in 0..=u8::MAX {
            let mut dna = NativeDhDna([255; 24]);
            dna.0[16] = byte;
            let atlas = NativeDpSkinAtlas::from_dna(&dna);
            assert_eq!(atlas, expected[usize::from(byte & 7)]);
            assert!(atlas.skin_path().starts_with("tex/"));
            assert_eq!(
                atlas.clothes_path(),
                atlas.skin_path().replace(".png", "Clothes.png")
            );
        }
    }

    #[test]
    #[ignore = "requires OPENSHORES_HAZERON_INSTALLATION; mounted bytes only"]
    fn mounted_native_skin_atlases_exist() {
        let mount =
            AssetMount::open(std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").unwrap()).unwrap();
        for atlas in [
            NativeDpSkinAtlas::Skin,
            NativeDpSkinAtlas::Exo,
            NativeDpSkinAtlas::Scales,
            NativeDpSkinAtlas::FurFeathers,
        ] {
            let bytes = atlas.read_skin(&mount).unwrap();
            assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
            assert!(
                mount
                    .read(&format!("loose/{}", atlas.clothes_path()))
                    .unwrap()
                    .starts_with(b"\x89PNG\r\n\x1a\n")
            );
        }
    }
}
