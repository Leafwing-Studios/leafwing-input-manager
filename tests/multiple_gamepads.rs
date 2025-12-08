#![cfg(feature = "gamepad")]

use bevy::input::gamepad::{
    GamepadConnection, GamepadConnectionEvent, GamepadEvent, RawGamepadEvent,
};
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

    let gamepad_1 = app.world_mut().spawn(()).id();
    let gamepad_2 = app.world_mut().spawn(()).id();

    let mut gamepad_messages = app.world_mut().resource_mut::<Messages<GamepadEvent>>();
    gamepad_messages.write(GamepadEvent::Connection(GamepadConnectionEvent {
        // Must be consistent with mocked messages
        gamepad: gamepad_1,
        connection: GamepadConnection::Connected {
            name: "FirstController".into(),
            vendor_id: None,
            product_id: None,
        },
    }));
    gamepad_messages.write(GamepadEvent::Connection(GamepadConnectionEvent {
        // Must be consistent with mocked messages
        gamepad: gamepad_2,
        connection: GamepadConnection::Connected {
            name: "SecondController".into(),
            vendor_id: None,
            product_id: None,
        },
    }));

    // Ensure the gamepads are picked up
    app.update();
    // Flush the gamepad connection messages
    app.update();

    app
}

fn jump_button_press_message(gamepad: Entity) -> RawGamepadEvent {
    use bevy::input::gamepad::RawGamepadButtonChangedEvent;

    RawGamepadEvent::Button(RawGamepadButtonChangedEvent::new(
        gamepad,
        GamepadButton::South,
        1.0,
    ))
}

#[test]
fn accepts_preferred_gamepad() {
    let mut app = create_test_app();

    let preferred_gamepad = app.world_mut().spawn(()).id();
    let mut gamepad_connection_messages = app
        .world_mut()
        .resource_mut::<Messages<GamepadConnectionEvent>>();
    gamepad_connection_messages.write(GamepadConnectionEvent {
        // This MUST be consistent with any other mocked messages
        gamepad: preferred_gamepad,
        connection: GamepadConnection::Connected {
            name: "Preferred gamepad".to_owned(),
            vendor_id: None,
            product_id: None,
        },
    });
    // Ensure that the gamepad is picked up by the appropriate system
    app.update();
    // Ensure that the connection message is flushed through
    app.update();

    let mut input_map = InputMap::new([(MyAction::Jump, GamepadButton::South)]);
    input_map.set_gamepad(preferred_gamepad);
    app.insert_resource(input_map);
    app.init_resource::<ActionState<MyAction>>();

    // When we press the Jump button...
    let mut messages = app.world_mut().resource_mut::<Messages<RawGamepadEvent>>();
    messages.write(jump_button_press_message(preferred_gamepad));
    app.update();

    // ... We should receive a Jump action!
    let action_state = app.world_mut().resource_mut::<ActionState<MyAction>>();
    assert!(action_state.pressed(&MyAction::Jump));
}

#[test]
fn filters_out_other_gamepads() {
    let mut app = create_test_app();

    let preferred_gamepad = app.world_mut().spawn(()).id();
    let other_gamepad = app.world_mut().spawn(()).id();

    let mut input_map = InputMap::new([(MyAction::Jump, GamepadButton::South)]);
    input_map.set_gamepad(preferred_gamepad);
    app.insert_resource(input_map);
    app.init_resource::<ActionState<MyAction>>();

    // When we press the Jump button...
    let mut messages = app.world_mut().resource_mut::<Messages<RawGamepadEvent>>();
    messages.write(jump_button_press_message(other_gamepad));
    app.update();

    // ... We should receive a Jump action!
    let action_state = app.world_mut().resource_mut::<ActionState<MyAction>>();
    assert!(action_state.released(&MyAction::Jump));
}
