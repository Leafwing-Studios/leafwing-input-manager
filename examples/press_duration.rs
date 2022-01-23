//! We can check how long each action has been pressed for,
//! and use that in our gameplay logic!

use bevy::prelude::*;
use leafwing_input_manager::{action_state::VirtualButtonState, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(InputManagerPlugin::<Action>::default())
        .add_startup_system(spawn_player)
        .add_startup_system(spawn_camera)
        // This is where the interesting stuff is!
        .add_system(hold_dash)
        .add_system(apply_velocity)
        .add_system(drag)
        .run();
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, EnumIter)]
enum Action {
    Left,
    Right,
}

#[derive(Component)]
struct Velocity {
    x: f32,
}

#[derive(Component)]
pub struct Player;

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    velocity: Velocity,
    #[bundle]
    input_manager: InputManagerBundle<Action>,
    #[bundle]
    sprite: SpriteBundle,
}

impl PlayerBundle {
    fn default_input_map() -> InputMap<Action> {
        use Action::*;
        let mut input_map = InputMap::default();

        input_map.insert(Left, KeyCode::A);
        input_map.insert(Left, KeyCode::Left);

        input_map.insert(Right, KeyCode::D);
        input_map.insert(Right, KeyCode::Right);

        input_map
    }
}

fn spawn_player(mut commands: Commands) {
    commands.spawn_bundle(PlayerBundle {
        player: Player,
        velocity: Velocity { x: 0.0 },
        input_manager: InputManagerBundle {
            input_map: PlayerBundle::default_input_map(),
            action_state: ActionState::default(),
        },
        sprite: SpriteBundle {
            transform: Transform {
                scale: Vec3::new(40.0, 80.0, 0.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: Color::rgb(0.5, 0.5, 1.0),
                ..Default::default()
            },
            ..Default::default()
        },
    });
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

/// The longer you hold, the faster you dash!
fn hold_dash(mut player_query: Query<(&ActionState<Action>, &mut Velocity), With<Player>>) {
    const VELOCITY_RATIO: f32 = 10.0;

    let (action_state, mut velocity) = player_query.single_mut();

    if let VirtualButtonState::Released(timing) = action_state.state(Action::Left) {
        // Move left
        velocity.x -= VELOCITY_RATIO * timing.previous_duration.as_secs_f32();
    }

    if let VirtualButtonState::Released(timing) = action_state.state(Action::Right) {
        // Move right
        velocity.x += VELOCITY_RATIO * timing.previous_duration.as_secs_f32();
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation.x += velocity.x * time.delta_seconds();
    }
}

fn drag(mut query: Query<&mut Velocity>, time: Res<Time>) {
    // FIXME: this is giving very unexpected results.
    const DRAG_COEFFICIENT: f32 = 0.5;
    for mut velocity in query.iter_mut() {
        // Reduce the velocity in proportion to the square of its speed,
        // applied in the opposite direction as the object is moving.
        velocity.x -= DRAG_COEFFICIENT * velocity.x * velocity.x.abs() * time.delta_seconds();
    }
}
