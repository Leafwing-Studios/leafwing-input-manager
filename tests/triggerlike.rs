use bevy::{
    input::{
        gamepad::{GamepadConnection, GamepadConnectionEvent, GamepadEvent, GamepadInfo},
        InputPlugin,
    },
    prelude::*,
};
use leafwing_input_manager::prelude::*;
use updating::CentralInputStore;

#[derive(Actionlike, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect)]
enum TestAction {
    #[actionlike(Trigger)]
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
        .with_trigger(TestAction::Throttle, GamepadButtonType::South)
        .with_trigger(TestAction::Throttle, GamepadButtonType::RightTrigger);

    input_map
        .insert_trigger(TestAction::Throttle, KeyCode::Space)
        .insert_trigger(TestAction::Throttle, MouseButton::Left);

    app.insert_resource(input_map)
        .init_resource::<ActionState<TestAction>>();

    let mut gamepad_events: Mut<'_, Events<GamepadEvent>> =
        app.world_mut().resource_mut::<Events<GamepadEvent>>();
    gamepad_events.send(GamepadEvent::Connection(GamepadConnectionEvent {
        gamepad: Gamepad { id: 1 },
        connection: GamepadConnection::Connected(GamepadInfo {
            name: "FirstController".into(),
        }),
    }));

    // Ensure the gamepads are picked up
    app.update();
    // Flush the gamepad connection events
    app.update();

    app
}

#[test]
fn set_trigger_updates_central_input_store() {
    let mut app = test_app();

    let gamepad_trigger = GamepadButtonType::RightTrigger;
    gamepad_trigger.set_trigger_value(app.world_mut(), 0.7);

    let keycode = KeyCode::Space;
    keycode.set_trigger_value(app.world_mut(), 1.0);

    app.update();

    let central_input_store = app.world().resource::<CentralInputStore>();
    assert_eq!(
        central_input_store.trigger_value(&GamepadButtonType::RightTrigger),
        0.7
    );
    assert_eq!(central_input_store.trigger_value(&KeyCode::Space), 1.0);
    assert_eq!(central_input_store.pressed(&KeyCode::Space), true);
}

#[test]
fn gamepad_button_triggerlike() {
    let mut app = test_app();

    let action_state = app.world().resource::<ActionState<TestAction>>();
    let trigger_value = action_state.trigger_value(&TestAction::Throttle);
    assert_eq!(trigger_value, 0.0);

    let relevant_button = GamepadButtonType::South;
    relevant_button.press(app.world_mut());

    app.update();

    let action_state = app.world().resource::<ActionState<TestAction>>();
    let trigger_value = action_state.trigger_value(&TestAction::Throttle);
    assert_eq!(trigger_value, 1.0);
}

#[test]
fn mouse_button_triggerlike() {
    let mut app = test_app();

    let action_state = app.world().resource::<ActionState<TestAction>>();
    let trigger_value = action_state.trigger_value(&TestAction::Throttle);
    assert_eq!(trigger_value, 0.0);

    let relevant_button = MouseButton::Left;
    relevant_button.press(app.world_mut());

    app.update();

    let action_state = app.world().resource::<ActionState<TestAction>>();
    let trigger_value = action_state.trigger_value(&TestAction::Throttle);
    assert_eq!(trigger_value, 1.0);
}

#[test]
fn keyboard_button_triggerlike() {
    let mut app = test_app();

    let action_state = app.world().resource::<ActionState<TestAction>>();
    let trigger_value = action_state.trigger_value(&TestAction::Throttle);
    assert_eq!(trigger_value, 0.0);

    let relevant_button = KeyCode::Space;
    relevant_button.press(app.world_mut());

    app.update();

    let action_state = app.world().resource::<ActionState<TestAction>>();
    let trigger_value = action_state.trigger_value(&TestAction::Throttle);
    assert_eq!(trigger_value, 1.0);
}

#[test]
fn gamepad_trigger() {
    let mut app = test_app();

    let action_state = app.world().resource::<ActionState<TestAction>>();
    let trigger_value = action_state.trigger_value(&TestAction::Throttle);
    assert_eq!(trigger_value, 0.0);

    let gamepad_trigger = GamepadButtonType::RightTrigger;
    gamepad_trigger.set_trigger_value(app.world_mut(), 0.7);

    app.update();

    let action_state = app.world().resource::<ActionState<TestAction>>();
    let trigger_value = action_state.trigger_value(&TestAction::Throttle);
    assert_eq!(trigger_value, 0.7);
}

#[test]
fn triggerlike_actions_can_be_pressed_and_released_when_pressed() {
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
fn triggerlike_actions_can_be_pressed_and_released_when_trigger_value_set() {
    let mut app = test_app();

    let gamepad_trigger = GamepadButtonType::RightTrigger;
    gamepad_trigger.set_trigger_value(app.world_mut(), 1.0);
    app.update();

    let action_state = app.world().resource::<ActionState<TestAction>>();
    assert!(action_state.just_pressed(&TestAction::Throttle));
    assert!(action_state.pressed(&TestAction::Throttle));
    assert!(!action_state.released(&TestAction::Throttle));
    assert!(!action_state.just_released(&TestAction::Throttle));

    let gamepad_trigger = GamepadButtonType::RightTrigger;
    gamepad_trigger.set_trigger_value(app.world_mut(), 0.0);
    app.update();

    let action_state = app.world().resource::<ActionState<TestAction>>();
    assert!(!action_state.just_pressed(&TestAction::Throttle));
    assert!(!action_state.pressed(&TestAction::Throttle));
    assert!(action_state.just_released(&TestAction::Throttle));
    assert!(action_state.released(&TestAction::Throttle));
}
