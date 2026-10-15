//! Ordered body-wide skin planning, preserving native combined-cache branches.
use super::*;
use std::io;

#[derive(Clone, Debug, PartialEq)]
pub enum NativeDpSkinPlan {
    Arm(Vec<NativeDpArmSkinGeometry>),
    Leg(Vec<NativeDpLegSkinGeometry>),
    Tail(Option<NativeDpTailSkinGeometry>),
    Torso(Option<NativeDpTorsoSkinPlan>),
    Head(NativeDpHeadSkinPlan),
}

#[derive(Clone, Debug, PartialEq)]
pub struct NativeDpPartSkinPlan {
    pub part_index: usize,
    pub plan: NativeDpSkinPlan,
}

impl NativeDpInitialization {
    /// Build all part plans in allocation order, after actual OneTimeCalcs.
    /// Inputs are owner fields, not defaults inferred from DNA. Torso/head
    /// combined keys, fallback branches, tolerances and eye references remain
    /// explicit: planning does not pretend those pieces have been merged.
    pub fn skin_geometry_plans(
        &self,
        field_4068: i32,
        field_406c: i32,
        fields_4120: [f64; 4],
    ) -> io::Result<Vec<NativeDpPartSkinPlan>> {
        if !self.parts_calculated() {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "native body skin planning requires completed part calculations",
            ));
        }
        self.parts()
            .iter()
            .enumerate()
            .map(|(index, part)| {
                let plan = match part.allocation.kind {
                    NativeDpPartKind::LeftArmAndHand | NativeDpPartKind::RightArmAndHand => {
                        NativeDpSkinPlan::Arm(self.arm_skin_geometry_requests(
                            index,
                            field_4068,
                            fields_4120,
                        )?)
                    }
                    NativeDpPartKind::LeftLegAndFoot | NativeDpPartKind::RightLegAndFoot => {
                        NativeDpSkinPlan::Leg(self.leg_skin_geometry_requests(
                            index,
                            field_4068,
                            fields_4120,
                        )?)
                    }
                    NativeDpPartKind::Tail => {
                        NativeDpSkinPlan::Tail(self.tail_skin_geometry_request(index, fields_4120)?)
                    }
                    NativeDpPartKind::Torso => NativeDpSkinPlan::Torso(
                        self.torso_skin_geometry_plan(index, field_4068, fields_4120)?,
                    ),
                    NativeDpPartKind::HeadAndNeck => NativeDpSkinPlan::Head(
                        self.head_skin_geometry_plan(index, field_406c, fields_4120)?,
                    ),
                };
                Ok(NativeDpPartSkinPlan {
                    part_index: index,
                    plan,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_body_skin_planning_rejects_uninitialized_parts() {
        let body = NativeDpInitialization::begin(&NativeDhDna([0; 24]));
        assert_eq!(
            body.skin_geometry_plans(0, 0, [1.; 4]).unwrap_err().kind(),
            io::ErrorKind::Unsupported
        );
    }
}
