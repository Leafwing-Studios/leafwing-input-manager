use bevy::input::gamepad::{
    GamepadAxisChangedEvent, GamepadConnection, GamepadConnectionEvent, GamepadEvent, GamepadInfo,
};
use bevy::input::InputPlugin;
use bevy::prelude::*;
use leafwing_input_manager::axislike::{AxisType, DeadZoneShape, DualAxisData};
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, Clone, Copy, Debug, Reflect, PartialEq, Eq, Hash)]
enum ButtonlikeTestAction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Actionlike, Clone, Copy, Debug, Reflect, PartialEq, Eq, Hash)]
enum AxislikeTestAction {
    X,
    Y,
    XY,
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
    let mut gamepad_events = app.world.resource_mut::<Events<GamepadEvent>>();
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
fn raw_gamepad_axis_events() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([(
        ButtonlikeTestAction::Up,
        SingleAxis::symmetric(GamepadAxisType::RightStickX, 0.1),
    )]));

    let mut events = app.world.resource_mut::<Events<GamepadEvent>>();
    events.send(GamepadEvent::Axis(GamepadAxisChangedEvent {
        gamepad: Gamepad { id: 1 },
        axis_type: GamepadAxisType::RightStickX,
        value: 1.0,
    }));

    app.update();
    let action_state = app.world.resource::<ActionState<ButtonlikeTestAction>>();
    assert!(action_state.pressed(&ButtonlikeTestAction::Up));
}

#[test]
#[ignore = "Broken upstream; tracked in https://github.com/Leafwing-Studios/leafwing-input-manager/issues/419"]
fn game_pad_single_axis_mocking() {
    let mut app = test_app();
    let mut events = app.world.resource_mut::<Events<GamepadEvent>>();
    assert_eq!(events.drain().count(), 0);

    let input = SingleAxis {
        axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickX),
        value: Some(-1.),
        positive_low: 0.0,
        negative_low: 0.0,
        sensitivity: 1.0,
        inverted: false,
    };

    app.send_input(input);
    let mut events = app.world.resource_mut::<Events<GamepadEvent>>();
    assert_eq!(events.drain().count(), 1);
}

#[test]
#[ignore = "Broken upstream; tracked in https://github.com/Leafwing-Studios/leafwing-input-manager/issues/419"]
fn game_pad_dual_axis_mocking() {
    let mut app = test_app();
    let mut events = app.world.resource_mut::<Events<GamepadEvent>>();
    assert_eq!(events.drain().count(), 0);

    let input = DualAxis {
        x: SingleAxis {
            axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickX),
            value: Some(1.),
            positive_low: 0.0,
            negative_low: 0.0,
            sensitivity: 1.0,
            inverted: false,
        },
        y: SingleAxis {
            axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickY),
            value: Some(0.),
            positive_low: 0.0,
            negative_low: 0.0,
            sensitivity: 1.0,
            inverted: false,
        },
        deadzone: DualAxis::DEFAULT_DEADZONE_SHAPE,
    };
    app.send_input(input);
    let mut events = app.world.resource_mut::<Events<GamepadEvent>>();
    // Dual axis events are split out
    assert_eq!(events.drain().count(), 2);
}

#[test]
fn game_pad_single_axis() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([
        (
            AxislikeTestAction::X,
            SingleAxis::symmetric(GamepadAxisType::LeftStickX, 0.1),
        ),
        (
            AxislikeTestAction::Y,
            SingleAxis::symmetric(GamepadAxisType::LeftStickY, 0.1),
        ),
    ]));

    // +X
    let input = SingleAxis {
        axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickX),
        value: Some(1.),
        positive_low: 0.0,
        negative_low: 0.0,
        inverted: false,
        sensitivity: 1.0,
    };
    app.send_input(input);
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(&AxislikeTestAction::X));

    // -X
    let input = SingleAxis {
        axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickX),
        value: Some(-1.),
        positive_low: 0.0,
        negative_low: 0.0,
        inverted: false,
        sensitivity: 1.0,
    };
    app.send_input(input);
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(&AxislikeTestAction::X));

    // +Y
    let input = SingleAxis {
        axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickY),
        value: Some(1.),
        positive_low: 0.0,
        negative_low: 0.0,
        inverted: false,
        sensitivity: 1.0,
    };
    app.send_input(input);
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(&AxislikeTestAction::Y));

    // -Y
    let input = SingleAxis {
        axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickY),
        value: Some(-1.),
        positive_low: 0.0,
        negative_low: 0.0,
        inverted: false,
        sensitivity: 1.0,
    };
    app.send_input(input);
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(&AxislikeTestAction::Y));

    // 0
    let input = SingleAxis {
        axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickY),
        value: Some(0.0),
        // Usually a small deadzone threshold will be set
        positive_low: 0.1,
        negative_low: 0.1,
        inverted: false,
        sensitivity: 1.0,
    };
    app.send_input(input);
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(!action_state.pressed(&AxislikeTestAction::Y));

    // None
    let input = SingleAxis {
        axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickY),
        value: None,
        positive_low: 0.0,
        negative_low: 0.0,
        inverted: false,
        sensitivity: 1.0,
    };
    app.send_input(input);
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(!action_state.pressed(&AxislikeTestAction::Y));

    // Scaled value
    let input = SingleAxis {
        axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickX),
        value: Some(0.2),
        positive_low: 0.1,
        negative_low: 0.1,
        inverted: false,
        sensitivity: 1.0,
    };
    app.send_input(input);
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(&AxislikeTestAction::X));
    assert!(action_state.value(&AxislikeTestAction::X) == 0.11111112);
}

