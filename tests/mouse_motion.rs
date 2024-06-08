use bevy::input::mouse::MouseMotion;
use bevy::input::InputPlugin;
use bevy::prelude::*;
use leafwing_input_manager::axislike::DualAxisData;
use leafwing_input_manager::buttonlike::MouseMotionDirection;
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
fn raw_mouse_motion_events() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([(
        AxislikeTestAction::X,
        SingleAxis::mouse_motion_y(),
    )]));

    let mut events = app.world_mut().resource_mut::<Events<MouseMotion>>();
    events.send(MouseMotion {
        delta: Vec2::new(0.0, 1.0),
    });

    app.update();
    let action_state = app.world().resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(&AxislikeTestAction::X));
}

#[test]
fn mouse_motion_discrete_mocking() {
    let mut app = test_app();
    let mut events = app.world_mut().resource_mut::<Events<MouseMotion>>();
    assert_eq!(events.drain().count(), 0);

    app.press_input(MouseMotionDirection::Up);
    let mut events = app.world_mut().resource_mut::<Events<MouseMotion>>();

    assert_eq!(events.drain().count(), 1);
}

#[test]
fn mouse_motion_single_axis_mocking() {
    let mut app = test_app();
    let mut events = app.world_mut().resource_mut::<Events<MouseMotion>>();
    assert_eq!(events.drain().count(), 0);

    let input = SingleAxis::mouse_motion_x();
    app.send_axis_values(input, [-1.0]);

    let mut events = app.world_mut().resource_mut::<Events<MouseMotion>>();
    assert_eq!(events.drain().count(), 1);
}

#[test]
fn mouse_motion_dual_axis_mocking() {
    let mut app = test_app();
    let mut events = app.world_mut().resource_mut::<Events<MouseMotion>>();
    assert_eq!(events.drain().count(), 0);

    let input = DualAxis::mouse_motion();
    app.send_axis_values(input, [1.0, 0.0]);

    let mut events = app.world_mut().resource_mut::<Events<MouseMotion>>();
    // Dual axis events are split out
    assert_eq!(events.drain().count(), 2);
}

#[test]
fn mouse_motion_buttonlike() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([
        (ButtonlikeTestAction::Up, MouseMotionDirection::Up),
        (ButtonlikeTestAction::Down, MouseMotionDirection::Down),
        (ButtonlikeTestAction::Left, MouseMotionDirection::Left),
        (ButtonlikeTestAction::Right, MouseMotionDirection::Right),
    ]));

    for action in ButtonlikeTestAction::variants() {
        let input_map = app.world().resource::<InputMap<ButtonlikeTestAction>>();
        // Get the first associated input
        let input = input_map.get(action).unwrap().first().unwrap().clone();

        app.press_input(input.clone());
        app.update();

        let action_state = app.world().resource::<ActionState<ButtonlikeTestAction>>();
        assert!(action_state.pressed(action), "failed for {input:?}");
    }
}

#[test]
fn mouse_motion_buttonlike_cancels() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([
        (ButtonlikeTestAction::Up, MouseMotionDirection::Up),
        (ButtonlikeTestAction::Down, MouseMotionDirection::Down),
        (ButtonlikeTestAction::Left, MouseMotionDirection::Left),
        (ButtonlikeTestAction::Right, MouseMotionDirection::Right),
    ]));

    app.press_input(MouseMotionDirection::Up);
    app.press_input(MouseMotionDirection::Down);

    // Correctly flushes the world
    app.update();

    let action_state = app.world().resource::<ActionState<ButtonlikeTestAction>>();

    assert!(!action_state.pressed(&ButtonlikeTestAction::Up));
    assert!(!action_state.pressed(&ButtonlikeTestAction::Down));
}

#[test]
fn mouse_motion_single_axis() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([
        (AxislikeTestAction::X, SingleAxis::mouse_motion_x()),
        (AxislikeTestAction::Y, SingleAxis::mouse_motion_y()),
    ]));

    // +X
    let input = SingleAxis::mouse_motion_x();
    app.send_axis_values(input, [1.0]);
    app.update();
    let action_state = app.world().resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(&AxislikeTestAction::X));

    // -X
    let input = SingleAxis::mouse_motion_x();
    app.send_axis_values(input, [-1.0]);
    app.update();
    let action_state = app.world().resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(&AxislikeTestAction::X));

    // +Y
    let input = SingleAxis::mouse_motion_y();
    app.send_axis_values(input, [-1.0]);
    app.update();
    let action_state = app.world().resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(&AxislikeTestAction::Y));

    // -Y
    let input = SingleAxis::mouse_motion_y();
    app.send_axis_values(input, [-1.0]);
    app.update();
    let action_state = app.world().resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(&AxislikeTestAction::Y));

    // 0
    let input = SingleAxis::mouse_motion_y();
    app.send_axis_values(input, [0.0]);
    app.update();
    let action_state = app.world().resource::<ActionState<AxislikeTestAction>>();
    assert!(!action_state.pressed(&AxislikeTestAction::Y));

    // None
    let input = SingleAxis::mouse_motion_y();
    app.send_axis_values(input, []);
    app.update();
    let action_state = app.world().resource::<ActionState<AxislikeTestAction>>();
    assert!(!action_state.pressed(&AxislikeTestAction::Y));
}

#[test]
fn mouse_motion_dual_axis() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([(
        AxislikeTestAction::XY,
        DualAxis::mouse_motion(),
    )]));

    let input = DualAxis::mouse_motion();
    app.send_axis_values(input, [5.0, 0.0]);
    app.update();

    let action_state = app.world().resource::<ActionState<AxislikeTestAction>>();

    assert!(action_state.pressed(&AxislikeTestAction::XY));
    assert_eq!(action_state.value(&AxislikeTestAction::XY), 5.0);
    assert_eq!(
        action_state.axis_pair(&AxislikeTestAction::XY).unwrap(),
        DualAxisData::new(5.0, 0.0)
    );
}

#[test]
fn mouse_motion_virtual_dpad() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([(
        AxislikeTestAction::XY,
        VirtualDPad::mouse_motion(),
    )]));

    let input = DualAxis::mouse_motion();
    app.send_axis_values(input, [0.0, -2.0]);
    app.update();

    let action_state = app.world().resource::<ActionState<AxislikeTestAction>>();

    assert!(action_state.pressed(&AxislikeTestAction::XY));
    // This should be a unit length, because we're working with a VirtualDPad
    assert_eq!(action_state.value(&AxislikeTestAction::XY), 1.0);
    assert_eq!(
        action_state.axis_pair(&AxislikeTestAction::XY).unwrap(),
        // This should be a unit length, because we're working with a VirtualDPad
        DualAxisData::new(0.0, -1.0)
    );
}

#[test]
fn mouse_drag() {
    let mut app = test_app();

    let mut input_map = InputMap::default();

    input_map.insert_chord(
        AxislikeTestAction::XY,
        [
            InputKind::from(DualAxis::mouse_motion()),
            InputKind::from(MouseButton::Right),
        ],
    );

    app.insert_resource(input_map);

    let input = DualAxis::mouse_motion();
    app.send_axis_values(input, [5.0, 0.0]);
    app.press_input(MouseButton::Right);
    app.update();

    let action_state = app.world().resource::<ActionState<AxislikeTestAction>>();

    assert!(action_state.pressed(&AxislikeTestAction::XY));
    assert_eq!(
        action_state.axis_pair(&AxislikeTestAction::XY),
        Some(DualAxisData::new(5.0, 0.0))
    );
}
