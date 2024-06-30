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

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum PlayerAction {
    Run,
    Jump,
    UseItem,
}

impl Actionlike for PlayerAction {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            PlayerAction::Run => InputControlKind::DualAxis,
            _ => InputControlKind::Button,
        }
    }
}

impl PlayerAction {
    /// Define the default bindings to the input
    fn default_input_map() -> InputMap<Self> {
        let mut input_map = InputMap::default();

        // Default gamepad input bindings
        input_map.insert_dual_axis(Self::Run, GamepadStick::LEFT);
        input_map.insert(Self::Jump, GamepadButtonType::South);
        input_map.insert(Self::UseItem, GamepadButtonType::RightTrigger2);

        // Default kbm input bindings
        input_map.insert_dual_axis(Self::Run, KeyboardVirtualDPad::WASD);
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
        .spawn(InputManagerBundle::with_map(
            PlayerAction::default_input_map(),
        ))
        .insert(Player);
}

fn use_actions(query: Query<&ActionState<PlayerAction>, With<Player>>) {
    let action_state = query.single();

    // When the default input for `PlayerAction::Run` is pressed, print the clamped direction of the axis
    if action_state.pressed(&PlayerAction::Run) {
        println!(
            "Moving in direction {}",
            action_state.clamped_axis_pair(&PlayerAction::Run).xy()
        );
    }

    // When the default input for `PlayerAction::Jump` is pressed, print "Jump!"
    if action_state.just_pressed(&PlayerAction::Jump) {
        println!("Jumped!");
    }

    // When the default input for `PlayerAction::UseItem` is pressed, print "Used an Item!"
    if action_state.just_pressed(&PlayerAction::UseItem) {
        println!("Used an Item!");
    }
}
