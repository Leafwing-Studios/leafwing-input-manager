#![cfg(all(feature = "gamepad", feature = "keyboard", feature = "mouse"))]

use bevy::{
    input::{
        InputPlugin,
        gamepad::{
            GamepadConnection, GamepadConnectionEvent, RawGamepadButtonChangedEvent,
            RawGamepadEvent,
        },
    },
    prelude::*,
};
use leafwing_input_manager::prelude::*;
use updating::CentralInputStore;

#[derive(Actionlike, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect)]
enum TestAction {
    Throttle,
}

struct TestContext {
    pub app: App,
}

impl TestContext {
    pub fn new() -> Self {
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

        app.update();
        app.update();

        Self { app }
    }

    pub fn send_gamepad_connection_event(&mut self, gamepad: Option<Entity>) -> Entity {
        let gamepad = gamepad.unwrap_or_else(|| self.app.world_mut().spawn_empty().id());
        self.app
            .world_mut()
            .resource_mut::<Messages<GamepadConnectionEvent>>()
            .write(GamepadConnectionEvent::new(
                gamepad,
                GamepadConnection::Connected {
                    name: "TestController".to_string(),
                    vendor_id: None,
                    product_id: None,
                },
            ));

        self.app.update();
        gamepad
    }

    pub fn update(&mut self) {
        self.app.update();
    }

    pub fn send_raw_gamepad_event(&mut self, event: RawGamepadEvent) {
        self.app
            .world_mut()
            .resource_mut::<Messages<RawGamepadEvent>>()
            .write(event);
    }
}

#[test]
fn set_gamepad_updates_central_input_store() {
    let mut ctx = TestContext::new();
    let gamepad = ctx.send_gamepad_connection_event(None);
    let gamepad_trigger = GamepadButton::RightTrigger;
    ctx.send_raw_gamepad_event(RawGamepadEvent::Button(RawGamepadButtonChangedEvent {
        gamepad,
        button: gamepad_trigger,
        value: 0.8,
    }));
    ctx.update();

    let central_input_store = ctx.app.world().resource::<CentralInputStore>();
    assert_eq!(central_input_store.button_value(&gamepad_trigger), 0.8);
    assert!(central_input_store.pressed(&gamepad_trigger).unwrap());
}

#[test]
fn set_keyboard_updates_central_input_store() {
    let mut app = TestContext::new().app;

    let keycode = KeyCode::Space;
    keycode.set_value(app.world_mut(), 1.0);

    app.update();

    let central_input_store = app.world().resource::<CentralInputStore>();

    assert_eq!(central_input_store.button_value(&keycode), 1.0);
    assert!(central_input_store.pressed(&keycode).unwrap());
}

#[test]
fn gamepad_button_value() {
    let mut ctx = TestContext::new();
    let gamepad = ctx.send_gamepad_connection_event(None);
    let mut app = ctx.app;

    let gamepad_button: GamepadButton = GamepadButton::South;
    gamepad_button.set_value_as_gamepad(app.world_mut(), 1.0, Some(gamepad));
    app.update();

    let action_state = app.world().resource::<ActionState<TestAction>>();
    let button_value = action_state.button_value(&TestAction::Throttle);
    assert_eq!(button_value, 1.0);
}

#[test]
fn mouse_button_value() {
    let mut app = TestContext::new().app;

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
    let mut app = TestContext::new().app;

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
fn gamepad_trigger() {
    let mut ctx = TestContext::new();
    let gamepad = ctx.send_gamepad_connection_event(None);
    let mut app = ctx.app;

    let action_state = app.world().resource::<ActionState<TestAction>>();
    let button_value = action_state.button_value(&TestAction::Throttle);
    assert_eq!(button_value, 0.0);

    let gamepad_trigger = GamepadButton::RightTrigger;
    gamepad_trigger.set_value_as_gamepad(app.world_mut(), 0.7, Some(gamepad));
    app.update();

    let action_state = app.world().resource::<ActionState<TestAction>>();
    let button_value = action_state.button_value(&TestAction::Throttle);
    assert_eq!(button_value, 0.7);
}

#[test]
fn buttonlike_actions_can_be_pressed_and_released_when_pressed() {
    let mut app = TestContext::new().app;

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
    let mut ctx = TestContext::new();
    let gamepad = ctx.send_gamepad_connection_event(None);
    let mut app = ctx.app;

    let gamepad_trigger = GamepadButton::RightTrigger;
    gamepad_trigger.set_value_as_gamepad(app.world_mut(), 1.0, Some(gamepad));
    app.update();

    let action_state = app.world().resource::<ActionState<TestAction>>();
    assert!(action_state.just_pressed(&TestAction::Throttle));
    assert!(action_state.pressed(&TestAction::Throttle));
    assert!(!action_state.released(&TestAction::Throttle));
    assert!(!action_state.just_released(&TestAction::Throttle));

    let gamepad_trigger = GamepadButton::RightTrigger;
    gamepad_trigger.set_value_as_gamepad(app.world_mut(), 0.0, Some(gamepad));
    app.update();

    let action_state = app.world().resource::<ActionState<TestAction>>();
    assert!(!action_state.just_pressed(&TestAction::Throttle));
    assert!(!action_state.pressed(&TestAction::Throttle));
    assert!(action_state.just_released(&TestAction::Throttle));
    assert!(action_state.released(&TestAction::Throttle));
}
