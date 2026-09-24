#[cfg(feature = "gamepad")]
use bevy::input::gamepad::{GamepadConnection, GamepadConnectionEvent};
use bevy::{input::InputPlugin, prelude::*};
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, Clone, Copy, Debug, Reflect, PartialEq, Eq, Hash)]
enum Action {
    Jump,
    Confirm,
    Dash,
    Pause,
}

#[derive(Component)]
struct TestPlayer;

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        InputManagerPlugin::<Action>::default(),
    ));
    app
}

fn spawn_player(app: &mut App, input_map: InputMap<Action>) -> Entity {
    app.world_mut().spawn((TestPlayer, input_map)).id()
}

fn player_action_state<'w>(app: &'w mut App) -> &'w ActionState<Action> {
    app.world_mut()
        .query_filtered::<&ActionState<Action>, With<TestPlayer>>()
        .single(app.world())
        .expect("ActionState not found")
}

#[cfg(feature = "gamepad")]
fn connect_gamepad(app: &mut App) -> Entity {
    let gamepad = app.world_mut().spawn_empty().id();
    app.world_mut()
        .resource_mut::<Messages<GamepadConnectionEvent>>()
        .write(GamepadConnectionEvent::new(
            gamepad,
            GamepadConnection::Connected {
                name: "TestController".to_owned(),
                vendor_id: None,
                product_id: None,
            },
        ));
    app.update();
    gamepad
}

#[cfg(feature = "keyboard")]
#[test]
fn keyboard_hold_across_input_map_replacement() {
    let mut app = test_app();
    let player = spawn_player(&mut app, InputMap::new([(Action::Jump, KeyCode::Space)]));
    app.update();

    KeyCode::Space.press(app.world_mut());
    app.update();

    let action_state = player_action_state(&mut app);
    assert!(action_state.just_pressed(&Action::Jump));

    app.world_mut()
        .entity_mut(player)
        .insert(InputMap::new([(Action::Confirm, KeyCode::Space)]));
    app.update();

    let action_state = player_action_state(&mut app);
    assert!(!action_state.pressed(&Action::Confirm));
    assert!(!action_state.just_pressed(&Action::Confirm));

    KeyCode::Space.release(app.world_mut());
    app.update();

    let action_state = player_action_state(&mut app);
    assert!(!action_state.pressed(&Action::Confirm));
    assert!(!action_state.just_pressed(&Action::Confirm));
    assert!(!action_state.just_released(&Action::Confirm));
}

#[cfg(feature = "gamepad")]
#[test]
fn gamepad_hold_across_input_map_replacement() {
    let mut app = test_app();
    let gamepad = connect_gamepad(&mut app);

    let mut jump_map = InputMap::new([(Action::Jump, GamepadButton::South)]);
    jump_map.set_gamepad(gamepad);
    let player = spawn_player(&mut app, jump_map);
    app.update();

    GamepadButton::South.press_as_gamepad(app.world_mut(), Some(gamepad));
    app.update();

    let action_state = player_action_state(&mut app);
    assert!(action_state.just_pressed(&Action::Jump));

    let mut confirm_map = InputMap::new([(Action::Confirm, GamepadButton::South)]);
    confirm_map.set_gamepad(gamepad);
    app.world_mut().entity_mut(player).insert(confirm_map);
    app.update();

    let action_state = player_action_state(&mut app);
    assert!(!action_state.pressed(&Action::Confirm));
    assert!(!action_state.just_pressed(&Action::Confirm));

    GamepadButton::South.release_as_gamepad(app.world_mut(), Some(gamepad));
    app.update();

    let action_state = player_action_state(&mut app);
    assert!(!action_state.pressed(&Action::Confirm));
    assert!(!action_state.just_pressed(&Action::Confirm));
    assert!(!action_state.just_released(&Action::Confirm));
}

#[cfg(feature = "keyboard")]
#[test]
fn multiple_held_inputs_stay_suppressed_after_input_map_replacement() {
    let mut app = test_app();
    let player = spawn_player(
        &mut app,
        InputMap::new([
            (Action::Jump, KeyCode::Space),
            (Action::Dash, KeyCode::KeyF),
        ]),
    );
    app.update();

    KeyCode::Space.press(app.world_mut());
    KeyCode::KeyF.press(app.world_mut());
    app.update();

    app.world_mut().entity_mut(player).insert(InputMap::new([
        (Action::Confirm, KeyCode::Space),
        (Action::Pause, KeyCode::KeyF),
    ]));
    app.update();

    let action_state = player_action_state(&mut app);
    assert!(!action_state.pressed(&Action::Confirm));
    assert!(!action_state.just_pressed(&Action::Confirm));
    assert!(!action_state.pressed(&Action::Pause));
    assert!(!action_state.just_pressed(&Action::Pause));

    KeyCode::Space.release(app.world_mut());
    KeyCode::KeyF.release(app.world_mut());
    app.update();

    let action_state = player_action_state(&mut app);
    assert!(!action_state.just_released(&Action::Confirm));
    assert!(!action_state.just_released(&Action::Pause));
}

#[cfg(feature = "keyboard")]
#[test]
fn input_becomes_active_again_after_release_and_repress() {
    let mut app = test_app();
    let player = spawn_player(&mut app, InputMap::new([(Action::Jump, KeyCode::Space)]));
    app.update();

    KeyCode::Space.press(app.world_mut());
    app.update();

    app.world_mut()
        .entity_mut(player)
        .insert(InputMap::new([(Action::Confirm, KeyCode::Space)]));
    app.update();

    KeyCode::Space.release(app.world_mut());
    app.update();

    let action_state = player_action_state(&mut app);
    assert!(!action_state.just_released(&Action::Confirm));

    KeyCode::Space.press(app.world_mut());
    app.update();

    let action_state = player_action_state(&mut app);
    assert!(action_state.pressed(&Action::Confirm));
    assert!(action_state.just_pressed(&Action::Confirm));

    KeyCode::Space.release(app.world_mut());
    app.update();

    let action_state = player_action_state(&mut app);
    assert!(!action_state.pressed(&Action::Confirm));
    assert!(action_state.just_released(&Action::Confirm));
}