#[test]
fn game_pad_single_axis_inverted() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([
        (
            AxislikeTestAction::X,
            SingleAxis::symmetric(GamepadAxisType::LeftStickX, 0.1).inverted(),
        ),
        (
            AxislikeTestAction::Y,
            SingleAxis::symmetric(GamepadAxisType::LeftStickY, 0.1).inverted(),
        ),
    ]));

    // +X
    let input = SingleAxis {
        axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickX),
        value: Some(1.),
        positive_low: 0.0,
        negative_low: 0.0,
        inverted: true,
        sensitivity: -1.0,
    }
    .inverted();
    app.send_input(input);
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(&AxislikeTestAction::X));
    assert!(action_state.value(&AxislikeTestAction::X) == -1.0);

    // -X
    let input = SingleAxis {
        axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickX),
        value: Some(-1.),
        positive_low: 0.0,
        negative_low: 0.0,
        inverted: true,
        sensitivity: -1.0,
    };
    app.send_input(input);
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(&AxislikeTestAction::X));
    assert!(action_state.value(&AxislikeTestAction::X) == 1.0);

    // +Y
    let input = SingleAxis {
        axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickY),
        value: Some(1.),
        positive_low: 0.0,
        negative_low: 0.0,
        inverted: true,
        sensitivity: -1.0,
    };
    app.send_input(input);
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(&AxislikeTestAction::Y));
    assert!(action_state.value(&AxislikeTestAction::Y) == -1.0);

    // -Y
    let input = SingleAxis {
        axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickY),
        value: Some(-1.),
        positive_low: 0.0,
        negative_low: 0.0,
        inverted: true,
        sensitivity: -1.0,
    };
    app.send_input(input);
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(&AxislikeTestAction::Y));
    assert!(action_state.value(&AxislikeTestAction::Y) == 1.0);
}

#[test]
fn game_pad_dual_axis_cross() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([(
        AxislikeTestAction::XY,
        DualAxis::left_stick().with_deadzone(DeadZoneShape::Cross {
            horizontal_width: 0.1,
            vertical_width: 0.1,
        }),
    )]));

    // Test that an input inside the cross deadzone is filtered out.
    app.send_input(DualAxis::from_value(
        GamepadAxisType::LeftStickX,
        GamepadAxisType::LeftStickY,
        0.04,
        0.1,
    ));

    app.update();

    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.released(&AxislikeTestAction::XY));
    assert_eq!(action_state.value(&AxislikeTestAction::XY), 0.0);
    assert_eq!(
        action_state.axis_pair(&AxislikeTestAction::XY).unwrap(),
        DualAxisData::new(0.0, 0.0)
    );

    // Test that an input outside the cross deadzone is not filtered out.
    app.send_input(DualAxis::from_value(
        GamepadAxisType::LeftStickX,
        GamepadAxisType::LeftStickY,
        1.0,
        0.2,
    ));

    app.update();

    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(&AxislikeTestAction::XY));
    assert_eq!(action_state.value(&AxislikeTestAction::XY), 1.006_154);
    assert_eq!(
        action_state.axis_pair(&AxislikeTestAction::XY).unwrap(),
        DualAxisData::new(1.0, 0.11111112)
    );

    // Test that each axis of the cross deadzone is filtered independently.
    app.send_input(DualAxis::from_value(
        GamepadAxisType::LeftStickX,
        GamepadAxisType::LeftStickY,
        0.8,
        0.1,
    ));

    app.update();

    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(&AxislikeTestAction::XY));
    assert_eq!(action_state.value(&AxislikeTestAction::XY), 0.7777778);
    assert_eq!(
        action_state.axis_pair(&AxislikeTestAction::XY).unwrap(),
        DualAxisData::new(0.7777778, 0.0)
    );
}

