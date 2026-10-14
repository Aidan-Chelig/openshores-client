use super::*;

pub(super) fn send(ui: &mut Presentation, bridge: &Bridge, request: Request) -> bool {
    ui.dirty = true;
    match bridge.0.lock().unwrap().send(request) {
        Ok(()) => true,
        Err(error) => {
            ui.model.status = error.to_string();
            false
        }
    }
}
pub(super) fn network_events(mut ui: ResMut<Presentation>, bridge: Res<Bridge>) {
    for _ in 0..128 {
        let event = match bridge.0.lock().unwrap().next_event() {
            Ok(event) => event,
            Err(_) => break,
        };
        let bootstrap = matches!(event, Event::Bootstrap(_));
        if let Err(error) = ui.model.apply(event) {
            ui.model.status = format!("Invalid server state: {error}");
            send(&mut ui, &bridge, Request::Disconnect);
        }
        if bootstrap {
            ui.local_position = ui.model.position();
            ui.hand = false;
            ui.yaw = 0.;
            ui.pitch = 0.;
            ui.mappings = [None; 10];
        }
        ui.dirty = true;
    }
}
pub(super) fn activate(action: Action, ui: &mut Presentation, bridge: &Bridge) {
    match action {
        Action::StoryControl(control) if ui.model.phase == Phase::Playing => {
            send(ui, bridge, Request::StoryControl(control));
        }
        Action::Hail if ui.model.phase == Phase::Playing => {
            send(ui, bridge, Request::Hail);
        }
        Action::Reply(reply) if ui.model.phase == Phase::Playing => {
            send(ui, bridge, Request::Choice(reply));
        }
        Action::ChatFocus => {
            ui.chat_focus = true;
            ui.hand = true;
        }
        Action::NextChannel => {
            ui.chat_channel = (ui.chat_channel + 1) % crate::chat::CHANNELS.len() as u8
        }
        Action::NextRange => ui.chat_range = ui.chat_range.next(),
        Action::SendMessage if ui.model.phase == Phase::Playing => {
            let message = crate::chat::Outgoing {
                channel: ui.chat_channel,
                range: ui.chat_range,
                text: ui.chat_draft.clone(),
            };
            if let Err(error) = message.encode(ui.model.avatar.unwrap_or(0)) {
                ui.model.status = error.to_string();
            } else if send(ui, bridge, Request::Message(message)) {
                ui.chat_draft.clear();
                ui.chat_focus = false;
            }
        }
        Action::AssetsPath => ui.asset_focus = true,
        Action::BrowseAssets => ui.asset_browse = true,
        Action::MountAssets => {
            ui.asset_focus = false;
            ui.asset_request = true;
        }
        Action::Username => {
            ui.asset_focus = false;
            ui.password_focus = false;
        }
        Action::Password => {
            ui.asset_focus = false;
            ui.password_focus = true;
        }
        Action::Login if matches!(ui.model.phase, Phase::Login | Phase::Disconnected) => {
            if !ui.assets_ready {
                ui.model.status = "Select and prepare your Hazeron installation first".into();
            } else if ui.username.is_empty() || ui.password.is_empty() {
                ui.model.status = "Enter username and password".into();
            } else {
                let password = std::mem::take(&mut ui.password);
                let username = ui.username.clone();
                if send(ui, bridge, Request::Login { username, password }) {
                    ui.model.phase = Phase::Connecting;
                    ui.model.status = "Authenticating with loopback server…".into();
                }
            }
        }
        Action::Join(id)
            if ui.model.phase == Phase::Roster
                && id != 0
                && send(ui, bridge, Request::Join(id)) =>
        {
            ui.model.phase = Phase::Joining;
            ui.model.status = "Initializing world…".into();
        }
        Action::SelectGear(slot, sub) => ui.model.gear_selection = Some((slot, sub)),
        Action::Eat if ui.model.phase == Phase::Playing => {
            // The source GD food scanner uses commodity flag bit 1. Do not
            // classify unknown rows or infer nutrition/quality client-side.
            let key = ui.model.gear().and_then(|gear| {
                gear.slots.iter().find_map(|slot| {
                    let item = slot.item.as_ref()?;
                    let row = ui
                        .model
                        .world
                        .commodities
                        .get(i32::from(item.common.commodity_id))?;
                    (row.flags & 2 != 0).then_some((slot.slot_type, slot.subindex))
                })
            });
            if let Some((source_slot, source_subindex)) = key {
                send(
                    ui,
                    bridge,
                    Request::Chat(ChatCommand::UseGear {
                        source_slot,
                        source_subindex,
                    }),
                );
            } else {
                ui.model.status = "No source-flagged food found in received gear".into();
            }
        }
        Action::SelectItem(id) => {
            ui.model.target = Some(id);
            send(
                ui,
                bridge,
                Request::Scene(SceneCommand::LookAt { target: id }),
            );
        }
        Action::Use | Action::Drop | Action::Move if ui.model.phase == Phase::Playing => {
            if let Some((source_slot, source_subindex)) = ui.model.gear_selection {
                let request = match action {
                    Action::Use => Request::Chat(ChatCommand::UseGear {
                        source_slot,
                        source_subindex,
                    }),
                    Action::Drop => Request::Chat(ChatCommand::DropGear {
                        source_slot,
                        source_subindex,
                    }),
                    _ => Request::Scene(SceneCommand::MoveGear {
                        source_slot,
                        source_subindex,
                        destination_slot: 1,
                    }),
                };
                send(ui, bridge, request);
            } else {
                ui.model.status = "Select a gear entry first".into();
            }
        }
        Action::Pickup if ui.model.phase == Phase::Playing => {
            if let Some(id) = ui.model.target {
                send(ui, bridge, Request::Pickup(id));
            } else {
                ui.model.status = "Select a visible world item first".into();
            }
        }
        Action::Disconnect => {
            send(ui, bridge, Request::Disconnect);
        }
        Action::Gear => ui.gear_open = !ui.gear_open,
        Action::Bio => ui.bio_open = !ui.bio_open,
        _ => {}
    }
    ui.dirty = true;
}
pub(super) fn buttons(
    mut ui: ResMut<Presentation>,
    bridge: Res<Bridge>,
    interactions: Query<(&Interaction, &Action), Changed<Interaction>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let can_click = ui.model.phase != Phase::Playing
        || ui.hand
        || keys.pressed(KeyCode::ControlLeft)
        || keys.pressed(KeyCode::ControlRight);
    for (interaction, action) in &interactions {
        if can_click && *interaction == Interaction::Pressed {
            activate(*action, &mut ui, &bridge);
        }
    }
}
