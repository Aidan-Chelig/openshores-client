use super::*;

impl NativeDpPartState {
    pub(super) fn calculate_arm(
        &mut self,
        dimensions: &NativeDpDimensions,
        hand_length: &mut impl FnMut() -> io::Result<f32>,
    ) -> io::Result<()> {
        let dna = &self.allocation.dna;
        let raw = &dna.0;
        let d = dimensions.fields_4080_through_40b8();
        let waist = native_dh_waist_vertex(dna);
        let wings = raw[1] & 3 != 0 && raw[2] & 6 != 0;
        let left = self.allocation.kind == NativeDpPartKind::LeftArmAndHand;
        let segmented = raw[20] & 0xe0 != 0;
        let (mut x, y, z) = if wings {
            let mut y = f64::from(waist as f32 * d[10]);
            if segmented {
                y -= f64::from(d[10]) * 0.5;
            }
            let mut x = d[12] * 0.25;
            if left {
                x = -x;
            }
            let mut x = f64::from(x);
            let mut z = d[11] * 0.5;
            if !segmented {
                x /= 3.0_f64;
                z /= 3.0_f32;
            }
            (x, y, f64::from(z) + 0.0)
        } else {
            let vertices = native_dh_arm_vertices(dna);
            if vertices == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "native arm vertex modulo zero",
                ));
            }
            // Native uint subtraction zero-extends into the 64-bit float
            // conversion, rather than becoming a signed negative position.
            let y = f64::from(waist.wrapping_sub(self.allocation.index % vertices) as f32 * d[10]);
            let width = (f64::from(d[12]) * 0.5 * 1.25) as f32;
            let mut x = d[14] / 3.0_f32 + width;
            if left {
                x = -x;
            }
            (f64::from(x), y, 0.)
        };
        if !wings && !segmented {
            x /= 3.0;
        }
        self.connect_30 = [x, y, z];
        let elbow = (if wings {
            f64::from(d[13]) * 1.5
        } else {
            f64::from(d[13])
        }) + y;
        let wrist = (if wings {
            f64::from(d[2]) * 1.5
        } else {
            f64::from(d[2])
        }) + elbow;
        let NativeDpPartJoints::Arm {
            field_58,
            field_70,
            field_88,
        } = &mut self.joints
        else {
            unreachable!()
        };
        *field_58 = [x, elbow, z];
        *field_70 = [x, wrist, z];
        // Required call occurs after those actual part writes. An error cannot
        // roll them back or publish an invented hand point.
        let length = hand_length()?;
        *field_88 = [x, f64::from(length) * 0.5 + wrist, z];
        Ok(())
    }
}
