use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use leafwing_input_manager::{errors::NearlySingularConversion, orientation::Direction};

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

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum ArpgAction {
    // Movement
    Up,
    Down,
    Left,
    Right,
    // Abilities
    Ability1,
    Ability2,
    Ability3,
    Ability4,
    Ultimate,
}

impl ArpgAction {
    // Lists like this can be very useful for quickly matching subsets of actions
    const DIRECTIONS: [Self; 4] = [
        ArpgAction::Up,
        ArpgAction::Down,
        ArpgAction::Left,
        ArpgAction::Right,
    ];

    fn direction(self) -> Option<Direction> {
        match self {
            ArpgAction::Up => Some(Direction::NORTH),
            ArpgAction::Down => Some(Direction::SOUTH),
            ArpgAction::Left => Some(Direction::EAST),
            ArpgAction::Right => Some(Direction::WEST),
            _ => None,
        }
    }
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
        // This allows us to replace `ArpgAction::Up` with `Up`,
        // significantly reducing boilerplate
        use ArpgAction::*;
        let mut input_map = InputMap::default();

        // Movement
        input_map.insert(KeyCode::Up, Up);
        input_map.insert(GamepadButtonType::DPadUp, Up);

        input_map.insert(KeyCode::Down, Down);
        input_map.insert(GamepadButtonType::DPadDown, Down);

        input_map.insert(KeyCode::Left, Left);
        input_map.insert(GamepadButtonType::DPadLeft, Left);

        input_map.insert(KeyCode::Right, Right);
        input_map.insert(GamepadButtonType::DPadRight, Right);

        // Abilities
        input_map.insert(KeyCode::Q, Ability1);
        input_map.insert(GamepadButtonType::West, Ability1);
        input_map.insert(MouseButton::Left, Ability1);

        input_map.insert(KeyCode::W, Ability2);
        input_map.insert(GamepadButtonType::North, Ability2);
        input_map.insert(MouseButton::Right, Ability2);

        input_map.insert(KeyCode::E, Ability3);
        input_map.insert(GamepadButtonType::East, Ability3);

        input_map.insert(KeyCode::Space, Ability4);
        input_map.insert(GamepadButtonType::South, Ability4);

        input_map.insert(KeyCode::R, Ultimate);
        input_map.insert(GamepadButtonType::LeftTrigger2, Ultimate);

        input_map
    }
}

fn spawn_player(mut commands: Commands) {
    commands.spawn_bundle(PlayerBundle {
        player: Player,
        input_manager: InputManagerBundle {
            input_map: PlayerBundle::default_input_map(),
            ..default()
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
        let mut direction_vector = Vec2::ZERO;

        for input_direction in ArpgAction::DIRECTIONS {
            if action_state.pressed(input_direction) {
                if let Some(direction) = input_direction.direction() {
                    // Sum the directions as 2D vectors
                    direction_vector += Vec2::from(direction);
                }
            }
        }

        // Then reconvert at the end, normalizing the magnitude
        let net_direction: Result<Direction, NearlySingularConversion> =
            direction_vector.try_into();

        if let Ok(direction) = net_direction {
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

    let mut direction_vector = Vec2::ZERO;

    for input_direction in ArpgAction::DIRECTIONS {
        if action_state.pressed(input_direction) {
            if let Some(direction) = input_direction.direction() {
                // Sum the directions as 2D vectors
                direction_vector += Vec2::from(direction);
            }
        }
    }

    // Then reconvert at the end, normalizing the magnitude
    let net_direction: Result<Direction, NearlySingularConversion> = direction_vector.try_into();

    if let Ok(direction) = net_direction {
        event_writer.send(PlayerWalk { direction });
    }
}
