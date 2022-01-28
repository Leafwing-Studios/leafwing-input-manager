#![cfg(test)]
use bevy::ecs::query::ChangeTrackers;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use strum::EnumIter;

#[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Debug)]
enum Action {
    PayRespects,
}

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands) {
    let mut input_map = InputMap::<Action>::default();
    input_map.insert(Action::PayRespects, KeyCode::F);

    commands
        .spawn()
        .insert(Player)
        .insert_bundle(InputManagerBundle::<Action> {
            input_map,
            ..Default::default()
        });
}

#[test]
fn action_state_change_detection() {
    use bevy::input::InputPlugin;
    use leafwing_input_manager::MockInput;

    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
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
