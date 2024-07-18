use bevy::input::mouse::MouseMotion;
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

    app.press_input(MouseMoveDirection::UP);
    let mut events = app.world_mut().resource_mut::<Events<MouseMotion>>();

    assert_eq!(events.drain().count(), 1);
}

#[test]
fn mouse_move_single_axis_mocking() {
    let mut app = test_app();
    let mut events = app.world_mut().resource_mut::<Events<MouseMotion>>();
    assert_eq!(events.drain().count(), 0);

    let input = MouseMoveAxis::X;
    app.send_axis_values(input, [-1.0]);

    let mut events = app.world_mut().resource_mut::<Events<MouseMotion>>();
    assert_eq!(events.drain().count(), 1);
}

#[test]
fn mouse_move_dual_axis_mocking() {
    let mut app = test_app();
    let mut events = app.world_mut().resource_mut::<Events<MouseMotion>>();
    assert_eq!(events.drain().count(), 0);

    let input = MouseMove::default();
    app.send_axis_values(input, [1.0, 0.0]);

    let mut events = app.world_mut().resource_mut::<Events<MouseMotion>>();
    // Dual axis events are split out
    assert_eq!(events.drain().count(), 2);
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

        app.press_input(*direction);
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

    app.press_input(MouseMoveDirection::UP);
    app.press_input(MouseMoveDirection::DOWN);

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
    app.send_axis_values(input, [1.0]);
    app.update();

    // -X
    let input = MouseMoveAxis::X;
    app.send_axis_values(input, [-1.0]);
    app.update();

    // +Y
    let input = MouseMoveAxis::Y;
    app.send_axis_values(input, [-1.0]);
    app.update();

    // -Y
    let input = MouseMoveAxis::Y;
    app.send_axis_values(input, [-1.0]);
    app.update();

    // 0
    let input = MouseMoveAxis::Y;
    app.send_axis_values(input, [0.0]);
    app.update();

    // No value
    let input = MouseMoveAxis::Y;
    app.send_axis_values(input, []);
    app.update();
}

#[test]
fn mouse_move_dual_axis() {
    let mut app = test_app();
    app.insert_resource(
        InputMap::default().with_dual_axis(AxislikeTestAction::XY, MouseMove::default()),
    );

    let input = MouseMove::default();
    app.send_axis_values(input, [5.0, 0.0]);
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
    app.send_axis_values(input, [0.0, -2.0]);
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
    app.send_axis_values(input, [5.0, 0.0]);
    app.press_input(MouseButton::Right);
    app.update();

    let action_state = app.world().resource::<ActionState<AxislikeTestAction>>();

    assert_eq!(
        action_state.axis_pair(&AxislikeTestAction::XY),
        Vec2::new(5.0, 0.0)
    );
}
