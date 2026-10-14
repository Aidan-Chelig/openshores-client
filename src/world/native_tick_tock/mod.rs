//! Native TickTock registration decisions, not scheduler execution.
//! Callbacks supply established native predicates/lookup results; no defaults
//! are inferred from source-profile entities or native base flags.
mod evaluation;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SolarSystem {
    pub id: u32,
    pub ticking_contents: bool,
    /// Native byte 0x3a9; nonzero suppresses tocker registration.
    pub field_3a9: u8,
}

pub trait Context {
    fn must_not_tick_tock(&mut self) -> bool;
    fn is_ticker(&mut self) -> bool;
    /// Raw 32-bit IsTocker result. Zero disables; remaining bits pass unchanged
    /// to UpdateOrAddTocker's unsigned argument. Units remain unestablished.
    fn is_tocker(&mut self) -> u32;
    fn find_parent_solar_system(&mut self) -> Option<SolarSystem>;
    fn is_ticking_contents(&mut self, solar: SolarSystem) -> bool;
    fn solar_field_3a9(&mut self, solar: SolarSystem) -> u8;
    fn atom_type(&mut self) -> u8;
}

/// Implemented by an owner with actual registration effects. Deliberately has
/// no default/no-op implementation. Effects can change subsequent getter values.
pub trait RegistrationContext: Context {
    fn apply_registration(&mut self, action: RegistrationAction);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegistrationAction {
    ReleaseTickTock {
        id: u32,
    },
    ReleaseTicker {
        id: u32,
    },
    ReleaseTocker {
        id: u32,
    },
    UpdateOrAddTicker {
        id: u32,
        atom_type: u8,
        solar_id: u32,
    },
    UpdateOrAddTocker {
        id: u32,
        atom_type: u8,
        solar_id: u32,
        raw_interval: u32,
    },
}

/// Evaluate the native non-instrumented path in getter/lookup order. Returns
/// ordered obligations, without claiming to mutate global registration hashes.
/// Solar predicates are read at their native evaluation points. Collecting a
/// plan does not apply effects; use apply_registrations when later getters can
/// depend on those effects. Native concurrent locks/logging remain unported.
pub fn registration_actions(id: u32, context: &mut impl Context) -> Vec<RegistrationAction> {
    let mut actions = Vec::with_capacity(2);
    evaluation::evaluate(id, context, |_, action| actions.push(action));
    actions
}

/// Apply each registration effect before evaluating the next native getter.
/// Unlike collecting a plan and replaying it, this preserves effects visible
/// to IsTocker, subsequent parent lookup, and the second AtomType call.
/// The caller must supply a real registration implementation; this function
/// does not implement global hashes, locks, or tick/tock scheduling itself.
pub fn apply_registrations(id: u32, context: &mut impl RegistrationContext) {
    evaluation::evaluate(id, context, |context, action| {
        context.apply_registration(action);
    });
}

#[cfg(test)]
mod tests;
