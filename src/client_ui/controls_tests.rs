use super::*;

fn app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(Presentation {
            model: ClientModel {
                phase: Phase::Playing,
                ..default()
            },
            windows_open: true,
            gear_open: true,
            bio_open: true,
            hud: true,
            ..default()
        })
        .insert_resource(Bridge(Mutex::new(
            ClientRuntime::start(LocalEndpoint {
                login: "127.0.0.1:1".parse().unwrap(),
                scene_port: 1,
                chat_port: 1,
            })
            .unwrap(),
        )))
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<AccumulatedMouseMotion>()
        .init_resource::<AccumulatedMouseScroll>()
        .add_message::<KeyboardInput>()
        .add_systems(Update, keyboard);
    app
}
fn keys(app: &mut App, keys: &[KeyCode]) {
    {
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.reset_all();
        for key in keys {
            input.press(*key);
        }
    }
    app.update();
}

#[test]
fn documented_window_shortcuts_and_modifier_exclusions_are_preserved() {
    let mut app = app();
    keys(&mut app, &[KeyCode::F4]);
    assert!(!app.world().resource::<Presentation>().gear_open);
    keys(&mut app, &[KeyCode::ShiftLeft, KeyCode::F4]);
    assert!(!app.world().resource::<Presentation>().gear_open);
    keys(&mut app, &[KeyCode::F8]);
    assert!(!app.world().resource::<Presentation>().bio_open);
    keys(&mut app, &[KeyCode::F1]);
    assert!(!app.world().resource::<Presentation>().windows_open);
    keys(&mut app, &[KeyCode::Escape]);
    assert!(app.world().resource::<Presentation>().hud);
    keys(&mut app, &[KeyCode::ShiftLeft, KeyCode::Escape]);
    assert!(!app.world().resource::<Presentation>().hud);
    keys(&mut app, &[KeyCode::Backspace]);
    assert!(app.world().resource::<Presentation>().top_down);
}

#[test]
fn cursor_toggle_temporary_control_and_position_hotbar_mapping_follow_reference() {
    let mut app = app();
    keys(&mut app, &[KeyCode::ControlLeft]);
    assert!(!app.world().resource::<Presentation>().hand);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Right);
    keys(&mut app, &[]);
    assert!(app.world().resource::<Presentation>().hand);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .reset_all();
    app.world_mut()
        .resource_mut::<Presentation>()
        .model
        .gear_selection = Some((5, 3));
    keys(&mut app, &[KeyCode::AltLeft, KeyCode::Digit1]);
    assert_eq!(
        app.world().resource::<Presentation>().mappings[0],
        Some((5, 3))
    );
    app.world_mut()
        .resource_mut::<Presentation>()
        .model
        .gear_selection = None;
    keys(&mut app, &[KeyCode::Digit1]);
    assert_eq!(
        app.world().resource::<Presentation>().model.gear_selection,
        Some((5, 3))
    );
    // Mapping stays attached to a position even with no item at that position.
    assert!(
        app.world()
            .resource::<Presentation>()
            .model
            .gear()
            .is_none()
    );
    assert_eq!(
        app.world().resource::<Presentation>().mappings[0],
        Some((5, 3))
    );
}
