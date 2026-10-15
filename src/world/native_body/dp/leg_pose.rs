//! Concrete native leg rotations, preserving Bones out-reference order.
use super::*;
use std::io;

impl NativeDpPartState {
    fn require_leg(&self) -> io::Result<()> {
        if matches!(
            self.allocation.kind,
            NativeDpPartKind::LeftLegAndFoot | NativeDpPartKind::RightLegAndFoot
        ) {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "native leg rotation requires LegAndFoot",
            ))
        }
    }

    pub fn rotate_leg_hip(&self, owner: &mut NativeDpBoneTransforms, angle: f32) -> io::Result<()> {
        self.require_leg()?;
        if angle == 0. {
            return Ok(());
        }
        let indices = self.limb_bones()?;
        // Same expanded pivot as RotateShoulder; unlike that arm consumer,
        // this always updates all three reference outputs.
        let rotation = super::arm_shoulder::shoulder_pivot(self.current_connect_loc(owner)?, angle);
        for index in indices {
            owner.slots[index] = rotation.multiply(&owner.slots[index]);
        }
        Ok(())
    }

    pub fn rotate_leg_knee(
        &self,
        owner: &mut NativeDpBoneTransforms,
        angle: f32,
    ) -> io::Result<()> {
        self.require_leg()?;
        if angle == 0. {
            return Ok(());
        }
        let indices = self.limb_bones()?;
        let NativeDpPartJoints::Leg { field_50, .. } = self.joints else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "native knee requires leg field50",
            ));
        };
        let pivot = super::arm_wrist::wrist_project(&owner.slots[indices[1]], field_50);
        let rotation = super::arm_wrist::wrist_pivot(pivot, angle);
        for &index in &indices[1..] {
            owner.slots[index] = rotation.multiply(&owner.slots[index]);
        }
        Ok(())
    }

    pub fn rotate_leg_ankle(
        &self,
        owner: &mut NativeDpBoneTransforms,
        angle: f32,
    ) -> io::Result<()> {
        self.require_leg()?;
        if angle == 0. {
            return Ok(());
        }
        let index = self.limb_bones()?[2];
        let NativeDpPartJoints::Leg { field_68, .. } = self.joints else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "native ankle requires leg field68",
            ));
        };
        let pivot = super::arm_wrist::wrist_project(&owner.slots[index], field_68);
        let rotation = super::arm_wrist::wrist_pivot(pivot, angle);
        owner.slots[index] = rotation.multiply(&owner.slots[index]);
        Ok(())
    }

    pub fn splay_leg_hip(&self, owner: &mut NativeDpBoneTransforms, angle: f32) -> io::Result<()> {
        self.require_leg()?;
        if angle == 0. {
            return Ok(());
        }
        let indices = self.limb_bones()?;
        let angle = if self.allocation.kind == NativeDpPartKind::LeftLegAndFoot {
            angle
        } else {
            -angle
        };
        let rotation = splay_pivot(self.current_connect_loc(owner)?, angle);
        for index in indices {
            owner.slots[index] = rotation.multiply(&owner.slots[index]);
        }
        Ok(())
    }

    /// Stand's order is hip, knee, conditional ankle, then stored splay.
    pub fn stand_leg(
        &self,
        owner: &mut NativeDpBoneTransforms,
        angles: NativeDpLegStandAngles,
    ) -> io::Result<()> {
        self.require_leg()?;
        self.rotate_leg_hip(owner, angles.hip)?;
        self.rotate_leg_knee(owner, angles.knee)?;
        if native_dh_has_ankle_bend(&self.allocation.dna) {
            self.rotate_leg_ankle(owner, angles.ankle)?;
        }
        let NativeDpPartJoints::Leg { splay_48, .. } = self.joints else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "native Stand requires leg field48",
            ));
        };
        self.splay_leg_hip(owner, splay_48)
    }
}

fn splay_pivot([x, y, z]: [f64; 3], angle: f32) -> NativeDpBoneTransform {
    let (s, c) = f64::from(angle).sin_cos();
    let (xz, yz, zz) = (x * 0., y * 0., z * 0.);
    let (cz, sz) = (c * 0., s * 0.);
    let a = cz + sz;
    let b = cz - sz;
    let d = c + sz;
    let e = cz - s;
    let f = s + cz;
    let g = c - sz;
    let initial_x = ((x + yz) + zz) + 0.;
    let initial_y = ((y + xz) + zz) + 0.;
    let initial_z = (z + (yz + xz)) + 0.;
    let initial_w = ((yz + xz) + zz) + 1.;
    let tx = ((d * -x + e * -y) - zz) + initial_x;
    let ty = ((f * -x + g * -y) - zz) + initial_y;
    let tz = initial_z + ((a * -x + b * -y) - z);
    let tw = initial_w + ((a * -x + b * -y) - zz);
    NativeDpBoneTransform {
        matrix_00: [d, f, a, a, e, g, b, b, 0., 0., 1., 0., tx, ty, tz, tw],
        field_100: 0,
    }
}

#[cfg(test)]
mod tests;
