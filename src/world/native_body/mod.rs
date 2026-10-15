//! Reconstructed body-model state, distinct from wire storage and UI previews.
//! Allocation is an initialization phase, not successful complete model creation.
mod dp;
pub use dp::{NativeDpAllocatedPart, NativeDpPartKind, native_allocate_dp_parts};
