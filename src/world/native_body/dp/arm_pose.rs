//! Native shoulder splay/lift and Relax/StandArms composition.
use super::*;
use std::io;

impl NativeDpPartState {
    fn require_arm(&self) -> io::Result<()> {
        if matches!(
            self.allocation.kind,
            NativeDpPartKind::LeftArmAndHand | NativeDpPartKind::RightArmAndHand
        ) {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "native shoulder operation requires ArmAndHand",
            ))
        }
    }

    pub fn splay_arm_shoulder(
        &self,
        owner: &mut NativeDpBoneTransforms,
        angle: f32,
    ) -> io::Result<()> {
        self.require_arm()?;
        if angle == 0. {
            return Ok(());
        }
        let indices = self.limb_bones()?;
        let angle = if self.allocation.kind == NativeDpPartKind::LeftArmAndHand {
            -angle
        } else {
            angle
        };
        let transform = splay_pivot(self.current_connect_loc(owner)?, angle);
        // The first reference is an inlined specialized product; later ones
        // call the existing primary-only AuMatrix4x4 operator*.
        owner.slots[indices[0]] = splay_first_product(&transform, &owner.slots[indices[0]]);
        for &index in &indices[1..] {
            owner.slots[index] = transform.multiply(&owner.slots[index]);
        }
        Ok(())
    }

    /// Native LiftShoulder deliberately executes even for +/-0 angles.
    pub fn lift_arm_shoulder(
        &self,
        owner: &mut NativeDpBoneTransforms,
        angle: f32,
    ) -> io::Result<()> {
        self.require_arm()?;
        let indices = self.limb_bones()?;
        let angle = if self.allocation.kind == NativeDpPartKind::LeftArmAndHand {
            angle
        } else {
            -angle
        };
        let transform = lift_pivot(self.current_connect_loc(owner)?, angle);
        for &index in &indices[..2] {
            owner.slots[index] = transform.multiply(&owner.slots[index]);
        }
        owner.slots[indices[2]] = lift_last_product(&transform, &owner.slots[indices[2]]);
        Ok(())
    }

    /// Applies already calculated angles. Owner field30 is needed only for
    /// the first-selected-arm override; it is not a DNA/torso-angle guess.
    pub fn relax_arm(
        &self,
        owner: &mut NativeDpBoneTransforms,
        mut angles: NativeDpArmStandAngles,
        override_field_30: Option<f32>,
    ) -> io::Result<()> {
        self.require_arm()?;
        if let Some(field_30) = override_field_30 {
            angles.elbow = field_30 - angles.shoulder;
            angles.wrist = f32::from_bits(0xbd0efa35);
        }
        self.rotate_arm_shoulder(owner, angles.shoulder)?;
        self.rotate_arm_wrist(owner, angles.wrist)?;
        self.rotate_arm_elbow(owner, angles.elbow)?;
        let NativeDpPartJoints::Arm {
            lift_48, splay_4c, ..
        } = self.joints
        else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "native Relax requires arm fields48/4c",
            ));
        };
        self.splay_arm_shoulder(owner, splay_4c)?;
        self.lift_arm_shoulder(owner, lift_48)
    }
}

impl NativeDpInitialization {
    /// Arm phase only; caller owns matrix reset and preceding legs/tail/torso/
    /// heads. An optional owner field30 override is consumed by the FIRST
    /// selected ordinary arm in native forward order, never every arm.
    pub fn stand_arms(
        &mut self,
        height: f32,
        side: i32,
        override_field_30: Option<f32>,
        field_4068: i32,
        hands: &mut NativeDpHandCache<'_>,
    ) -> io::Result<()> {
        let lift = if native_dh_walks_on_hands(&self.dna) {
            f32::from_bits(0x3c8efa35)
        } else {
            f32::from_bits(0x3e567750)
        };
        self.spread_arms(f32::from_bits(0x3eb2b8c2), lift, side)?;
        let mut override_field_30 = override_field_30;
        for part in self
            .parts
            .iter()
            .skip(1)
            .filter(|p| super::spread_arms::ordinary_arm(p))
        {
            if super::spread_arms::selected(part, side) {
                let angles = part.arm_stand_angles(
                    &self.dna,
                    &self.dimensions,
                    &self.bones,
                    field_4068,
                    height,
                    hands,
                )?;
                part.relax_arm(&mut self.bones, angles, override_field_30.take())?;
            }
        }
        Ok(())
    }
}

