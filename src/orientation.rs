//! Direction and rotation for spinning around in 2 dimensions

pub use direction::Direction;
pub use orientation_trait::Orientation;
pub use rotation::Rotation;
pub use rotation_direction::RotationDirection;

mod orientation_trait {
    use super::{Direction, Rotation, RotationDirection};
    use bevy::math::Quat;
    use bevy::transform::components::{GlobalTransform, Transform};
    use core::fmt::Debug;

    /// A type that can represent an orientation in 2D space
    pub trait Orientation: Sized + Debug + From<Rotation> + Into<Rotation> + Copy {
        /// Returns the absolute distance between `self` and `other` as a [`Rotation`]
        ///
        /// The shortest path will always be taken, and so this value ranges between 0 and 180 degrees.
        /// Simply subtract the two rotations if you want a signed value instead.
        ///
        /// # Example
        /// ```rust
        /// use leafwing_input_manager::orientation::{Orientation, Direction, Rotation};
        ///
        /// Direction::NORTH.distance(Direction::SOUTHWEST).assert_approx_eq(Rotation::from_degrees(135.));
        /// ```
        #[must_use]
        fn distance(&self, other: Self) -> Rotation;

        /// Asserts that `self` is approximately equal to `other`
        ///
        /// # Panics
        ///
        /// Panics if the distance between `self` and `other` is greater than a hundredth of a degree.
        #[track_caller]
        fn assert_approx_eq(self, other: impl Orientation) {
            let self_rotation: Rotation = self.into();
            let other_rotation: Rotation = other.into();

            let distance: Rotation = self_rotation.distance(other_rotation);
            assert!(
                distance <= Rotation::new(Rotation::DEGREE / 100),
                "{self:?} (converted to {self_rotation}) was {distance} away from {other:?} (converted to {other_rotation})."
            );
        }

        /// Which [`RotationDirection`] is the shortest to rotate towards to reach `target`?
        ///
        /// In the case of ties, [`RotationDirection::Clockwise`] will be returned.
        ///
        /// # Example
        /// ```rust
        /// use leafwing_input_manager::orientation::{Direction, Orientation, RotationDirection};
        ///
        /// assert_eq!(Direction::NORTH.rotation_direction(Direction::NORTH), RotationDirection::Clockwise);
        /// assert_eq!(Direction::NORTH.rotation_direction(Direction::SOUTH), RotationDirection::Clockwise);
        ///
        /// assert_eq!(Direction::NORTH.rotation_direction(Direction::EAST), RotationDirection::Clockwise);
        /// assert_eq!(Direction::NORTH.rotation_direction(Direction::WEST), RotationDirection::CounterClockwise);
        ///
        /// assert_eq!(Direction::WEST.rotation_direction(Direction::SOUTH), RotationDirection::CounterClockwise);
        /// assert_eq!(Direction::SOUTH.rotation_direction(Direction::WEST), RotationDirection::Clockwise);
        /// ```
        #[inline]
        #[must_use]
        fn rotation_direction(&self, target: Self) -> RotationDirection {
            let self_rotation: Rotation = (*self).into();
            let target_rotation: Rotation = target.into();

            let rotation_to = target_rotation - self_rotation;

            if rotation_to.micro_degrees == 0 || rotation_to.micro_degrees >= Rotation::HALF_CIRCLE
            {
                RotationDirection::Clockwise
            } else {
                RotationDirection::CounterClockwise
            }
        }

