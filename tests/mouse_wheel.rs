use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::input::InputPlugin;
use bevy::prelude::*;
use leafwing_input_manager::axislike::DualAxisData;
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

    app
}

#[test]
fn raw_mouse_wheel_events() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([(
        ButtonlikeTestAction::Up,
        MouseWheelDirection::Up,
    )]));

    let mut events = app.world.resource_mut::<Events<MouseWheel>>();
    events.send(MouseWheel {
        unit: MouseScrollUnit::Pixel,
        x: 0.0,
        y: 10.0,
        window: Entity::PLACEHOLDER,
    });

    app.update();
    let action_state = app.world.resource::<ActionState<ButtonlikeTestAction>>();
    assert!(action_state.pressed(&ButtonlikeTestAction::Up));
}

#[test]
fn mouse_wheel_discrete_mocking() {
    let mut app = test_app();
    let mut events = app.world.resource_mut::<Events<MouseWheel>>();
    assert_eq!(events.drain().count(), 0);

    app.press_input(MouseWheelDirection::Up);
    let mut events = app.world.resource_mut::<Events<MouseWheel>>();

    assert_eq!(events.drain().count(), 1);
}

#[test]
fn mouse_wheel_single_axis_mocking() {
    let mut app = test_app();
    let mut events = app.world.resource_mut::<Events<MouseWheel>>();
    assert_eq!(events.drain().count(), 0);

    let input = SingleAxis::mouse_wheel_x();
    app.send_axis_values(input, [-1.0]);

    let mut events = app.world.resource_mut::<Events<MouseWheel>>();
    assert_eq!(events.drain().count(), 1);
}

#[test]
fn mouse_wheel_dual_axis_mocking() {
    let mut app = test_app();
    let mut events = app.world.resource_mut::<Events<MouseWheel>>();
    assert_eq!(events.drain().count(), 0);

    let input = DualAxis::mouse_wheel();
    app.send_axis_values(input, [-1.0]);

    let mut events = app.world.resource_mut::<Events<MouseWheel>>();
    // Dual axis events are split out
    assert_eq!(events.drain().count(), 2);
}

#[test]
fn mouse_wheel_buttonlike() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([
        (ButtonlikeTestAction::Up, MouseWheelDirection::Up),
        (ButtonlikeTestAction::Down, MouseWheelDirection::Down),
        (ButtonlikeTestAction::Left, MouseWheelDirection::Left),
        (ButtonlikeTestAction::Right, MouseWheelDirection::Right),
    ]));

    for action in ButtonlikeTestAction::variants() {
        let input_map = app.world.resource::<InputMap<ButtonlikeTestAction>>();
        // Get the first associated input
        let input = input_map.get(action).unwrap().first().unwrap().clone();

        app.press_input(input.clone());
        app.update();

        let action_state = app.world.resource::<ActionState<ButtonlikeTestAction>>();
        assert!(action_state.pressed(action), "failed for {input:?}");
    }
}

#[test]
fn mouse_wheel_buttonlike_cancels() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([
        (ButtonlikeTestAction::Up, MouseWheelDirection::Up),
        (ButtonlikeTestAction::Down, MouseWheelDirection::Down),
        (ButtonlikeTestAction::Left, MouseWheelDirection::Left),
        (ButtonlikeTestAction::Right, MouseWheelDirection::Right),
    ]));

    app.press_input(MouseWheelDirection::Up);
    app.press_input(MouseWheelDirection::Down);

    // Correctly flushes the world
    app.update();

    let action_state = app.world.resource::<ActionState<ButtonlikeTestAction>>();

    assert!(!action_state.pressed(&ButtonlikeTestAction::Up));
    assert!(!action_state.pressed(&ButtonlikeTestAction::Down));
}

#[test]
fn mouse_wheel_single_axis() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([
        (AxislikeTestAction::X, SingleAxis::mouse_wheel_x()),
        (AxislikeTestAction::Y, SingleAxis::mouse_wheel_y()),
    ]));

    // +X
    let input = SingleAxis::mouse_wheel_x();
    app.send_axis_values(input, [1.0]);
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(&AxislikeTestAction::X));

    // -X
    let input = SingleAxis::mouse_wheel_x();
    app.send_axis_values(input, [-1.0]);
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(&AxislikeTestAction::X));

    // +Y
    let input = SingleAxis::mouse_wheel_y();
    app.send_axis_values(input, [1.0]);
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(&AxislikeTestAction::Y));

    // -Y
    let input = SingleAxis::mouse_wheel_y();
    app.send_axis_values(input, [-1.0]);
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(&AxislikeTestAction::Y));

    // 0
    let input = SingleAxis::mouse_wheel_y();
    app.send_axis_values(input, [0.0]);
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(!action_state.pressed(&AxislikeTestAction::Y));

    // None
    let input = SingleAxis::mouse_wheel_y();
    app.send_axis_values(input, []);
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(!action_state.pressed(&AxislikeTestAction::Y));
}

#[test]
fn mouse_wheel_dual_axis() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([(
        AxislikeTestAction::XY,
        DualAxis::mouse_wheel(),
    )]));

    let input = DualAxis::mouse_wheel();
    app.send_axis_values(input, [5.0, 0.0]);
    app.update();

    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();

    assert!(action_state.pressed(&AxislikeTestAction::XY));
    assert_eq!(action_state.value(&AxislikeTestAction::XY), 5.0);
    assert_eq!(
        action_state.axis_pair(&AxislikeTestAction::XY).unwrap(),
        DualAxisData::new(5.0, 0.0)
    );
}

#[test]
fn mouse_wheel_virtualdpad() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([(
        AxislikeTestAction::XY,
        VirtualDPad::mouse_wheel(),
    )]));

    let input = DualAxis::mouse_wheel();
    app.send_axis_values(input, [0.0, -2.0]);
    app.update();

    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();

    assert!(action_state.pressed(&AxislikeTestAction::XY));
    // This should be a unit length, because we're working with a VirtualDPad
    assert_eq!(action_state.value(&AxislikeTestAction::XY), 1.0);
    assert_eq!(
        action_state.axis_pair(&AxislikeTestAction::XY).unwrap(),
        // This should be a unit length, because we're working with a VirtualDPad
        DualAxisData::new(0.0, -1.0)
    );
}
