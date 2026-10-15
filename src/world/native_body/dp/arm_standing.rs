//! DpArmAndHand::CalcStandAngles, VA1412d2000; static GNU evidence retained.
use super::*;
use std::io;

const HALF: f32 = f32::from_bits(0x3fc90fdb);
const PI: f32 = f32::from_bits(0x40490fdb);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativeDpArmStandAngles {
    pub shoulder: f32,
    pub elbow: f32,
    pub wrist: f32,
}

impl NativeDpPartState {
    /// Reads owner HandLength(false) and the primary first-reference bone's
    /// projected origin Z. This is not the stored shoulder attachment height.
    #[allow(clippy::too_many_arguments)]
    pub fn arm_stand_angles(
        &self,
        owner_dna: &NativeDhDna,
        dimensions: &NativeDpDimensions,
        owner: &NativeDpBoneTransforms,
        field_4068: i32,
        height: f32,
        hands: &mut NativeDpHandCache<'_>,
    ) -> io::Result<NativeDpArmStandAngles> {
        if !matches!(
            self.allocation.kind,
            NativeDpPartKind::LeftArmAndHand | NativeDpPartKind::RightArmAndHand
        ) {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "native arm standing angles require ArmAndHand",
            ));
        }
        let first = self.limb_bones()?[0];
        let hand = hands.hand_length(owner_dna, field_4068, false)?;
        let origin_z = owner.slots[first].project([0.; 3])[2];
        let d = dimensions.fields_4080_through_40b8();
        calculate(&self.allocation.dna, height, origin_z, d[13], d[2], hand)
    }
}

fn calculate(
    dna: &NativeDhDna,
    height: f32,
    origin_z: f64,
    upper: f32,
    lower: f32,
    hand: f32,
) -> io::Result<NativeDpArmStandAngles> {
    let unsupported = || {
        io::Error::new(
            io::ErrorKind::Unsupported,
            "nonfinite native standing-arm solution",
        )
    };
    if ![height, upper, lower, hand].iter().all(|x| x.is_finite()) || !origin_z.is_finite() {
        return Err(unsupported());
    }
    let wrist_bend = native_dh_has_wrist_bend(dna);
    let extended = if wrist_bend { lower } else { hand + lower };
    let nominal = (extended + upper) * f32::from_bits(0x3f733333);
    let requested = (f64::from(height) + origin_z) as f32;
    let limited = f64::from(nominal) - f64::from(requested) >= -1e-5_f64;
    let reach = if limited { requested } else { nominal };
    let upper2 = upper * upper;
    let lower2 = extended * extended;
    let reach2 = reach * reach;
    let denominator = -2_f32 * upper;
    let mut shoulder = (((lower2 - upper2) - reach2) / (denominator * reach)).acos();
    let mut elbow = (((reach2 - upper2) - lower2) / (denominator * extended)).acos();
    // Native evaluates the acos values before deciding whether to overwrite
    // them with its alternate solution. Do not reject those intermediates.
    let wrist = if limited {
        let check_lower = if wrist_bend { lower } else { lower + hand };
        let triangle = reach < upper + check_lower
            && upper < reach + check_lower
            && check_lower < reach + upper;
        if (super::layout::native_dh_is_exoskeletal(dna) || !triangle) && extended > 0. {
            let diagonal = reach + upper * f32::from_bits(0x3f3504f3);
            if diagonal > extended {
                let angle = ((extended - reach) / upper).asin();
                shoulder = angle + HALF;
                elbow = HALF - angle;
            } else {
                elbow = (diagonal / extended).acos() + f32::from_bits(0x3f490fdb);
                shoulder = f32::from_bits(0x4016cbe4);
            }
        }
        if wrist_bend {
            (elbow + shoulder) - HALF
        } else {
            f32::from_bits(0x3d0efa35)
        }
    } else {
        f32::from_bits(0x3d0efa35)
    };
    let result = NativeDpArmStandAngles {
        shoulder: -HALF - shoulder,
        elbow: PI - elbow,
        wrist,
    };
    if [result.shoulder, result.elbow, result.wrist]
        .iter()
        .all(|x| x.is_finite())
    {
        Ok(result)
    } else {
        Err(unsupported())
    }
}

#[cfg(test)]
mod tests;
