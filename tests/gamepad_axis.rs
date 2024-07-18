use bevy::input::gamepad::{GamepadConnection, GamepadConnectionEvent, GamepadEvent, GamepadInfo};
use bevy::input::InputPlugin;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, Clone, Copy, Debug, Reflect, PartialEq, Eq, Hash)]
enum ButtonlikeTestAction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Reflect, PartialEq, Eq, Hash)]
enum AxislikeTestAction {
    X,
    Y,
    XY,
}

impl Actionlike for AxislikeTestAction {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            AxislikeTestAction::X | AxislikeTestAction::Y => InputControlKind::Axis,
            AxislikeTestAction::XY => InputControlKind::DualAxis,
        }
    }
}

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(InputPlugin)
        .add_plugins(InputManagerPlugin::<ButtonlikeTestAction>::default())
        .add_plugins(InputManagerPlugin::<AxislikeTestAction>::default())
        .init_resource::<ActionState<ButtonlikeTestAction>>()
        .init_resource::<ActionState<AxislikeTestAction>>();

    // WARNING: you MUST register your gamepad during tests, or all gamepad input mocking will fail
    let mut gamepad_events = app.world_mut().resource_mut::<Events<GamepadEvent>>();
    gamepad_events.send(GamepadEvent::Connection(GamepadConnectionEvent {
        // This MUST be consistent with any other mocked events
        gamepad: Gamepad { id: 1 },
        connection: GamepadConnection::Connected(GamepadInfo {
            name: "TestController".into(),
        }),
    }));

    // Ensure that the gamepad is picked up by the appropriate system
    app.update();
    // Ensure that the connection event is flushed through
    app.update();

    app
}

#[test]
#[ignore = "Broken upstream; tracked in https://github.com/Leafwing-Studios/leafwing-input-manager/issues/419"]
fn gamepad_single_axis_mocking() {
    let mut app = test_app();
    let mut events = app.world_mut().resource_mut::<Events<GamepadEvent>>();
    assert_eq!(events.drain().count(), 0);

    let input = GamepadControlAxis::LEFT_X;
    app.send_axis_values(input, [-1.0]);

    let mut events = app.world_mut().resource_mut::<Events<GamepadEvent>>();
    assert_eq!(events.drain().count(), 1);
}

#[test]
#[ignore = "Broken upstream; tracked in https://github.com/Leafwing-Studios/leafwing-input-manager/issues/419"]
fn gamepad_dual_axis_mocking() {
    let mut app = test_app();
    let mut events = app.world_mut().resource_mut::<Events<GamepadEvent>>();
    assert_eq!(events.drain().count(), 0);

    let input = GamepadStick::LEFT;
    app.send_axis_values(input, [1.0, 0.0]);

    let mut events = app.world_mut().resource_mut::<Events<GamepadEvent>>();
    // Dual axis events are split out
    assert_eq!(events.drain().count(), 2);
}

#[test]
fn gamepad_single_axis() {
    let mut app = test_app();
    app.insert_resource(
        InputMap::default()
            .with_axis(
                AxislikeTestAction::X,
                GamepadControlAxis::LEFT_X.with_deadzone_symmetric(0.1),
            )
            .with_axis(
                AxislikeTestAction::Y,
                GamepadControlAxis::LEFT_Y.with_deadzone_symmetric(0.1),
            ),
    );

    // +X
    let input = GamepadControlAxis::LEFT_X;
    app.send_axis_values(input, [1.0]);
    app.update();

    // -X
    let input = GamepadControlAxis::LEFT_X;
    app.send_axis_values(input, [-1.0]);
    app.update();

    // +Y
    let input = GamepadControlAxis::LEFT_Y;
    app.send_axis_values(input, [1.0]);
    app.update();

    // -Y
    let input = GamepadControlAxis::LEFT_Y;
    app.send_axis_values(input, [-1.0]);
    app.update();

    // 0
    // Usually a small deadzone threshold will be set
    let input = GamepadControlAxis::LEFT_Y;
    app.send_axis_values(input, [0.0]);
    app.update();

    // No value
    let input = GamepadControlAxis::LEFT_Y;
    app.send_axis_values(input, []);
    app.update();

    // Scaled value
    let input = GamepadControlAxis::LEFT_X;
    app.send_axis_values(input, [0.2]);
    app.update();
    let action_state = app.world().resource::<ActionState<AxislikeTestAction>>();
    assert_eq!(action_state.value(&AxislikeTestAction::X), 0.11111112);
}

#[test]
fn gamepad_single_axis_inverted() {
    let mut app = test_app();
    app.insert_resource(
        InputMap::default()
            .with_axis(
                AxislikeTestAction::X,
                GamepadControlAxis::LEFT_X
                    .with_deadzone_symmetric(0.1)
                    .inverted(),
            )
            .with_axis(
                AxislikeTestAction::Y,
                GamepadControlAxis::LEFT_Y
                    .with_deadzone_symmetric(0.1)
                    .inverted(),
            ),
    );

    // +X
    let input = GamepadControlAxis::LEFT_X;
    app.send_axis_values(input, [1.0]);
    app.update();
    let action_state = app.world().resource::<ActionState<AxislikeTestAction>>();
    assert_eq!(action_state.value(&AxislikeTestAction::X), -1.0);

    // -X
    let input = GamepadControlAxis::LEFT_X;
    app.send_axis_values(input, [-1.0]);
    app.update();
    let action_state = app.world().resource::<ActionState<AxislikeTestAction>>();
    assert_eq!(action_state.value(&AxislikeTestAction::X), 1.0);

    // +Y
    let input = GamepadControlAxis::LEFT_Y;
    app.send_axis_values(input, [1.0]);
    app.update();
    let action_state = app.world().resource::<ActionState<AxislikeTestAction>>();
    assert_eq!(action_state.value(&AxislikeTestAction::Y), -1.0);

    // -Y
    let input = GamepadControlAxis::LEFT_Y;
    app.send_axis_values(input, [-1.0]);
    app.update();
    let action_state = app.world().resource::<ActionState<AxislikeTestAction>>();
    assert_eq!(action_state.value(&AxislikeTestAction::Y), 1.0);
}

