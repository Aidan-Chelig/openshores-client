//! DpLegAndFoot::CalcStandAngles, VA 1412d8dc0. See retained GNU evidence.
use super::{
    NativeDhDna, NativeDpDimensions, NativeDpFootCache, native_dh_has_ankle_bend,
    native_dh_leg_vertices,
};
use std::io;

const QUARTER: f32 = f32::from_bits(0x3f490fdb);
const HALF: f32 = f32::from_bits(0x3fc90fdb);
const PI: f32 = f32::from_bits(0x40490fdb);
const DIAGONAL: f32 = f32::from_bits(0x3f3504f3);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativeDpLegStandAngles {
    pub hip: f32,
    pub knee: f32,
    pub ankle: f32,
}

impl NativeDpFootCache<'_> {
    /// The native caller resolves concrete Bones before this calculation.
    /// This computes angles only; it does not alter bones or apply hip splay.
    pub fn leg_stand_angles(
        &mut self,
        owner_dna: &NativeDhDna,
        part_dna: &NativeDhDna,
        dimensions: &NativeDpDimensions,
        field_4068: i32,
        height: f32,
    ) -> io::Result<NativeDpLegStandAngles> {
        let foot = self.foot_length(owner_dna, field_4068, false)?;
        let d = dimensions.fields_4080_through_40b8();
        calculate(part_dna, height, d[7], d[0], foot)
    }
}

