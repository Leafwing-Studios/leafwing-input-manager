use bevy::input::mouse::MouseMotion;
use bevy::input::InputPlugin;
use bevy::prelude::*;
use leafwing_input_manager::axislike::{AxisType, DualAxisData, MouseMotionAxisType};
use leafwing_input_manager::buttonlike::MouseMotionDirection;
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

    app
}

#[test]
fn raw_mouse_motion_events() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([(
        SingleAxis::from_value(AxisType::MouseMotion(MouseMotionAxisType::Y), 1.0),
        AxislikeTestAction::X,
    )]));

    let mut events = app.world.resource_mut::<Events<MouseMotion>>();
    events.send(MouseMotion {
        delta: Vec2::new(0.0, 1.0),
    });

    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(AxislikeTestAction::X));
}

#[test]
fn mouse_motion_discrete_mocking() {
    let mut app = test_app();
    let mut events = app.world.resource_mut::<Events<MouseMotion>>();
    assert_eq!(events.drain().count(), 0);

    app.send_input(MouseMotionDirection::Up);
    let mut events = app.world.resource_mut::<Events<MouseMotion>>();

    assert_eq!(events.drain().count(), 1);
}

#[test]
fn mouse_motion_single_axis_mocking() {
    let mut app = test_app();
    let mut events = app.world.resource_mut::<Events<MouseMotion>>();
    assert_eq!(events.drain().count(), 0);

    let input = SingleAxis {
        axis_type: AxisType::MouseMotion(MouseMotionAxisType::X),
        value: Some(-1.),
        positive_low: 0.0,
        negative_low: 0.0,
    };

    app.send_input(input);
    let mut events = app.world.resource_mut::<Events<MouseMotion>>();
    assert_eq!(events.drain().count(), 1);
}

#[test]
fn mouse_motion_dual_axis_mocking() {
    let mut app = test_app();
    let mut events = app.world.resource_mut::<Events<MouseMotion>>();
    assert_eq!(events.drain().count(), 0);

    let input = DualAxis {
        x: SingleAxis {
            axis_type: AxisType::MouseMotion(MouseMotionAxisType::X),
            value: Some(1.),
            positive_low: 0.0,
            negative_low: 0.0,
        },
        y: SingleAxis {
            axis_type: AxisType::MouseMotion(MouseMotionAxisType::Y),
            value: Some(0.),
            positive_low: 0.0,
            negative_low: 0.0,
        },
    };
    app.send_input(input);
    let mut events = app.world.resource_mut::<Events<MouseMotion>>();
    // Dual axis events are split out
    assert_eq!(events.drain().count(), 2);
}

#[test]
fn mouse_motion_buttonlike() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([
        (MouseMotionDirection::Up, ButtonlikeTestAction::Up),
        (MouseMotionDirection::Down, ButtonlikeTestAction::Down),
        (MouseMotionDirection::Left, ButtonlikeTestAction::Left),
        (MouseMotionDirection::Right, ButtonlikeTestAction::Right),
    ]));

    for action in ButtonlikeTestAction::variants() {
        let input_map = app.world.resource::<InputMap<ButtonlikeTestAction>>();
        // Get the first associated input
        let input = input_map.get(action).get_at(0).unwrap().clone();

        app.send_input(input.clone());
        app.update();

        let action_state = app.world.resource::<ActionState<ButtonlikeTestAction>>();
        assert!(action_state.pressed(action), "failed for {input:?}");
    }
}

#[test]
fn mouse_motion_buttonlike_cancels() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([
        (MouseMotionDirection::Up, ButtonlikeTestAction::Up),
        (MouseMotionDirection::Down, ButtonlikeTestAction::Down),
        (MouseMotionDirection::Left, ButtonlikeTestAction::Left),
        (MouseMotionDirection::Right, ButtonlikeTestAction::Right),
    ]));

    app.send_input(MouseMotionDirection::Up);
    app.send_input(MouseMotionDirection::Down);

    // Correctly flushes the world
    app.update();

    let action_state = app.world.resource::<ActionState<ButtonlikeTestAction>>();

    assert!(!action_state.pressed(ButtonlikeTestAction::Up));
    assert!(!action_state.pressed(ButtonlikeTestAction::Down));
}

#[test]
fn mouse_motion_single_axis() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([
        (SingleAxis::mouse_motion_x(), AxislikeTestAction::X),
        (SingleAxis::mouse_motion_y(), AxislikeTestAction::Y),
    ]));

    // +X
    let input = SingleAxis {
        axis_type: AxisType::MouseMotion(MouseMotionAxisType::X),
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
        axis_type: AxisType::MouseMotion(MouseMotionAxisType::X),
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
        axis_type: AxisType::MouseMotion(MouseMotionAxisType::Y),
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
        axis_type: AxisType::MouseMotion(MouseMotionAxisType::Y),
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
        axis_type: AxisType::MouseMotion(MouseMotionAxisType::Y),
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
        axis_type: AxisType::MouseMotion(MouseMotionAxisType::Y),
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
fn mouse_motion_dual_axis() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([(
        DualAxis::mouse_motion(),
        AxislikeTestAction::XY,
    )]));

    app.send_input(DualAxis::from_value(
        MouseMotionAxisType::X,
        MouseMotionAxisType::Y,
        5.0,
        0.0,
    ));

    app.update();

    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();

    assert!(action_state.pressed(AxislikeTestAction::XY));
    assert_eq!(action_state.value(AxislikeTestAction::XY), 5.0);
    assert_eq!(
        action_state.axis_pair(AxislikeTestAction::XY).unwrap(),
        DualAxisData::new(5.0, 0.0)
    );
}

#[test]
fn mouse_motion_virtualdpad() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([(
        VirtualDPad::mouse_motion(),
        AxislikeTestAction::XY,
    )]));

    app.send_input(DualAxis::from_value(
        MouseMotionAxisType::X,
        MouseMotionAxisType::Y,
        0.0,
        -2.0,
    ));
    app.update();

    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();

    assert!(action_state.pressed(AxislikeTestAction::XY));
    // This should be unit length, because we're working with a VirtualDpad
    assert_eq!(action_state.value(AxislikeTestAction::XY), 1.0);
    assert_eq!(
        action_state.axis_pair(AxislikeTestAction::XY).unwrap(),
        // This should be unit length, because we're working with a VirtualDpad
        DualAxisData::new(0.0, -1.0)
    );
}
