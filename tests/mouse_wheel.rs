#![cfg(feature = "mouse")]

use bevy::input::InputPlugin;
use bevy::input::mouse::MouseWheel;
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

#[derive(Actionlike, Clone, Copy, Debug, Reflect, PartialEq, Eq, Hash)]
#[actionlike(Axis)]
enum AxislikeTestAction {
    X,
    Y,
    #[actionlike(DualAxis)]
    XY,
}

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(InputPlugin)
        .add_plugins(InputManagerPlugin::<ButtonlikeTestAction>::default())
        .add_plugins(InputManagerPlugin::<AxislikeTestAction>::default());

    app
}

/// Returns a clone of the single [`ActionState<A>`] component in the app.
fn get_action_state<A: Actionlike>(app: &mut App) -> ActionState<A> {
    let world = app.world_mut();
    let mut query = world.query::<&ActionState<A>>();
    query.single(world).unwrap().clone()
}

/// Returns a clone of the single [`InputMap<A>`] component in the app.
fn get_input_map<A: Actionlike>(app: &mut App) -> InputMap<A> {
    let world = app.world_mut();
    let mut query = world.query::<&InputMap<A>>();
    query.single(world).unwrap().clone()
}

#[test]
fn mouse_scroll_discrete_mocking() {
    let mut app = test_app();
    let mut messages = app.world_mut().resource_mut::<Messages<MouseWheel>>();
    assert_eq!(messages.drain().count(), 0);

    MouseScrollDirection::UP.press(app.world_mut());

    let mut messages = app.world_mut().resource_mut::<Messages<MouseWheel>>();

    assert_eq!(messages.drain().count(), 1);
}

#[test]
fn mouse_scroll_single_axis_mocking() {
    let mut app = test_app();
    let mut messages = app.world_mut().resource_mut::<Messages<MouseWheel>>();
    assert_eq!(messages.drain().count(), 0);

    let input = MouseScrollAxis::X;
    input.set_value(app.world_mut(), -1.0);

    let mut messages = app.world_mut().resource_mut::<Messages<MouseWheel>>();
    assert_eq!(messages.drain().count(), 1);
}

#[test]
fn mouse_scroll_buttonlike() {
    let mut app = test_app();
    app.world_mut().spawn(InputMap::new([
        (ButtonlikeTestAction::Up, MouseScrollDirection::UP),
        (ButtonlikeTestAction::Down, MouseScrollDirection::DOWN),
        (ButtonlikeTestAction::Left, MouseScrollDirection::LEFT),
        (ButtonlikeTestAction::Right, MouseScrollDirection::RIGHT),
    ]));

    for action in ButtonlikeTestAction::variants() {
        let input_map = get_input_map::<ButtonlikeTestAction>(&mut app);
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

        let action_state = get_action_state::<ButtonlikeTestAction>(&mut app);
        assert!(action_state.pressed(action), "failed for {input:?}");
    }
}

#[test]
fn mouse_scroll_buttonlike_cancels() {
    let mut app = test_app();
    app.world_mut().spawn(InputMap::new([
        (ButtonlikeTestAction::Up, MouseScrollDirection::UP),
        (ButtonlikeTestAction::Down, MouseScrollDirection::DOWN),
        (ButtonlikeTestAction::Left, MouseScrollDirection::LEFT),
        (ButtonlikeTestAction::Right, MouseScrollDirection::RIGHT),
    ]));

    MouseScrollDirection::UP.press(app.world_mut());
    MouseScrollDirection::DOWN.press(app.world_mut());

    // Correctly flushes the world
    app.update();

    let action_state = get_action_state::<ButtonlikeTestAction>(&mut app);

    assert!(!action_state.pressed(&ButtonlikeTestAction::Up));
    assert!(!action_state.pressed(&ButtonlikeTestAction::Down));
}

#[test]
fn mouse_scroll_dual_axis() {
    let mut app = test_app();
    app.world_mut().spawn(
        InputMap::default().with_dual_axis(AxislikeTestAction::XY, MouseScroll::default()),
    );

    let input = MouseScroll::default();
    input.set_axis_pair(app.world_mut(), Vec2::new(5.0, 0.0));
    app.update();

    let action_state = get_action_state::<AxislikeTestAction>(&mut app);

    assert_eq!(
        action_state.axis_pair(&AxislikeTestAction::XY),
        Vec2::new(5.0, 0.0)
    );
}
