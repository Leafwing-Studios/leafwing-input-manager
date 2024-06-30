//! Oftentimes your input actions can be handled globally and are
//! best represented as a [`Resource`].
//!
//! This example demonstrates how to create a simple `ActionLike`
//! and include it as a resource in a bevy app.

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(InputManagerPlugin::<PlayerAction>::default())
        // Initialize the ActionState resource
        .init_resource::<ActionState<PlayerAction>>()
        // Insert the InputMap resource
        .insert_resource(PlayerAction::mkb_input_map())
        .add_systems(Update, move_player)
        .run();
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum PlayerAction {
    Move,
    Jump,
}

// Exhaustively match `PlayerAction` and define the default bindings to the input
impl PlayerAction {
    fn mkb_input_map() -> InputMap<Self> {
        InputMap::new([(Self::Jump, KeyCode::Space)])
            .with_dual_axis(Self::Move, KeyboardVirtualDPad::WASD)
    }
}

fn move_player(
    // action_state is stored as a resource
    action_state: Res<ActionState<PlayerAction>>,
) {
    if action_state.pressed(&PlayerAction::Move) {
        // We're working with gamepads, so we want to defensively ensure that we're using the clamped values
        let axis_pair = action_state.clamped_axis_pair(&PlayerAction::Move);
        println!("Move: ({}, {})", axis_pair.x, axis_pair.y);
    }

    if action_state.pressed(&PlayerAction::Jump) {
        println!("Jumping!");
    }
}
