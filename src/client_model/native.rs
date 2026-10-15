//! Explicit native presentation boundary, never converted to source WorldState.
use super::*;
use crate::world::{
    native_creature::NativeCreatureBodyKind, native_dispatch::NativeAtomDispatchOutcome,
    native_person::NativePersonObjects,
};

#[derive(Clone, Debug, PartialEq)]
pub struct NativeAvatarPresentation {
    pub handle: u32,
    pub wire_id: u32,
    pub timestamp_ms: i64,
    pub parent: u32,
    pub local_position: [f64; 3],
    pub unit_text: Option<Option<String>>,
    pub body_kind: Option<NativeCreatureBodyKind>,
    pub hit_points: Option<i16>,
    pub condition_count: Option<usize>,
    pub gear_count: Option<usize>,
    pub inventory_count: Option<usize>,
}
impl NativeAvatarPresentation {
    /// Publish only after successful native dispatch, never an errored partial
    /// receive. Cross-check stable handle, wire identity and attached ownership.
    /// Position remains explicitly local; no globe/environment frame is guessed.
    pub fn from_dispatched(
        people: &NativePersonObjects,
        outcome: NativeAtomDispatchOutcome,
    ) -> io::Result<Self> {
        let person = people.get(outcome.handle).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "native presentation handle is missing",
            )
        })?;
        if !person.base.owner_attached || person.base.wire_id != outcome.wire_id {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "native presentation identity/owner mismatch",
            ));
        }
        let position = person.base.position.position.local[3][..3]
            .try_into()
            .expect("three coordinates");
        Ok(Self {
            handle: outcome.handle,
            wire_id: outcome.wire_id,
            timestamp_ms: person.base.received_time,
            parent: person.base.position.position.parent,
            local_position: position,
            unit_text: person.body.creature.unit.fields.field_3b0.clone(),
            body_kind: person.body.creature.body.as_ref().map(|body| body.kind),
            hit_points: person
                .body
                .creature
                .body
                .as_ref()
                .map(|body| body.hit_points),
            condition_count: person
                .body
                .creature
                .unit
                .conditions
                .as_ref()
                .map(|values| values.0.len()),
            gear_count: person
                .body
                .person
                .gear_1390
                .as_ref()
                .map(|values| values.0.len()),
            inventory_count: person
                .body
                .person
                .inventory_1458
                .as_ref()
                .map(|values| values.0.values().map(Vec::len).sum()),
        })
    }

    pub fn summary(&self) -> String {
        format!(
            "Native Person #{} • handle {}\nBody {:?} • HP {:?}\nLocal parent {} • position {:?}\nConditions {:?} • gear {:?} • inventory {:?}",
            self.wire_id,
            self.handle,
            self.body_kind,
            self.hit_points,
            self.parent,
            self.local_position,
            self.condition_count,
            self.gear_count,
            self.inventory_count
        )
    }

    /// Bound HUD work without truncating the actual decoded/retained field.
    pub fn unit_text_summary(&self) -> String {
        match &self.unit_text {
            None => "unknown".into(),
            Some(None) => "null".into(),
            Some(Some(text)) => {
                let preview: String = text.chars().take(256).collect();
                format!(
                    "{preview:?}{}",
                    if text.chars().nth(256).is_some() {
                        "… (display truncated)"
                    } else {
                        ""
                    }
                )
            }
        }
    }
}

impl ClientModel {
    pub(super) fn present_native(&mut self, avatar: Box<NativeAvatarPresentation>) {
        self.native_avatar = Some(avatar);
        self.phase = Phase::NativePreview;
        self.world = WorldState::default();
        self.roster = None;
        self.avatar = None; // source-profile selected-avatar field is not reused
        self.target = None;
        self.gear_selection = None;
        self.reported_position = None;
        self.reported_rotation = None;
        self.chat.clear();
        self.story.clear();
        self.choices.clear();
        self.hail_environments.clear();
        self.opaque_chat.clear();
        self.story_open = false;
        self.status =
            "Read-only native avatar preview; diagnostic marker, not a reconstructed body mesh"
                .into();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_unit_text_display_is_bounded_and_keeps_unknown_null_and_empty_distinct() {
        let mut avatar = NativeAvatarPresentation {
            handle: 90,
            wire_id: 7,
            timestamp_ms: 0,
            parent: 0,
            local_position: [0.; 3],
            unit_text: None,
            body_kind: None,
            hit_points: None,
            condition_count: None,
            gear_count: None,
            inventory_count: None,
        };
        assert_eq!(avatar.unit_text_summary(), "unknown");
        avatar.unit_text = Some(None);
        assert_eq!(avatar.unit_text_summary(), "null");
        avatar.unit_text = Some(Some(String::new()));
        assert_eq!(avatar.unit_text_summary(), "\"\"");
        let raw = "🌊".repeat(1000);
        avatar.unit_text = Some(Some(raw.clone()));
        let summary = avatar.unit_text_summary();
        assert_eq!(summary.matches('🌊').count(), 256);
        assert!(summary.ends_with("(display truncated)"));
        assert_eq!(avatar.unit_text, Some(Some(raw)));
    }
}
