//! Oftentimes your input actions can be handled globally and are
//! best represented as a [`Resource`].
//!
//! This example demonstrates how to create a simple `ActionLike`
//! and include it as a resource in a bevy app.

use bevy::prelude::*;
use leafwing_input_manager::{common_conditions::action_pressed, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(InputManagerPlugin::<PlayerAction>::default())
        // Initialize the ActionState resource
        .init_resource::<ActionState<PlayerAction>>()
        // Insert the InputMap resource
        .insert_resource(PlayerAction::mkb_input_map())
        // System to read the ActionState resource
        .add_systems(Update, move_player)
        .add_systems(Update, jump.run_if(action_pressed(PlayerAction::Jump)))
        .run();
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum PlayerAction {
    #[actionlike(DualAxis)]
    Move,
    Jump,
}

// Exhaustively match `PlayerAction` and define the default bindings to the input
impl PlayerAction {
    fn mkb_input_map() -> InputMap<Self> {
        InputMap::new([(Self::Jump, KeyCode::Space)])
            .with_dual_axis(Self::Move, VirtualDPad::wasd())
    }
}

fn move_player(
    // action_state is stored as a resource
    action_state: Res<ActionState<PlayerAction>>,
) {
    let axis_pair = action_state.clamped_axis_pair(&PlayerAction::Move);
    println!("Move: ({}, {})", axis_pair.x, axis_pair.y);
}

// System that runs when the Jump action is just pressed as a result of the run condition
fn jump() {
    println!("Jumping!");
}
