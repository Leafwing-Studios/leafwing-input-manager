//! Demonstrates how to create default controls for an `Actionlike` and add it to an `InputMap`

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(InputManagerPlugin::<PlayerAction>::default())
        .add_systems(Startup, spawn_player)
        .add_systems(Update, use_actions)
        .run();
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum PlayerAction {
    #[actionlike(DualAxis)]
    Run,
    Jump,
    UseItem,
}

impl PlayerAction {
    /// Define the default bindings to the input
    fn default_input_map() -> InputMap<Self> {
        let mut input_map = InputMap::default();

        // Default gamepad input bindings
        input_map.insert_dual_axis(Self::Run, GamepadStick::LEFT);
        input_map.insert(Self::Jump, GamepadButton::South);
        input_map.insert(Self::UseItem, GamepadButton::RightTrigger2);

        // Default kbm input bindings
        input_map.insert_dual_axis(Self::Run, VirtualDPad::wasd());
        input_map.insert(Self::Jump, KeyCode::Space);
        input_map.insert(Self::UseItem, MouseButton::Left);

        input_map
    }
}

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands) {
    // Spawn the player with the default input_map
    commands
        .spawn(PlayerAction::default_input_map())
        .insert(Player);
}

fn use_actions(query: Query<&ActionState<PlayerAction>, With<Player>>) {
    let action_state = query.single();

    if action_state.axis_pair(&PlayerAction::Run) != Vec2::ZERO {
        println!(
            "Moving in direction {}",
            action_state.clamped_axis_pair(&PlayerAction::Run)
        );
    }

    if action_state.just_pressed(&PlayerAction::Jump) {
        println!("Jumped!");
    }

    if action_state.just_pressed(&PlayerAction::UseItem) {
        println!("Used an Item!");
    }
}