        /// Rotates `self` towards `target_orientation` by up to `max_rotation`
        ///
        /// # Example
        /// ```rust
        /// use leafwing_input_manager::orientation::{Rotation, Orientation};
        ///
        /// let mut rotation = Rotation::SOUTH;
        ///
        /// // Without a `max_rotation`, the orientation snaps
        /// rotation.rotate_towards(Rotation::WEST, None);
        /// assert_eq!(rotation, Rotation::WEST);
        ///
        /// // With a `max_rotation`, we don't get all the way there
        /// rotation.rotate_towards(Rotation::SOUTH, Some(Rotation::from_degrees_int(45)));
        /// assert_eq!(rotation, Rotation::SOUTHWEST);
        /// ```
        #[inline]
        fn rotate_towards(&mut self, target_orientation: Self, max_rotation: Option<Rotation>) {
            if let Some(max_rotation) = max_rotation {
                if self.distance(target_orientation) > max_rotation {
                    let sign = self.rotation_direction(target_orientation).sign() as f32;
                    let delta_rotation = sign * max_rotation;
                    let current_rotation: Rotation = (*self).into();
                    let new_rotation: Rotation = current_rotation + delta_rotation;

                    *self = new_rotation.into();
                }
                return;
            }
            *self = target_orientation;
        }
    }

    impl Orientation for Rotation {
        #[inline]
        fn distance(&self, other: Rotation) -> Rotation {
            let difference = self.micro_degrees_difference(other);
            let micro_degrees = difference.min(Rotation::neg_micro_degrees(difference));
            Rotation { micro_degrees }
        }
    }

    impl Orientation for Direction {
        fn distance(&self, other: Direction) -> Rotation {
            let self_rotation: Rotation = (*self).into();
            let other_rotation: Rotation = other.into();
            self_rotation.distance(other_rotation)
        }
    }

    impl Orientation for Quat {
        fn distance(&self, other: Quat) -> Rotation {
            let self_rotation: Rotation = (*self).into();
            let other_rotation: Rotation = other.into();
            self_rotation.distance(other_rotation)
        }
    }

    impl Orientation for Transform {
        fn distance(&self, other: Transform) -> Rotation {
            let self_rotation: Rotation = (*self).into();
            let other_rotation: Rotation = other.into();
            self_rotation.distance(other_rotation)
        }
    }

    impl Orientation for GlobalTransform {
        fn distance(&self, other: GlobalTransform) -> Rotation {
            let self_rotation: Rotation = (*self).into();
            let other_rotation: Rotation = other.into();
            self_rotation.distance(other_rotation)
        }
    }
}

mod rotation_direction {
    /// A direction that a [`Rotation`](crate::orientation::Rotation) can be applied in.
    ///
    /// # Example
    /// ```rust
    /// use leafwing_input_manager::orientation::{Orientation, Rotation, RotationDirection};
    ///
    /// assert_eq!(Rotation::NORTH.rotation_direction(Rotation::NORTH), RotationDirection::Clockwise);
    /// assert_eq!(Rotation::NORTH.rotation_direction(Rotation::EAST), RotationDirection::Clockwise);
    /// assert_eq!(Rotation::NORTH.rotation_direction(Rotation::WEST), RotationDirection::CounterClockwise);
    /// assert_eq!(Rotation::NORTH.rotation_direction(Rotation::SOUTH), RotationDirection::Clockwise);
    /// ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub enum RotationDirection {
        /// Corresponds to a positive rotation
        #[default]
        Clockwise,
        /// Corresponds to a negative rotation
        CounterClockwise,
    }

    impl RotationDirection {
        /// The sign of the corresponding [`Rotation`](super::Rotation)
        ///
        /// Returns 1 if [`RotationDirection::Clockwise`],
        /// or -1 if [`RotationDirection::CounterClockwise`]
        #[inline]
        #[must_use]
        pub fn sign(self) -> isize {
            match self {
                RotationDirection::Clockwise => -1,
                RotationDirection::CounterClockwise => 1,
            }
        }

        /// Reverse the direction into the opposite enum variant
        #[inline]
        pub fn reverse(self) -> RotationDirection {
            use RotationDirection::*;

            match self {
                Clockwise => CounterClockwise,
                CounterClockwise => Clockwise,
            }
        }
    }
}

