use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(InputManagerPlugin::<Action>::default())
        .add_systems(Startup, spawn_players)
        .run();
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
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
    input_manager: InputManagerBundle<Action>,
}

impl PlayerBundle {
    fn input_map(player: Player) -> InputMap<Action> {
        let mut input_map = match player {
            Player::One => InputMap::new([
                (Action::Left, KeyCode::KeyA),
                (Action::Right, KeyCode::KeyD),
                (Action::Jump, KeyCode::KeyW),
            ])
            // This is a quick and hacky solution:
            // you should coordinate with the `Gamepads` resource to determine the correct gamepad for each player
            // and gracefully handle disconnects
            // Note that this step is not required:
            // if it is skipped, all input maps will read from all connected gamepads
            .set_gamepad(Gamepad { id: 0 })
            .build(),
            Player::Two => InputMap::new([
                (Action::Left, KeyCode::ArrowLeft),
                (Action::Right, KeyCode::ArrowRight),
                (Action::Jump, KeyCode::ArrowUp),
            ])
            .set_gamepad(Gamepad { id: 1 })
            .build(),
        };

        // Each player will use the same gamepad controls, but on separate gamepads.
        input_map.insert_multiple([
            (Action::Left, GamepadButtonType::DPadLeft),
            (Action::Right, GamepadButtonType::DPadRight),
            (Action::Jump, GamepadButtonType::DPadUp),
            (Action::Jump, GamepadButtonType::South),
        ]);

        input_map
    }
}

fn spawn_players(mut commands: Commands) {
    commands.spawn(PlayerBundle {
        player: Player::One,
        input_manager: InputManagerBundle {
            input_map: PlayerBundle::input_map(Player::One),
            ..Default::default()
        },
    });

    commands.spawn(PlayerBundle {
        player: Player::Two,
        input_manager: InputManagerBundle {
            input_map: PlayerBundle::input_map(Player::Two),
            ..Default::default()
        },
    });
}
