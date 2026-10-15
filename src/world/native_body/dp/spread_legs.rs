//! DpBody::SpreadLegs and StandLegs. No guessed attachment offsets.
use super::*;
use std::io;

fn is_leg(part: &NativeDpPartState) -> bool {
    matches!(
        part.allocation.kind,
        NativeDpPartKind::LeftLegAndFoot | NativeDpPartKind::RightLegAndFoot
    )
}

fn selected(part: &NativeDpPartState, selector: i32) -> bool {
    match selector {
        0 => part.allocation.index != 0,
        1 => part.allocation.index == 0,
        2 => true,
        _ => false,
    }
}

impl NativeDpInitialization {
    /// Groups every leg by its exact, unposed ConnectLoc, then updates only
    /// the selected pairs. Skipped parts retain their previous field48.
    pub fn spread_legs(
        &mut self,
        angle: f32,
        selector: i32,
        field_4068: i32,
        feet: &mut NativeDpFootCache<'_>,
    ) -> io::Result<()> {
        let groups = connection_groups(&self.parts)?;
        // Both owner measurements happen even if there are no matching groups.
        let height = feet.standing_height(&self.dna, &self.dimensions, field_4068, false)?;
        let foot = feet.foot_length(&self.dna, field_4068, false)?;
        apply_spread(
            &mut self.parts,
            &self.dimensions,
            &groups,
            angle,
            selector,
            height,
            foot,
        )
    }

    /// This is the leg phase only: no matrix reset, body translation, torso,
    /// head or arm pose is implied. Earlier writes survive a resource error.
    pub fn stand_legs(
        &mut self,
        height: f32,
        selector: i32,
        field_4068: i32,
        feet: &mut NativeDpFootCache<'_>,
    ) -> io::Result<()> {
        // Literal 1422bfc94, not a separately rounded radians conversion.
        self.spread_legs(f32::from_bits(0x3eb2b8c2), selector, field_4068, feet)?;
        // StandLegs' end sentinel is begin()+1 (unlike SpreadLegs' begin()).
        for part in self.parts.iter().skip(2).rev().take_while(|p| is_leg(p)) {
            if selected(part, selector) {
                // Native CalcStandAngles resolves concrete Bones first.
                part.limb_bones()?;
                let angles = feet.leg_stand_angles(
                    &self.dna,
                    &part.allocation.dna,
                    &self.dimensions,
                    field_4068,
                    height,
                )?;
                part.stand_leg(&mut self.bones, angles)?;
            }
        }
        Ok(())
    }
}

fn connection_groups(parts: &[NativeDpPartState]) -> io::Result<Vec<([f64; 3], u32)>> {
    let mut groups: Vec<([f64; 3], u32)> = Vec::new();
    for part in parts.iter().skip(1).rev().take_while(|p| is_leg(p)) {
        if !part.connect_30.iter().all(|v| v.is_finite()) {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "nonfinite native leg attachment map key",
            ));
        }
        if let Some((_, count)) = groups
            .iter_mut()
            .find(|(point, _)| *point == part.connect_30)
        {
            *count += 1;
        } else {
            groups.push((part.connect_30, 1));
        }
    }
    // QMap<AuPoint,int> ordering: Z, then Y, then X. Signed zeros compare equal.
    groups.sort_by(|(a, _), (b, _)| {
        a[2].partial_cmp(&b[2])
            .unwrap()
            .then(a[1].partial_cmp(&b[1]).unwrap())
            .then(a[0].partial_cmp(&b[0]).unwrap())
    });
    Ok(groups)
}

#[allow(clippy::too_many_arguments)]
fn apply_spread(
    parts: &mut [NativeDpPartState],
    dimensions: &NativeDpDimensions,
    groups: &[([f64; 3], u32)],
    angle: f32,
    selector: i32,
    height: f32,
    foot: f32,
) -> io::Result<()> {
    if ![angle, height, foot].iter().all(|v| v.is_finite()) {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "nonfinite native leg spread input",
        ));
    }
    let d = dimensions.fields_4080_through_40b8();
    let pi = f32::from_bits(0x40490fdb);
    for &(point, count) in groups {
        let delta = if angle == 0. {
            pi / (count + 1) as f32
        } else {
            angle
        };
        let mut current = 0_f32;
        for part in parts.iter_mut().skip(1).rev().take_while(|p| is_leg(p)) {
            if part.connect_30 != point || !selected(part, selector) {
                continue;
            }
            let dna = &part.allocation.dna;
            let ankle = native_dh_has_ankle_bend(dna);
            let extension = if ankle {
                d[0] / (foot / d[0]).atan().cos()
            } else {
                foot + d[0]
            };
            let triangular = !super::layout::native_dh_is_exoskeletal(dna)
                && u32::from((dna.0[12] >> 2) & 3) <= native_dh_leg_vertices(dna)
                && (!ankle || foot <= height)
                && height < d[7] + extension
                && d[7] < height + extension
                && extension < height + d[7];
            current += if triangular {
                delta
            } else {
                pi / (count + 1) as f32
            };
            let NativeDpPartJoints::Leg { splay_48, .. } = &mut part.joints else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "native SpreadLegs requires leg field48",
                ));
            };
            *splay_48 = current;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
