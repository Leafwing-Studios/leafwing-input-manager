#![cfg(feature = "mouse")]

use bevy::input::mouse::MouseMotion;
use bevy::input::InputPlugin;
use bevy::prelude::*;
use leafwing_input_manager::input_processing::WithDualAxisProcessingPipelineExt;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, Clone, Copy, Debug, Reflect, PartialEq, Eq, Hash)]
enum ButtonlikeTestAction {
    Up,
    Down,
    Left,
    Right,
}

impl ButtonlikeTestAction {
    fn variants() -> &'static [ButtonlikeTestAction] {
        &[
            ButtonlikeTestAction::Up,
            ButtonlikeTestAction::Down,
            ButtonlikeTestAction::Left,
            ButtonlikeTestAction::Right,
        ]
    }
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

    app
}

#[test]
fn mouse_move_discrete_mocking() {
    let mut app = test_app();
    let mut events = app.world_mut().resource_mut::<Events<MouseMotion>>();
    assert_eq!(events.drain().count(), 0);

    MouseMoveDirection::UP.press(app.world_mut());
    let mut events = app.world_mut().resource_mut::<Events<MouseMotion>>();

    assert_eq!(events.drain().count(), 1);
}

#[test]
fn mouse_move_single_axis_mocking() {
    let mut app = test_app();
    let mut events = app.world_mut().resource_mut::<Events<MouseMotion>>();
    assert_eq!(events.drain().count(), 0);

    let input = MouseMoveAxis::X;
    input.set_value(app.world_mut(), -1.0);

    let mut events = app.world_mut().resource_mut::<Events<MouseMotion>>();
    assert_eq!(events.drain().count(), 1);
}

#[test]
fn mouse_move_buttonlike() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([
        (ButtonlikeTestAction::Up, MouseMoveDirection::UP),
        (ButtonlikeTestAction::Down, MouseMoveDirection::DOWN),
        (ButtonlikeTestAction::Left, MouseMoveDirection::LEFT),
        (ButtonlikeTestAction::Right, MouseMoveDirection::RIGHT),
    ]));

    for action in ButtonlikeTestAction::variants() {
        let input_map = app.world().resource::<InputMap<ButtonlikeTestAction>>();
        // Get the first associated input
        let input = input_map
            .get_buttonlike(action)
            .unwrap()
            .first()
            .unwrap()
            .clone();
        let direction = Reflect::as_any(input.as_ref())
            .downcast_ref::<MouseMoveDirection>()
            .unwrap();

        direction.press(app.world_mut());
        app.update();

        let action_state = app.world().resource::<ActionState<ButtonlikeTestAction>>();
        assert!(action_state.pressed(action), "failed for {input:?}");
    }
}

#[test]
fn mouse_move_buttonlike_cancels() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([
        (ButtonlikeTestAction::Up, MouseMoveDirection::UP),
        (ButtonlikeTestAction::Down, MouseMoveDirection::DOWN),
        (ButtonlikeTestAction::Left, MouseMoveDirection::LEFT),
        (ButtonlikeTestAction::Right, MouseMoveDirection::RIGHT),
    ]));

    MouseMoveDirection::UP.press(app.world_mut());
    MouseMoveDirection::DOWN.press(app.world_mut());

    // Correctly flushes the world
    app.update();

    let action_state = app.world().resource::<ActionState<ButtonlikeTestAction>>();

    assert!(!action_state.pressed(&ButtonlikeTestAction::Up));
    assert!(!action_state.pressed(&ButtonlikeTestAction::Down));
}

#[test]
fn mouse_move_single_axis() {
    let mut app = test_app();
    app.insert_resource(
        InputMap::default()
            .with_axis(AxislikeTestAction::X, MouseMoveAxis::X)
            .with_axis(AxislikeTestAction::Y, MouseMoveAxis::Y),
    );

    // +X
    let input = MouseMoveAxis::X;
    input.set_value(app.world_mut(), 1.0);
    app.update();

    // -X
    let input = MouseMoveAxis::X;
    input.set_value(app.world_mut(), -1.0);
    app.update();

    // +Y
    let input = MouseMoveAxis::Y;
    input.set_value(app.world_mut(), 1.0);
    app.update();

    // -Y
    let input = MouseMoveAxis::Y;
    input.set_value(app.world_mut(), -1.0);
    app.update();

    // 0
    let input = MouseMoveAxis::Y;
    input.set_value(app.world_mut(), 0.0);
    app.update();
}

#[test]
fn mouse_move_dual_axis() {
    let mut app = test_app();
    app.insert_resource(
        InputMap::default().with_dual_axis(AxislikeTestAction::XY, MouseMove::default()),
    );

    let input = MouseMove::default();
    input.set_axis_pair(app.world_mut(), Vec2::new(5.0, 0.0));
    app.update();

    let action_state = app.world().resource::<ActionState<AxislikeTestAction>>();

    assert_eq!(
        action_state.axis_pair(&AxislikeTestAction::XY),
        Vec2::new(5.0, 0.0)
    );
}

#[test]
fn mouse_move_discrete() {
    let mut app = test_app();
    app.insert_resource(
        InputMap::default().with_dual_axis(AxislikeTestAction::XY, MouseMove::default().digital()),
    );

    let input = MouseMove::default();
    input.set_axis_pair(app.world_mut(), Vec2::new(0.0, -2.0));
    app.update();

    let action_state = app.world().resource::<ActionState<AxislikeTestAction>>();

    assert_eq!(
        action_state.axis_pair(&AxislikeTestAction::XY),
        // This should be a unit length, because we're working with a VirtualDPad
        Vec2::new(0.0, -1.0)
    );
}

#[test]
fn mouse_drag() {
    let mut app = test_app();

    let mut input_map = InputMap::default();

    input_map.insert_dual_axis(
        AxislikeTestAction::XY,
        DualAxislikeChord::new(MouseButton::Right, MouseMove::default()),
    );

    app.insert_resource(input_map);

    let input = MouseMove::default();
    input.set_axis_pair(app.world_mut(), Vec2::new(5.0, 0.0));
    MouseButton::Right.press(app.world_mut());
    app.update();

    let action_state = app.world().resource::<ActionState<AxislikeTestAction>>();

    assert_eq!(
        action_state.axis_pair(&AxislikeTestAction::XY),
        Vec2::new(5.0, 0.0)
    );
}
