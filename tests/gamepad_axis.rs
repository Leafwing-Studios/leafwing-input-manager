use bevy::input::gamepad::GamepadEventRaw;
use bevy::input::InputPlugin;
use bevy::prelude::*;
use leafwing_input_manager::axislike::{AxisType, DualAxisData};
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, Clone, Copy, Debug)]
enum ButtonlikeTestAction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Actionlike, Clone, Copy, Debug)]
enum AxislikeTestAction {
    X,
    Y,
    XY,
}

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugin(InputPlugin)
        .add_plugin(InputManagerPlugin::<ButtonlikeTestAction>::default())
        .add_plugin(InputManagerPlugin::<AxislikeTestAction>::default())
        .init_resource::<ActionState<ButtonlikeTestAction>>()
        .init_resource::<ActionState<AxislikeTestAction>>();

    // WARNING: you MUST register your gamepad during tests, or all gamepad input mocking will fail
    let mut gamepad_events = app.world.resource_mut::<Events<GamepadEventRaw>>();
    gamepad_events.send(GamepadEventRaw {
        // This MUST be consistent with any other mocked events
        gamepad: Gamepad { id: 1 },
        event_type: GamepadEventType::Connected,
    });

    // Ensure that the gamepad is picked up by the appropriate system
    app.update();
    // Ensure that the connection event is flushed through
    app.update();

    app
}

#[test]
fn raw_gamepad_axis_events() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([(
        SingleAxis::symmetric(GamepadAxisType::RightStickX, 0.1),
        ButtonlikeTestAction::Up,
    )]));

    let mut events = app.world.resource_mut::<Events<GamepadEventRaw>>();
    events.send(GamepadEventRaw {
        gamepad: Gamepad { id: 1 },
        event_type: GamepadEventType::AxisChanged(GamepadAxisType::RightStickX, 1.0),
    });

    app.update();
    let action_state = app.world.resource::<ActionState<ButtonlikeTestAction>>();
    assert!(action_state.pressed(ButtonlikeTestAction::Up));
}

#[test]
fn game_pad_single_axis_mocking() {
    let mut app = test_app();
    let mut events = app.world.resource_mut::<Events<GamepadEventRaw>>();
    assert_eq!(events.drain().count(), 0);

    let input = SingleAxis {
        axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickX),
        value: Some(-1.),
        positive_low: 0.0,
        negative_low: 0.0,
    };

    app.send_input(input);
    let mut events = app.world.resource_mut::<Events<GamepadEventRaw>>();
    assert_eq!(events.drain().count(), 1);
}

#[test]
fn game_pad_dual_axis_mocking() {
    let mut app = test_app();
    let mut events = app.world.resource_mut::<Events<GamepadEventRaw>>();
    assert_eq!(events.drain().count(), 0);

    let input = DualAxis {
        x: SingleAxis {
            axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickX),
            value: Some(1.),
            positive_low: 0.0,
            negative_low: 0.0,
        },
        y: SingleAxis {
            axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickY),
            value: Some(0.),
            positive_low: 0.0,
            negative_low: 0.0,
        },
    };
    app.send_input(input);
    let mut events = app.world.resource_mut::<Events<GamepadEventRaw>>();
    // Dual axis events are split out
    assert_eq!(events.drain().count(), 2);
}

#[test]
fn game_pad_single_axis() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([
        (
            SingleAxis::symmetric(GamepadAxisType::LeftStickX, 0.1),
            AxislikeTestAction::X,
        ),
        (
            SingleAxis::symmetric(GamepadAxisType::LeftStickY, 0.1),
            AxislikeTestAction::Y,
        ),
    ]));

    // +X
    let input = SingleAxis {
        axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickX),
        value: Some(1.),
        positive_low: 0.0,
        negative_low: 0.0,
    };
    app.send_input(input);
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(AxislikeTestAction::X));

    // -X
    let input = SingleAxis {
        axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickX),
        value: Some(-1.),
        positive_low: 0.0,
        negative_low: 0.0,
    };
    app.send_input(input);
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(AxislikeTestAction::X));

    // +Y
    let input = SingleAxis {
        axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickY),
        value: Some(1.),
        positive_low: 0.0,
        negative_low: 0.0,
    };
    app.send_input(input);
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(AxislikeTestAction::Y));

    // -Y
    let input = SingleAxis {
        axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickY),
        value: Some(-1.),
        positive_low: 0.0,
        negative_low: 0.0,
    };
    app.send_input(input);
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(AxislikeTestAction::Y));

    // 0
    let input = SingleAxis {
        axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickY),
        value: Some(0.0),
        // Usually a small deadzone threshold will be set
        positive_low: 0.1,
        negative_low: 0.1,
    };
    app.send_input(input);
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(!action_state.pressed(AxislikeTestAction::Y));

    // None
    let input = SingleAxis {
        axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickY),
        value: None,
        positive_low: 0.0,
        negative_low: 0.0,
    };
    app.send_input(input);
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(!action_state.pressed(AxislikeTestAction::Y));
}

#[test]
fn game_pad_dual_axis() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([(
        DualAxis::left_stick(),
        AxislikeTestAction::XY,
    )]));

    app.send_input(DualAxis::from_value(
        GamepadAxisType::LeftStickX,
        GamepadAxisType::LeftStickY,
        0.8,
        0.0,
    ));

    app.update();

    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();

    assert!(action_state.pressed(AxislikeTestAction::XY));
    assert_eq!(action_state.value(AxislikeTestAction::XY), 0.8);
    assert_eq!(
        action_state.axis_pair(AxislikeTestAction::XY).unwrap(),
        DualAxisData::new(0.8, 0.0)
    );
}

#[test]
fn game_pad_virtualdpad() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([(
        VirtualDPad::dpad(),
        AxislikeTestAction::XY,
    )]));

    app.send_input(GamepadButtonType::DPadLeft);
    app.update();

    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();

    assert!(action_state.pressed(AxislikeTestAction::XY));
    // This should be unit length, because we're working with a VirtualDpad
    assert_eq!(action_state.value(AxislikeTestAction::XY), 1.0);
    assert_eq!(
        action_state.axis_pair(AxislikeTestAction::XY).unwrap(),
        // This should be unit length, because we're working with a VirtualDpad
        DualAxisData::new(-1.0, 0.0)
    );
}
