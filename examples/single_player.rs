use bevy::prelude::*;
use direction::Direction;
use leafwing_input_manager::prelude::*;

use derive_more::Display;
use strum_macros::EnumIter;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        // This plugin maps inputs to an input-type agnostic action-state
        // We need to provide it with an enum which stores the possible actions a player could take
        .add_plugin(InputManagerPlugin::<ARPGAction>::default())
        // The InputMap and ActionState components will be added to any entity with the Player component
        .add_startup_system(spawn_player)
        .add_startup_system(initialize_controls)
        // The ActionState can be used directly
        .add_system(cast_fireball)
        // Or multiple parts of it can be inspected
        .add_system(player_dash)
        // Or it can be used to emit events for later processing
        .add_event::<PlayerWalk>()
        .add_system(player_walks)
        .run();
}

#[derive(Component)]
pub struct Player;

fn spawn_player(mut commands: Commands) {
    commands
        .spawn()
        .insert(Player)
        // This bundle must be added to your player entity
        // (or whatever else you wish to control)
        .insert_bundle(InputManagerBundle::<ARPGAction>::default());
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, Display, EnumIter)]
enum ARPGAction {
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
    // Utilities
    AimLock,
    Emote,
    Interact,
}

// We need to implement this trait (and meet its bounds) in order to use our enum as an A
impl Actionlike for ARPGAction {}

impl ARPGAction {
    // Lists like this can be very useful for quickly matching subsets of actions
    const DIRECTIONS: [Self; 4] = [
        ARPGAction::Up,
        ARPGAction::Down,
        ARPGAction::Left,
        ARPGAction::Right,
    ];

    fn direction(self) -> Direction {
        match self {
            ARPGAction::Up => Direction::UP,
            ARPGAction::Down => Direction::DOWN,
            ARPGAction::Left => Direction::LEFT,
            ARPGAction::Right => Direction::RIGHT,
            _ => Direction::NEUTRAL,
        }
    }
}

fn initialize_controls(mut query: Query<&mut InputMap<ARPGAction>>) {
    // This allows us to replace `ARPGAction::Up` with `Up`,
    // significantly reducing boilerplate
    use ARPGAction::*;
    // Movement
    keyboard_map.insert(Up, KeyCode::Up);
    gamepad_map.insert(Up, GamepadButtonType::DPadUp);

    keyboard_map.insert(Down, KeyCode::Down);
    gamepad_map.insert(Down, GamepadButtonType::DPadDown);

    keyboard_map.insert(Left, KeyCode::Left);
    gamepad_map.insert(Left, GamepadButtonType::DPadLeft);

    keyboard_map.insert(Right, KeyCode::Right);
    gamepad_map.insert(Right, GamepadButtonType::DPadRight);

    // Abilities
    keyboard_map.insert(Ability1, KeyCode::Q);
    gamepad_map.insert(Ability1, GamepadButtonType::West);
    mousebutton_map.insert(Ability1, MouseButton::Left);

    keyboard_map.insert(Ability2, KeyCode::W);
    gamepad_map.insert(Ability2, GamepadButtonType::North);
    mousebutton_map.insert(Ability2, MouseButton::Right);

    keyboard_map.insert(Ability3, KeyCode::E);
    gamepad_map.insert(Ability3, GamepadButtonType::East);

    keyboard_map.insert(Ability4, KeyCode::Space);
    gamepad_map.insert(Ability4, GamepadButtonType::South);

    keyboard_map.insert(Ultimate, KeyCode::R);
    gamepad_map.insert(Ultimate, GamepadButtonType::LeftTrigger2);

    // Utilities
    keyboard_map.insert(AimLock, KeyCode::Grave);
    gamepad_map.insert(AimLock, GamepadButtonType::RightTrigger);

    keyboard_map.insert(Emote, KeyCode::LShift);
    gamepad_map.insert(Emote, GamepadButtonType::LeftTrigger);

    keyboard_map.insert(Interact, KeyCode::F);
    keyboard_map.insert(Interact, KeyCode::Return); // We can bind multiple keys to the same action
    mousebutton_map.insert(Interact, MouseButton::Left); // We can bind multiple actions to the same key
    gamepad_map.insert(Interact, GamepadButtonType::RightTrigger2);
}

