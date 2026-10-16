use super::*;
use bevy::ecs::system::SystemParam;

#[cfg(test)]
#[path = "controls_tests.rs"]
mod tests;

fn elevation_step(held: &mut f32, direction: f32, seconds: f32) -> f32 {
    if direction == 0. {
        *held = 0.;
        return 0.;
    }
    let before = if held.signum() == direction {
        held.abs()
    } else {
        0.
    };
    let ramp = seconds.min((2. - before).max(0.));
    // 24.3 -> 89.1 world units/second over two seconds. Integrate the ramp so
    // distance does not depend on frame rate, including crossing its cap.
    let distance = (24.3 + 32.4 * before) * seconds + 32.4 * (ramp * seconds - 0.5 * ramp * ramp);
    *held = direction * (before + seconds).min(2.);
    direction * distance
}

#[derive(SystemParam)]
pub(super) struct InputFrame<'w, 's> {
    keys: Res<'w, ButtonInput<KeyCode>>,
    mouse: Res<'w, ButtonInput<MouseButton>>,
    motion: Res<'w, AccumulatedMouseMotion>,
    scroll: Res<'w, AccumulatedMouseScroll>,
    typed: MessageReader<'w, 's, KeyboardInput>,
    time: Res<'w, Time>,
    window: Query<'w, 's, &'static mut CursorOptions, With<PrimaryWindow>>,
}

fn cycle_gear(ui: &mut Presentation, forward: bool) {
    let keys: Vec<_> = ui
        .model
        .gear()
        .map(|gear| {
            gear.slots
                .iter()
                .filter(|slot| slot.item.is_some())
                .map(|slot| (slot.slot_type, slot.subindex))
                .collect()
        })
        .unwrap_or_default();
    if keys.is_empty() {
        ui.model.gear_selection = None;
        return;
    }
    let index = ui
        .model
        .gear_selection
        .and_then(|key| keys.iter().position(|entry| *entry == key));
    let next = match (index, forward) {
        (Some(index), true) => (index + 1) % keys.len(),
        (Some(index), false) => (index + keys.len() - 1) % keys.len(),
        (None, true) => 0,
        (None, false) => keys.len() - 1,
    };
    ui.model.gear_selection = Some(keys[next]);
    ui.dirty = true;
}