fn splay_pivot([x, y, z]: [f64; 3], angle: f32) -> NativeDpBoneTransform {
    let (s, c) = f64::from(angle).sin_cos();
    let (xz, yz, zz) = (x * 0., y * 0., z * 0.);
    let (cz, sz) = (c * 0., s * 0.);
    let (a, b, d, e, f, g) = (cz + sz, cz - sz, c + sz, cz - s, s + cz, c - sz);
    let initial_x = ((x + yz) + zz) + 0.;
    let initial_y = ((y + xz) + zz) + 0.;
    let initial_z = (z + (yz + xz)) + 0.;
    let initial_w = ((yz + xz) + zz) + 1.;
    let tx = initial_x + ((e * -y + d * -x) - zz);
    let ty = ((f * -x + g * -y) - zz) + initial_y;
    let tz = initial_z + ((a * -x + b * -y) - z);
    let tw = initial_w + ((a * -x + b * -y) - zz);
    NativeDpBoneTransform {
        matrix_00: [d, f, a, a, e, g, b, b, 0., 0., 1., 0., tx, ty, tz, tw],
        field_100: 0,
    }
}

fn lift_pivot([x, y, z]: [f64; 3], angle: f32) -> NativeDpBoneTransform {
    let (s, c) = f64::from(angle).sin_cos();
    let (xz, yz, zz) = (x * 0., y * 0., z * 0.);
    let (cz, nsz) = (c * 0., -s * 0.);
    let (a, b, d, e, f, g) = (nsz + cz, cz - nsz, c + nsz, cz - s, s + cz, c - nsz);
    let initial_x = ((x + yz) + zz) + 0.;
    let initial_y = ((y + xz) + zz) + 0.;
    let initial_z = (z + (yz + xz)) + 0.;
    let initial_w = (zz + (yz + xz)) + 1.;
    let tx = ((d * -x - yz) + f * -z) + initial_x;
    let ty = initial_y + ((a * -x - y) + b * -z);
    let tz = ((e * -x - yz) + g * -z) + initial_z;
    let tw = ((a * -x - yz) + b * -z) + initial_w;
    NativeDpBoneTransform {
        matrix_00: [d, a, e, a, 0., 1., 0., 0., f, b, g, b, tx, ty, tz, tw],
        field_100: 0,
    }
}

fn splay_first_product(
    left: &NativeDpBoneTransform,
    right: &NativeDpBoneTransform,
) -> NativeDpBoneTransform {
    let a = &left.matrix_00;
    let b = &right.matrix_00;
    let mut out = left.multiply(right);
    for col in 0..4 {
        let i = col * 4;
        // Native removes the constant 1*bZ term and adds bZ first.
        out.matrix_00[i + 2] = (b[i + 2] + (b[i] * a[2] + b[i + 1] * a[6])) + b[i + 3] * a[14];
    }
    out
}

// Native SIMD lane groups retain different addition operand orders.
#[allow(clippy::if_same_then_else)]
fn lift_last_product(
    left: &NativeDpBoneTransform,
    right: &NativeDpBoneTransform,
) -> NativeDpBoneTransform {
    let a = &left.matrix_00;
    let b = &right.matrix_00;
    let mut out = left.multiply(right);
    for row in 0..4 {
        if row >= 2 {
            out.matrix_00[row] =
                (a[8 + row] * b[2] + (a[row] * b[0] + a[4 + row] * b[1])) + a[12 + row] * b[3];
        }
        let sum = a[row] * b[4] + a[4 + row] * b[5];
        out.matrix_00[4 + row] = (a[8 + row] * b[6] + sum) + a[12 + row] * b[7];
        let sum = a[4 + row] * b[13] + a[row] * b[12];
        out.matrix_00[12 + row] = if row < 2 {
            (sum + a[8 + row] * b[14]) + a[12 + row] * b[15]
        } else {
            (a[8 + row] * b[14] + sum) + a[12 + row] * b[15]
        };
    }
    out
}

#[cfg(test)]
mod tests;
