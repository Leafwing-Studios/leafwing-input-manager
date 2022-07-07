use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // This plugin maps inputs to an input-type agnostic action-state
        // We need to provide it with an enum which stores the possible actions a player could take
        .add_plugin(InputManagerPlugin::<Action>::default())
        // Spawn an entity with Player, InputMap, and ActionState components
        .add_startup_system(spawn_player)
        // Read the ActionState in your systems using queries!
        .add_system(move_player)
        .run();
}

// This is the list of "things in the game I want to be able to do based on input"
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum Action {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands) {
    commands
        .spawn()
        .insert(Player)
        .insert_bundle(InputManagerBundle::<Action> {
            // Stores "which actions are currently activated"
            action_state: ActionState::default(),
            // Describes how to convert from player inputs into those actions
            input_map: InputMap::new([
                (
                    // This describes which gamepad axis must be moved how much to trigger the action
                    GamepadAxisThreshold {
                        axis: GamepadAxisType::LeftStickY,
                        comparison: GamepadAxisComparison::Greater,
                        threshold: 0.1,
                    },
                    Action::Up,
                ),
                (
                    GamepadAxisThreshold {
                        axis: GamepadAxisType::LeftStickY,
                        comparison: GamepadAxisComparison::Less,
                        threshold: -0.1,
                    },
                    Action::Down,
                ),
                (
                    GamepadAxisThreshold {
                        axis: GamepadAxisType::LeftStickX,
                        comparison: GamepadAxisComparison::Less,
                        threshold: -0.1,
                    },
                    Action::Left,
                ),
                (
                    GamepadAxisThreshold {
                        axis: GamepadAxisType::LeftStickX,
                        comparison: GamepadAxisComparison::Greater,
                        threshold: 0.1,
                    },
                    Action::Right,
                ),
            ])
            // Listen for events on the first gamepad
            .set_gamepad(Gamepad(0))
            .build(),
        });
}

// Query for the `ActionState` component in your game logic systems!
fn move_player(query: Query<&ActionState<Action>, With<Player>>) {
    let action_state = query.single();
    // Each action has a button-like state of its own that you can check
    if action_state.pressed(Action::Up) {
        println!("Up");
    }
    if action_state.pressed(Action::Down) {
        println!("Down");
    }
    if action_state.pressed(Action::Left) {
        println!("Left");
    }
    if action_state.pressed(Action::Right) {
        println!("Right");
    }
}
