#![cfg(feature = "keyboard")]

use bevy::input::InputPlugin;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use bevy::MinimalPlugins;
use leafwing_input_manager::action_state::ActionState;
use leafwing_input_manager::input_map::InputMap;
use leafwing_input_manager::plugin::InputManagerPlugin;
use leafwing_input_manager::prelude::Buttonlike;
use leafwing_input_manager_macros::Actionlike;
use std::time::Duration;

#[derive(Actionlike, Clone, Copy, Debug, Reflect, PartialEq, Eq, Hash)]
enum TestAction {
    Up,
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
        .insert_resource(InputMap::<TestAction>::new([(
            TestAction::Up,
            KeyCode::ArrowUp,
        )]))
        .insert_resource(Time::<Fixed>::from_duration(fixed_timestep))
        .insert_resource(TimeUpdateStrategy::ManualDuration(frame_timestep));

    app.add_systems(Update, update_counter);
    app.add_systems(FixedUpdate, fixed_update_counter);

    // we have to set an initial time for TimeUpdateStrategy::ManualDuration to work properly
    let startup = app.world().resource::<Time<Real>>().startup();
    app.world_mut()
        .resource_mut::<Time<Real>>()
        .update_with_instant(startup);
    app
}

/// Keep track of some events that happen in the FixedUpdate schedule
/// - did the FixedUpdate schedule run during the frame?
/// - was the button marked as `just_pressed` in the FixedUpdate schedule?
fn fixed_update_counter(
    mut counter: ResMut<FixedUpdateCounter>,
    action: Res<ActionState<TestAction>>,
) {
    if action.just_pressed(&TestAction::Up) {
        counter.just_pressed += 1;
    }
    counter.run += 1;
}

/// Keep track of some events that happen in the Update schedule
/// - was the button marked as `just_pressed` in the Update schedule?
fn update_counter(mut counter: ResMut<UpdateCounter>, action: Res<ActionState<TestAction>>) {
    if action.just_pressed(&TestAction::Up) {
        counter.just_pressed += 1;
    }
}

/// Reset the counters at the end of the frame
fn reset_counters(app: &mut App) {
    let mut fixed_update_counters = app.world_mut().resource_mut::<FixedUpdateCounter>();
    fixed_update_counters.run = 0;
    fixed_update_counters.just_pressed = 0;
    let mut update_counters = app.world_mut().resource_mut::<UpdateCounter>();
    update_counters.just_pressed = 0;
}

/// Assert that the FixedUpdate schedule run the expected number of times
fn check_fixed_update_run_count(app: &mut App, expected: usize) {
    assert_eq!(
        app.world()
            .get_resource::<FixedUpdateCounter>()
            .unwrap()
            .run,
        expected,
        "FixedUpdate schedule should have run {expected} times"
    );
}

/// Assert that the button was just_pressed the expected number of times during the FixedUpdate schedule
fn check_fixed_update_just_pressed_count(app: &mut App, expected: usize) {
    assert_eq!(
        app.world()
            .get_resource::<FixedUpdateCounter>()
            .unwrap()
            .just_pressed,
        expected,
        "Button should have been just_pressed {expected} times during the FixedUpdate schedule"
    );
}

/// Assert that the button was just_pressed the expected number of times during the Update schedule
fn check_update_just_pressed_count(app: &mut App, expected: usize) {
    assert_eq!(
        app.world()
            .get_resource::<UpdateCounter>()
            .unwrap()
            .just_pressed,
        expected,
        "Button should have been just_pressed {expected} times during the Update schedule"
    );
}

/// We have 2 frames without a FixedUpdate schedule in between (F1 - F2 - FU - F3)
///
/// A button pressed in F1 should still be marked as `just_pressed` in FU
#[test]
fn frame_without_fixed_timestep() {
    let mut app = build_app(Duration::from_millis(10), Duration::from_millis(9));

    KeyCode::ArrowUp.press(app.world_mut());

    // Frame 1: button is just_pressed and the FixedUpdate schedule does not run
    app.update();
    check_update_just_pressed_count(&mut app, 1);
    check_fixed_update_run_count(&mut app, 0);
    reset_counters(&mut app);

    // Frame 2: the FixedUpdate schedule should run once, the button is `just_pressed` for FixedUpdate
    // because we tick independently in FixedUpdate and Update.
    // Button is not `just_pressed` anymore in the Update schedule.
    app.update();
    check_update_just_pressed_count(&mut app, 0);
    check_fixed_update_run_count(&mut app, 1);
    check_fixed_update_just_pressed_count(&mut app, 1);
    reset_counters(&mut app);

    // Frame 3: the FixedUpdate schedule should run once, the button should now be `pressed` in both schedules
    app.update();
    check_update_just_pressed_count(&mut app, 0);
    check_fixed_update_run_count(&mut app, 1);
    check_fixed_update_just_pressed_count(&mut app, 0);
    reset_counters(&mut app);

    // make sure that the timings didn't get updated twice (once in Update and once in FixedUpdate)
    // (the `tick` function has been called twice, but it uses `Time<Real>` to update the time,
    // which is only updated in `PreUpdate`, which is what we want)
    #[cfg(feature = "timing")]
    assert_eq!(
        app.world()
            .get_resource::<ActionState<TestAction>>()
            .unwrap()
            .current_duration(&TestAction::Up),
        Duration::from_millis(18)
    );
}

/// We have a frame with two FixedUpdate schedule executions in between (F1 - FU1 - FU2 - F2)
///
/// A button pressed in F1 should still be marked as `just_pressed` in FU1, and should just be
/// `pressed` in FU2
#[test]
fn frame_with_two_fixed_timestep() {
    let mut app = build_app(Duration::from_millis(4), Duration::from_millis(9));

    KeyCode::ArrowUp.press(app.world_mut());

    // Frame 1: the FixedUpdate schedule should run twice, but the button should be just_pressed only once
    // in FixedUpdate
    app.update();
    check_update_just_pressed_count(&mut app, 1);
    check_fixed_update_run_count(&mut app, 2);
    check_fixed_update_just_pressed_count(&mut app, 1);
    reset_counters(&mut app);

    // Frame 2: the FixedUpdate schedule should run twice, but the button is not just_pressed anymore
    app.update();
    check_update_just_pressed_count(&mut app, 0);
    check_fixed_update_run_count(&mut app, 2);
    check_fixed_update_just_pressed_count(&mut app, 0);
    reset_counters(&mut app);

    // make sure that the timings didn't get updated twice (once in Update and once in FixedUpdate)
    // (the `tick` function has been called twice, but it uses `Time<Real>` to update the time,
    // which is only updated in `PreUpdate`, which is what we want)
    #[cfg(feature = "timing")]
    assert_eq!(
        app.world()
            .get_resource::<ActionState<TestAction>>()
            .unwrap()
            .current_duration(&TestAction::Up),
        Duration::from_millis(18)
    );
}