mod rotation {
    use crate::errors::NearlySingularConversion;
    use bevy::ecs::prelude::Component;
    use bevy::math::Vec2;
    use core::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};
    use derive_more::Display;
    use std::f32::consts::TAU;

    /// A discretized 2-dimensional rotation
    ///
    /// Internally, these are stored in millionths of a degree, and so can be cleanly added
    /// and reversed without accumulating error.
    ///
    /// # Example
    /// ```rust
    /// use leafwing_input_manager::orientation::{Rotation, Direction, Orientation};
    /// use core::f32::consts::{FRAC_PI_2, PI, TAU};
    ///
    /// let east = Rotation::from_radians(0.0);
    /// let north = Rotation::from_radians(FRAC_PI_2);
    /// let west = Rotation::from_radians(PI);
    ///
    /// Rotation::default().assert_approx_eq(Rotation::from_radians(0.0));
    /// Rotation::default().assert_approx_eq(Rotation::from_radians(TAU));
    /// Rotation::default().assert_approx_eq(500.0 * Rotation::from_radians(TAU));
    ///
    /// (north + north).assert_approx_eq(west);
    /// (west - east).assert_approx_eq(west);
    /// (2.0 * north).assert_approx_eq(west);
    /// (west / 2.0).assert_approx_eq(north);
    ///
    /// north.assert_approx_eq(Rotation::NORTH);
    ///
    /// Direction::from(west).assert_approx_eq(Direction::WEST);
    /// ```
    #[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Default, Display)]
    pub struct Rotation {
        /// Millionths of a degree, measured clockwise from midnight (x=0, y=1)
        ///
        /// `360_000_000` make up a full circle
        pub(crate) micro_degrees: u32,
    }

    // Useful methods
    impl Rotation {
        /// Creates a new [`Rotation`] from a whole number of millionths of a degree
        ///
        /// Measured clockwise from midnight.
        #[inline]
        #[must_use]
        pub const fn new(micro_degrees: u32) -> Rotation {
            Rotation {
                micro_degrees: micro_degrees % Rotation::FULL_CIRCLE,
            }
        }

        /// Returns the exact internal measurement, stored in millionths of a degree
        ///
        /// Measured clockwise from midnight (x=0, y=1).
        /// `360_000_000` make up a full circle.
        #[inline]
        #[must_use]
        pub const fn micro_degrees(&self) -> u32 {
            self.micro_degrees
        }
    }

    // Constants
    impl Rotation {
        /// The number of micro-degrees in one degree
        pub const DEGREE: u32 = 1_000_000;

        /// The number of micro-degrees that make up a half-circle
        pub const HALF_CIRCLE: u32 = 180 * Rotation::DEGREE;

        /// The number of micro-degrees that make up a full circle
        pub const FULL_CIRCLE: u32 = 360 * Rotation::DEGREE;

        /// The direction that points straight up
        pub const NORTH: Rotation = Rotation::from_degrees_int(90);

        /// The direction that points straight right
        pub const EAST: Rotation = Rotation::from_degrees_int(0);
        /// The direction that points straight down
        pub const SOUTH: Rotation = Rotation::from_degrees_int(270);
        /// The direction that points straight left
        pub const WEST: Rotation = Rotation::from_degrees_int(180);

        /// The direction that points halfway between up and right
        pub const NORTHEAST: Rotation = Rotation::from_degrees_int(45);
        /// The direction that points halfway between down and right
        pub const SOUTHEAST: Rotation = Rotation::from_degrees_int(315);
        /// The direction that points halfway between down and left
        pub const SOUTHWEST: Rotation = Rotation::from_degrees_int(225);
        /// The direction that points halfway between left and up
        pub const NORTHWEST: Rotation = Rotation::from_degrees_int(135);
    }

    // Conversion methods
    impl Rotation {
        /// Constructs a [`Rotation`](crate::orientation::Direction) from an (x,y) Euclidean coordinate
        ///
        /// If both x and y are nearly 0 (the magnitude is less than [`EPSILON`](f32::EPSILON)),
        /// [`Err(NearlySingularConversion)`] will be returned instead.
        ///
        /// # Example
        /// ```rust
        /// use bevy::math::Vec2;
        /// use leafwing_input_manager::orientation::Rotation;
        ///
        /// assert_eq!(Rotation::from_xy(Vec2::new(0.0, 1.0)), Ok(Rotation::NORTH));
        /// ```
        #[inline]
        pub fn from_xy(xy: Vec2) -> Result<Rotation, NearlySingularConversion> {
            if xy.length_squared() < f32::EPSILON * f32::EPSILON {
                Err(NearlySingularConversion)
            } else {
                let radians = f32::atan2(xy.y, xy.x);
                Ok(Rotation::from_radians(radians))
            }
        }

        /// Converts this direction into an (x, y) pair with magnitude 1
        #[inline]
        #[must_use]
        pub fn into_xy(self) -> Vec2 {
            let radians = self.into_radians();
            Vec2::new(radians.cos(), radians.sin())
        }

        /// Construct a [`Direction`](crate::orientation::Direction) from radians,
        /// measured counterclockwise from the positive x-axis
        #[must_use]
        #[inline]
        pub fn from_radians(radians: impl Into<f32>) -> Rotation {
            let normalized_radians = radians.into().rem_euclid(TAU);

            Rotation {
                micro_degrees: (normalized_radians * (Rotation::FULL_CIRCLE as f32 / TAU)) as u32,
            }
        }

        /// Converts this direction into radians, measured counterclockwise from the positive x-axis
        #[inline]
        #[must_use]
        pub fn into_radians(self) -> f32 {
            self.micro_degrees as f32 * (TAU / Rotation::FULL_CIRCLE as f32)
        }

        /// Construct a [`Direction`](crate::orientation::Direction) from degrees, measured counterclockwise from the positive x-axis
        #[must_use]
        #[inline]
        pub fn from_degrees(degrees: impl Into<f32>) -> Rotation {
            let normalized_degrees: f32 = degrees.into().rem_euclid(360.0);

            Rotation {
                micro_degrees: (normalized_degrees * Rotation::DEGREE as f32) as u32,
            }
        }

        /// Construct a [`Direction`](crate::orientation::Direction) from a whole number of degrees, measured counterclockwise from the positive x-axis
        #[must_use]
        #[inline]
        pub const fn from_degrees_int(degrees: u32) -> Rotation {
            Rotation {
                micro_degrees: degrees.rem_euclid(360) * Rotation::DEGREE,
            }
        }

        /// Converts this direction into degrees, measured counterclockwise from the positive x-axis
        #[inline]
        #[must_use]
        pub fn into_degrees(self) -> f32 {
            self.micro_degrees as f32 / Rotation::DEGREE as f32
        }

        /// Calculates the difference in micro-degrees between `self` and `rhs`.
        #[inline]
        #[must_use]
        pub(crate) fn micro_degrees_difference(&self, rhs: Self) -> u32 {
            (self.micro_degrees as i32 - rhs.micro_degrees as i32).unsigned_abs()
        }

        /// Returns the negation of the given micro-degrees.
        #[inline]
        #[must_use]
        pub(crate) fn neg_micro_degrees(micro_degrees: u32) -> u32 {
            Rotation::FULL_CIRCLE - micro_degrees
        }

        /// Calculates the micro-degrees from `self` subtracted by `rhs`.
        #[inline]
        #[must_use]
        pub(crate) fn micro_degrees_sub_by(&self, rhs: Self) -> u32 {
            let difference = self.micro_degrees_difference(rhs);
            if self.micro_degrees >= rhs.micro_degrees {
                difference
            } else {
                Rotation::neg_micro_degrees(difference)
            }
        }
    }

    impl Add for Rotation {
        type Output = Rotation;
        fn add(self, rhs: Self) -> Rotation {
            Rotation::new(self.micro_degrees + rhs.micro_degrees)
        }
    }

    impl Sub for Rotation {
        type Output = Rotation;
        fn sub(self, rhs: Self) -> Rotation {
            Rotation::new(self.micro_degrees_sub_by(rhs))
        }
    }

    impl AddAssign for Rotation {
        fn add_assign(&mut self, rhs: Self) {
            self.micro_degrees = (self.micro_degrees + rhs.micro_degrees) % Rotation::FULL_CIRCLE;
        }
    }

    impl SubAssign for Rotation {
        fn sub_assign(&mut self, rhs: Self) {
            self.micro_degrees = self.micro_degrees_sub_by(rhs);
        }
    }

    impl Neg for Rotation {
        type Output = Rotation;
        fn neg(self) -> Rotation {
            Rotation {
                micro_degrees: Rotation::neg_micro_degrees(self.micro_degrees),
            }
        }
    }

    impl Mul<f32> for Rotation {
        type Output = Rotation;
        fn mul(self, rhs: f32) -> Rotation {
            Rotation::from_degrees(self.into_degrees() * rhs)
        }
    }

    impl Mul<Rotation> for f32 {
        type Output = Rotation;
        fn mul(self, rhs: Rotation) -> Rotation {
            Rotation::from_degrees(rhs.into_degrees() * self)
        }
    }

    impl Div<f32> for Rotation {
        type Output = Rotation;
        fn div(self, rhs: f32) -> Rotation {
            Rotation::from_degrees(self.into_degrees() / rhs)
        }
    }

    impl Div<Rotation> for f32 {
        type Output = Rotation;
        fn div(self, rhs: Rotation) -> Rotation {
            Rotation::from_degrees(self / rhs.into_degrees())
        }
    }
}

