use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use strum_macros::EnumIter;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugin(InputManagerPlugin::<Action>::default())
        .add_startup_system(spawn_players)
        .run();
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, EnumIter)]
enum Action {
    Left,
    Right,
    Jump,
}

impl Actionlike for Action {}

#[derive(Component)]
enum Player {
    One,
    Two,
}

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    #[bundle]
    input_manager: InputManagerBundle<Action>,
}

impl PlayerBundle {
    fn input_map(player: Player) -> InputMap<Action> {
        let mut input_map = InputMap::default();

        match player {
            Player::One => {
                input_map.insert(Action::Left, KeyCode::A);
                input_map.insert(Action::Right, KeyCode::D);
                input_map.insert(Action::Jump, KeyCode::W);

                // This is a quick and hacky solution:
                // you should coordinate with the `Gamepads` resource to determine the correct gamepad for each player
                // and gracefully handle disconnects
                input_map.assign_gamepad(Gamepad(0));
            }
            Player::Two => {
                input_map.insert(Action::Left, KeyCode::Left);
                input_map.insert(Action::Right, KeyCode::Right);
                input_map.insert(Action::Jump, KeyCode::Up);

                input_map.assign_gamepad(Gamepad(1));
            }
        };

        // Each player will use the same gamepad controls, but on seperate gamepads
        input_map.insert(Action::Left, GamepadButtonType::DPadLeft);
        input_map.insert(Action::Right, GamepadButtonType::DPadLeft);
        input_map.insert(Action::Jump, GamepadButtonType::DPadUp);
        input_map.insert(Action::Jump, GamepadButtonType::South);

        input_map
    }
}

fn spawn_players(mut commands: Commands) {
    commands.spawn_bundle(PlayerBundle {
        player: Player::One,
        input_manager: InputManagerBundle {
            input_map: PlayerBundle::input_map(Player::One),
            ..Default::default()
        },
    });

    commands.spawn_bundle(PlayerBundle {
        player: Player::Two,
        input_manager: InputManagerBundle {
            input_map: PlayerBundle::input_map(Player::Two),
            ..Default::default()
        },
    });
}
