//! Clashes occur when two actions would be triggered by the same combination of buttons
//! and one input is a strict subset of the other.
//!
//! See [`ClashStrategy`] for more details.

use bevy::prelude::*;
use leafwing_input_manager::clashing_inputs::ClashStrategy;
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(InputManagerPlugin::<TestAction>::default())
        .add_startup_system(spawn_input_map)
        .add_system(report_pressed_actions)
        // Change the value of this resource to change how clashes should be handled in your game
        .insert_resource(ClashStrategy::PrioritizeLongest)
        .run()
}

#[derive(Actionlike, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TestAction {
    One,
    Two,
    Three,
    OneAndTwo,
    OneAndThree,
    TwoAndThree,
    OneAndTwoAndThree,
}

fn spawn_input_map(mut commands: Commands) {
    use KeyCode::*;
    use TestAction::*;

    let mut input_map = InputMap::default();

    // Setting up input mappings in the obvious way
    input_map.insert_multiple([(Key1, One), (Key2, Two), (Key3, Three)]);

    input_map.insert_chord([Key1, Key2], OneAndTwo);
    input_map.insert_chord([Key1, Key3], OneAndThree);
    input_map.insert_chord([Key2, Key3], TwoAndThree);

    input_map.insert_chord([Key1, Key2, Key3], OneAndTwoAndThree);

    commands.spawn_bundle(InputManagerBundle {
        input_map,
        ..Default::default()
    });
}

fn report_pressed_actions(
    query: Query<&ActionState<TestAction>, Changed<ActionState<TestAction>>>,
) {
    let action_state = query.single();
    for action in TestAction::variants() {
        if action_state.just_pressed(action) {
            dbg!(action);
        }
    }
}