mod direction {
    use bevy::ecs::prelude::Component;
    use bevy::math::{Vec2, Vec3};
    use core::ops::{Add, Div, Mul, Neg, Sub};
    use derive_more::Display;
    use std::f32::consts::SQRT_2;

    /// A 2D unit vector that represents a direction
    ///
    /// Its magnitude is always `1.0`.
    ///
    /// # Example
    /// ```rust
    /// use leafwing_input_manager::orientation::Direction;
    /// use bevy::math::Vec2;
    ///
    /// assert_eq!(Direction::NORTH.unit_vector(), Vec2::new(0.0, 1.0));
    /// assert_eq!(Direction::try_from(Vec2::ONE), Ok(Direction::NORTHEAST));
    ///
    /// assert_eq!(Direction::SOUTH * 3.0, Vec2::new(0.0, -3.0));
    /// assert_eq!(Direction::EAST / 2.0, Vec2::new(0.5, 0.0));
    /// ```
    #[derive(Component, Clone, Copy, Debug, PartialEq, Display)]
    pub struct Direction {
        pub(crate) unit_vector: Vec2,
    }

    impl Default for Direction {
        /// [`Direction::EAST`] is the default direction,
        /// as it is consistent with the default [`Rotation`](crate::orientation::Rotation)
        fn default() -> Direction {
            Direction::EAST
        }
    }

