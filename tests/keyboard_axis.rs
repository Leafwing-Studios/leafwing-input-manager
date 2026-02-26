#![cfg(feature = "keyboard")]

use bevy::input::InputPlugin;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, Clone, Copy, Debug, Reflect, PartialEq, Eq, Hash)]
#[actionlike(DualAxis)]
enum TestAction {
    Move,
}

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(InputPlugin)
        .add_plugins(InputManagerPlugin::<TestAction>::default())
        .init_resource::<ActionState<TestAction>>()
        .insert_resource(InputMap::default().with_dual_axis(TestAction::Move, VirtualDPad::wasd()));
    app.update();
    app
}

// Regression: a nonzero pair seeded outside the input pipeline (e.g. rollback restore) must be cleared.
#[test]
fn dual_axis_resets_to_zero_when_no_input_in_store() {
    let mut app = test_app();

    app.world_mut()
        .resource_mut::<ActionState<TestAction>>()
        .set_axis_pair(&TestAction::Move, Vec2::new(1.0, 0.0));

    app.update();

    assert_eq!(
        app.world()
            .resource::<ActionState<TestAction>>()
            .axis_pair(&TestAction::Move),
        Vec2::ZERO,
    );
}
