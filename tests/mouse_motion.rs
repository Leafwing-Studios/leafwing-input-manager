use bevy::input::mouse::MouseMotion;
use bevy::input::InputPlugin;
use bevy::prelude::*;
use leafwing_input_manager::axislike::DualAxisData;
use leafwing_input_manager::input_like::mouse_motion::{MouseMotionAxis, MouseMotionDirection};
use leafwing_input_manager::input_like::virtual_dpad::VirtualDPad;
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
    app.insert_resource(InputMap::new([(MouseMotionAxis::Y, AxislikeTestAction::X)]));

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

    app.world
        .resource_mut::<Input<MouseMotionDirection>>()
        .press(MouseMotionDirection::Up);
    let mut events = app.world.resource_mut::<Events<MouseMotion>>();

    assert_eq!(events.drain().count(), 1);
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
        let input = input.as_reflect().downcast_ref::<KeyCode>().unwrap();

        app.world.resource_mut::<Input<KeyCode>>().press(*input);
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

    let mut events = app.world.resource_mut::<Events<MouseMotion>>();
    events.send(MouseMotion {
        delta: Vec2::new(1.0, 1.0),
    });

    // Apply the event
    app.update();
    let action_state = app.world.resource::<ActionState<ButtonlikeTestAction>>();
    assert!(action_state.pressed(ButtonlikeTestAction::Up));
    assert!(action_state.pressed(ButtonlikeTestAction::Down));

    // Ensure it doesn't stay pressed for more than one frame
    app.update();
    let action_state = app.world.resource::<ActionState<ButtonlikeTestAction>>();
    assert!(!action_state.pressed(ButtonlikeTestAction::Up));
    assert!(!action_state.pressed(ButtonlikeTestAction::Down));
}

#[test]
fn mouse_motion_axis() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([
        (MouseMotionAxis::X, AxislikeTestAction::X),
        (MouseMotionAxis::Y, AxislikeTestAction::Y),
    ]));

    // +X
    let mut events = app.world.resource_mut::<Events<MouseMotion>>();
    events.send(MouseMotion {
        delta: Vec2::new(1.0, 0.0),
    });
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(AxislikeTestAction::X));

    // -X
    let mut events = app.world.resource_mut::<Events<MouseMotion>>();
    events.send(MouseMotion {
        delta: Vec2::new(-1.0, 0.0),
    });
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(AxislikeTestAction::X));

    // +Y
    let mut events = app.world.resource_mut::<Events<MouseMotion>>();
    events.send(MouseMotion {
        delta: Vec2::new(0.0, 1.0),
    });
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(AxislikeTestAction::Y));

    // -Y
    let mut events = app.world.resource_mut::<Events<MouseMotion>>();
    events.send(MouseMotion {
        delta: Vec2::new(0.0, -1.0),
    });
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(AxislikeTestAction::Y));

    // 0
    let mut events = app.world.resource_mut::<Events<MouseMotion>>();
    events.send(MouseMotion {
        delta: Vec2::new(0.0, 0.0),
    });
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

    let mut events = app.world.resource_mut::<Events<MouseMotion>>();
    events.send(MouseMotion {
        delta: Vec2::new(5.0, 1.0),
    });

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

    let mut events = app.world.resource_mut::<Events<MouseMotion>>();
    events.send(MouseMotion {
        delta: Vec2::new(0.0, -2.0),
    });
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

#[test]
fn mouse_drag() {
    let mut app = test_app();

    let mut input_map = InputMap::default();
    let chord: [Box<dyn InputLikeObject>; 2] =
        [DualAxis::mouse_motion().into(), MouseButton::Right.into()];
    input_map.insert_chord(chord, AxislikeTestAction::XY);

    app.insert_resource(input_map);

    let mut events = app.world.resource_mut::<Events<MouseMotion>>();
    events.send(MouseMotion {
        delta: Vec2::new(5.0, 0.0),
    });
    app.world
        .resource_mut::<Input<MouseButton>>()
        .press(MouseButton::Right);
    app.update();

    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();

    assert!(action_state.pressed(AxislikeTestAction::XY));
    assert_eq!(
        action_state.axis_pair(AxislikeTestAction::XY),
        Some(DualAxisData::new(5.0, 0.0))
    );
}
