//! Presentation state for the connected slice, independent of the UI framework.
use crate::{
    client_runtime::Event,
    commands::SceneCommand,
    login::Avatar,
    world::{AtomKind, WorldState},
};
use std::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Phase {
    #[default]
    Login,
    Connecting,
    Roster,
    Joining,
    Playing,
    Disconnected,
}
#[derive(Default)]
pub struct ClientModel {
    pub phase: Phase,
    pub roster: Option<Box<[Avatar; 6]>>,
    pub world: WorldState,
    pub avatar: Option<u32>,
    pub target: Option<u32>,
    pub gear_selection: Option<(u8, u8)>,
    /// Last successfully written client report, never a server transform echo.
    pub reported_position: Option<[f64; 3]>,
    pub reported_rotation: Option<[f64; 3]>,
    pub status: String,
    pub revision: u64,
}
impl ClientModel {
    pub fn apply(&mut self, event: Event) -> io::Result<()> {
        match event {
            Event::Roster(roster) => {
                self.roster = Some(roster);
                self.phase = Phase::Roster;
                self.status = "Select an avatar (empty slots remain empty)".into();
            }
            Event::Bootstrap(bootstrap) => {
                let mut world = WorldState::default();
                for packet in &bootstrap.packets {
                    world.ingest(packet)?;
                }
                if world.active_unit != Some(bootstrap.avatar_id) {
                    return Err(io::Error::other("bootstrap avatar mismatch"));
                }
                self.avatar = Some(bootstrap.avatar_id);
                self.world = world;
                self.phase = Phase::Playing;
                self.reported_position = None;
                self.reported_rotation = None;
                self.target = None;
                self.gear_selection = None;
                self.status = "Connected to local world; actions await server updates".into();
            }
            Event::Packet(packet) => {
                self.world.ingest(&packet)?;
                if let Some(target) = self.target
                    && !self.world.visible_ids.contains(&target)
                {
                    self.target = None;
                }
                if let Some(key) = self.gear_selection
                    && !self.gear().is_some_and(|gear| {
                        gear.slots.iter().any(|slot| {
                            (slot.slot_type, slot.subindex) == key && slot.item.is_some()
                        })
                    })
                {
                    self.gear_selection = None;
                }
            }
            Event::SentScene(command) => {
                if let SceneCommand::PositionReport {
                    projected,
                    rotation,
                } = command
                {
                    self.reported_position = Some(projected);
                    self.reported_rotation = Some(rotation);
                }
                self.status = format!("Sent {command:?}; not an acknowledgement");
            }
            Event::SentChat(command) => {
                self.status = format!("Sent {command:?}; waiting for server state")
            }
            Event::Status(status) => self.status = status,
            Event::Error(error) => {
                if matches!(self.phase, Phase::Connecting | Phase::Joining) {
                    self.phase = Phase::Login;
                }
                self.status = format!("Error: {error}");
            }
            Event::Disconnected => {
                self.phase = Phase::Disconnected;
                self.target = None;
                self.gear_selection = None;
            }
        }
        self.revision += 1;
        Ok(())
    }
    pub fn gear(&self) -> Option<&crate::inventory::Gear> {
        self.world
            .entities
            .get(&self.avatar?)
            .and_then(|entity| entity.person.as_ref())
            .and_then(|person| person.gear.as_ref())
    }
    pub fn home_globe(&self) -> Option<u32> {
        self.world
            .ancestry(self.avatar?)
            .ok()?
            .into_iter()
            .find(|id| self.world.entities[id].globe_surface.is_some())
    }
    pub fn position(&self) -> Option<[f64; 3]> {
        self.reported_position.or_else(|| {
            self.world
                .entities
                .get(&self.avatar?)
                .and_then(|entity| entity.transform)
                .map(|transform| transform.position.map(f64::from))
        })
    }
    pub fn rotation(&self) -> Option<[f64; 3]> {
        self.reported_rotation.or_else(|| {
            self.world
                .entities
                .get(&self.avatar?)
                .and_then(|entity| entity.transform)
                .map(|transform| transform.rotation.map(f64::from))
        })
    }
    pub fn visible_items(&self) -> impl Iterator<Item = &crate::world::Entity> {
        self.world.entities.values().filter(|entity| {
            entity.kind == AtomKind::Item && self.world.visible_ids.contains(&entity.id)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn local_reports_do_not_mutate_received_entity_transforms() {
        let mut model = ClientModel::default();
        let report = SceneCommand::PositionReport {
            projected: [1., 2., 3.],
            rotation: [4., 5., 6.],
        };
        model.apply(Event::SentScene(report)).unwrap();
        assert!(model.world.entities.is_empty());
        assert_eq!(model.position(), Some([1., 2., 3.]));
        assert!(model.status.contains("not an acknowledgement"));
    }
}
