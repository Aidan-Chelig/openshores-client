use super::RegistrationStore;
use crate::world::native_tick_tock::{
    Context, FallibleRegistrationContext, RegistrationAction, SolarSystem, try_apply_registrations,
};
use std::io;

/// Native predicates/lookup supplied by the eventual world owner. Every getter
/// receives current registration storage, including earlier inline effects.
/// No predicate is guessed from source-profile entities or registration presence.
pub trait StoreContext {
    fn must_not_tick_tock(&mut self, store: &RegistrationStore) -> bool;
    fn is_ticker(&mut self, store: &RegistrationStore) -> bool;
    fn is_tocker(&mut self, store: &RegistrationStore) -> u32;
    fn find_parent_solar_system(&mut self, store: &RegistrationStore) -> Option<SolarSystem>;
    fn is_ticking_contents(&mut self, store: &RegistrationStore, solar: SolarSystem) -> bool;
    fn solar_field_3a9(&mut self, store: &RegistrationStore, solar: SolarSystem) -> u8;
    fn atom_type(&mut self, store: &RegistrationStore) -> u8;
}

struct Owner<'a, C> {
    store: &'a mut RegistrationStore,
    context: &'a mut C,
}
impl<C: StoreContext> Context for Owner<'_, C> {
    fn must_not_tick_tock(&mut self) -> bool {
        self.context.must_not_tick_tock(self.store)
    }
    fn is_ticker(&mut self) -> bool {
        self.context.is_ticker(self.store)
    }
    fn is_tocker(&mut self) -> u32 {
        self.context.is_tocker(self.store)
    }
    fn find_parent_solar_system(&mut self) -> Option<SolarSystem> {
        self.context.find_parent_solar_system(self.store)
    }
    fn is_ticking_contents(&mut self, solar: SolarSystem) -> bool {
        self.context.is_ticking_contents(self.store, solar)
    }
    fn solar_field_3a9(&mut self, solar: SolarSystem) -> u8 {
        self.context.solar_field_3a9(self.store, solar)
    }
    fn atom_type(&mut self) -> u8 {
        self.context.atom_type(self.store)
    }
}
impl<C: StoreContext> FallibleRegistrationContext for Owner<'_, C> {
    type Error = io::Error;
    fn try_apply_registration(&mut self, action: RegistrationAction) -> io::Result<()> {
        self.store.apply(action)
    }
}

impl RegistrationStore {
    /// Execute the recovered registration decision path directly against this
    /// store. Safety failures stop evaluation; earlier effects are not rolled
    /// back. Native world predicates and scheduler execution remain separate.
    pub fn apply_tick_tock(&mut self, id: u32, context: &mut impl StoreContext) -> io::Result<()> {
        try_apply_registrations(
            id,
            &mut Owner {
                store: self,
                context,
            },
        )
    }
}

#[cfg(test)]
#[path = "integration_tests.rs"]
mod tests;
