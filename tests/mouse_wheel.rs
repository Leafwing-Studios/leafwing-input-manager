use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::input::InputPlugin;
use bevy::prelude::*;
use leafwing_input_manager::axislike::DualAxisData;
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
fn raw_mouse_wheel_events() {
    let mut app = test_app();
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
    app.insert_resource(InputMap::new([
        (MouseWheelDirection::Up, ButtonlikeTestAction::Up),
        (MouseWheelDirection::Down, ButtonlikeTestAction::Down),
        (MouseWheelDirection::Left, ButtonlikeTestAction::Left),
        (MouseWheelDirection::Right, ButtonlikeTestAction::Right),
    ]));

    for action in ButtonlikeTestAction::variants() {
        let input_map = app.world.resource::<InputMap<ButtonlikeTestAction>>();
        // Get the first associated input
        let input = input_map.get(action).get_at(0).unwrap().clone();
        let input = input
            .as_reflect()
            .downcast_ref::<MouseWheelDirection>()
            .unwrap();

        let mut events = app.world.resource_mut::<Events<MouseWheel>>();
        let scroll_direction: Vec2 = Vec2::from(*input) * 10.0;
        events.send(MouseWheel {
            unit: MouseScrollUnit::Pixel,
            x: scroll_direction.x,
            y: scroll_direction.y,
        });

        app.update();

        let action_state = app.world.resource::<ActionState<ButtonlikeTestAction>>();
        assert!(
            action_state.pressed(action),
            "failed for {input:?} {action:?}"
        );
    }
}

#[test]
fn mouse_wheel_buttonlike_cancels() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([
        (MouseWheelDirection::Up, ButtonlikeTestAction::Up),
        (MouseWheelDirection::Down, ButtonlikeTestAction::Down),
        (MouseWheelDirection::Left, ButtonlikeTestAction::Left),
        (MouseWheelDirection::Right, ButtonlikeTestAction::Right),
    ]));

    let mut events = app.world.resource_mut::<Events<MouseWheel>>();
    events.send(MouseWheel {
        unit: MouseScrollUnit::Pixel,
        x: 0.0,
        y: 10.0,
    });
    let mut events = app.world.resource_mut::<Events<MouseWheel>>();
    events.send(MouseWheel {
        unit: MouseScrollUnit::Pixel,
        x: 0.0,
        y: -10.0,
    });

    // Correctly flushes the world
    app.update();

    let action_state = app.world.resource::<ActionState<ButtonlikeTestAction>>();

    assert!(!action_state.pressed(ButtonlikeTestAction::Up));
    assert!(!action_state.pressed(ButtonlikeTestAction::Down));
}

#[test]
fn mouse_wheel_single_axis() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([
        (MouseWheelAxis::X, AxislikeTestAction::X),
        (MouseWheelAxis::Y, AxislikeTestAction::Y),
    ]));

    // +X
    let mut events = app.world.resource_mut::<Events<MouseWheel>>();
    events.send(MouseWheel {
        unit: MouseScrollUnit::Pixel,
        x: 1.0,
        y: 0.0,
    });
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(AxislikeTestAction::X));

    // -X
    let mut events = app.world.resource_mut::<Events<MouseWheel>>();
    events.send(MouseWheel {
        unit: MouseScrollUnit::Pixel,
        x: -1.0,
        y: 0.0,
    });
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(AxislikeTestAction::X));

    // +Y
    let mut events = app.world.resource_mut::<Events<MouseWheel>>();
    events.send(MouseWheel {
        unit: MouseScrollUnit::Pixel,
        x: 0.0,
        y: 1.0,
    });
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(AxislikeTestAction::Y));

    // -Y
    let mut events = app.world.resource_mut::<Events<MouseWheel>>();
    events.send(MouseWheel {
        unit: MouseScrollUnit::Pixel,
        x: 0.0,
        y: -1.0,
    });
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(AxislikeTestAction::Y));

    // 0
    let mut events = app.world.resource_mut::<Events<MouseWheel>>();
    events.send(MouseWheel {
        unit: MouseScrollUnit::Pixel,
        x: 0.0,
        y: 0.0,
    });
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(!action_state.pressed(AxislikeTestAction::Y));
}

#[test]
fn mouse_wheel_dual_axis() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([(
        DualAxis::mouse_wheel(),
        AxislikeTestAction::XY,
    )]));

    let mut events = app.world.resource_mut::<Events<MouseWheel>>();
    events.send(MouseWheel {
        unit: MouseScrollUnit::Pixel,
        x: 5.0,
        y: 0.0,
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
fn mouse_wheel_virtualdpad() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([(
        VirtualDPad::mouse_wheel(),
        AxislikeTestAction::XY,
    )]));

    let mut events = app.world.resource_mut::<Events<MouseWheel>>();
    events.send(MouseWheel {
        unit: MouseScrollUnit::Pixel,
        x: 0.0,
        y: -2.0,
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
