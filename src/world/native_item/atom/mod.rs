//! Concrete established Item receive state, not a full native world owner.
use super::{NativeItemBody, NativeItemInitialization, NativeItemState};
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
pub struct NativeItemAtom {
    pub base: NativeBaseAtomInitialization,
    pub received: NativeBaseReceiveFields,
    pub item: NativeItemState,
}
impl NativeItemAtom {
    /// Only the confirmed Item constructor subset. Unestablished scalar base
    /// defaults remain unobserved; no automatic owner/map registration.
    pub fn from_default_item_constructor() -> Self {
        let initial = NativeItemInitialization::from_default_item_constructor();
        Self {
            base: initial.base,
            received: NativeBaseReceiveFields::default(),
            item: initial.state,
        }
    }
    pub fn receive_base(
        &mut self,
        input: &mut QtReader<'_>,
        position: impl FnMut(&mut NativeBaseAtomInitialization, PackedPosition) -> io::Result<()>,
        clock: impl FnMut() -> (i64, i64),
    ) -> io::Result<NativeBaseReceiveOutcome> {
        receive_base(
            input,
            FloatEncoding::TypeWidths,
            &mut NativeBaseReceiveAdapter {
                atom: &mut self.base,
                fields: &mut self.received,
                position,
                clock,
            },
        )
    }
    /// Known Item body after all base layers. None explicitly requires an
    /// unsupported-subtype handler; it is not successful native reception and
    /// does not emulate native old-value deletion/null factory behavior.
    /// Known bodies update this object's sparse Item state and retain suffixes.
    pub fn receive_known_body(&mut self, input: &mut QtReader<'_>) -> io::Result<Option<usize>> {
        let Some(body) = NativeItemBody::read(input)? else {
            return Ok(None);
        };
        let consumed = body.consumed_bytes;
        self.item.apply(body);
        Ok(Some(consumed))
    }
}

#[cfg(test)]
mod tests;
