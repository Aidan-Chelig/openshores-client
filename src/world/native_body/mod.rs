//! Reconstructed body-model state, distinct from wire storage and UI previews.
//! Allocation is an initialization phase, not successful complete model creation.
mod dp;
mod textures;
pub use dp::NativeDpArmSkinGeometry;
pub use dp::NativeDpArmStandAngles;
pub use dp::NativeDpLegSkinGeometry;
pub use dp::NativeDpLegStandAngles;
pub use dp::NativeDpTailSkinGeometry;
pub use dp::native_dp_torso_segment;
pub use dp::{
    NativeDpAllocatedPart, NativeDpDimensions, NativeDpHandCache, NativeDpHandGeometry,
    NativeDpHandResource, NativeDpModelDna, NativeDpPartKind, native_allocate_dp_parts,
    native_dh_arm_vertices, native_dh_leg_vertices, native_dh_waist_vertex, native_dp_size_factor,
};
pub use dp::{NativeDpBoneTransform, NativeDpBoneTransforms, NativeDpSkinnedVertex};
pub use dp::{
    NativeDpFootCache, NativeDpFootGeometry, NativeDpFootResource, native_dh_has_ankle_bend,
};
pub use dp::{NativeDpHeadSkinPiece, NativeDpHeadSkinPlan};
pub use dp::{NativeDpInitialization, NativeDpPartJoints, NativeDpPartState};
pub use dp::{NativeDpPartSkinPlan, NativeDpSkinPlan};
pub use dp::{NativeDpTorsoSkinPiece, NativeDpTorsoSkinPlan};
pub use textures::{NativeDpSkinAtlas, native_dp_skin_offsets};
