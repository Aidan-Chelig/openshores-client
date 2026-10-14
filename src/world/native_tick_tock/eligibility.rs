//! MustNotTickTock over explicit, caller-established native observations.
//! This is a stable snapshot evaluator, not the native locking/lookup routine.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Eligibility {
    /// Raw atom field 0x1c; meaning and wire source remain unestablished.
    pub field_1c: i32,
    pub has_owner: bool,
    pub parent_id: u32,
    /// None is a failed Parent lookup, not a missing packed parent field.
    pub parent_field_1c: Option<i32>,
    /// Virtual slot 0x668 (DoesQueryEnvironment).
    pub does_query_environment: bool,
    /// None means atom query pointer at 0x98 is null. Some is the established
    /// query field at offset 0x34, not a fabricated scene/base flag mapping.
    pub environment_field_34: Option<u32>,
    /// None means parent FindParentCapsuleInEnv returned null. Some is the
    /// resolved capsule's virtual slot 0xb88 result. Overrides remain separate.
    pub capsule_slot_b88: Option<bool>,
}

impl Eligibility {
    /// Recovered non-instrumented branch. No class predicate/default is inferred
    /// from native atom kind. True requests releasing both registrations.
    pub fn must_not_tick_tock(self) -> bool {
        if matches!(self.field_1c, 2 | 3 | 5..=i32::MAX) || !self.has_owner || self.parent_id == 0 {
            return true;
        }
        let Some(parent_field) = self.parent_field_1c else {
            return true;
        };
        if parent_field == 2 {
            return true;
        }
        if !self.does_query_environment {
            return false;
        }
        if self
            .environment_field_34
            .is_some_and(|flags| flags & 0x28 == 0)
        {
            return false;
        }
        self.capsule_slot_b88.is_some_and(|value| !value)
    }
}

#[cfg(test)]
#[path = "eligibility_tests.rs"]
mod tests;