fn cast_fireball(query: Query<&ActionState<ARPGAction>, With<Player>>) {
    let action_state = query.single();

    if action_state.just_pressed(ARPGAction::Ability1) {
        println!("Fwoosh!");
    }
}

fn player_dash(query: Query<&ActionState<ARPGAction>, With<Player>>) {
    let action_state = query.single();

    if action_state.just_pressed(ARPGAction::Ability4) {
        let mut direction = Direction::NEUTRAL;

        for input_direction in ARPGAction::DIRECTIONS {
            if action_state.pressed(input_direction) {
                direction += input_direction.direction();
            }
        }

        println!("Dashing in {}", direction);
    }
}

pub struct PlayerWalk {
    pub direction: Direction,
}

fn player_walks(
    query: Query<&ActionState<ARPGAction>, With<Player>>,
    mut event_writer: EventWriter<PlayerWalk>,
) {
    let action_state = query.single();

    let mut direction = Direction::NEUTRAL;

    for input_direction in ARPGAction::DIRECTIONS {
        if action_state.pressed(input_direction) {
            direction += input_direction.direction();
        }
    }

    if direction != Direction::NEUTRAL {
        event_writer.send(PlayerWalk { direction });
    }
}

/// A well-behaved [Direction] primitive for use in 2D games
mod direction {
    use bevy::math::const_vec2;
    use bevy::prelude::*;
    use core::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};
    use derive_more::Display;

    /// A direction vector, defined relative to the XY plane.
    ///
    /// Its magnitude is either zero or one.
    #[derive(Component, Clone, Copy, Debug, Display, PartialEq, Default)]
    pub struct Direction {
        unit_vector: Vec2,
    }

    impl Direction {
        #[inline]
        pub fn new(vec2: Vec2) -> Self {
            Self {
                unit_vector: vec2.normalize_or_zero(),
            }
        }

        pub const NEUTRAL: Direction = Direction {
            unit_vector: Vec2::ZERO,
        };

        pub const UP: Direction = Direction {
            unit_vector: const_vec2!([0.0, 1.0]),
        };

        pub const DOWN: Direction = Direction {
            unit_vector: const_vec2!([0.0, -1.0]),
        };

        pub const RIGHT: Direction = Direction {
            unit_vector: const_vec2!([1.0, 0.0]),
        };

        pub const LEFT: Direction = Direction {
            unit_vector: const_vec2!([-1.0, 0.0]),
        };
    }

    impl Add for Direction {
        type Output = Direction;
        fn add(self, other: Direction) -> Direction {
            Self {
                unit_vector: (self.unit_vector + other.unit_vector).normalize_or_zero(),
            }
        }
    }

    impl AddAssign for Direction {
        fn add_assign(&mut self, other: Direction) {
            *self = *self + other;
        }
    }

    impl Sub for Direction {
        type Output = Direction;

        fn sub(self, rhs: Direction) -> Direction {
            Self {
                unit_vector: (self.unit_vector - rhs.unit_vector).normalize_or_zero(),
            }
        }
    }

    impl SubAssign for Direction {
        fn sub_assign(&mut self, other: Direction) {
            *self = *self - other;
        }
    }

    impl Mul<f32> for Direction {
        type Output = Vec2;

        fn mul(self, rhs: f32) -> Self::Output {
            Vec2::new(self.unit_vector.x * rhs, self.unit_vector.y * rhs)
        }
    }

    impl Mul<Direction> for f32 {
        type Output = Vec2;

        fn mul(self, rhs: Direction) -> Self::Output {
            Vec2::new(self * rhs.unit_vector.x, self * rhs.unit_vector.y)
        }
    }

    impl From<Direction> for Vec3 {
        fn from(direction: Direction) -> Vec3 {
            Vec3::new(direction.unit_vector.x, direction.unit_vector.y, 0.0)
        }
    }

    impl Neg for Direction {
        type Output = Self;

        fn neg(self) -> Self {
            Self {
                unit_vector: -self.unit_vector,
            }
        }
    }
}
