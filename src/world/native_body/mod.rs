//! Reconstructed body-model state, distinct from wire storage and UI previews.
//! Allocation is an initialization phase, not successful complete model creation.
mod dp;
pub use dp::{
    NativeDpAllocatedPart, NativeDpDimensions, NativeDpPartKind, native_allocate_dp_parts,
    native_dh_arm_vertices, native_dh_waist_vertex, native_dp_size_factor,
};
