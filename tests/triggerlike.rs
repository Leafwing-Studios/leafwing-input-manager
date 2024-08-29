use bevy::{
    input::{
        gamepad::{GamepadConnection, GamepadConnectionEvent, GamepadEvent, GamepadInfo},
        InputPlugin,
    },
    prelude::*,
};
use leafwing_input_manager::prelude::*;

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
        // Must be consistent with mocked events
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
