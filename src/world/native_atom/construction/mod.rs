//! Established base constructor fields, not a complete derived atom allocation.
use crate::atom_position::NativePositionState;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeBaseAtomInitialization {
    pub wire_id: u32,
    /// Native offset 0x1c; constructor writes 5. Meaning remains unestablished.
    pub field_1c: i32,
    /// Explicit null owner at construction; no hierarchy membership is inferred.
    pub owner_attached: bool,
    /// Native AuTime storage at 0x1e8 starts at zero, not a local clock epoch.
    pub received_time: i64,
    pub position: NativePositionState,
    /// Initialized even while the environment-cache valid flag is false.
    pub environment_matrix: [[f64; 4]; 4],
}
impl NativeBaseAtomInitialization {
    /// Recovered DaAtom() subset used by position and owner lifecycle. Does not
    /// pick a derived kind, assign a packet ID, create owner lists or register
    /// scheduling. Qt locks/timers and unmodeled base fields remain separate.
    pub fn from_default_base_constructor() -> Self {
        let mut position = NativePositionState::default();
        position.orientation = Some([0.; 3]);
        Self {
            wire_id: 0,
            field_1c: 5,
            owner_attached: false,
            received_time: 0,
            environment_matrix: position.position.local,
            position,
        }
    }
}

#[cfg(test)]
mod tests;
