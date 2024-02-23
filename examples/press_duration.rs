//! We can check how long each action has been pressed for,
//! and use that in our gameplay logic!
//!
//! Press Left / Right or A / D to move your character to the left and right!

use bevy::{prelude::*, window::PrimaryWindow};
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(InputManagerPlugin::<Action>::default())
        .add_systems(Startup, spawn_player)
        .add_systems(Startup, spawn_camera)
        // This is where the interesting stuff is!
        .add_systems(Update, hold_dash)
        .add_systems(Update, apply_velocity)
        .add_systems(Update, drag)
        .add_systems(Update, wall_collisions)
        .run();
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
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
    input_manager: InputManagerBundle<Action>,
    sprite: SpriteBundle,
}

impl PlayerBundle {
    fn default_input_map() -> InputMap<Action> {
        use Action::*;

        InputMap::new([
            (Left, KeyCode::KeyA),
            (Left, KeyCode::ArrowLeft),
            (Right, KeyCode::KeyD),
            (Right, KeyCode::ArrowRight),
        ])
    }
}

fn spawn_player(mut commands: Commands) {
    commands.spawn(PlayerBundle {
        player: Player,
        velocity: Velocity { x: 0.0 },
        input_manager: InputManagerBundle::with_map(PlayerBundle::default_input_map()),
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
    commands.spawn(Camera2dBundle::default());
}

/// The longer you hold, the faster you dash when released!
fn hold_dash(mut player_query: Query<(&ActionState<Action>, &mut Velocity), With<Player>>) {
    const VELOCITY_RATIO: f32 = 1000.0;

    let (action_state, mut velocity) = player_query.single_mut();

    if action_state.just_released(&Action::Left) {
        // Accelerate left
        velocity.x -= VELOCITY_RATIO * action_state.previous_duration(&Action::Left).as_secs_f32();
    }

    if action_state.just_released(&Action::Right) {
        // Accelerate right
        velocity.x += VELOCITY_RATIO * action_state.previous_duration(&Action::Right).as_secs_f32();
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation.x += velocity.x * time.delta_seconds();
    }
}

fn drag(mut query: Query<&mut Velocity>, time: Res<Time>) {
    const DRAG_COEFFICIENT: f32 = 0.8;
    for mut velocity in query.iter_mut() {
        // Reduce the velocity in proportion to its speed,
        // applied in the opposite direction as the object is moving.
        velocity.x -= DRAG_COEFFICIENT * velocity.x * time.delta_seconds();
    }
}

fn wall_collisions(
    mut query: Query<(&Transform, &mut Velocity)>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let window_width = windows.single().width();
    let left_side = 0.0 - window_width / 2.0;
    let right_side = 0.0 + window_width / 2.0;

    for (transform, mut velocity) in query.iter_mut() {
        // This doesn't account for sprite width, but this is a simple example
        if (transform.translation.x < left_side) | (transform.translation.x > right_side) {
            // Boing!
            velocity.x *= -1.0;
        }
    }
}
