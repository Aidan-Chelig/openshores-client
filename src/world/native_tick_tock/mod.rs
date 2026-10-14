//! Native TickTock registration decisions, not scheduler execution.
//! Callbacks supply established native predicates/lookup results; no defaults
//! are inferred from source-profile entities or native base flags.

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
    fn atom_type(&mut self) -> u8;
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
/// Solar predicates are caller snapshots: concurrent lock interleaving and the
/// optional native slow-call logging branch are not reconstructed here.
pub fn registration_actions(id: u32, context: &mut impl Context) -> Vec<RegistrationAction> {
    use RegistrationAction::*;
    if context.must_not_tick_tock() {
        return vec![ReleaseTickTock { id }];
    }
    let mut actions = Vec::with_capacity(2);
    let mut solar = None;
    if context.is_ticker() {
        solar = context.find_parent_solar_system();
        if let Some(system) = solar.filter(|system| system.ticking_contents) {
            actions.push(UpdateOrAddTicker {
                id,
                atom_type: context.atom_type(),
                solar_id: system.id,
            });
        } else {
            actions.push(ReleaseTicker { id });
        }
    } else {
        actions.push(ReleaseTicker { id });
    }
    let raw_interval = context.is_tocker();
    if raw_interval == 0 {
        actions.push(ReleaseTocker { id });
        return actions;
    }
    // A failed ticker lookup is retried for the tocker path; a resolved solar
    // system is reused even when IsTickingContents was false.
    let solar = solar.or_else(|| context.find_parent_solar_system());
    if let Some(system) = solar.filter(|system| system.field_3a9 == 0) {
        actions.push(UpdateOrAddTocker {
            id,
            atom_type: context.atom_type(),
            solar_id: system.id,
            raw_interval,
        });
    } else {
        actions.push(ReleaseTocker { id });
    }
    actions
}

#[cfg(test)]
mod tests;