#[test]
fn game_pad_dual_axis_ellipse() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([(
        AxislikeTestAction::XY,
        DualAxis::left_stick().with_deadzone(DeadZoneShape::Ellipse {
            radius_x: 0.1,
            radius_y: 0.1,
        }),
    )]));

    // Test that an input inside the ellipse deadzone is filtered out, assuming values of 0.1
    app.send_input(DualAxis::from_value(
        GamepadAxisType::LeftStickX,
        GamepadAxisType::LeftStickY,
        0.06,
        0.06,
    ));

    app.update();

    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.released(&AxislikeTestAction::XY));
    assert_eq!(action_state.value(&AxislikeTestAction::XY), 0.0);
    assert_eq!(
        action_state.axis_pair(&AxislikeTestAction::XY).unwrap(),
        DualAxisData::new(0.0, 0.0)
    );

    // Test that an input outside the ellipse deadzone is not filtered out, assuming values of 0.1
    app.send_input(DualAxis::from_value(
        GamepadAxisType::LeftStickX,
        GamepadAxisType::LeftStickY,
        0.2,
        0.0,
    ));

    app.update();

    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(&AxislikeTestAction::XY));
    assert_eq!(action_state.value(&AxislikeTestAction::XY), 0.11111112);
    assert_eq!(
        action_state.axis_pair(&AxislikeTestAction::XY).unwrap(),
        DualAxisData::new(0.11111112, 0.0)
    );
}

#[test]
fn test_zero_cross() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([(
        AxislikeTestAction::XY,
        DualAxis::left_stick().with_deadzone(DeadZoneShape::Cross {
            horizontal_width: 0.0,
            vertical_width: 0.0,
        }),
    )]));

    // Test that an input of zero will be `None` even with no deadzone.
    app.send_input(DualAxis::from_value(
        GamepadAxisType::LeftStickX,
        GamepadAxisType::LeftStickY,
        0.0,
        0.0,
    ));

    app.update();

    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.released(&AxislikeTestAction::XY));
    assert_eq!(action_state.value(&AxislikeTestAction::XY), 0.0);
    assert_eq!(
        action_state.axis_pair(&AxislikeTestAction::XY).unwrap(),
        DualAxisData::new(0.0, 0.0)
    );
}

#[test]
fn test_zero_ellipse() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([(
        AxislikeTestAction::XY,
        DualAxis::left_stick().with_deadzone(DeadZoneShape::Ellipse {
            radius_x: 0.0,
            radius_y: 0.0,
        }),
    )]));

    // Test that an input of zero will be `None` even with no deadzone.
    app.send_input(DualAxis::from_value(
        GamepadAxisType::LeftStickX,
        GamepadAxisType::LeftStickY,
        0.0,
        0.0,
    ));

    app.update();

    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.released(&AxislikeTestAction::XY));
    assert_eq!(action_state.value(&AxislikeTestAction::XY), 0.0);
    assert_eq!(
        action_state.axis_pair(&AxislikeTestAction::XY).unwrap(),
        DualAxisData::new(0.0, 0.0)
    );
}

#[test]
#[ignore = "Input mocking is subtly broken: https://github.com/Leafwing-Studios/leafwing-input-manager/issues/407"]
fn game_pad_virtualdpad() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([(
        AxislikeTestAction::XY,
        VirtualDPad::dpad(),
    )]));

    app.send_input(GamepadButtonType::DPadLeft);
    app.update();

    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();

    assert!(action_state.pressed(&AxislikeTestAction::XY));
    // This should be unit length, because we're working with a VirtualDpad
    assert_eq!(action_state.value(&AxislikeTestAction::XY), 1.0);
    assert_eq!(
        action_state.axis_pair(&AxislikeTestAction::XY).unwrap(),
        // This should be unit length, because we're working with a VirtualDpad
        DualAxisData::new(-1.0, 0.0)
    );
}
