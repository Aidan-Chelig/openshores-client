//! Reconstructed body-model state, distinct from wire storage and UI previews.
//! Allocation is an initialization phase, not successful complete model creation.
mod dp;
pub use dp::NativeDpArmSkinGeometry;
pub use dp::NativeDpLegSkinGeometry;
pub use dp::NativeDpTailSkinGeometry;
pub use dp::native_dp_torso_segment;
pub use dp::{
    NativeDpAllocatedPart, NativeDpDimensions, NativeDpHandCache, NativeDpHandGeometry,
    NativeDpHandResource, NativeDpModelDna, NativeDpPartKind, native_allocate_dp_parts,
    native_dh_arm_vertices, native_dh_waist_vertex, native_dp_size_factor,
};
pub use dp::{NativeDpBoneTransform, NativeDpBoneTransforms};
pub use dp::{
    NativeDpFootCache, NativeDpFootGeometry, NativeDpFootResource, native_dh_has_ankle_bend,
};
pub use dp::{NativeDpHeadSkinPiece, NativeDpHeadSkinPlan};
pub use dp::{NativeDpInitialization, NativeDpPartJoints, NativeDpPartState};
pub use dp::{NativeDpPartSkinPlan, NativeDpSkinPlan};
pub use dp::{NativeDpTorsoSkinPiece, NativeDpTorsoSkinPlan};
