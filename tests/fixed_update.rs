use bevy::app::PreUpdate;
use bevy::input::InputPlugin;
use bevy::prelude::{
    App, Fixed, FixedPostUpdate, FixedUpdate, IntoSystemConfigs, KeyCode, Real, Reflect, Res,
    ResMut, Resource, Time, Update,
};
use bevy::time::TimeUpdateStrategy;
use bevy::MinimalPlugins;
use leafwing_input_manager::action_state::ActionState;
use leafwing_input_manager::input_map::InputMap;
use leafwing_input_manager::input_mocking::MockInput;
use leafwing_input_manager::plugin::{InputManagerPlugin, InputManagerSystem};
use leafwing_input_manager_macros::Actionlike;
use std::time::Duration;

#[derive(Actionlike, Clone, Copy, Debug, Reflect, PartialEq, Eq, Hash)]
enum TestAction {
    Up,
    Down,
}

#[derive(Resource, Default)]
struct UpdateCounter {
    just_pressed: usize,
}

#[derive(Resource, Default)]
struct FixedUpdateCounter {
    /// how many times did the FixedUpdate schedule run?
    run: usize,
    /// how many times did the Up button get just_pressed?
    just_pressed: usize,
}

fn build_app(fixed_timestep: Duration, frame_timestep: Duration) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(InputPlugin)
        .add_plugins(InputManagerPlugin::<TestAction>::default())
        .init_resource::<UpdateCounter>()
        .init_resource::<FixedUpdateCounter>()
        .init_resource::<ActionState<TestAction>>()
        .insert_resource(InputMap::<TestAction>::new([
            (TestAction::Up, KeyCode::ArrowUp),
            (TestAction::Down, KeyCode::ArrowDown),
        ]))
        .insert_resource(Time::<Fixed>::from_duration(fixed_timestep))
        .insert_resource(TimeUpdateStrategy::ManualDuration(frame_timestep));

    app.add_systems(Update, update_counter);
    app.add_systems(FixedUpdate, fixed_update_counter);

    // we have to set an initial time for TimeUpdateStrategy::ManualDuration to work properly
    let startup = app.world.resource::<Time<Real>>().startup();
    app.world
        .resource_mut::<Time<Real>>()
        .update_with_instant(startup);
    app
}

fn fixed_update_counter(
    mut counter: ResMut<FixedUpdateCounter>,
    action: Res<ActionState<TestAction>>,
) {
    if action.just_pressed(&TestAction::Up) {
        counter.just_pressed += 1;
    }
    counter.run += 1;
}

fn update_counter(mut counter: ResMut<UpdateCounter>, action: Res<ActionState<TestAction>>) {
    if action.just_pressed(&TestAction::Up) {
        counter.just_pressed += 1;
    }
}

/// We have 2 frames without a FixedUpdate schedule in between (F1 - F2 - FU - F3)
///
/// A button pressed in F1 should still be marked as `just_pressed` in FU
#[test]
fn frame_without_fixed_timestep() {
    let mut app = build_app(Duration::from_millis(10), Duration::from_millis(9));

    app.press_input(KeyCode::ArrowUp);

    // the FixedUpdate schedule shouldn't run
    app.update();
    assert_eq!(
        app.world.get_resource::<FixedUpdateCounter>().unwrap().run,
        0
    );
    assert_eq!(
        app.world
            .get_resource::<UpdateCounter>()
            .unwrap()
            .just_pressed,
        1
    );

    // the FixedUpdate schedule should run once and the action still be counted as `just_pressed`
    app.update();
    assert_eq!(
        app.world.get_resource::<FixedUpdateCounter>().unwrap().run,
        1
    );
    assert_eq!(
        app.world
            .get_resource::<FixedUpdateCounter>()
            .unwrap()
            .just_pressed,
        1
    );
    assert_eq!(
        app.world
            .get_resource::<UpdateCounter>()
            .unwrap()
            .just_pressed,
        1
    );

    // the FixedUpdate schedule should run once, the button should now be `pressed`
    app.update();
    assert_eq!(
        app.world.get_resource::<FixedUpdateCounter>().unwrap().run,
        2
    );
    assert_eq!(
        app.world
            .get_resource::<FixedUpdateCounter>()
            .unwrap()
            .just_pressed,
        1
    );
    assert_eq!(
        app.world
            .get_resource::<UpdateCounter>()
            .unwrap()
            .just_pressed,
        1
    );

    // make sure that the timings didn't get updated twice (once in Update and once in FixedUpdate)
    // (the `tick` function has been called twice, but it uses `Time<Real>` to update the time,
    // which is only updated in `PreUpdate`, which is what we want)
    assert_eq!(
        app.world
            .get_resource::<ActionState<TestAction>>()
            .unwrap()
            .current_duration(&TestAction::Up),
        Duration::from_millis(18)
    );
}

