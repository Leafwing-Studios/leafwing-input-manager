#![cfg(feature = "mouse")]

use bevy::input::mouse::MouseWheel;
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
        &[Self::Up, Self::Down, Self::Left, Self::Right]
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
fn mouse_scroll_discrete_mocking() {
    let mut app = test_app();
    let mut events = app.world_mut().resource_mut::<Events<MouseWheel>>();
    assert_eq!(events.drain().count(), 0);

    MouseScrollDirection::UP.press(app.world_mut());

    let mut events = app.world_mut().resource_mut::<Events<MouseWheel>>();

    assert_eq!(events.drain().count(), 1);
}

#[test]
fn mouse_scroll_single_axis_mocking() {
    let mut app = test_app();
    let mut events = app.world_mut().resource_mut::<Events<MouseWheel>>();
    assert_eq!(events.drain().count(), 0);

    let input = MouseScrollAxis::X;
    input.set_value(app.world_mut(), -1.0);

    let mut events = app.world_mut().resource_mut::<Events<MouseWheel>>();
    assert_eq!(events.drain().count(), 1);
}

#[test]
fn mouse_scroll_buttonlike() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([
        (ButtonlikeTestAction::Up, MouseScrollDirection::UP),
        (ButtonlikeTestAction::Down, MouseScrollDirection::DOWN),
        (ButtonlikeTestAction::Left, MouseScrollDirection::LEFT),
        (ButtonlikeTestAction::Right, MouseScrollDirection::RIGHT),
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
            .downcast_ref::<MouseScrollDirection>()
            .unwrap();

        direction.press(app.world_mut());
        app.update();

        let action_state = app.world().resource::<ActionState<ButtonlikeTestAction>>();
        assert!(action_state.pressed(action), "failed for {input:?}");
    }
}

#[test]
fn mouse_scroll_buttonlike_cancels() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([
        (ButtonlikeTestAction::Up, MouseScrollDirection::UP),
        (ButtonlikeTestAction::Down, MouseScrollDirection::DOWN),
        (ButtonlikeTestAction::Left, MouseScrollDirection::LEFT),
        (ButtonlikeTestAction::Right, MouseScrollDirection::RIGHT),
    ]));

    MouseScrollDirection::UP.press(app.world_mut());
    MouseScrollDirection::DOWN.press(app.world_mut());

    // Correctly flushes the world
    app.update();

    let action_state = app.world().resource::<ActionState<ButtonlikeTestAction>>();

    assert!(!action_state.pressed(&ButtonlikeTestAction::Up));
    assert!(!action_state.pressed(&ButtonlikeTestAction::Down));
}

#[test]
fn mouse_scroll_dual_axis() {
    let mut app = test_app();
    app.insert_resource(
        InputMap::default().with_dual_axis(AxislikeTestAction::XY, MouseScroll::default()),
    );

    let input = MouseScroll::default();
    input.set_axis_pair(app.world_mut(), Vec2::new(5.0, 0.0));
    app.update();

    let action_state = app.world().resource::<ActionState<AxislikeTestAction>>();

    assert_eq!(
        action_state.axis_pair(&AxislikeTestAction::XY),
        Vec2::new(5.0, 0.0)
    );
}
