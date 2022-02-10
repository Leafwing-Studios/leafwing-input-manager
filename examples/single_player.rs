use bevy::prelude::*;
use leafwing_input_manager::geometric_primitives::Direction;
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy::input::InputPlugin)
        // This plugin maps inputs to an input-type agnostic action-state
        // We need to provide it with an enum which stores the possible actions a player could take
        .add_plugin(InputManagerPlugin::<ArpgAction>::default())
        // The InputMap and ActionState components will be added to any entity with the Player component
        .add_startup_system(spawn_player)
        // The ActionState can be used directly
        .add_system(cast_fireball)
        // Or multiple parts of it can be inspected
        .add_system(player_dash)
        // Or it can be used to emit events for later processing
        .add_event::<PlayerWalk>()
        .add_system(player_walks)
        .run();
}

#[derive(Actionlike, PartialEq, Clone, Copy, Debug)]
enum ArpgAction {
    // Movement
    Movement(Direction),
    // Abilities
    Ability1,
    Ability2,
    Ability3,
    Ability4,
    Ultimate,
}

#[derive(Component)]
pub struct Player;

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    // This bundle must be added to your player entity
    // (or whatever else you wish to control)
    #[bundle]
    input_manager: InputManagerBundle<ArpgAction>,
}

impl PlayerBundle {
    fn default_input_map() -> InputMap<ArpgAction> {
        // This allows us to significantly reducing boilerplate
        use ArpgAction::*;
        let mut input_map = InputMap::default();

        // This is a quick and hacky solution:
        // you should coordinate with the `Gamepads` resource to determine the correct gamepad for each player
        // and gracefully handle disconnects
        input_map.set_gamepad(Gamepad(0));

        // Movement
        // FIXME: this doesn't work
        input_map.insert(Movement(Direction::NORTH), KeyCode::Up);
        input_map.insert(Movement(Direction::NORTH), GamepadButtonType::DPadUp);

        input_map.insert(Movement(Direction::SOUTH), KeyCode::Down);
        input_map.insert(Movement(Direction::SOUTH), GamepadButtonType::DPadDown);

        input_map.insert(Movement(Direction::EAST), KeyCode::Left);
        input_map.insert(Movement(Direction::EAST), GamepadButtonType::DPadLeft);

        input_map.insert(Movement(Direction::WEST), KeyCode::Right);
        input_map.insert(Movement(Direction::WEST), GamepadButtonType::DPadRight);

        // Abilities
        input_map.insert(Ability1, KeyCode::Q);
        input_map.insert(Ability1, GamepadButtonType::West);
        input_map.insert(Ability1, MouseButton::Left);

        input_map.insert(Ability2, KeyCode::W);
        input_map.insert(Ability2, GamepadButtonType::North);
        input_map.insert(Ability2, MouseButton::Right);

        input_map.insert(Ability3, KeyCode::E);
        input_map.insert(Ability3, GamepadButtonType::East);

        input_map.insert(Ability4, KeyCode::Space);
        input_map.insert(Ability4, GamepadButtonType::South);

        input_map.insert(Ultimate, KeyCode::R);
        input_map.insert(Ultimate, GamepadButtonType::LeftTrigger2);

        input_map
    }
}

fn spawn_player(mut commands: Commands) {
    commands.spawn_bundle(PlayerBundle {
        player: Player,
        input_manager: InputManagerBundle {
            input_map: PlayerBundle::default_input_map(),
            action_state: ActionState::default(),
        },
    });
}

fn cast_fireball(query: Query<&ActionState<ArpgAction>, With<Player>>) {
    let action_state = query.single();

    if action_state.just_pressed(ArpgAction::Ability1) {
        println!("Fwoosh!");
    }
}

fn player_dash(query: Query<&ActionState<ArpgAction>, With<Player>>) {
    let action_state = query.single();

    if action_state.just_pressed(ArpgAction::Ability4) {
        // The internal `Direction::NEUTRAL` value is irrelevant;
        // we merely need to get the information from the data
        // FIXME: this is extremely ugly and unintuitive.
        if let ArpgAction::Movement(direction) =
            action_state.action_value(ArpgAction::Movement(Direction::NEUTRAL))
        {
            println!("Dashing in {direction:?}");
        }
    }
}

pub struct PlayerWalk {
    pub direction: Direction,
}

fn player_walks(
    query: Query<&ActionState<ArpgAction>, With<Player>>,
    mut event_writer: EventWriter<PlayerWalk>,
) {
    let action_state = query.single();

    if let ArpgAction::Movement(direction) =
        action_state.action_value(ArpgAction::Movement(Direction::NEUTRAL))
    {
        if direction != Direction::NEUTRAL {
            event_writer.send(PlayerWalk { direction });
        }
    }
}