/// We have a frames with two FixedUpdate schedule executions in between (F1 - FU1 - FU2 - F2)
///
/// A button pressed in F1 should still be marked as `just_pressed` in FU1, and should just be
/// `pressed` in FU2
#[test]
fn frame_with_two_fixed_timestep() {
    let mut app = build_app(Duration::from_millis(4), Duration::from_millis(9));

    app.press_input(KeyCode::ArrowUp);

    // the FixedUpdate schedule should run twice
    // the button should be just_pressed only once
    app.update();
    assert_eq!(
        app.world.get_resource::<FixedUpdateCounter>().unwrap().run,
        2
    );
    assert_eq!(
        app.world
            .get_resource::<FixedUpdateCounter>()
            .unwrap()
            .just_pressed,
        1
    );
    assert_eq!(
        app.world
            .get_resource::<UpdateCounter>()
            .unwrap()
            .just_pressed,
        1
    );

    // the FixedUpdate schedule should run twice
    app.update();
    assert_eq!(
        app.world.get_resource::<FixedUpdateCounter>().unwrap().run,
        4
    );
    assert_eq!(
        app.world
            .get_resource::<FixedUpdateCounter>()
            .unwrap()
            .just_pressed,
        1
    );
    assert_eq!(
        app.world
            .get_resource::<UpdateCounter>()
            .unwrap()
            .just_pressed,
        1
    );

    // make sure that the timings didn't get updated twice (once in Update and once in FixedUpdate)
    // (the `tick` function has been called twice, but it uses `Time<Real>` to update the time,
    // which is only updated in `PreUpdate`, which is what we want)
    assert_eq!(
        app.world
            .get_resource::<ActionState<TestAction>>()
            .unwrap()
            .current_duration(&TestAction::Up),
        Duration::from_millis(18)
    );
}

/// Check that if the action is consumed in FU1, it will still be consumed in F2.
/// (i.e. consuming is shared between the `FixedMain` and `Main` schedules)
#[test]
fn test_consume_in_fixed_update() {
    let mut app = build_app(Duration::from_millis(5), Duration::from_millis(5));

    app.add_systems(
        FixedPostUpdate,
        |mut action: ResMut<ActionState<TestAction>>| {
            action.consume(&TestAction::Up);
        },
    );

    app.press_input(KeyCode::ArrowUp);

    // the FixedUpdate schedule should run once
    // the button should be just_pressed only once
    app.update();
    assert_eq!(
        app.world.get_resource::<FixedUpdateCounter>().unwrap().run,
        1
    );
    assert_eq!(
        app.world
            .get_resource::<FixedUpdateCounter>()
            .unwrap()
            .just_pressed,
        1
    );
    assert_eq!(
        app.world
            .get_resource::<UpdateCounter>()
            .unwrap()
            .just_pressed,
        1
    );

    // the button should still be consumed, even after we exit the FixedUpdate schedule
    assert!(
        app.world
            .get_resource::<ActionState<TestAction>>()
            .unwrap()
            .action_data(&TestAction::Up)
            .unwrap()
            .consumed,
    );
}

/// Check that if the action is consumed in F1, it will still be consumed in FU1.
/// (i.e. consuming is shared between the `FixedMain` and `Main` schedules)
#[test]
fn test_consume_in_update() {
    let mut app = build_app(Duration::from_millis(5), Duration::from_millis(5));

    app.press_input(KeyCode::ArrowUp);
    fn consume_action(mut action: ResMut<ActionState<TestAction>>) {
        action.consume(&TestAction::Up);
    }

    app.add_systems(
        PreUpdate,
        consume_action.in_set(InputManagerSystem::ManualControl),
    );

    app.add_systems(FixedUpdate, |action: Res<ActionState<TestAction>>| {
        // check that the action is still consumed in the FixedMain schedule
        assert!(
            action.consumed(&TestAction::Up),
            "Action should still be consumed in FixedUpdate"
        );
    });

    // the FixedUpdate schedule should run once
    app.update();
}
