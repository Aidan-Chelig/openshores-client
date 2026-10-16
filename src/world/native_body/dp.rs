//! DpBody::Init's ordered part allocation and per-part DNA copies.
//! REA ev_5ac6fc428ff8926b29782cbbd39115ee7f7c4e15713693bb2e110319920e20c6;
//! corroborating assembly: docs/evidence/native-dp-initialization-gnu.txt.
use crate::world::native_dna::NativeDhDna;
mod bones;
pub use bones::{NativeDpBoneTransform, NativeDpBoneTransforms, NativeDpSkinnedVertex};
mod dimensions;
pub use dimensions::NativeDpDimensions;
mod layout;
pub use layout::{
    native_dh_arm_vertices, native_dh_has_wrist_bend, native_dh_leg_vertices,
    native_dh_waist_vertex, native_dh_walks_on_hands, native_dp_size_factor,
};
mod model_dna;
pub use model_dna::NativeDpModelDna;
mod arm_skin;
mod hand_resource;
pub use arm_skin::NativeDpArmSkinGeometry;
mod leg_skin;
pub use hand_resource::NativeDpHandResource;
pub use leg_skin::NativeDpLegSkinGeometry;
mod tail_skin;
pub use tail_skin::NativeDpTailSkinGeometry;
mod torso_segment;
pub use torso_segment::native_dp_torso_segment;
mod torso_skin;
pub use torso_skin::{NativeDpTorsoSkinPiece, NativeDpTorsoSkinPlan};
mod head_skin;
mod skin_plan;
pub use head_skin::{NativeDpHeadSkinPiece, NativeDpHeadSkinPlan};
pub use skin_plan::{NativeDpPartSkinPlan, NativeDpSkinPlan};
mod hand_geometry;
pub use hand_geometry::{NativeDpFootGeometry, NativeDpHandGeometry};
mod arm_elbow;
mod arm_locomotion;
mod arm_move;
mod arm_pose;
pub use arm_locomotion::NativeDpArmLocomotion;
mod locomotion_pose;
mod neutral_pose;
pub use locomotion_pose::NativeDpLocomotionSample;
pub use neutral_pose::NativeDpStandingPose;
mod arm_shoulder;
mod arm_standing;
mod arm_wrist;
mod body_waist;
mod foot;
mod height;
mod leg_locomotion;
mod leg_pose;
pub use leg_locomotion::{NativeDpGait, NativeDpLegLocomotion, NativeDpLegLocomotionAngles};
mod leg_standing;
mod spread_arms;
pub use arm_standing::NativeDpArmStandAngles;
mod position_tail;
mod position_torso;
mod spread_legs;
mod standing;
pub use leg_standing::NativeDpLegStandAngles;
mod tail_bend;
mod tail_fold;
mod tail_motion;
mod tail_slither;
mod tail_slither_pose;
mod tail_special;
mod torso_base;
mod waist;
mod wing_bow;
pub use foot::{NativeDpFootCache, NativeDpFootResource, native_dh_has_ankle_bend};
mod hand_cache;
pub use hand_cache::NativeDpHandCache;
mod connection;
mod head_pose;
mod initialization;
mod neck;
mod neck_bend;
mod position_heads;
pub use initialization::{NativeDpInitialization, NativeDpPartJoints, NativeDpPartState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeDpPartKind {
    Torso,
    Tail,
    HeadAndNeck,
    LeftArmAndHand,
    RightArmAndHand,
    LeftLegAndFoot,
    RightLegAndFoot,
}

/// Actual initialized part identity/DNA before body and virtual part
/// OneTimeCalcs. No geometry, volume, pose or mounted resource is fabricated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeDpAllocatedPart {
    pub kind: NativeDpPartKind,
    /// Head field48, arm field50 or leg field4c; zero for torso/tail.
    pub index: u32,
    pub dna: NativeDhDna,
}

/// Allocation phase only. A production body constructor must still initialize
/// bone matrices, body OneTimeCalcs and each part's virtual10 OneTimeCalcs;
/// returning these records is not a substitute for those required operations.
#[must_use]
pub fn native_allocate_dp_parts(dna: &NativeDhDna) -> Vec<NativeDpAllocatedPart> {
    use NativeDpPartKind::*;
    let mut parts = Vec::new();
    let mut append = |kind, index, dna: &NativeDhDna| {
        parts.push(NativeDpAllocatedPart {
            kind,
            index,
            dna: dna.clone(),
        });
    };
    append(Torso, 0, dna);
    if dna.0[14] & 0x30 != 0 {
        append(Tail, 0, dna);
    }
    for index in 0..=u32::from((dna.0[5] >> 1) & 1) {
        append(HeadAndNeck, index, dna);
    }
    let arm_pairs = u32::from(dna.0[1] & 3);
    for index in 0..arm_pairs {
        let mut arm_dna = dna.clone();
        if arm_pairs > 1 && dna.0[2] & 6 != 0 && index < arm_pairs - 1 {
            arm_dna.0[2] &= 0xf8;
        }
        append(LeftArmAndHand, index, &arm_dna);
        append(RightArmAndHand, index, &arm_dna);
    }
    for index in 0..u32::from((dna.0[12] >> 2) & 3) {
        append(LeftLegAndFoot, index, dna);
        append(RightLegAndFoot, index, dna);
    }
    parts
}

#[cfg(test)]
mod tests;
