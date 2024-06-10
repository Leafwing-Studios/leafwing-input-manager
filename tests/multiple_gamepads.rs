use bevy::input::gamepad::{GamepadConnection, GamepadConnectionEvent, GamepadEvent, GamepadInfo};
use bevy::input::InputPlugin;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, Clone, Copy, Debug, Reflect, PartialEq, Eq, Hash)]
enum MyAction {
    Jump,
}

fn create_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(InputPlugin);
    app.add_plugins(InputManagerPlugin::<MyAction>::default());

    let mut gamepad_events = app.world_mut().resource_mut::<Events<GamepadEvent>>();
    gamepad_events.send(GamepadEvent::Connection(GamepadConnectionEvent {
        // Must be consistent with mocked events
        gamepad: Gamepad { id: 1 },
        connection: GamepadConnection::Connected(GamepadInfo {
            name: "FirstController".into(),
        }),
    }));
    gamepad_events.send(GamepadEvent::Connection(GamepadConnectionEvent {
        // Must be consistent with mocked events
        gamepad: Gamepad { id: 2 },
        connection: GamepadConnection::Connected(GamepadInfo {
            name: "SecondController".into(),
        }),
    }));

    // Ensure the gamepads are picked up
    app.update();
    // Flush the gamepad connection events
    app.update();

    app
}

fn jump_button_press_event(gamepad: Gamepad) -> GamepadEvent {
    use bevy::input::gamepad::GamepadButtonChangedEvent;

    GamepadEvent::Button(GamepadButtonChangedEvent::new(
        gamepad,
        GamepadButtonType::South,
        1.0,
    ))
}

#[test]
fn default_accepts_any() {
    let mut app = create_test_app();

    const FIRST_GAMEPAD: Gamepad = Gamepad { id: 1 };
    const SECOND_GAMEPAD: Gamepad = Gamepad { id: 2 };

    let input_map = InputMap::new([(MyAction::Jump, GamepadButtonType::South)]);
    app.insert_resource(input_map);
    app.init_resource::<ActionState<MyAction>>();

    // When we press the Jump button...
    let mut events = app.world_mut().resource_mut::<Events<GamepadEvent>>();
    events.send(jump_button_press_event(FIRST_GAMEPAD));
    app.update();

    // ... We should receive a Jump action!
    let mut action_state = app.world_mut().resource_mut::<ActionState<MyAction>>();
    assert!(action_state.pressed(&MyAction::Jump));

    action_state.release(&MyAction::Jump);
    app.update();

    // This is maintained for any gamepad.
    let mut events = app.world_mut().resource_mut::<Events<GamepadEvent>>();
    events.send(jump_button_press_event(SECOND_GAMEPAD));
    app.update();

    let action_state = app.world_mut().resource_mut::<ActionState<MyAction>>();
    assert!(action_state.pressed(&MyAction::Jump));
}

#[test]
fn accepts_preferred_gamepad() {
    let mut app = create_test_app();

    const PREFERRED_GAMEPAD: Gamepad = Gamepad { id: 2 };

    let mut input_map = InputMap::new([(MyAction::Jump, GamepadButtonType::South)]);
    input_map.set_gamepad(PREFERRED_GAMEPAD);
    app.insert_resource(input_map);
    app.init_resource::<ActionState<MyAction>>();

    // When we press the Jump button...
    let mut events = app.world_mut().resource_mut::<Events<GamepadEvent>>();
    events.send(jump_button_press_event(PREFERRED_GAMEPAD));
    app.update();

    // ... We should receive a Jump action!
    let action_state = app.world_mut().resource_mut::<ActionState<MyAction>>();
    assert!(action_state.pressed(&MyAction::Jump));
}

#[test]
fn filters_out_other_gamepads() {
    let mut app = create_test_app();

    const PREFERRED_GAMEPAD: Gamepad = Gamepad { id: 2 };
    const OTHER_GAMEPAD: Gamepad = Gamepad { id: 1 };

    let mut input_map = InputMap::new([(MyAction::Jump, GamepadButtonType::South)]);
    input_map.set_gamepad(PREFERRED_GAMEPAD);
    app.insert_resource(input_map);
    app.init_resource::<ActionState<MyAction>>();

    // When we press the Jump button...
    let mut events = app.world_mut().resource_mut::<Events<GamepadEvent>>();
    events.send(jump_button_press_event(OTHER_GAMEPAD));
    app.update();

    // ... We should receive a Jump action!
    let action_state = app.world_mut().resource_mut::<ActionState<MyAction>>();
    assert!(action_state.released(&MyAction::Jump));
}
