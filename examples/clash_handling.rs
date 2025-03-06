//! Clashes occur when two actions would be triggered by the same combination of buttons
//! and one input is a strict subset of the other.
//!
//! See [`ClashStrategy`] for more details.

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(InputManagerPlugin::<TestAction>::default())
        .add_systems(Startup, spawn_input_map)
        .add_systems(Update, report_pressed_actions)
        // Change the value of this resource to change how clashes should be handled in your game
        .insert_resource(ClashStrategy::PrioritizeLongest)
        .run();
}

#[derive(Actionlike, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
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

    // Setting up input mappings in the obvious way
    let mut input_map = InputMap::new([(One, Digit1), (Two, Digit2), (Three, Digit3)]);

    input_map.insert(OneAndTwo, ButtonlikeChord::new([Digit1, Digit2]));
    input_map.insert(OneAndThree, ButtonlikeChord::new([Digit1, Digit3]));
    input_map.insert(TwoAndThree, ButtonlikeChord::new([Digit2, Digit3]));

    input_map.insert(
        OneAndTwoAndThree,
        ButtonlikeChord::new([Digit1, Digit2, Digit3]),
    );

    commands.spawn(input_map);
}

fn report_pressed_actions(
    query: Query<&ActionState<TestAction>, Changed<ActionState<TestAction>>>,
) {
    let action_state = query.single();
    dbg!(action_state.get_just_pressed());
}