fn calculate(
    dna: &NativeDhDna,
    height: f32,
    upper: f32,
    lower: f32,
    foot: f32,
) -> io::Result<NativeDpLegStandAngles> {
    let unsupported = || {
        io::Error::new(
            io::ErrorKind::Unsupported,
            "nonfinite native standing-leg solution",
        )
    };
    if ![height, upper, lower, foot].iter().all(|x| x.is_finite()) {
        return Err(unsupported());
    }
    let ankle = native_dh_has_ankle_bend(dna);
    let (foot_angle, extended) = if ankle {
        let angle = (foot / lower).atan();
        (angle, lower / angle.cos())
    } else {
        (0., lower + foot)
    };
    let triangle = !super::layout::native_dh_is_exoskeletal(dna)
        && u32::from((dna.0[12] >> 2) & 3) <= native_dh_leg_vertices(dna)
        && (!ankle || foot <= height)
        && height < upper + extended
        && upper < height + extended
        && extended < upper + height;
    let (hip, knee) = if triangle {
        // The triangle gate uses the extended calf, but native deliberately
        // uses the unextended calf if both segments are longer than the foot.
        let (segment, correction) = if ankle && upper > foot && lower > foot {
            (lower, 0.)
        } else {
            (extended, foot_angle)
        };
        let upper2 = upper * upper;
        let height2 = height * height;
        let segment2 = segment * segment;
        let denominator = f64::from(upper) * -2.;
        let hip =
            (f64::from((segment2 - upper2) - height2) / (f64::from(height) * denominator)) as f32;
        let knee =
            (f64::from((height2 - upper2) - segment2) / (f64::from(segment) * denominator)) as f32;
        (hip.acos() - HALF, knee.acos() - correction)
    } else if extended > 0. {
        let diagonal_height = height + upper * DIAGONAL;
        if diagonal_height <= extended {
            (
                QUARTER,
                ((diagonal_height / extended).acos() + QUARTER) - foot_angle,
            )
        } else if extended + upper > height {
            let hip = ((extended - height) / upper).asin();
            // Constant at 1422bfde0 is the widened f32 HALF, not f64 PI/2.
            (
                hip,
                ((f64::from(HALF) - f64::from(hip)) - f64::from(foot_angle)) as f32,
            )
        } else {
            (QUARTER, 0.)
        }
    } else {
        (0., 0.)
    };
    let result = NativeDpLegStandAngles {
        hip,
        knee: knee - PI,
        ankle: if ankle { HALF } else { 0. },
    };
    if [result.hip, result.knee, result.ankle]
        .iter()
        .all(|x| x.is_finite())
    {
        Ok(result)
    } else {
        // Preview safety boundary: native does not clamp invalid trig domains.
        Err(unsupported())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dna(species: u8, ankle: bool) -> NativeDhDna {
        let mut dna = NativeDhDna([0; 24]);
        dna.0[12] = 4;
        dna.0[16] = species;
        dna.0[10] = if ankle { 0 } else { 0x80 };
        dna
    }

    fn close(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 1e-6, "{actual} != {expected}");
    }

    #[test]
    fn endoskeletal_triangle_and_ankle_segment_selection() {
        // Equilateral no-ankle triangle: calf plus foot makes the lower side.
        let a = calculate(&dna(1, false), 1., 1., 0.75, 0.25).unwrap();
        close(a.hip, std::f32::consts::FRAC_PI_3 - HALF);
        close(a.knee, std::f32::consts::FRAC_PI_3 - PI);
        assert_eq!(a.ankle, 0.);
        // Gate uses the extended length, solution uses the plain calf here.
        let a = calculate(&dna(1, true), 1., 1., 1., 0.25).unwrap();
        close(a.hip, std::f32::consts::FRAC_PI_3 - HALF);
        close(a.knee, std::f32::consts::FRAC_PI_3 - PI);
        assert_eq!(a.ankle, HALF);
        // Foot as long as calf selects extension AND subtracts its angle.
        let a = calculate(&dna(1, true), 1., 1., 0.5, 0.5).unwrap();
        let side = 0.5 / (1_f32.atan()).cos();
        let hip = (((side * side - 1.) - 1.) / -2.).acos() - HALF;
        let knee = (((1. - 1.) - side * side) / (-2. * side)).acos() - QUARTER - PI;
        close(a.hip, hip);
        close(a.knee, knee);
    }

    #[test]
    fn exoskeletal_fallback_covers_three_height_regions_and_zero_segment() {
        for species in [0, 5] {
            let dna = dna(species, false);
            let a = calculate(&dna, 0.1, 1., 1., 0.).unwrap();
            assert_eq!(a.hip, QUARTER);
            close(a.knee, ((0.1 + DIAGONAL).acos() + QUARTER) - PI);
            let a = calculate(&dna, 1., 1., 1., 0.).unwrap();
            assert_eq!(a.hip, 0.);
            assert_eq!(a.knee, HALF - PI);
            let a = calculate(&dna, 2., 1., 1., 0.).unwrap();
            assert_eq!(a.hip, QUARTER);
            assert_eq!(a.knee, -PI);
            let a = calculate(&dna, 0., 1., 0., 0.).unwrap();
            assert_eq!(a.hip, 0.);
            assert_eq!(a.knee, -PI);
        }
        for species in [1, 2, 3, 4, 6, 7] {
            assert!(!super::super::layout::native_dh_is_exoskeletal(&dna(
                species, false
            )));
        }
    }

    #[test]
    fn extra_leg_pairs_and_foot_above_height_select_fallback() {
        let mut extra = dna(1, false);
        extra.0[12] = 8;
        assert_eq!(native_dh_leg_vertices(&extra), 1);
        assert_eq!(calculate(&extra, 1., 1., 1., 0.).unwrap().hip, 0.);
        let a = calculate(&dna(1, true), 0.25, 1., 1., 0.5).unwrap();
        let exo = calculate(&dna(0, true), 0.25, 1., 1., 0.5).unwrap();
        assert_eq!(a, exo);
        // Equality at the triangle boundary goes to the fallback too.
        assert_eq!(
            calculate(&dna(1, false), 2., 1., 1., 0.).unwrap().hip,
            QUARTER
        );
    }

    #[test]
    fn invalid_domains_are_not_clamped_to_a_fabricated_pose() {
        for height in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -10.] {
            assert_eq!(
                calculate(&dna(0, false), height, 1., 1., 0.)
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::Unsupported
            );
        }
    }
}
