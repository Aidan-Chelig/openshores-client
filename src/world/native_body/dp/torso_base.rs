//! The geometric prelude of PositionTorso, before its pose dispatch.
//! Static source: docs/evidence/native-dp-position-torso{.json,-gnu.txt}.
use super::{
    NativeDhDna, NativeDpDimensions, NativeDpHandCache, native_dh_has_wrist_bend,
    native_dh_waist_vertex, native_dh_walks_on_hands,
};
use std::io;

impl NativeDpHandCache<'_> {
    /// Compute the native hand-walking torso base from actual mounted hand
    /// geometry when required. This does not select a pose or mutate bones.
    pub fn torso_base_angle(
        &mut self,
        dna: &NativeDhDna,
        dimensions: &NativeDpDimensions,
        field_4068: i32,
        height: f32,
    ) -> io::Result<f32> {
        calculate(dna, dimensions.fields_4080_through_40b8(), height, || {
            self.hand_length(dna, field_4068, false)
        })
    }
}

fn calculate(
    dna: &NativeDhDna,
    dimensions: &[f32; 15],
    height: f32,
    mut hand_length: impl FnMut() -> io::Result<f32>,
) -> io::Result<f32> {
    if !native_dh_walks_on_hands(dna) {
        return Ok(0.);
    }
    if dna.0[20] & 0xe0 == 0 || native_dh_waist_vertex(dna) == 0 {
        return Ok(f32::from_bits(0x3fc9_0fdb));
    }
    let length = dimensions[9];
    if length == 0. {
        return Ok(0.);
    }
    let segments = dimensions[13] + dimensions[2];
    let span = if native_dh_has_wrist_bend(dna) {
        segments
    } else {
        // GNU addss order differs from flattened decompiler pseudocode.
        hand_length()? + segments
    };
    let scaled = span * 0.95_f32;
    // COMISS/JA and COMISS/JAE reject only ordered > and >= respectively.
    // Do not rewrite as <= / <: unordered values reach the native asinf.
    if height - length > scaled {
        return Ok(0.);
    }
    let offset = scaled - height;
    if offset.abs() >= length {
        return Ok(0.);
    }
    Ok((offset / length).asin())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn walker() -> NativeDhDna {
        let mut dna = NativeDhDna([0; 24]);
        dna.0[1] = 2;
        dna.0[20] = 0x60;
        dna
    }

    #[test]
    fn packed_locomotion_and_wrist_selectors_cover_all_bytes() {
        for class in 0..=255 {
            for arms in 0..=255 {
                for wings in [0, 2, 4, 6, 0xf8, 0xff] {
                    let mut dna = walker();
                    dna.0[11] = class;
                    dna.0[1] = arms;
                    dna.0[2] = wings;
                    let expected = !matches!(
                        (class >> 5, arms & 3, wings & 6),
                        (3 | 6, _, _) | (_, 0, _) | (_, 1, 2 | 4 | 6)
                    );
                    assert_eq!(native_dh_walks_on_hands(&dna), expected);
                    assert_eq!(native_dh_has_wrist_bend(&dna), !matches!(class >> 5, 4 | 6));
                }
            }
        }
    }

    #[test]
    fn resource_calls_follow_geometry_guards_and_propagate_errors() {
        let mut dna = walker();
        let mut d = [0.; 15];
        d[9] = 1.;
        d[13] = 0.5;
        assert!(native_dh_waist_vertex(&dna) != 0);
        assert!(calculate(&dna, &d, 0.25, || panic!()).unwrap() > 0.);
        dna.0[11] = 0x80;
        assert_eq!(
            calculate(&dna, &d, 0.25, || Err(io::ErrorKind::NotFound.into()))
                .unwrap_err()
                .kind(),
            io::ErrorKind::NotFound
        );
        let mut calls = 0;
        calculate(&dna, &d, 0.25, || {
            calls += 1;
            Ok(0.125)
        })
        .unwrap();
        assert_eq!(calls, 1);
        d[9] = -0.;
        assert_eq!(calculate(&dna, &d, 0., || panic!()).unwrap(), 0.);
        dna.0[20] = 0;
        assert_eq!(
            calculate(&dna, &d, 0., || panic!()).unwrap().to_bits(),
            0x3fc90fdb
        );
        dna.0[1] = 0;
        assert_eq!(calculate(&dna, &d, 0., || panic!()).unwrap(), 0.);
    }

    #[test]
    fn ordered_boundaries_and_unordered_values_match_instruction_branches() {
        let dna = walker();
        let mut d = [0.; 15];
        d[9] = 1.;
        for height in [-2., -1., 0., 0.5, 1., 2., f32::INFINITY, f32::NEG_INFINITY] {
            let actual = calculate(&dna, &d, height, || panic!()).unwrap();
            let expected = if height.abs() < 1. {
                (0_f32 - height).asin()
            } else {
                0.
            };
            assert_eq!(actual.to_bits(), expected.to_bits());
        }
        assert!(calculate(&dna, &d, f32::NAN, || panic!()).unwrap().is_nan());
        d[9] = f32::NAN;
        assert!(calculate(&dna, &d, 0., || panic!()).unwrap().is_nan());
    }

    #[test]
    fn hand_is_added_after_the_limb_sum_not_before() {
        let mut dna = walker();
        dna.0[11] = 0x80;
        let mut d = [0.; 15];
        d[9] = 1.;
        d[13] = 16_777_216.;
        d[2] = -16_777_216.;
        let actual = calculate(&dna, &d, 0., || Ok(1.)).unwrap();
        assert_eq!(actual.to_bits(), 0.95_f32.asin().to_bits());
        assert_eq!((1_f32 + d[13]) + d[2], 0.);
    }
}
