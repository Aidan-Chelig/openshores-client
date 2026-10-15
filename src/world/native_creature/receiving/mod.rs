use super::NativeCreaturePose;
use crate::{
    atom_position::FloatEncoding,
    protocol::QtReader,
    world::{
        native_dna::{NativeAuDna, NativeDhDna, NativeDna},
        native_item::NativeItemValue,
        native_unit::{NativeUnitReceive, NativeUnitState},
    },
};
use std::io;
mod context;
pub use context::NativeCreatureReceiveContext;
mod receive;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeCreatureBodyKind {
    DpBody,
    Animalia,
}

/// Receive-visible subset of a selected body, not its skeletal/model object.
/// HP/time zero are established by both default constructors; DNA None means
/// no received DNA, not a claim about constructor-internal DNA bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeCreatureBodyReceiveState {
    pub kind: NativeCreatureBodyKind,
    pub hit_points: i16,
    pub timestamp_ms: i64,
    pub received_dna: Option<NativeDna>,
    pub received_configuration: Option<u8>,
    /// DpBody byte3a and byte64. Animalia applies its real setters in context.
    pub dp_configuration: Option<[u8; 2]>,
}

impl NativeCreatureBodyReceiveState {
    fn selected(kind: NativeCreatureBodyKind) -> Self {
        Self {
            kind,
            hit_points: 0,
            timestamp_ms: 0,
            received_dna: None,
            received_configuration: None,
            dp_configuration: (kind == NativeCreatureBodyKind::DpBody).then_some([0, 0]),
        }
    }
}

/// Observed receive state. Default is deliberately NOT a DaCreature constructor:
/// body/options are unobserved until received. Concrete native object creation
/// must seed the inherited defaults and construct its real body/context.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NativeCreatureReceiveState {
    pub unit: NativeUnitState,
    pub body: Option<NativeCreatureBodyReceiveState>,
    pub scalar_1e0: Option<f32>,
    pub field_496: Option<u8>,
    pub field_494: Option<u8>,
    pub field_495: Option<u8>,
    pub item_698: Option<NativeItemValue>,
    pub field_5b8: Option<u32>,
    pub field_5bc: Option<u32>,
}

impl NativeCreatureReceiveState {
    /// Receive-visible DaCreature constructor subset. A real runtime context
    /// must separately construct the DpBody model; this is its observation state.
    /// Virtual scalar setter state is not inferred from unrelated scalar writes.
    pub fn from_default_constructor_subset() -> Self {
        Self {
            unit: NativeUnitState::from_default_constructor_subset(),
            body: Some(NativeCreatureBodyReceiveState::selected(
                NativeCreatureBodyKind::DpBody,
            )),
            field_496: Some(0),
            field_494: Some(0),
            field_495: Some(0),
            field_5b8: Some(4),
            field_5bc: Some(4),
            ..Default::default()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeCreatureReceive {
    pub unit: NativeUnitReceive,
    pub flags: u16,
    pub consumed_bytes: usize,
}

#[cfg(test)]
mod tests;
