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

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum Action {
    Move,
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
            input_map: InputMap::new([(
                UserInput::VirtualDPad {
                    up: KeyCode::W.into(),
                    down: KeyCode::S.into(),
                    left: KeyCode::A.into(),
                    right: KeyCode::D.into(),
                },
                Action::Move,
            )])
            .build(),
        });
}

// Query for the `ActionState` component in your game logic systems!
fn move_player(query: Query<&ActionState<Action>, With<Player>>) {
    let action_state = query.single();
    // Each action has a button-like state of its own that you can check
    if action_state.pressed(Action::Move) {
        let axis_pair = action_state.action_axis_pair(Action::Move).unwrap();
        println!("Move:");
        println!("   distance: {}", axis_pair.magnitude());
        println!("          x: {}", axis_pair.x());
        println!("          y: {}", axis_pair.y());
    }
}
