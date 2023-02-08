//! For some controls, such as the classical WASD movement controls,
//! we don't care about the actual output of the keys, but rather where they are positioned.
//!
//! For example, on the French AZERTY keyboard layout, these keys are not in the standard triangle pattern,
//! so using them for player movement would feel very awkward.
//!
//! Instead, we can base our bindings on the physical position of the keys.
//! This functionality is provided by _scan codes_.
//!
//! In order to not deal with arbitrary numbers to define the key positions,
//! we can use [`QwertyScanCode`] to define the name of the keys on the US QWERTY layout.
//! The mapping to the other keyboard layouts is done automatically.

use bevy::prelude::*;
use leafwing_input_manager::{prelude::*, scan_codes::QwertyScanCode};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(InputManagerPlugin::<Action>::default())
        // The InputMap and ActionState components will be added to any entity with the Player component
        .add_startup_system(spawn_player)
        // Read the ActionState in your systems using queries!
        .add_system(jump)
        .run();
}

// This is the list of "things in the game I want to be able to do based on input"
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum Action {
    Forward,
    Left,
    Backward,
    Right,
}

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands) {
    commands
        .spawn(InputManagerBundle::<Action> {
            // Stores "which actions are currently pressed"
            action_state: ActionState::default(),
            // We define the name of the keys based on the US QWERTY layout.
            // The keys the user will actually have to press depends on their selected keyboard layout.
            // However, the _position_ of the keys will be the same, regardless of layout.
            // This way, every player can use the classic triangle shaped key arrangement.
            input_map: InputMap::new([
                (QwertyScanCode::W, Action::Forward),
                (QwertyScanCode::A, Action::Left),
                (QwertyScanCode::S, Action::Backward),
                (QwertyScanCode::D, Action::Right),
            ]),
        })
        .insert(Player);
}

// Query for the `ActionState` component in your game logic systems!
fn jump(query: Query<&ActionState<Action>, With<Player>>) {
    let action_state = query.single();

    // Each action has a button-like state of its own that you can check
    if action_state.just_pressed(Action::Forward) {
        println!("Going forward!");
    } else if action_state.just_pressed(Action::Left) {
        println!("Going left!");
    } else if action_state.just_pressed(Action::Backward) {
        println!("Going backward!");
    } else if action_state.just_pressed(Action::Right) {
        println!("Going right!");
    }
}
