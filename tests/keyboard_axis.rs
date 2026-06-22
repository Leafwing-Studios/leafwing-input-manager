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
        .add_plugins(InputManagerPlugin::<TestAction>::default());
    // Spawn a single input entity. The `InputMap` requires an `ActionState`,
    // so it is added automatically.
    app.world_mut()
        .spawn(InputMap::default().with_dual_axis(TestAction::Move, VirtualDPad::wasd()));
    app.update();
    app
}

/// Returns a clone of the single [`ActionState<TestAction>`] component in the app.
fn action_state(app: &mut App) -> ActionState<TestAction> {
    let world = app.world_mut();
    let mut query = world.query::<&ActionState<TestAction>>();
    query.single(world).unwrap().clone()
}

// Regression: a nonzero pair seeded outside the input pipeline (e.g. rollback restore) must be cleared.
#[test]
fn dual_axis_resets_to_zero_when_no_input_in_store() {
    let mut app = test_app();

    {
        let world = app.world_mut();
        let mut query = world.query::<&mut ActionState<TestAction>>();
        query
            .single_mut(world)
            .unwrap()
            .set_axis_pair(&TestAction::Move, Vec2::new(1.0, 0.0));
    }

    app.update();

    assert_eq!(
        action_state(&mut app).axis_pair(&TestAction::Move),
        Vec2::ZERO,
    );
}
