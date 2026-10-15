//! DpBody::SpreadArms, forward scan and side selection, not leg-pair selection.
use super::*;
use std::io;

pub(super) fn ordinary_arm(part: &NativeDpPartState) -> bool {
    matches!(
        part.allocation.kind,
        NativeDpPartKind::LeftArmAndHand | NativeDpPartKind::RightArmAndHand
    ) && (part.allocation.dna.0[1] & 3 == 0 || part.allocation.dna.0[2] & 6 == 0)
}

pub(super) fn selected(part: &NativeDpPartState, side: i32) -> bool {
    match side {
        0 => part.allocation.kind == NativeDpPartKind::LeftArmAndHand,
        1 => part.allocation.kind == NativeDpPartKind::RightArmAndHand,
        2 => true,
        _ => false,
    }
}

impl NativeDpInitialization {
    /// Only changes native arm fields48/4c, never attachment geometry or bones.
    /// Excluded sides still consume spacing increments within their group.
    pub fn spread_arms(&mut self, angle: f32, lift: f32, side: i32) -> io::Result<()> {
        if ![angle, lift].iter().all(|x| x.is_finite()) {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "nonfinite native arm spread input",
            ));
        }
        let mut groups = Vec::new();
        for part in self.parts.iter().skip(1).filter(|p| ordinary_arm(p)) {
            if !part.connect_30.iter().all(|x| x.is_finite()) {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "nonfinite native arm attachment map key",
                ));
            }
            if !groups.contains(&part.connect_30) {
                groups.push(part.connect_30);
            }
        }
        // Same QMap<AuPoint,int> Z/Y/X ordering as leg spacing. Counts have no
        // consumers in this arm routine; no count-based angular division.
        groups.sort_by(|a, b| {
            a[2].partial_cmp(&b[2])
                .unwrap()
                .then(a[1].partial_cmp(&b[1]).unwrap())
                .then(a[0].partial_cmp(&b[0]).unwrap())
        });
        for point in groups {
            let mut current_lift = (lift * 3_f32) * 0.25_f32;
            let mut current_splay = 0_f32;
            for part in self.parts.iter_mut().skip(1).filter(|p| ordinary_arm(p)) {
                if part.connect_30 != point {
                    continue;
                }
                current_splay += angle;
                current_lift += lift * 0.25_f32;
                if selected(part, side) {
                    let NativeDpPartJoints::Arm {
                        lift_48, splay_4c, ..
                    } = &mut part.joints
                    else {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "native SpreadArms requires arm fields48/4c",
                        ));
                    };
                    *lift_48 = current_lift;
                    *splay_4c = current_splay;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
