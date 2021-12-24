use bevy::math::const_vec2;
use bevy::prelude::*;

use derive_more::Display;
use derive_more::{Add, Sub};
use std::ops::{Add, AddAssign, Mul, Neg};

use crate::input::InputLabel;
pub struct MovementPlugin;

#[derive(SystemLabel, Clone, Hash, Debug, PartialEq, Eq)]
pub enum MovementLabel {
    Velocity,
}

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::PreUpdate,
            reset_direction.before(InputLabel::Processing),
        )
        .add_system(apply_direction.before(MovementLabel::Velocity))
        .add_system(apply_velocity.label(MovementLabel::Velocity));
    }
}

#[derive(Component, Clone, Copy, Debug, Display, PartialEq, Default)]
/// A direction vector, defined relative to the XY plane.
///
/// Its magnitude is either zero or one.
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
    fn add(self, other: Direction) -> Self {
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

fn reset_direction(mut query: Query<&mut Direction>) {
    for mut direction in query.iter_mut() {
        *direction = Direction::NEUTRAL;
    }
}

fn apply_direction(mut query: Query<(&Speed, &Direction, &mut Transform)>, time: Res<Time>) {
    for (&speed, &direction, mut transform) in query.iter_mut() {
        let rate = speed * time.delta_seconds();
        transform.translation += (rate * direction).extend(0.0);

        // Facing towards itself causes the entity to disappear due to broken transforms
        if direction != Direction::NEUTRAL {
            // Face towards direction that the entity moved towards
            // These are the local unit vectors of the vector space
            // ointing towards the new direction
            let forward: Vec3 = direction.into();
            let up = Vec3::Z;
            let right = up.cross(forward).normalize();

            // Points directly at the specified direction
            let full_rotation = Quat::from_mat3(&Mat3::from_cols(right, up, forward)).normalize();

            let full_turn_angle = transform.rotation.angle_between(full_rotation);

            // Maximum rate that entities can turn, in radians per second
            // This is set aggresively high, to allow for responsive controls
            const MAX_TURN_SPEED: f32 = 15.0;
            let max_turn_angle = MAX_TURN_SPEED * time.delta_seconds();

            // Set the entity's rotation, clamping if needed
            transform.rotation = if full_turn_angle <= max_turn_angle {
                full_rotation
            } else {
                // Interpolate between the starting rotation and the full rotation
                // such that the maximum turn speed is respected
                transform
                    .rotation
                    .slerp(full_rotation, max_turn_angle / full_turn_angle)
            };
        }
    }
}

/// The base speed of an entity
///
/// The player's base movement speed is set to the default value,
/// providing a useful scale
#[derive(Component, Clone, Copy, Debug, Add)]
pub struct Speed {
    rate: f32,
}

impl Speed {
    pub fn new(rate: f32) -> Self {
        if rate >= 0.0 {
            Speed { rate }
        } else {
            Speed { rate: 0.0 }
        }
    }
}

impl Default for Speed {
    fn default() -> Self {
        Speed { rate: 5.0 }
    }
}

impl Mul<f32> for Speed {
    type Output = Speed;
    fn mul(self, rhs: f32) -> Self::Output {
        Speed {
            rate: self.rate * rhs,
        }
    }
}

impl Mul<Speed> for f32 {
    type Output = Speed;
    fn mul(self, rhs: Speed) -> Self::Output {
        Speed {
            rate: self * rhs.rate,
        }
    }
}

impl Mul<Direction> for Speed {
    type Output = Vec2;
    fn mul(self, rhs: Direction) -> Self::Output {
        self.rate * rhs
    }
}

impl Mul<Speed> for Direction {
    type Output = Vec2;
    fn mul(self, rhs: Speed) -> Self::Output {
        self * rhs.rate
    }
}

#[derive(PartialEq, Clone, Copy, Component, Debug, Default, Display, Add, Sub)]
pub struct Velocity {
    vector: Vec3,
}

impl Velocity {
    /// Creates a velocity pointing from a translation to another translation
    pub fn along(from: Vec3, to: Vec3) -> Self {
        Self { vector: to - from }
    }

    /// Creates a z-locked velocity pointing from a translation to another translation
    pub fn along_flat(from: Vec3, to: Vec3) -> Self {
        Self {
            vector: Vec3::new(to.x - from.x, to.y - from.y, 0.0),
        }
    }
}

impl Mul<Speed> for Velocity {
    type Output = Velocity;
    fn mul(self, rhs: Speed) -> Self::Output {
        Velocity {
            vector: self.vector * rhs.rate,
        }
    }
}

impl Mul<Velocity> for Speed {
    type Output = Velocity;
    fn mul(self, rhs: Velocity) -> Self::Output {
        Velocity {
            vector: self.rate * rhs.vector,
        }
    }
}

impl Mul<f32> for Velocity {
    type Output = Vec3;
    fn mul(self, rhs: f32) -> Self::Output {
        self.vector * rhs
    }
}

impl Mul<Velocity> for f32 {
    type Output = Vec3;
    fn mul(self, rhs: Velocity) -> Self::Output {
        self * rhs.vector
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, &velocity) in query.iter_mut() {
        transform.translation += velocity * time.delta_seconds();
    }
}
