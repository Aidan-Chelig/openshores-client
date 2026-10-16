use super::*;

pub(super) fn label(commands: &mut Commands, parent: Entity, text: impl Into<String>) {
    let child = commands
        .spawn((
            Text::new(text),
            TextFont {
                font_size: FontSize::Px(15.),
                ..default()
            },
            TextColor(Color::srgb(0.88, 0.91, 0.95)),
            Node {
                flex_shrink: 0.,
                ..default()
            },
        ))
        .id();
    commands.entity(parent).add_child(child);
}
pub(super) fn button(
    commands: &mut Commands,
    parent: Entity,
    action: Action,
    text: impl Into<String>,
) {
    let child = commands
        .spawn((
            Button,
            action,
            Node {
                padding: px(7).all(),
                margin: px(2).all(),
                flex_shrink: 0.,
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.2, 0.26)),
        ))
        .id();
    label(commands, child, text);
    commands.entity(parent).add_child(child);
}
pub(super) fn rebuild_panels(
    mut commands: Commands,
    mut ui: ResMut<Presentation>,
    panels: Query<Entity, With<Panel>>,
) {
    if !ui.dirty {
        return;
    }
    ui.dirty = false;
    for entity in &panels {
        commands.entity(entity).despawn();
    }
    if ui.model.phase == Phase::Playing && ui.hud {
        commands.spawn((
            Panel,
            Text::new("+"),
            TextFont {
                font_size: FontSize::Px(24.),
                ..default()
            },
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                top: percent(50),
                ..default()
            },
        ));
    }
    let root = commands
        .spawn((
            Panel,
            Node {
                position_type: PositionType::Absolute,
                left: px(15),
                top: px(15),
                width: px(360),
                max_height: percent(97),
                flex_direction: FlexDirection::Column,
                padding: px(12).all(),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.035, 0.05, 0.92)),
            ScrollPosition(Vec2::new(0., ui.panel_scroll)),
        ))
        .id();
    match ui.model.phase {
        Phase::NativePreview => {
            label(
                &mut commands,
                root,
                "Native avatar • offline read-only preview",
            );
            if let Some(avatar) = ui.model.native_avatar.as_ref() {
                label(&mut commands, root, avatar.summary());
                label(
                    &mut commands,
                    root,
                    format!(
                        "Native Unit text (raw nullable field): {}",
                        avatar.unit_text_summary()
                    ),
                );
            }
            label(&mut commands, root, &ui.model.status);
            label(
                &mut commands,
                root,
                "Marker centered in a diagnostic frame; not native world placement or body geometry.",
            );
        }
        Phase::Login | Phase::Disconnected => {
            label(&mut commands, root, "OpenShores Rust • local slice");
            label(
                &mut commands,
                root,
                "Loopback only. Minimal development UI.",
            );
            button(
                &mut commands,
                root,
                Action::BrowseAssets,
                "Choose Hazeron installation…",
            );
            button(
                &mut commands,
                root,
                Action::AssetsPath,
                format!(
                    "{} Hazeron installation: {}",
                    if ui.asset_focus { ">" } else { " " },
                    ui.asset_path
                ),
            );
            button(
                &mut commands,
                root,
                Action::MountAssets,
                "Prepare installation assets",
            );
            label(
                &mut commands,
                root,
                "Click the installation field, type its folder path, then prepare.",
            );
            label(&mut commands, root, &ui.asset_status);
            button(
                &mut commands,
                root,
                Action::Username,
                format!(
                    "{} Username: {}",
                    if ui.password_focus { " " } else { ">" },
                    ui.username
                ),
            );
            button(
                &mut commands,
                root,
                Action::Password,
                format!(
                    "{} Password: {}",
                    if ui.password_focus { ">" } else { " " },
                    "•".repeat(ui.password.chars().count())
                ),
            );
            button(&mut commands, root, Action::Login, "Log in");
            label(
                &mut commands,
                root,
                "Click a field; Tab changes field. Enter logs in.",
            );
            label(&mut commands, root, &ui.model.status);
        }
        Phase::Connecting | Phase::Joining => label(&mut commands, root, &ui.model.status),
        Phase::Roster => {
            label(
                &mut commands,
                root,
                "Choose avatar • six original roster slots",
            );
            if let Some(roster) = &ui.model.roster {
                for (index, avatar) in roster.iter().enumerate() {
                    if avatar.id == 0 {
                        label(&mut commands, root, format!("{}. Empty", index + 1));
                    } else {
                        button(
                            &mut commands,
                            root,
                            Action::Join(avatar.id),
                            format!(
                                "{}. {} ({:#010x})",
                                index + 1,
                                avatar.name.as_deref().unwrap_or("Unnamed"),
                                avatar.id
                            ),
                        );
                    }
                }
            }
        }
        Phase::Playing => {
            if ui.hud {
                label(
                    &mut commands,
                    root,
                    "Right-click: cursor/look • Ctrl: temporary cursor",
                );
                label(
                    &mut commands,
                    root,
                    "WASD / C Z • Shift+W run • U/middle click use • T eat",
                );
                label(
                    &mut commands,
                    root,
                    "F4 Gear • F8 Bio • F1 windows • Shift+Esc HUD",
                );
                label(
                    &mut commands,
                    root,
                    "Backspace: elevated view • hold [ / ]: lower / raise",
                );
                label(
                    &mut commands,
                    root,
                    "Base terrain / raw item models / prototype physics",
                );
                label(&mut commands, root, &ui.model.status);
                label(&mut commands, root, &ui.avatar_status);
                label(&mut commands, root, &ui.avatar_animation_status);
                label(&mut commands, root, &ui.avatar_placement_status);
            }
            if ui.windows_open {
                super::chat::panel(&mut commands, root, &ui);
                if ui.bio_open {
                    label(&mut commands, root, "Bio (F8)");
                    if let Some(person) = ui
                        .model
                        .avatar
                        .and_then(|id| ui.model.world.entities.get(&id))
                        .and_then(|entity| entity.person.as_ref())
                    {
                        label(
                            &mut commands,
                            root,
                            format!(
                                "HP {:?} • Stamina {:?} • Hunger raw {:?}",
                                person.hp, person.stamina, person.hunger_raw
                            ),
                        );
                    }
                }
                if ui.gear_open {
                    label(&mut commands, root, "Gear (F4) • select an entry");
                    if let Some(gear) = ui.model.gear() {
                        for slot in gear.slots.iter().take(16) {
                            if let Some(item) = &slot.item {
                                let name = ui
                                    .model
                                    .world
                                    .commodities
                                    .get(i32::from(item.common.commodity_id))
                                    .and_then(|row| row.name.as_deref())
                                    .unwrap_or("Unknown commodity");
                                button(
                                    &mut commands,
                                    root,
                                    Action::SelectGear(slot.slot_type, slot.subindex),
                                    format!(
                                        "{} {}/{}: {}",
                                        if ui.model.gear_selection
                                            == Some((slot.slot_type, slot.subindex))
                                        {
                                            ">"
                                        } else {
                                            " "
                                        },
                                        slot.slot_type,
                                        slot.subindex,
                                        name
                                    ),
                                );
                            }
                        }
                    }
                    button(&mut commands, root, Action::Use, "Use selected (U)");
                    button(
                        &mut commands,
                        root,
                        Action::Drop,
                        "Drop selected (no default key)",
                    );
                    button(&mut commands, root, Action::Move, "Move selected to slot 1");
                }
                label(
                    &mut commands,
                    root,
                    "Visible world items • drop an item to test pickup",
                );
                for item in ui.model.visible_items().take(6) {
                    button(
                        &mut commands,
                        root,
                        Action::SelectItem(item.id),
                        format!(
                            "{} {:#010x}",
                            if ui.model.target == Some(item.id) {
                                ">"
                            } else {
                                " "
                            },
                            item.id
                        ),
                    );
                }
                button(
                    &mut commands,
                    root,
                    Action::Pickup,
                    "Pick up selected world item",
                );
                button(&mut commands, root, Action::Disconnect, "Disconnect");
            }
        }
    }
}