    impl Direction {
        /// Creates a new [`Direction`] from a [`Vec2`]
        ///
        /// The [`Vec2`] stored internally will be normalized to have a magnitude of `1.0`.
        ///
        /// # Panics
        ///
        /// Panics if the length of the supplied vector has length zero or cannot be determined.
        /// Use [`try_from`](TryFrom) to get a [`Result`] instead.
        #[must_use]
        #[inline]
        pub fn new(vec2: Vec2) -> Self {
            Self::try_from(vec2).unwrap()
        }

        /// Returns the raw underlying [`Vec2`] unit vector of this direction
        ///
        /// This will always have a length of `1.0`
        #[must_use]
        #[inline]
        pub const fn unit_vector(&self) -> Vec2 {
            self.unit_vector
        }
    }

    // Constants
    impl Direction {
        /// The direction that points straight up
        pub const NORTH: Direction = Direction {
            unit_vector: Vec2::new(0.0, 1.0),
        };
        /// The direction that points straight right
        pub const EAST: Direction = Direction {
            unit_vector: Vec2::new(1.0, 0.0),
        };
        /// The direction that points straight down
        pub const SOUTH: Direction = Direction {
            unit_vector: Vec2::new(0.0, -1.0),
        };
        /// The direction that points straight left
        pub const WEST: Direction = Direction {
            unit_vector: Vec2::new(-1.0, 0.0),
        };

