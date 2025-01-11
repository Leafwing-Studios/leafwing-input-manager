//! Demonstrates how to register gamepads in local multiplayer fashion

use bevy::{prelude::*, utils::HashMap};
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(InputManagerPlugin::<Action>::default())
        .init_resource::<JoinedPlayers>()
        .add_systems(Update, (join, jump, disconnect))
        .run();
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum Action {
    Jump,
    Disconnect,
}

// This is used to check if a player already exists and which entity to disconnect
//
// This maps gamepad entity to player.
#[derive(Resource, Default)]
struct JoinedPlayers(pub HashMap<Entity, Entity>);

#[derive(Component)]
struct Player {
    // This gamepad is used to index each player
    gamepad: Entity,
}

fn join(
    mut commands: Commands,
    mut joined_players: ResMut<JoinedPlayers>,
    gamepads: Query<(Entity, &Gamepad)>,
) {
    for (gamepad_entity, gamepad) in gamepads.iter() {
        // Join the game when both bumpers (L+R) on the controller are pressed
        // We drop down the Bevy's input to get the input from each gamepad
        if gamepad.pressed(GamepadButton::LeftTrigger)
            && gamepad.pressed(GamepadButton::RightTrigger)
        {
            // Make sure a player cannot join twice
            if !joined_players.0.contains_key(&gamepad_entity) {
                println!("Player {} has joined the game!", gamepad_entity);

                let input_map = InputMap::new([
                    (Action::Jump, GamepadButton::South),
                    (Action::Disconnect, GamepadButton::Select),
                ])
                // Make sure to set the gamepad or all gamepads will be used!
                .with_gamepad(gamepad_entity);
                let player = commands
                    .spawn(input_map)
                    .insert(Player {
                        gamepad: gamepad_entity,
                    })
                    .id();

                // Insert the created player and its gamepad to the hashmap of joined players
                // Since uniqueness was already checked above, we can insert here unchecked
                joined_players
                    .0
                    .insert_unique_unchecked(gamepad_entity, player);
            }
        }
    }
}

fn jump(action_query: Query<(&ActionState<Action>, &Player)>) {
    // Iterate through each player to see if they jumped
    for (action_state, player) in action_query.iter() {
        if action_state.just_pressed(&Action::Jump) {
            println!("Player {} jumped!", player.gamepad);
        }
    }
}

fn disconnect(
    mut commands: Commands,
    action_query: Query<(&ActionState<Action>, &Player)>,
    mut joined_players: ResMut<JoinedPlayers>,
) {
    for (action_state, player) in action_query.iter() {
        if action_state.pressed(&Action::Disconnect) {
            let player_entity = *joined_players.0.get(&player.gamepad).unwrap();

            // Despawn the disconnected player and remove them from the joined player list
            commands.entity(player_entity).despawn();
            joined_players.0.remove(&player.gamepad);

            println!("Player {} has disconnected!", player.gamepad);
        }
    }
}
