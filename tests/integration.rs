#![cfg(test)]
use bevy::prelude::*;
use bevy_ecs::query::ChangeTrackers;
use leafwing_input_manager::prelude::*;
use leafwing_input_manager::MockInput;

#[derive(Actionlike, Clone, Copy, Debug)]
enum Action {
    PayRespects,
}

// A resource that represents whether respects have been paid or not
#[derive(Default, PartialEq, Debug)]
struct Respect(bool);

fn pay_respects(
    action_state_query: Query<&ActionState<Action>, With<Player>>,
    action_state_resource: Option<Res<ActionState<Action>>>,
    mut respect: ResMut<Respect>,
) {
    if let Ok(action_state) = action_state_query.get_single() {
        if action_state.pressed(Action::PayRespects) {
            respect.0 = true;
        }
    }
    if let Some(action_state) = action_state_resource {
        if action_state.pressed(Action::PayRespects) {
            respect.0 = true;
        }
    }
}

fn respect_fades(mut respect: ResMut<Respect>) {
    respect.0 = false;
}

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands) {
    commands
        .spawn()
        .insert(Player)
        .insert_bundle(InputManagerBundle::<Action> {
            input_map: InputMap::<Action>::new([(Action::PayRespects, KeyCode::F)]),
            ..Default::default()
        });
}

#[test]
fn action_state_change_detection() {
    use bevy_input::InputPlugin;

    let mut app = App::new();

    app.add_plugins(MinimalPlugins)
        .add_plugin(InputPlugin)
        .add_plugin(InputManagerPlugin::<Action>::default())
        .add_startup_system(spawn_player)
        .add_system(action_state_changed_iff_input_changed);

    for i in 0..10 {
        if i % 2 == 0 {
            app.send_input(KeyCode::F);
        }

        app.update();
    }

    fn action_state_changed_iff_input_changed(
        query: Query<ChangeTrackers<ActionState<Action>>>,
        input: Res<Input<KeyCode>>,
    ) {
        let action_state_tracker = query.single();

        if input.is_changed() {
            assert!(action_state_tracker.is_changed());
        } else {
            assert!(!action_state_tracker.is_changed());
        }
    }
}

#[test]
fn disable_input() {
    use bevy_input::InputPlugin;

    let mut app = App::new();

    // Here we spawn a player and creating a global action state to check if [`DisableInput`]
    // releases correctly both
    app.add_plugins(MinimalPlugins)
        .add_plugin(InputPlugin)
        .add_plugin(InputManagerPlugin::<Action>::default())
        .add_startup_system(spawn_player)
        .init_resource::<ActionState<Action>>()
        .insert_resource(InputMap::<Action>::new([(Action::PayRespects, KeyCode::F)]))
        .init_resource::<Respect>()
        .add_system(pay_respects)
        .add_system_to_stage(CoreStage::PreUpdate, respect_fades);

    // Press F to pay respects
    app.send_input(KeyCode::F);
    app.update();
    let respect = app.world.resource::<Respect>();
    assert_eq!(*respect, Respect(true));

    // Disable the input
    let mut toggle_actions = app.world.resource_mut::<ToggleActions<Action>>();
    toggle_actions.enabled = false;

    // Now, all respect has faded
    app.update();
    let respect = app.world.resource::<Respect>();
    assert_eq!(*respect, Respect(false));

    // And even pressing F cannot bring it back
    app.send_input(KeyCode::F);
    app.update();
    let respect = app.world.resource::<Respect>();
    assert_eq!(*respect, Respect(false));
}

#[test]
#[cfg(feature = "ui")]
fn action_state_driver() {
    use bevy_input::InputPlugin;
    use bevy_ui::Interaction;

    let mut app = App::new();

    #[derive(Component)]
    struct ButtonMarker;

    fn setup(mut commands: Commands) {
        let player_entity = commands
            .spawn()
            .insert(Player)
            .insert_bundle(InputManagerBundle::<Action> {
                input_map: InputMap::<Action>::new([(Action::PayRespects, KeyCode::F)]),
                ..Default::default()
            })
            .id();

        commands
            .spawn()
            .insert(ButtonMarker)
            .insert(Interaction::None)
            .insert(ActionStateDriver::<Action> {
                action: Action::PayRespects,
                entity: player_entity,
            });
    }

    app.add_plugins(MinimalPlugins)
        .add_plugin(InputManagerPlugin::<Action>::default())
        .add_plugin(InputPlugin)
        .add_startup_system(setup)
        .add_system(pay_respects)
        .add_system_to_stage(CoreStage::PreUpdate, respect_fades)
        .init_resource::<Respect>();

    app.update();

    let respect = app.world.resource::<Respect>();
    assert_eq!(*respect, Respect(false));

    // Click button to pay respects
    app.click_button::<ButtonMarker>();

    // Verify that the button was in fact clicked
    let mut button_query = app.world.query::<&Interaction>();
    let interaction = button_query.iter(&app.world).next().unwrap();
    assert_eq!(*interaction, Interaction::Clicked);

    // Run the app once to process the clicks
    app.update();

    // Check the action state
    let mut action_state_query = app.world.query::<&ActionState<Action>>();
    let action_state = action_state_query.iter(&app.world).next().unwrap();
    assert!(action_state.pressed(Action::PayRespects));

    // Check the effects of that action state
    let respect = app.world.resource::<Respect>();
    assert_eq!(*respect, Respect(true));

    // Clear inputs
    app.reset_inputs();
    app.update();

    let respect = app.world.resource::<Respect>();
    assert_eq!(*respect, Respect(false));
}

