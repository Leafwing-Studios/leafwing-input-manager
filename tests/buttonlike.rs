#![cfg(all(feature = "gamepad", feature = "keyboard", feature = "mouse"))]

use bevy::{
    input::{
        gamepad::{GamepadConnection, GamepadConnectionEvent, GamepadEvent},
        InputPlugin,
    },
    prelude::*,
};
use leafwing_input_manager::prelude::*;
use updating::CentralInputStore;

#[derive(Actionlike, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect)]
enum TestAction {
    Throttle,
}

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        InputManagerPlugin::<TestAction>::default(),
    ));

    let mut input_map = InputMap::<TestAction>::default()
        .with(TestAction::Throttle, GamepadButton::South)
        .with(TestAction::Throttle, GamepadButton::RightTrigger);

    input_map
        .insert(TestAction::Throttle, KeyCode::Space)
        .insert(TestAction::Throttle, MouseButton::Left);

    app.insert_resource(input_map)
        .init_resource::<ActionState<TestAction>>();

    let gamepad = app.world_mut().spawn(Gamepad::default()).id();
    let mut gamepad_messages: Mut<'_, Messages<GamepadEvent>> =
        app.world_mut().resource_mut::<Messages<GamepadEvent>>();
    gamepad_messages.write(GamepadEvent::Connection(GamepadConnectionEvent {
        gamepad,
        connection: GamepadConnection::Connected {
            name: "FirstController".into(),
            product_id: None,
            vendor_id: None,
        },
    }));

    // Ensure the gamepads are picked up
    app.update();
    // Flush the gamepad connection messages
    app.update();

    app
}

#[test]
#[ignore = "Input mocking is subtly broken: https://github.com/Leafwing-Studios/leafwing-input-manager/issues/516"]
fn set_gamepad_updates_central_input_store() {
    let mut app = test_app();

    let gamepad_trigger = GamepadButton::RightTrigger;
    gamepad_trigger.set_value(app.world_mut(), 0.7);

    app.update();

    let central_input_store = app.world().resource::<CentralInputStore>();

    assert_eq!(central_input_store.button_value(&gamepad_trigger), 0.7);
    assert!(central_input_store.pressed(&gamepad_trigger).unwrap());
}

#[test]
fn set_keyboard_updates_central_input_store() {
    let mut app = test_app();

    let keycode = KeyCode::Space;
    keycode.set_value(app.world_mut(), 1.0);

    app.update();

    let central_input_store = app.world().resource::<CentralInputStore>();

    assert_eq!(central_input_store.button_value(&keycode), 1.0);
    assert!(central_input_store.pressed(&keycode).unwrap());
}

#[test]
fn gamepad_button_value() {
    let mut app = test_app();

    let action_state = app.world().resource::<ActionState<TestAction>>();
    let button_value = action_state.button_value(&TestAction::Throttle);
    assert_eq!(button_value, 0.0);

    let relevant_button = GamepadButton::South;
    relevant_button.press(app.world_mut());

    app.update();

    let action_state = app.world().resource::<ActionState<TestAction>>();
    let button_value = action_state.button_value(&TestAction::Throttle);
    assert_eq!(button_value, 1.0);
}

#[test]
fn mouse_button_value() {
    let mut app = test_app();

    let action_state = app.world().resource::<ActionState<TestAction>>();
    let button_value = action_state.button_value(&TestAction::Throttle);
    assert_eq!(button_value, 0.0);

    let relevant_button = MouseButton::Left;
    relevant_button.press(app.world_mut());

    app.update();

    let action_state = app.world().resource::<ActionState<TestAction>>();
    let button_value = action_state.button_value(&TestAction::Throttle);
    assert_eq!(button_value, 1.0);
}

#[test]
fn keyboard_button_value() {
    let mut app = test_app();

    let action_state = app.world().resource::<ActionState<TestAction>>();
    let button_value = action_state.button_value(&TestAction::Throttle);
    assert_eq!(button_value, 0.0);

    let relevant_button = KeyCode::Space;
    relevant_button.press(app.world_mut());

    app.update();

    let action_state = app.world().resource::<ActionState<TestAction>>();
    let button_value = action_state.button_value(&TestAction::Throttle);
    assert_eq!(button_value, 1.0);
}

#[test]
#[ignore = "Input mocking is subtly broken: https://github.com/Leafwing-Studios/leafwing-input-manager/issues/516"]
fn gamepad_trigger() {
    let mut app = test_app();

    let action_state = app.world().resource::<ActionState<TestAction>>();
    let button_value = action_state.button_value(&TestAction::Throttle);
    assert_eq!(button_value, 0.0);

    let gamepad_trigger = GamepadButton::RightTrigger;
    gamepad_trigger.set_value(app.world_mut(), 0.7);

    app.update();

    let action_state = app.world().resource::<ActionState<TestAction>>();
    let button_value = action_state.button_value(&TestAction::Throttle);
    assert_eq!(button_value, 0.7);
}

#[test]
fn buttonlike_actions_can_be_pressed_and_released_when_pressed() {
    let mut app = test_app();

    let relevant_button = MouseButton::Left;

    relevant_button.press(app.world_mut());
    app.update();

    let action_state = app.world().resource::<ActionState<TestAction>>();
    assert!(action_state.just_pressed(&TestAction::Throttle));
    assert!(action_state.pressed(&TestAction::Throttle));
    assert!(!action_state.released(&TestAction::Throttle));
    assert!(!action_state.just_released(&TestAction::Throttle));

    relevant_button.release(app.world_mut());
    app.update();

    let action_state = app.world().resource::<ActionState<TestAction>>();
    assert!(!action_state.just_pressed(&TestAction::Throttle));
    assert!(!action_state.pressed(&TestAction::Throttle));
    assert!(action_state.just_released(&TestAction::Throttle));
    assert!(action_state.released(&TestAction::Throttle));
}

#[test]
fn buttonlike_actions_can_be_pressed_and_released_when_button_value_set() {
    let mut app = test_app();

    let gamepad_trigger = GamepadButton::RightTrigger;
    gamepad_trigger.set_value(app.world_mut(), 1.0);
    app.update();

    let action_state = app.world().resource::<ActionState<TestAction>>();
    assert!(action_state.just_pressed(&TestAction::Throttle));
    assert!(action_state.pressed(&TestAction::Throttle));
    assert!(!action_state.released(&TestAction::Throttle));
    assert!(!action_state.just_released(&TestAction::Throttle));

    let gamepad_trigger = GamepadButton::RightTrigger;
    gamepad_trigger.set_value(app.world_mut(), 0.0);
    app.update();

    let action_state = app.world().resource::<ActionState<TestAction>>();
    assert!(!action_state.just_pressed(&TestAction::Throttle));
    assert!(!action_state.pressed(&TestAction::Throttle));
    assert!(action_state.just_released(&TestAction::Throttle));
    assert!(action_state.released(&TestAction::Throttle));
}
