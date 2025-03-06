#![cfg(feature = "keyboard")]

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, Clone, Copy, Debug, Reflect, PartialEq, Eq, Hash)]
enum Action {
    PayRespects,
}

// A resource that represents whether respects have been paid or not
#[derive(Resource, Default, PartialEq, Debug)]
struct Respect(bool);

fn pay_respects(
    action_state_query: Query<&ActionState<Action>, With<Player>>,
    action_state_resource: Option<Res<ActionState<Action>>>,
    mut respect: ResMut<Respect>,
) {
    if let Ok(action_state) = action_state_query.get_single() {
        if action_state.pressed(&Action::PayRespects) {
            respect.0 = true;
        }
    }
    if let Some(action_state) = action_state_resource {
        if action_state.pressed(&Action::PayRespects) {
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
        .spawn(InputMap::new([(Action::PayRespects, KeyCode::KeyF)]))
        .insert(Player);
}

#[test]
fn disable_input() {
    use bevy::input::InputPlugin;

    let mut app = App::new();

    // Here we spawn a player and create a global action state to check if [`DisableInput`]
    // releases correctly both
    app.add_plugins(MinimalPlugins)
        .add_plugins(InputPlugin)
        .add_plugins(InputManagerPlugin::<Action>::default())
        .add_systems(Startup, spawn_player)
        .init_resource::<ActionState<Action>>()
        .insert_resource(InputMap::<Action>::new([(
            Action::PayRespects,
            KeyCode::KeyF,
        )]))
        .init_resource::<Respect>()
        .add_systems(Update, pay_respects)
        .add_systems(PreUpdate, respect_fades);

    // Press F to pay respects
    KeyCode::KeyF.press(app.world_mut());
    app.update();
    let respect = app.world().resource::<Respect>();
    assert_eq!(*respect, Respect(true));

    // Disable the global input
    let mut action_state = app.world_mut().resource_mut::<ActionState<Action>>();
    action_state.disable_all_actions();

    // But the player is still paying respects
    app.update();
    let respect = app.world().resource::<Respect>();
    assert_eq!(*respect, Respect(true));

    // Disable the player's input too
    let mut action_state = app
        .world_mut()
        .query_filtered::<&mut ActionState<Action>, With<Player>>()
        .single_mut(app.world_mut());
    action_state.disable_all_actions();

    // Now, all respect has faded
    app.update();
    let respect = app.world().resource::<Respect>();
    assert_eq!(*respect, Respect(false));

    // And even pressing F cannot bring it back
    KeyCode::KeyF.press(app.world_mut());
    app.update();
    let respect = app.world().resource::<Respect>();
    assert_eq!(*respect, Respect(false));

    // Re-enable the global input
    let mut action_state = app.world_mut().resource_mut::<ActionState<Action>>();
    action_state.enable_all_actions();

    // And it will start paying respects again
    app.update();
    let respect = app.world().resource::<Respect>();
    assert_eq!(*respect, Respect(true));
}

#[test]
fn release_when_input_map_removed() {
    use bevy::input::InputPlugin;

    let mut app = App::new();

    // Spawn a player and create a global action state.
    app.add_plugins(MinimalPlugins)
        .add_plugins(InputPlugin)
        .add_plugins(InputManagerPlugin::<Action>::default())
        .add_systems(Startup, spawn_player)
        .init_resource::<ActionState<Action>>()
        .insert_resource(InputMap::<Action>::new([(
            Action::PayRespects,
            KeyCode::KeyF,
        )]))
        .init_resource::<Respect>()
        .add_systems(Update, (pay_respects, remove_input_map))
        .add_systems(PreUpdate, respect_fades);

    // Press F to pay respects
    KeyCode::KeyF.press(app.world_mut());
    app.update();
    let respect = app.world().resource::<Respect>();
    assert_eq!(*respect, Respect(true));

    // Remove the InputMap
    app.world_mut().remove_resource::<InputMap<Action>>();
    // Needs an extra frame for the resource removed detection to release inputs
    app.update();

    // Now, all respect has faded
    app.update();
    let respect = app.world().resource::<Respect>();
    assert_eq!(*respect, Respect(false));

    // And even pressing F cannot bring it back
    KeyCode::KeyF.press(app.world_mut());
    app.update();
    let respect = app.world().resource::<Respect>();
    assert_eq!(*respect, Respect(false));
}

#[cfg(feature = "timing")]
#[test]
fn duration() {
    use bevy::input::InputPlugin;
    use bevy::utils::Duration;

    const RESPECTFUL_DURATION: Duration = Duration::from_millis(5);

    fn hold_f_to_pay_respects(
        action_state: Res<ActionState<Action>>,
        mut respect: ResMut<Respect>,
    ) {
        if action_state.pressed(&Action::PayRespects)
            // Unrealistically disrespectful, but makes the tests faster
            && action_state.current_duration(&Action::PayRespects) > RESPECTFUL_DURATION
        {
            respect.0 = true;
        }
    }

    let mut app = App::new();

    app.add_plugins(MinimalPlugins)
        .add_plugins(InputPlugin)
        .add_plugins(InputManagerPlugin::<Action>::default())
        .add_systems(Startup, spawn_player)
        .init_resource::<ActionState<Action>>()
        .insert_resource(InputMap::<Action>::new([(
            Action::PayRespects,
            KeyCode::KeyF,
        )]))
        .init_resource::<Respect>()
        .add_systems(Update, hold_f_to_pay_respects);

    // Initializing
    app.update();

    // Press
    KeyCode::KeyF.press(app.world_mut());

    // Hold
    std::thread::sleep(2 * RESPECTFUL_DURATION);

    // Check
    app.update();
    assert!(app
        .world()
        .resource::<ActionState<Action>>()
        .pressed(&Action::PayRespects));
}
