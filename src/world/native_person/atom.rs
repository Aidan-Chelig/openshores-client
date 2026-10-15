//! Concrete receive storage; owner/dispatch and model context are separate.
use super::{NativePersonBodyReceive, NativePersonBodyState, NativePersonReceiveContext};
use crate::{
    atom_position::{FloatEncoding, PackedPosition},
    protocol::QtReader,
    world::{
        native_atom::NativeBaseAtomInitialization,
        native_dispatch::base::{
            NativeBaseReceiveAdapter, NativeBaseReceiveFields, NativeBaseReceiveOutcome,
            receive_base,
        },
    },
};
use std::io;

#[derive(Clone, Debug)]
pub struct NativePersonAtom {
    pub base: NativeBaseAtomInitialization,
    pub received: NativeBaseReceiveFields,
    pub body: NativePersonBodyState,
}

impl NativePersonAtom {
    /// Compose proven base and inherited receive-visible constructor subsets.
    /// Does not silently allocate/attach runtime models or an owner graph.
    pub fn from_default_person_constructor_subset() -> Self {
        Self {
            base: NativeBaseAtomInitialization::from_default_base_constructor(),
            received: NativeBaseReceiveFields::default(),
            body: NativePersonBodyState::from_default_constructor_subset(),
        }
    }

    /// Dispatch must select this object and apply identity/time first. Actual
    /// ChangePos/owner work is required from the callback, not bypassed here.
    pub fn receive_base(
        &mut self,
        input: &mut QtReader<'_>,
        profile: FloatEncoding,
        position: impl FnMut(&mut NativeBaseAtomInitialization, PackedPosition) -> io::Result<()>,
        clock: impl FnMut() -> (i64, i64),
    ) -> io::Result<NativeBaseReceiveOutcome> {
        receive_base(
            input,
            profile,
            &mut NativeBaseReceiveAdapter {
                atom: &mut self.base,
                fields: &mut self.received,
                position,
                clock,
            },
        )
    }

    /// Only after all enclosing base layers complete. Concrete runtime context
    /// must perform model/console effects or explicitly report Unsupported.
    pub fn receive_body(
        &mut self,
        input: &mut QtReader<'_>,
        profile: FloatEncoding,
        context: &mut impl NativePersonReceiveContext,
    ) -> io::Result<NativePersonBodyReceive> {
        self.body.receive_qt7(input, profile, context)
    }
}
