use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy::input::InputPlugin)
        .add_plugin(InputManagerPlugin::<Action>::default())
        .add_startup_system(spawn_players)
        .run();
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum Action {
    Left,
    Right,
    Jump,
}

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
        let mut input_map = match player {
            Player::One => InputMap::new([
                (KeyCode::A, Action::Left),
                (KeyCode::D, Action::Right),
                (KeyCode::W, Action::Jump),
            ])
            // This is a quick and hacky solution:
            // you should coordinate with the `Gamepads` resource to determine the correct gamepad for each player
            // and gracefully handle disconnects
            // Note that this step is not required:
            // if it is skipped all input maps will read from all connected gamepads
            .set_gamepad(Gamepad { id: 0 })
            .build(),
            Player::Two => InputMap::new([
                (KeyCode::Left, Action::Left),
                (KeyCode::Right, Action::Right),
                (KeyCode::Up, Action::Jump),
            ])
            .set_gamepad(Gamepad { id: 1 })
            .build(),
        };

        // Each player will use the same gamepad controls, but on seperate gamepads
        input_map.insert_multiple([
            (GamepadButtonType::DPadLeft, Action::Left),
            (GamepadButtonType::DPadRight, Action::Right),
            (GamepadButtonType::DPadUp, Action::Jump),
            (GamepadButtonType::South, Action::Jump),
        ]);

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
