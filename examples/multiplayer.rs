use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(InputManagerPlugin::<Action>::default())
        .add_systems(Startup, spawn_players)
        .add_systems(Update, log_actions)
        .run();
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum Action {
    Left,
    Right,
    Jump,
}

#[derive(Component, Debug)]
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
        // Each player will use the same gamepad controls, but on separate gamepads.
        let mut input_map = InputMap::new([
            (GamepadButtonType::DPadLeft, Action::Left),
            (GamepadButtonType::DPadRight, Action::Right),
            (GamepadButtonType::DPadUp, Action::Jump),
            (GamepadButtonType::South, Action::Jump),
        ]);
        match player {
            Player::One => input_map
                .insert_multiple([
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
            Player::Two => input_map
                .insert_multiple([
                    (KeyCode::Left, Action::Left),
                    (KeyCode::Right, Action::Right),
                    (KeyCode::Up, Action::Jump),
                ])
                .set_gamepad(Gamepad { id: 1 })
                .build(),
        };

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

fn log_actions(query: Query<(&Player, &ActionState<Action>)>) {
    for (player, action_state) in query.iter() {
        if action_state.just_pressed(Action::Left) {
            println!("Player {:?} pressed left", player);
        }
        if action_state.just_pressed(Action::Right) {
            println!("Player {:?} pressed right", player);
        }
        if action_state.just_pressed(Action::Jump) {
            println!("Player {:?} pressed jump", player);
        }
    }
}
