//! Actual inherited receive-stage composition, after enclosing atom/base.
use super::*;
use crate::world::{
    native_creature::{
        NativeCreatureReceive, NativeCreatureReceiveContext, NativeCreatureReceiveState,
    },
    native_sentient::{NativeOrderSeed, NativeSentientState},
};
use std::time::Instant;

/// Concrete runtime effects are required, not supplied by the body codec.
/// No default successful implementation exists for unresolved model effects.
pub trait NativePersonReceiveContext: NativeCreatureReceiveContext {
    fn order_seed(&mut self) -> io::Result<NativeOrderSeed>;
    fn monotonic_now(&mut self) -> Instant;
}

/// Observation state, not reconstructed QObject/native constructor defaults.
/// Caller must supply actual prior Person660/state1130 before masked updates.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NativePersonBodyState {
    /// Includes the real Unit receive state/stage.
    pub creature: NativeCreatureReceiveState,
    pub sentient: NativeSentientState,
    pub person: NativePersonSuffixState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[must_use = "inspect nested diagnostics; inherited body reception is not complete atom dispatch"]
pub struct NativePersonBodyReceive {
    pub creature: NativeCreatureReceive,
    pub sentient_bytes: usize,
    pub person: NativePersonReceive,
    pub consumed_bytes: usize,
}

impl NativePersonBodyState {
    /// Proven receive-visible constructor chain. Does not construct the base
    /// atom, skeletal/model objects, owner membership or signal connections.
    pub fn from_default_constructor_subset() -> Self {
        Self {
            creature: NativeCreatureReceiveState::from_default_constructor_subset(),
            sentient: NativeSentientState::from_default_constructor_subset(),
            person: NativePersonSuffixState::from_default_constructor_subset(),
        }
    }

    /// Enclosing base/atom reception must already have succeeded. All four
    /// stages share one cursor and actual retained state in native order.
    /// Earlier state/effects survive later errors; later stages never run
    /// after an earlier failure. Does not source-decode or bypass Unsupported.
    pub fn receive_qt7(
        &mut self,
        input: &mut QtReader<'_>,
        profile: FloatEncoding,
        context: &mut impl NativePersonReceiveContext,
    ) -> io::Result<NativePersonBodyReceive> {
        let length = input.remaining();
        let creature = self.creature.receive_qt7(input, profile, context)?;
        let sentient_bytes = self
            .sentient
            .receive_suffix(input, profile, || context.order_seed())?;
        let person = self
            .person
            .receive_suffix(input, profile, || context.monotonic_now())?;
        Ok(NativePersonBodyReceive {
            creature,
            sentient_bytes,
            person,
            consumed_bytes: length - input.remaining(),
        })
    }
}

#[cfg(test)]
mod tests;
