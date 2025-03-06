use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // This plugin maps inputs to an input-type agnostic action-state
        // We need to provide it with an enum which stores the possible actions a player could take
        .add_plugins(InputManagerPlugin::<Action>::default())
        // Spawn an entity with Player, InputMap, and ActionState components
        .add_systems(Startup, spawn_player)
        // Read the ActionState in your systems using queries!
        .add_systems(Update, move_player)
        .run();
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
#[actionlike(DualAxis)]
enum Action {
    Move,
}

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands) {
    // Stores "which actions are currently activated"
    let input_map = InputMap::default().with_dual_axis(
        Action::Move,
        // Define a virtual D-pad using four arbitrary buttons.
        VirtualDPad::new(
            KeyCode::KeyW,
            KeyCode::KeyS,
            GamepadButton::DPadLeft,
            GamepadButton::DPadRight,
        ),
    );
    commands.spawn(input_map).insert(Player);
}

// Query for the `ActionState` component in your game logic systems!
fn move_player(query: Query<&ActionState<Action>, With<Player>>) {
    let action_state = query.single();
    if action_state.axis_pair(&Action::Move) != Vec2::ZERO {
        // Virtual direction pads are one of the types which return a DualAxis. The values will be
        // represented as `-1.0`, `0.0`, or `1.0` depending on the combination of buttons pressed.
        let axis_pair = action_state.axis_pair(&Action::Move);
        println!("Move:");
        println!("   distance: {}", axis_pair.length());
        println!("          x: {}", axis_pair.x);
        println!("          y: {}", axis_pair.y);
    }
}
