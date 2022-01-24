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

fn press_f(mut input: ResMut<Input<KeyCode>>) {
    input.press(KeyCode::F);
}

#[test]
fn action_state_change_detection() {
    use bevy::ecs::schedule::ShouldRun;
    use bevy::input::InputPlugin;

    let mut app = App::new();

    app.add_plugins(MinimalPlugins)
        .add_plugin(InputPlugin)
        .add_plugin(InputManagerPlugin::<Action>::default())
        .add_startup_system(spawn_player)
        .add_system(
            press_f.with_run_criteria(|mut run_this_frame: Local<bool>| {
                // Run this system every other frame
                let run_next_frame = !*run_this_frame;
                *run_this_frame = run_next_frame;
                if run_next_frame {
                    ShouldRun::No
                } else {
                    ShouldRun::Yes
                }
            }),
        )
        .add_system(action_state_changed_iff_input_changed);

    for _ in 0..10 {
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
