use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use strum::EnumIter;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // This plugin maps inputs to an input-type agnostic action-state
        // We need to provide it with an enum which stores the possible actions a player could take
        .add_plugin(InputManagerPlugin::<Action>::default())
        // The InputMap and ActionState components will be added to any entity with the Player component
        .add_startup_system(spawn_player)
        // Read the ActionState in your systems using queries!
        .add_system(jump)
        .run();
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, EnumIter)]
enum Action {
    Run,
    Jump,
}

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands) {
    // Adding new bindings is easy!
    let mut input_map = InputMap::default();
    input_map.insert(Action::Jump, KeyCode::Space);

    commands
        .spawn()
        .insert(Player)
        .insert_bundle(InputManagerBundle::<Action> {
            // Stores "which virtual action buttons are currently pressed"
            action_state: ActionState::default(),
            // Stores input bindings
            input_map,
        });
}

// Query for the ActionState component in your game logic systems!
fn jump(query: Query<&ActionState<Action>, With<Player>>) {
    let action_state = query.single();
    // Each action variant has a virtual button of its own
    if action_state.just_pressed(Action::Jump) {
        println!("I'm jumping!");
    }
}