#[test]
fn duration() {
    use bevy_input::InputPlugin;
    use bevy_utils::Duration;

    const RESPECTFUL_DURATION: Duration = Duration::from_micros(5);

    fn hold_f_to_pay_respects(
        action_state: Res<ActionState<Action>>,
        mut respect: ResMut<Respect>,
    ) {
        if action_state.pressed(Action::PayRespects)
            // Unrealistically disrespectful, but makes the tests faster
            && action_state.current_duration(Action::PayRespects) > RESPECTFUL_DURATION
        {
            respect.0 = true;
        }
    }

    let mut app = App::new();

    app.add_plugins(MinimalPlugins)
        .add_plugin(InputPlugin)
        .add_plugin(InputManagerPlugin::<Action>::default())
        .add_startup_system(spawn_player)
        .init_resource::<ActionState<Action>>()
        .insert_resource(InputMap::<Action>::new([(Action::PayRespects, KeyCode::F)]))
        .init_resource::<Respect>()
        .add_system(hold_f_to_pay_respects);

    // Initializing
    app.update();
    assert!(!app
        .world
        .resource::<ActionState<Action>>()
        .pressed(Action::PayRespects));
    let respect = app.world.resource::<Respect>();
    assert_eq!(*respect, Respect(false));

    // We haven't held for long enough
    app.send_input(KeyCode::F);
    app.update();
    assert!(app
        .world
        .resource::<ActionState<Action>>()
        .pressed(Action::PayRespects));

    std::thread::sleep(Duration::from_micros(1));

    app.update();
    assert!(app
        .world
        .resource::<ActionState<Action>>()
        .pressed(Action::PayRespects));

    let duration_held = app
        .world
        .resource::<ActionState<Action>>()
        .current_duration(Action::PayRespects);
    assert!(duration_held <= RESPECTFUL_DURATION);

    let respect = app.world.resource::<Respect>();
    assert_eq!(*respect, Respect(false));

    // Waiting while released doesn't work
    app.release_input(KeyCode::F);
    app.update();
    app.update();
    std::thread::sleep(Duration::from_micros(10));

    app.update();
    assert!(!app
        .world
        .resource::<ActionState<Action>>()
        .pressed(Action::PayRespects));
    let respect = app.world.resource::<Respect>();
    assert_eq!(*respect, Respect(false));

    let duration_released = app
        .world
        .resource::<ActionState<Action>>()
        .current_duration(Action::PayRespects);
    assert!(
        duration_released > RESPECTFUL_DURATION,
        "duration_released ({:?}) > RESPECTFUL_DURATION ({:?})",
        duration_released,
        RESPECTFUL_DURATION
    );

    // Press and hold
    app.send_input(KeyCode::F);

    app.update();
    assert!(app
        .world
        .resource::<ActionState<Action>>()
        .pressed(Action::PayRespects));

    std::thread::sleep(Duration::from_micros(10));
    app.update();
    assert!(app
        .world
        .resource::<ActionState<Action>>()
        .pressed(Action::PayRespects));

    app.update();
    assert!(app
        .world
        .resource::<ActionState<Action>>()
        .pressed(Action::PayRespects));

    let duration_held = app
        .world
        .resource::<ActionState<Action>>()
        .current_duration(Action::PayRespects);
    assert!(duration_held > RESPECTFUL_DURATION);

    // Now it works!
    let respect = app.world.resource::<Respect>();
    assert_eq!(*respect, Respect(true));

    // Double-check on the duration...
    let current_duration = app
        .world
        .resource::<ActionState<Action>>()
        .current_duration(Action::PayRespects);
    assert!(current_duration > Duration::from_micros(5));

    // Double-checking that the swap to previous_duration works
    app.release_input(KeyCode::F);
    app.update();
    app.update();

    let previous_duration = app
        .world
        .resource::<ActionState<Action>>()
        .previous_duration(Action::PayRespects);
    assert_eq!(current_duration, previous_duration);
}

#[test]
fn do_nothing() {
    use bevy_input::InputPlugin;
    use bevy_utils::Duration;

    let mut app = App::new();

    app.add_plugins(MinimalPlugins)
        .add_plugin(InputPlugin)
        .add_plugin(InputManagerPlugin::<Action>::default())
        .add_startup_system(spawn_player)
        .init_resource::<ActionState<Action>>()
        .insert_resource(InputMap::<Action>::new([(Action::PayRespects, KeyCode::F)]));

    app.update();
    let action_state = app.world.resource::<ActionState<Action>>();
    let t0 = action_state.instant_started(Action::PayRespects);
    assert!(t0.is_some());
    let mut duration_last_update = Duration::ZERO;

    for _ in 0..3 {
        app.update();
        let action_state = app.world.resource::<ActionState<Action>>();

        // Sanity checking state to catch wonkiness
        assert!(!action_state.pressed(Action::PayRespects));
        assert!(!action_state.just_pressed(Action::PayRespects));
        assert!(action_state.released(Action::PayRespects));
        assert!(!action_state.just_released(Action::PayRespects));

        assert_eq!(action_state.reasons_pressed(Action::PayRespects).len(), 0);

        assert_eq!(action_state.instant_started(Action::PayRespects), t0);
        assert_eq!(
            action_state.previous_duration(Action::PayRespects),
            Duration::ZERO
        );
        assert!(action_state.current_duration(Action::PayRespects) > duration_last_update);

        duration_last_update = action_state.current_duration(Action::PayRespects);
        dbg!(duration_last_update);
    }
}
