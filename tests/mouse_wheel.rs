use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::input::InputPlugin;
use bevy::prelude::*;
use leafwing_input_manager::axislike::{DualAxisData, MouseWheelAxisType};
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
        .add_plugin(InputManagerPlugin::<ButtonlikeTestAction>::default());

    app
}

#[test]
fn raw_events() {
    let mut app = test_app();
    app.init_resource::<ActionState<ButtonlikeTestAction>>();
    app.insert_resource(InputMap::new([(
        MouseWheelDirection::Up,
        ButtonlikeTestAction::Up,
    )]));

    let mut events = app.world.resource_mut::<Events<MouseWheel>>();
    events.send(MouseWheel {
        unit: MouseScrollUnit::Pixel,
        x: 0.0,
        y: 10.0,
    });

    app.update();
    let action_state = app.world.resource::<ActionState<ButtonlikeTestAction>>();
    assert!(action_state.pressed(ButtonlikeTestAction::Up));
}

#[test]
fn mouse_wheel_buttonlike() {
    let mut app = test_app();
    app.init_resource::<ActionState<ButtonlikeTestAction>>();
    app.insert_resource(InputMap::new([
        (MouseWheelDirection::Up, ButtonlikeTestAction::Up),
        (MouseWheelDirection::Down, ButtonlikeTestAction::Down),
        (MouseWheelDirection::Left, ButtonlikeTestAction::Left),
        (MouseWheelDirection::Right, ButtonlikeTestAction::Right),
    ]));

    for action in ButtonlikeTestAction::variants() {
        let input_map = app.world.resource::<InputMap<ButtonlikeTestAction>>();
        let input = input_map.get(action).get_at(0).unwrap();

        app.send_input(input.clone());
        app.update();

        let action_state = app.world.resource::<ActionState<ButtonlikeTestAction>>();
        assert!(action_state.pressed(action));
    }
}

#[test]
fn mouse_wheel_buttonlike_cancels() {
    let mut app = test_app();
    app.init_resource::<ActionState<ButtonlikeTestAction>>();
    app.insert_resource(InputMap::new([
        (MouseWheelDirection::Up, ButtonlikeTestAction::Up),
        (MouseWheelDirection::Down, ButtonlikeTestAction::Down),
        (MouseWheelDirection::Left, ButtonlikeTestAction::Left),
        (MouseWheelDirection::Right, ButtonlikeTestAction::Right),
    ]));

    // FIXME: fails because sending this as input doesn't naively work
    app.send_input(MouseWheelDirection::Up);
    app.send_input(MouseWheelDirection::Down);

    app.update();

    let action_state = app.world.resource::<ActionState<ButtonlikeTestAction>>();

    assert!(!action_state.pressed(ButtonlikeTestAction::Up));
    assert!(!action_state.pressed(ButtonlikeTestAction::Down));
}

#[test]
fn mouse_wheel_single_axis() {
    let mut app = test_app();
    app.init_resource::<ActionState<AxislikeTestAction>>();
    app.insert_resource(InputMap::new([
        (SingleAxis::mouse_wheel_x(), AxislikeTestAction::X),
        (SingleAxis::mouse_wheel_y(), AxislikeTestAction::Y),
    ]));

    for action in [AxislikeTestAction::X, AxislikeTestAction::Y] {
        let input_map = app.world.resource::<InputMap<AxislikeTestAction>>();
        let input = input_map.get(action).get_at(0).unwrap();

        // FIXME: fails because sending this as input doesn't naively work
        app.send_input(input.clone());
        app.update();

        let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
        assert!(action_state.pressed(action));
    }
}

#[test]
fn mouse_wheel_dual_axis() {
    let mut app = test_app();
    app.init_resource::<ActionState<AxislikeTestAction>>();
    app.insert_resource(InputMap::new([(
        DualAxis::mouse_wheel(),
        AxislikeTestAction::XY,
    )]));

    app.send_input(DualAxis::from_value(
        MouseWheelAxisType::X,
        MouseWheelAxisType::Y,
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
fn mouse_wheel_virtualdpad() {
    let mut app = test_app();
    app.init_resource::<ActionState<AxislikeTestAction>>();
    app.insert_resource(InputMap::new([(
        VirtualDPad::mouse_wheel(),
        AxislikeTestAction::XY,
    )]));

    app.send_input(DualAxis::from_value(
        MouseWheelAxisType::X,
        MouseWheelAxisType::Y,
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