pub(super) fn keyboard(mut ui: ResMut<Presentation>, bridge: Res<Bridge>, input: InputFrame) {
    ui.avatar_motion_speed = 0.;
    let InputFrame {
        keys,
        mouse,
        motion,
        scroll,
        mut typed,
        time,
        mut window,
    } = input;
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let alt = keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight);
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    if ui.chat_focus {
        ui.top_height_hold = 0.;
        typed.clear();
        if let Ok(mut cursor) = window.single_mut() {
            cursor.visible = true;
            cursor.grab_mode = CursorGrabMode::None;
        }
        return;
    }
    if matches!(ui.model.phase, Phase::Login | Phase::Disconnected) {
        for event in typed
            .read()
            .filter(|event| event.state == ButtonState::Pressed)
        {
            if event.key_code == KeyCode::Tab {
                ui.asset_focus = false;
                ui.password_focus = !ui.password_focus;
            } else if ui.asset_focus {
                if event.key_code == KeyCode::Backspace {
                    ui.asset_path.pop();
                } else if event.key_code == KeyCode::Enter {
                    activate(Action::MountAssets, &mut ui, &bridge);
                } else if ctrl && event.key_code == KeyCode::KeyA {
                    ui.asset_path.clear();
                } else if !ctrl
                    && !alt
                    && let Some(text) = &event.text
                {
                    let text: String = text.chars().filter(|c| !c.is_control()).collect();
                    if ui.asset_path.len() + text.len() <= 4096 {
                        ui.asset_path.push_str(&text);
                    }
                }
            } else if event.key_code == KeyCode::Backspace {
                if ui.password_focus {
                    ui.password.pop();
                } else {
                    ui.username.pop();
                }
            } else if event.key_code == KeyCode::Enter {
                activate(Action::Login, &mut ui, &bridge);
            } else if ctrl && event.key_code == KeyCode::KeyA {
                if ui.password_focus {
                    ui.password.clear();
                } else {
                    ui.username.clear();
                }
            } else if !ctrl
                && !alt
                && let Some(text) = &event.text
            {
                let text: String = text.chars().filter(|c| !c.is_control()).collect();
                if ui.password_focus {
                    if ui.password.len() + text.len() <= 1024 {
                        ui.password.push_str(&text);
                    }
                } else if ui.username.len() + text.len() <= 256 {
                    ui.username.push_str(&text);
                }
            }
            ui.dirty = true;
        }
    } else {
        typed.clear();
    }
    if ui.model.phase != Phase::Playing {
        ui.top_height_hold = 0.;
        if let Ok(mut cursor) = window.single_mut() {
            cursor.visible = true;
            cursor.grab_mode = CursorGrabMode::None;
        }
        return;
    }
    if mouse.just_pressed(MouseButton::Right) {
        ui.hand = !ui.hand;
        ui.dirty = true;
    }
    if keys.just_pressed(KeyCode::Enter) {
        ui.hand = true;
        ui.dirty = true;
    }
    if keys.just_pressed(KeyCode::F1) && !shift && !alt && !ctrl {
        ui.windows_open = !ui.windows_open;
        ui.dirty = true;
    }
    if keys.just_pressed(KeyCode::F4) && !shift && !alt && !ctrl {
        activate(Action::Gear, &mut ui, &bridge);
    }
    if keys.just_pressed(KeyCode::F8) && !shift && !alt && !ctrl {
        activate(Action::Bio, &mut ui, &bridge);
    }
    if keys.just_pressed(KeyCode::Escape) && shift && !alt && !ctrl {
        ui.hud = !ui.hud;
        ui.dirty = true;
    }
    if keys.just_pressed(KeyCode::Backspace) && !alt && !ctrl && !shift {
        ui.top_down = !ui.top_down;
    }
    if keys.just_pressed(KeyCode::Backspace) && shift && !alt && !ctrl {
        ui.avatar_inspect = !ui.avatar_inspect;
        ui.dirty = true;
    }
    if ui.top_down && !alt && !ctrl && !shift {
        let direction = i32::from(keys.pressed(KeyCode::BracketRight))
            - i32::from(keys.pressed(KeyCode::BracketLeft));
        // Camera-only world units per second; cap a stalled frame like movement.
        let step = elevation_step(
            &mut ui.top_height_hold,
            direction as f32,
            time.delta_secs().min(0.05),
        );
        ui.top_height = (ui.top_height + step).clamp(0.1, 100.);
    } else {
        ui.top_height_hold = 0.;
    }
    let hand = ui.hand || ctrl;
    if hand && scroll.delta.y != 0. {
        ui.panel_scroll = (ui.panel_scroll - scroll.delta.y * 30.).max(0.);
        ui.dirty = true;
    }
    if let Ok(mut cursor) = window.single_mut() {
        cursor.visible = hand;
        cursor.grab_mode = if hand {
            CursorGrabMode::None
        } else {
            CursorGrabMode::Locked
        };
    }
    let digit_keys = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
        KeyCode::Digit0,
    ];
    if !ctrl && !shift {
        for (index, key) in digit_keys.into_iter().enumerate() {
            if keys.just_pressed(key) {
                if alt {
                    ui.mappings[index] = ui.model.gear_selection;
                } else {
                    ui.model.gear_selection = ui.mappings[index];
                }
                ui.dirty = true;
            }
        }
    }
    if hand || alt {
        return;
    }
    ui.yaw += motion.delta.x * 0.003;
    ui.pitch = (ui.pitch - motion.delta.y * 0.003).clamp(-1.4, 1.4);
    if (keys.just_pressed(KeyCode::KeyU) && !shift) || mouse.just_pressed(MouseButton::Middle) {
        activate(Action::Use, &mut ui, &bridge);
    }
    if keys.just_pressed(KeyCode::KeyT) && !shift {
        activate(Action::Eat, &mut ui, &bridge);
    }
    if scroll.delta.y > 0. || (keys.just_pressed(KeyCode::BracketRight) && shift) {
        cycle_gear(&mut ui, true);
    }
    if scroll.delta.y < 0. || (keys.just_pressed(KeyCode::BracketLeft) && shift) {
        cycle_gear(&mut ui, false);
    }
    let Some(position) = ui.local_position else {
        return;
    };
    let Some(frame) = placement::tangent_frame(position, ui.yaw) else {
        return;
    };
    let up = frame.z_axis.as_dvec3();
    let forward = frame.y_axis.as_dvec3();
    let lateral = frame.x_axis.as_dvec3();
    let mut direction = DVec3::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        direction += forward;
    }
    if keys.pressed(KeyCode::KeyS) && !shift {
        direction -= forward;
    }
    if keys.pressed(KeyCode::KeyA) {
        direction -= lateral;
    }
    if keys.pressed(KeyCode::KeyD) {
        direction += lateral;
    }
    if keys.pressed(KeyCode::KeyC) {
        direction += up;
    }
    if keys.pressed(KeyCode::KeyZ) {
        direction -= up;
    }
    let seconds = time.delta_secs().min(0.05);
    ui.report_timer += seconds;
    if direction.length_squared() > 0. {
        let speed = if shift && keys.pressed(KeyCode::KeyW) {
            12.
        } else {
            4.
        };
        if (direction - up * direction.dot(up)).length_squared() > 0. {
            ui.avatar_motion_speed = speed as f32;
        }
        ui.local_position = Some(
            (DVec3::from_array(position) + direction.normalize() * f64::from(seconds) * speed)
                .to_array(),
        );
    }
    if direction.length_squared() > 0. || motion.delta.length_squared() > 0. {
        ui.local_rotation = placement::heading_rotation(ui.local_position.unwrap(), ui.yaw);
        ui.report_dirty = true;
    }
    if ui.report_timer >= 0.1 && ui.report_dirty {
        ui.report_timer = 0.;
        let projected = ui.local_position.unwrap();
        let rotation = ui
            .local_rotation
            .or_else(|| ui.model.rotation())
            .unwrap_or([0.; 3]);
        if send(
            &mut ui,
            &bridge,
            Request::Scene(SceneCommand::PositionReport {
                projected,
                rotation,
            }),
        ) {
            ui.report_dirty = false;
        }
    }
}
