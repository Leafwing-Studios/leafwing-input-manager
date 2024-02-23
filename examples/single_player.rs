use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use leafwing_input_manager::{errors::NearlySingularConversion, orientation::Direction};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // This plugin maps inputs to an input-type agnostic action-state
        // We need to provide it with an enum which stores the possible actions a player could take
        .add_plugins(InputManagerPlugin::<ArpgAction>::default())
        // The InputMap and ActionState components will be added to any entity with the Player component
        .add_systems(Startup, spawn_player)
        // The ActionState can be used directly
        .add_systems(Update, cast_fireball)
        // Or multiple parts of it can be inspected
        .add_systems(Update, player_dash)
        // Or it can be used to emit events for later processing
        .add_event::<PlayerWalk>()
        .add_systems(Update, player_walks)
        .run();
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
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
            ArpgAction::Left => Some(Direction::WEST),
            ArpgAction::Right => Some(Direction::EAST),
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
    input_manager: InputManagerBundle<ArpgAction>,
}

impl PlayerBundle {
    fn default_input_map() -> InputMap<ArpgAction> {
        // This allows us to replace `ArpgAction::Up` with `Up`,
        // significantly reducing boilerplate
        use ArpgAction::*;
        let mut input_map = InputMap::default();

        // Movement
        input_map.insert(Up, KeyCode::ArrowUp);
        input_map.insert(Up, GamepadButtonType::DPadUp);

        input_map.insert(Down, KeyCode::ArrowDown);
        input_map.insert(Down, GamepadButtonType::DPadDown);

        input_map.insert(Left, KeyCode::ArrowLeft);
        input_map.insert(Left, GamepadButtonType::DPadLeft);

        input_map.insert(Right, KeyCode::ArrowRight);
        input_map.insert(Right, GamepadButtonType::DPadRight);

        // Abilities
        input_map.insert(Ability1, KeyCode::KeyQ);
        input_map.insert(Ability1, GamepadButtonType::West);
        input_map.insert(Ability1, MouseButton::Left);

        input_map.insert(Ability2, KeyCode::KeyW);
        input_map.insert(Ability2, GamepadButtonType::North);
        input_map.insert(Ability2, MouseButton::Right);

        input_map.insert(Ability3, KeyCode::KeyE);
        input_map.insert(Ability3, GamepadButtonType::East);

        input_map.insert(Ability4, KeyCode::Space);
        input_map.insert(Ability4, GamepadButtonType::South);

        input_map.insert(Ultimate, KeyCode::KeyR);
        input_map.insert(Ultimate, GamepadButtonType::LeftTrigger2);

        input_map
    }
}

fn spawn_player(mut commands: Commands) {
    commands.spawn(PlayerBundle {
        player: Player,
        input_manager: InputManagerBundle::with_map(PlayerBundle::default_input_map()),
    });
}

fn cast_fireball(query: Query<&ActionState<ArpgAction>, With<Player>>) {
    let action_state = query.single();

    if action_state.just_pressed(&ArpgAction::Ability1) {
        println!("Fwoosh!");
    }
}

fn player_dash(query: Query<&ActionState<ArpgAction>, With<Player>>) {
    let action_state = query.single();

    if action_state.just_pressed(&ArpgAction::Ability4) {
        let mut direction_vector = Vec2::ZERO;

        for input_direction in ArpgAction::DIRECTIONS {
            if action_state.pressed(&input_direction) {
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

#[derive(Event)]
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
        if action_state.pressed(&input_direction) {
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
