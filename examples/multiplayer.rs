use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(InputManagerPlugin::<Action>::default())
        .add_systems(Startup, spawn_players)
        .add_systems(Update, move_players)
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

impl Player {
    fn input_map(player: Player, gamepad_0: Entity, gamepad_1: Entity) -> InputMap<Action> {
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
            .with_gamepad(gamepad_0),
            Player::Two => InputMap::new([
                (Action::Left, KeyCode::ArrowLeft),
                (Action::Right, KeyCode::ArrowRight),
                (Action::Jump, KeyCode::ArrowUp),
            ])
            .with_gamepad(gamepad_1),
        };

        // Each player will use the same gamepad controls, but on separate gamepads.
        input_map.insert_multiple([
            (Action::Left, GamepadButton::DPadLeft),
            (Action::Right, GamepadButton::DPadRight),
            (Action::Jump, GamepadButton::DPadUp),
            (Action::Jump, GamepadButton::South),
        ]);

        input_map
    }
}

fn spawn_players(mut commands: Commands) {
    let gamepad_0 = commands.spawn(()).id();
    let gamepad_1 = commands.spawn(()).id();

    commands.spawn((
        Player::One,
        Player::input_map(Player::One, gamepad_0, gamepad_1),
    ));

    commands.spawn((
        Player::Two,
        Player::input_map(Player::Two, gamepad_0, gamepad_1),
    ));
}

fn move_players(player_query: Query<(&Player, &ActionState<Action>)>) {
    for (player, action_state) in player_query.iter() {
        let actions = action_state.get_just_pressed();
        if !actions.is_empty() {
            info!("Player {player:?} performed actions {actions:?}");
        }
    }
}
