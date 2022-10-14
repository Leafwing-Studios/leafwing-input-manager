#![cfg(test)]
use bevy::ecs::query::ChangeTrackers;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

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

fn remove_input_map(mut commands: Commands, query: Query<Entity, With<InputMap<Action>>>) {
    for entity in query.iter() {
        commands.entity(entity).remove::<InputMap<Action>>();
    }
}

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands) {
    commands
        .spawn()
        .insert(Player)
        .insert_bundle(InputManagerBundle::<Action> {
            input_map: InputMap::<Action>::new([(KeyCode::F, Action::PayRespects)]),
            ..Default::default()
        });
}

#[test]
fn do_nothing() {
    use bevy::input::InputPlugin;
    use bevy::utils::Duration;

    let mut app = App::new();

    app.add_plugins(MinimalPlugins)
        .add_plugin(InputPlugin)
        .add_plugin(InputManagerPlugin::<Action>::default())
        .add_startup_system(spawn_player)
        .init_resource::<ActionState<Action>>()
        .insert_resource(InputMap::<Action>::new([(KeyCode::F, Action::PayRespects)]));

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

#[test]
fn action_state_change_detection() {
    use bevy::input::InputPlugin;

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
    use bevy::input::InputPlugin;

    let mut app = App::new();

    // Here we spawn a player and creating a global action state to check if [`DisableInput`]
    // releases correctly both
    app.add_plugins(MinimalPlugins)
        .add_plugin(InputPlugin)
        .add_plugin(InputManagerPlugin::<Action>::default())
        .add_startup_system(spawn_player)
        .init_resource::<ActionState<Action>>()
        .insert_resource(InputMap::<Action>::new([(KeyCode::F, Action::PayRespects)]))
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
fn release_when_input_map_removed() {
    use bevy::input::InputPlugin;

    let mut app = App::new();

    // Spawn a player and create a global action state.
    app.add_plugins(MinimalPlugins)
        .add_plugin(InputPlugin)
        .add_plugin(InputManagerPlugin::<Action>::default())
        .add_startup_system(spawn_player)
        .init_resource::<ActionState<Action>>()
        .insert_resource(InputMap::<Action>::new([(KeyCode::F, Action::PayRespects)]))
        .init_resource::<Respect>()
        .add_system(pay_respects)
        .add_system(remove_input_map)
        .add_system_to_stage(CoreStage::PreUpdate, respect_fades);

    // Press F to pay respects
    app.send_input(KeyCode::F);
    app.update();
    let respect = app.world.resource::<Respect>();
    assert_eq!(*respect, Respect(true));

    // Remove the InputMap
    app.world.remove_resource::<InputMap<Action>>();
    // Needs an extra frame for the resource removed detection to release inputs
    app.update();

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
    use bevy::input::InputPlugin;
    use bevy::ui::Interaction;

    let mut app = App::new();

    #[derive(Component)]
    struct ButtonMarker;

    fn setup(mut commands: Commands) {
        let player_entity = commands
            .spawn()
            .insert(Player)
            .insert_bundle(InputManagerBundle::<Action> {
                input_map: InputMap::<Action>::new([(KeyCode::F, Action::PayRespects)]),
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
    use bevy::input::InputPlugin;
    use bevy::utils::Duration;

    const RESPECTFUL_DURATION: Duration = Duration::from_millis(5);

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
        .insert_resource(InputMap::<Action>::new([(KeyCode::F, Action::PayRespects)]))
        .init_resource::<Respect>()
        .add_system(hold_f_to_pay_respects);

    // Initializing
    app.update();

    // Press
    app.send_input(KeyCode::F);

    // Hold
    std::thread::sleep(2 * RESPECTFUL_DURATION);

    // Check
    app.update();
    assert!(app
        .world
        .resource::<ActionState<Action>>()
        .pressed(Action::PayRespects));
}