        /// The direction that points halfway between up and right
        pub const NORTHEAST: Direction = Direction {
            unit_vector: Vec2::new(SQRT_2 / 2.0, SQRT_2 / 2.0),
        };
        /// The direction that points halfway between down and right
        pub const SOUTHEAST: Direction = Direction {
            unit_vector: Vec2::new(SQRT_2 / 2.0, -SQRT_2 / 2.0),
        };
        /// The direction that points halfway between down and left
        pub const SOUTHWEST: Direction = Direction {
            unit_vector: Vec2::new(-SQRT_2 / 2.0, -SQRT_2 / 2.0),
        };
        /// The direction that points halfway between left and up
        pub const NORTHWEST: Direction = Direction {
            unit_vector: Vec2::new(-SQRT_2 / 2.0, SQRT_2 / 2.0),
        };
    }

    impl Add for Direction {
        type Output = Vec2;
        fn add(self, other: Direction) -> Vec2 {
            self.unit_vector + other.unit_vector
        }
    }

    impl Sub for Direction {
        type Output = Vec2;

        fn sub(self, rhs: Direction) -> Vec2 {
            self.unit_vector - rhs.unit_vector
        }
    }

    impl Mul<f32> for Direction {
        type Output = Vec2;

        fn mul(self, rhs: f32) -> Vec2 {
            self.unit_vector * rhs
        }
    }

    impl Mul<Direction> for f32 {
        type Output = Vec2;

        fn mul(self, rhs: Direction) -> Vec2 {
            self * rhs.unit_vector
        }
    }

    impl Div<f32> for Direction {
        type Output = Vec2;

        fn div(self, rhs: f32) -> Vec2 {
            self.unit_vector / rhs
        }
    }

    impl Div<Direction> for f32 {
        type Output = Vec2;

        fn div(self, rhs: Direction) -> Vec2 {
            self / rhs.unit_vector
        }
    }

