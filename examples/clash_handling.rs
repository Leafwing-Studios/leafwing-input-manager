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
    // Setting the clash strategy; swap out the variant
    // to play with different behavior!
    input_map.clash_strategy = ClashStrategy::PrioritizeLongest;
    //input_map.clash_strategy = ClashStrategy::PressAll;
    //input_map.clash_strategy = ClashStrategy::UseActionOrder;

    // Setting up input mappings in the obvious way
    input_map.insert_multiple([(One, Key1), (Two, Key2), (Three, Key3)]);

    input_map.insert_chord(OneAndTwo, [Key1, Key2]);
    input_map.insert_chord(OneAndThree, [Key1, Key3]);
    input_map.insert_chord(TwoAndThree, [Key2, Key3]);

    input_map.insert_chord(OneAndTwoAndThree, [Key1, Key2, Key3]);

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