#[test]
fn gamepad_dual_axis_deadzone() {
    let mut app = test_app();
    app.insert_resource(InputMap::default().with_dual_axis(
        AxislikeTestAction::XY,
        GamepadStick::LEFT.with_deadzone_symmetric(0.1),
    ));

    // Test that an input inside the dual-axis deadzone is filtered out.
    let input = GamepadStick::LEFT;
    app.send_axis_values(input, [0.04, 0.1]);
    app.update();

    let action_state = app.world().resource::<ActionState<AxislikeTestAction>>();
    assert_eq!(
        action_state.axis_pair(&AxislikeTestAction::XY),
        Vec2::new(0.0, 0.0)
    );

    // Test that an input outside the dual-axis deadzone is not filtered out.
    let input = GamepadStick::LEFT;
    app.send_axis_values(input, [1.0, 0.2]);
    app.update();

    let action_state = app.world().resource::<ActionState<AxislikeTestAction>>();
    assert_eq!(
        action_state.axis_pair(&AxislikeTestAction::XY),
        Vec2::new(1.0, 0.11111112)
    );

    // Test that each axis of the dual-axis deadzone is filtered independently.
    let input = GamepadStick::LEFT;
    app.send_axis_values(input, [0.8, 0.1]);
    app.update();

    let action_state = app.world().resource::<ActionState<AxislikeTestAction>>();
    assert_eq!(
        action_state.axis_pair(&AxislikeTestAction::XY),
        Vec2::new(0.7777778, 0.0)
    );
}

#[test]
fn gamepad_circle_deadzone() {
    let mut app = test_app();
    app.insert_resource(InputMap::default().with_dual_axis(
        AxislikeTestAction::XY,
        GamepadStick::LEFT.with_circle_deadzone(0.1),
    ));

    // Test that an input inside the circle deadzone is filtered out, assuming values of 0.1
    let input = GamepadStick::LEFT;
    app.send_axis_values(input, [0.06, 0.06]);
    app.update();

    let action_state = app.world().resource::<ActionState<AxislikeTestAction>>();
    assert_eq!(
        action_state.axis_pair(&AxislikeTestAction::XY),
        Vec2::new(0.0, 0.0)
    );

    // Test that an input outside the circle deadzone is not filtered out, assuming values of 0.1
    let input = GamepadStick::LEFT;
    app.send_axis_values(input, [0.2, 0.0]);
    app.update();

    let action_state = app.world().resource::<ActionState<AxislikeTestAction>>();
    assert_eq!(
        action_state.axis_pair(&AxislikeTestAction::XY),
        Vec2::new(0.11111112, 0.0)
    );
}

#[test]
fn test_zero_dual_axis_deadzone() {
    let mut app = test_app();
    app.insert_resource(InputMap::default().with_dual_axis(
        AxislikeTestAction::XY,
        GamepadStick::LEFT.with_deadzone_symmetric(0.0),
    ));

    // Test that an input of zero will be `None` even with no deadzone.
    let input = GamepadStick::LEFT;
    app.send_axis_values(input, [0.0, 0.0]);
    app.update();

    let action_state = app.world().resource::<ActionState<AxislikeTestAction>>();
    assert_eq!(
        action_state.axis_pair(&AxislikeTestAction::XY),
        Vec2::new(0.0, 0.0)
    );
}

#[test]
fn test_zero_circle_deadzone() {
    let mut app = test_app();
    app.insert_resource(InputMap::default().with_dual_axis(
        AxislikeTestAction::XY,
        GamepadStick::LEFT.with_circle_deadzone(0.0),
    ));

    // Test that an input of zero will be `None` even with no deadzone.
    let input = GamepadStick::LEFT;
    app.send_axis_values(input, [0.0, 0.0]);
    app.update();

    let action_state = app.world().resource::<ActionState<AxislikeTestAction>>();
    assert_eq!(
        action_state.axis_pair(&AxislikeTestAction::XY),
        Vec2::new(0.0, 0.0)
    );
}

#[test]
#[ignore = "Input mocking is subtly broken: https://github.com/Leafwing-Studios/leafwing-input-manager/issues/516"]
fn gamepad_virtual_dpad() {
    let mut app = test_app();
    app.insert_resource(
        InputMap::default().with_dual_axis(AxislikeTestAction::XY, GamepadVirtualDPad::DPAD),
    );

    app.press_input(GamepadButtonType::DPadLeft);
    app.update();

    let action_state = app.world().resource::<ActionState<AxislikeTestAction>>();

    // This should be a unit length, because we're working with a VirtualDPad
    assert_eq!(
        action_state.axis_pair(&AxislikeTestAction::XY),
        // This should be a unit length, because we're working with a VirtualDPad
        Vec2::new(-1.0, 0.0)
    );
}