    impl From<Direction> for Vec3 {
        fn from(direction: Direction) -> Vec3 {
            direction.unit_vector.extend(0.0)
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

mod conversions {
    use super::{Direction, Rotation};
    use crate::errors::NearlySingularConversion;
    use bevy::math::{Quat, Vec2, Vec3};
    use bevy::transform::components::{GlobalTransform, Transform};

    impl From<Rotation> for Direction {
        fn from(rotation: Rotation) -> Direction {
            Direction {
                unit_vector: rotation.into_xy(),
            }
        }
    }

    impl From<Direction> for Rotation {
        fn from(direction: Direction) -> Rotation {
            let radians = direction.unit_vector.y.atan2(direction.unit_vector.x);
            // This dirty little trick helps us nudge the two (of eight) cardinal directions onto
            // the correct microdegree. 32-bit floating point math rounds to the wrong microdegree,
            // which usually isn't a big deal, but can result in unexpected surprises when people
            // are dealing only with cardinal directions. The underlying problem is that f32 values
            // for 1.0 and -1.0 can't be represented exactly, so our unit vectors start with an
            // approximate value and both `atan2` above and `from_radians` below magnify the
            // imprecision. So, we cheat.
            use std::f32::consts::FRAC_PI_2;
            const APPROX_SOUTH: f32 = -FRAC_PI_2;
            const APPROX_NORTHWEST: f32 = 1.5 * FRAC_PI_2;
            if radians == APPROX_NORTHWEST {
                Rotation::from_degrees_int(135)
            } else if radians == APPROX_SOUTH {
                Rotation::from_degrees_int(270)
            } else {
                Rotation::from_radians(radians)
            }
        }
    }

    impl TryFrom<Vec2> for Rotation {
        type Error = NearlySingularConversion;

        fn try_from(vec2: Vec2) -> Result<Rotation, NearlySingularConversion> {
            Rotation::from_xy(vec2)
        }
    }

    impl From<Rotation> for Vec2 {
        fn from(rotation: Rotation) -> Vec2 {
            rotation.into_xy()
        }
    }

    impl TryFrom<Vec2> for Direction {
        type Error = NearlySingularConversion;

        fn try_from(vec2: Vec2) -> Result<Direction, NearlySingularConversion> {
            vec2.try_normalize()
                .map(|unit_vector| Direction { unit_vector })
                .ok_or(NearlySingularConversion)
        }
    }

    impl From<Direction> for Vec2 {
        fn from(direction: Direction) -> Vec2 {
            direction.unit_vector()
        }
    }

    impl From<Quat> for Rotation {
        fn from(quaternion: Quat) -> Rotation {
            let direction: Direction = quaternion.into();
            direction.into()
        }
    }

    impl From<Rotation> for Quat {
        fn from(rotation: Rotation) -> Self {
            Quat::from_rotation_z(rotation.into_radians())
        }
    }

    impl From<Quat> for Direction {
        fn from(quaternion: Quat) -> Self {
            quaternion.mul_vec3(Vec3::X).truncate().try_into().unwrap()
        }
    }

    impl From<Direction> for Quat {
        fn from(direction: Direction) -> Quat {
            let rotation: Rotation = direction.into();
            rotation.into()
        }
    }

    impl From<Transform> for Direction {
        fn from(transform: Transform) -> Self {
            transform.rotation.into()
        }
    }

    impl From<GlobalTransform> for Direction {
        fn from(transform: GlobalTransform) -> Self {
            transform.to_scale_rotation_translation().1.into()
        }
    }

    impl From<Direction> for Transform {
        fn from(direction: Direction) -> Self {
            Transform::from_rotation(direction.into())
        }
    }

    impl From<Direction> for GlobalTransform {
        fn from(direction: Direction) -> Self {
            GlobalTransform::from_rotation(direction.into())
        }
    }

    impl From<Transform> for Rotation {
        fn from(transform: Transform) -> Self {
            transform.rotation.into()
        }
    }

    impl From<GlobalTransform> for Rotation {
        fn from(transform: GlobalTransform) -> Self {
            transform.to_scale_rotation_translation().1.into()
        }
    }

    impl From<Rotation> for Transform {
        fn from(rotation: Rotation) -> Self {
            Transform::from_rotation(rotation.into())
        }
    }

    impl From<Rotation> for GlobalTransform {
        fn from(rotation: Rotation) -> Self {
            GlobalTransform::from_rotation(rotation.into())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn directions_end_up_even() {
        let north_rot: Rotation = Direction::NORTH.into();
        assert_eq!(
            north_rot,
            Rotation::from_degrees_int(90),
            "we want north to end up exact in microdegrees"
        );
        let northeast_rot: Rotation = Direction::NORTHEAST.into();
        assert_eq!(
            northeast_rot,
            Rotation::from_degrees_int(45),
            "we want northeast to end up exact in microdegrees"
        );
        let northwest_rot: Rotation = Direction::NORTHWEST.into();
        assert_eq!(
            northwest_rot,
            Rotation::from_degrees_int(135),
            "we want northwest to end up exact in microdegrees"
        );
        let south_rot: Rotation = Direction::SOUTH.into();
        assert_eq!(
            south_rot,
            Rotation::from_degrees_int(270),
            "we want south to end up exact in microdegrees"
        );
        let southeast_rot: Rotation = Direction::SOUTHEAST.into();
        assert_eq!(
            southeast_rot,
            Rotation::from_degrees_int(315),
            "we want southeast to end up exact in microdegrees"
        );
        let southwest_rot: Rotation = Direction::SOUTHWEST.into();
        assert_eq!(
            southwest_rot,
            Rotation::from_degrees_int(225),
            "we want southwest to end up exact in microdegrees"
        );
        let east_rot: Rotation = Direction::EAST.into();
        assert_eq!(
            east_rot,
            Rotation::from_degrees_int(0),
            "we want east to end up exact in microdegrees"
        );
        let west_rot: Rotation = Direction::WEST.into();
        assert_eq!(
            west_rot,
            Rotation::from_degrees_int(180),
            "we want west to end up exact in microdegrees"
        );
    }
}
