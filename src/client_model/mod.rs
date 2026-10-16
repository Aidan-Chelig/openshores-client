//! Presentation state for the connected slice, independent of the UI framework.
use crate::{
    client_runtime::Event,
    commands::SceneCommand,
    login::Avatar,
    world::{AtomKind, WorldState},
};
use std::io;
mod avatar_preparation;
mod native;
mod player_appearance;
pub use avatar_preparation::{
    AvatarContentKey, AvatarPreparation, AvatarPreparationStatus, AvatarRequest,
    PreparedNormalAvatar,
};
pub use native::NativeAvatarPresentation;
pub use player_appearance::LocalPlayerDna;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Phase {
    #[default]
    Login,
    Connecting,
    Roster,
    Joining,
    Playing,
    NativePreview,
    Disconnected,
}
#[derive(Default)]
pub struct ClientModel {
    /// Successful source-session bootstrap generation; independent of movement.
    pub session_generation: u64,
    pub native_avatar: Option<Box<NativeAvatarPresentation>>,
    pub story_open: bool,
    pub story: std::collections::VecDeque<crate::chat::StoryText>,
    pub hail_environments: std::collections::BTreeMap<u32, u32>,
    pub choices: std::collections::VecDeque<crate::chat::Choice>,
    pub chat: std::collections::VecDeque<crate::chat::Message>,
    pub opaque_chat: std::collections::BTreeMap<u8, u64>,
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
        if self.phase == Phase::NativePreview
            && !matches!(
                &event,
                Event::NativeAvatar(_) | Event::Status(_) | Event::Error(_) | Event::Disconnected
            )
        {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "source-profile event in native avatar preview",
            ));
        }
        match event {
            Event::NativeAvatar(avatar) => self.present_native(avatar),
            Event::IncomingChat(crate::chat::Incoming::Story(text)) => {
                if Some(text.header.target) == self.avatar {
                    self.story_open = true;
                    if self.story.len() == 32 {
                        self.story.pop_front();
                    }
                    self.story.push_back(text);
                }
            }
            Event::IncomingChat(crate::chat::Incoming::Hail(reply)) => {
                if self.hail_environments.len() == 100 {
                    self.hail_environments.pop_first();
                }
                self.hail_environments
                    .insert(reply.header.sender, reply.environment);
                if self.chat.len() == 100 {
                    self.chat.pop_front();
                }
                self.chat.push_back(reply.header);
            }
            Event::IncomingChat(crate::chat::Incoming::Choice(choice)) => {
                if Some(choice.offered_to) == self.avatar
                    && Some(choice.header.target) == self.avatar
                {
                    if choice.narrated {
                        self.story_open = true;
                    }
                    self.choices.retain(|old| old.reply() != choice.reply());
                    if self.choices.len() == 32 {
                        self.choices.pop_front();
                    }
                    self.choices.push_back(choice);
                }
            }
            Event::ChoiceSent(reply) => {
                self.choices.retain(|choice| {
                    choice.instance as u32 != reply.instance
                        || choice.header.sender != reply.offered
                });
                self.status = "Choice written; awaiting server dialogue".into();
            }
            Event::StoryControlSent(control) => {
                if control == crate::chat::StoryControl::Close {
                    self.story_open = false;
                    self.choices.retain(|choice| choice.instance == 1);
                }
                self.status = format!("Story {control:?} written; not a server acknowledgement");
            }
            Event::IncomingChat(crate::chat::Incoming::Message(message)) => {
                if self.chat.len() == 100 {
                    self.chat.pop_front();
                }
                self.chat.push_back(message);
            }
            Event::IncomingChat(crate::chat::Incoming::Opaque { kind, .. }) => {
                *self.opaque_chat.entry(kind).or_default() += 1;
            }
            Event::Roster(roster) => {
                self.roster = Some(roster);
                self.phase = Phase::Roster;
                self.status = "Select an avatar (empty slots remain empty)".into();
            }
            Event::Bootstrap(bootstrap) => {
                let generation = self
                    .session_generation
                    .checked_add(1)
                    .ok_or_else(|| io::Error::other("source session generation exhausted"))?;
                let mut world = WorldState::default();
                for packet in &bootstrap.packets {
                    world.ingest(packet)?;
                }
                if world.active_unit != Some(bootstrap.avatar_id) {
                    return Err(io::Error::other("bootstrap avatar mismatch"));
                }
                self.avatar = Some(bootstrap.avatar_id);
                self.session_generation = generation;
                self.world = world;
                self.chat.clear();
                self.story.clear();
                self.story_open = false;
                self.hail_environments.clear();
                self.choices.clear();
                self.opaque_chat.clear();
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
                self.native_avatar = None;
                self.story_open = false;
                self.choices.clear();
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
mod tests;
