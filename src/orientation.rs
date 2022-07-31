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

    /// A type that can represent a orientation in 2D space
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
        /// Panics if the distance between `self` and `other` is greater than 2 deci-degrees.
        fn assert_approx_eq(self, other: impl Orientation) {
            let self_rotation: Rotation = self.into();
            let other_rotation: Rotation = other.into();

            let distance: Rotation = self_rotation.distance(other_rotation);
            assert!(
                distance <= Rotation::new(2),
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

            if rotation_to.deci_degrees == 0 || rotation_to.deci_degrees >= 1800 {
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
        /// rotation.rotate_towards(Rotation::SOUTH, Some(Rotation::new(450)));
        /// assert_eq!(rotation, Rotation::SOUTHWEST);
        /// ```
        #[inline]
        fn rotate_towards(&mut self, target_orientation: Self, max_rotation: Option<Rotation>) {
            if let Some(max_rotation) = max_rotation {
                if self.distance(target_orientation) <= max_rotation {
                    *self = target_orientation;
                } else {
                    let delta_rotation = match self.rotation_direction(target_orientation) {
                        RotationDirection::CounterClockwise => max_rotation,
                        RotationDirection::Clockwise => -max_rotation,
                    };
                    let current_rotation: Rotation = (*self).into();
                    let new_rotation: Rotation = current_rotation + delta_rotation;

                    *self = new_rotation.into();
                }
            } else {
                *self = target_orientation;
            }
        }
    }

    impl Orientation for Rotation {
        #[inline]
        fn distance(&self, other: Rotation) -> Rotation {
            let initial_distance = if self.deci_degrees >= other.deci_degrees {
                self.deci_degrees - other.deci_degrees
            } else {
                other.deci_degrees - self.deci_degrees
            };

            if initial_distance <= Rotation::FULL_CIRCLE / 2 {
                Rotation {
                    deci_degrees: initial_distance,
                }
            } else {
                Rotation {
                    deci_degrees: Rotation::FULL_CIRCLE - initial_distance,
                }
            }
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
    /// A direction that a [`Rotation`] can be applied in
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
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum RotationDirection {
        /// Corresponds to a positive rotation
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

        /// Reverese the direction into the opposite enum variant
        #[inline]
        pub fn reverse(self) -> RotationDirection {
            use RotationDirection::*;

            match self {
                Clockwise => CounterClockwise,
                CounterClockwise => Clockwise,
            }
        }
    }

    impl Default for RotationDirection {
        fn default() -> RotationDirection {
            RotationDirection::Clockwise
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
    /// Internally, these are stored in tenths of a degree, and so can be cleanly added
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
        /// Tenths of a degree, measured clockwise from midnight (x=0, y=1)
        ///
        /// 3600 make up a full circle
        pub(crate) deci_degrees: u16,
    }

    // Useful methods
    impl Rotation {
        /// Creates a new [`Rotation`] from a whole number of tenths of a degree
        ///
        /// Measured clockwise from midnight.
        #[inline]
        #[must_use]
        pub const fn new(deci_degrees: u16) -> Rotation {
            Rotation {
                deci_degrees: deci_degrees % Rotation::FULL_CIRCLE,
            }
        }

        /// Returns the exact internal mesaurement, stored in tenths of a degree
        ///
        /// Measured clockwise from midnight (x=0, y=1).
        /// 3600 make up a full circle.
        #[inline]
        #[must_use]
        pub const fn deci_degrees(&self) -> u16 {
            self.deci_degrees
        }
    }

    // Constants
    impl Rotation {
        /// The number of deci-degrees that make up a full circle
        pub const FULL_CIRCLE: u16 = 3600;

        /// The direction that points straight up
        pub const NORTH: Rotation = Rotation { deci_degrees: 900 };

        /// The direction that points straight right
        pub const EAST: Rotation = Rotation { deci_degrees: 0 };
        /// The direction that points straight down
        pub const SOUTH: Rotation = Rotation { deci_degrees: 2700 };
        /// The direction that points straight left
        pub const WEST: Rotation = Rotation { deci_degrees: 1800 };

        /// The direction that points halfway between up and right
        pub const NORTHEAST: Rotation = Rotation { deci_degrees: 450 };
        /// The direction that points halfway between down and right
        pub const SOUTHEAST: Rotation = Rotation { deci_degrees: 3150 };
        /// The direction that points halfway between down and left
        pub const SOUTHWEST: Rotation = Rotation { deci_degrees: 2250 };
        /// The direction that points halfway between left and up
        pub const NORTHWEST: Rotation = Rotation { deci_degrees: 1350 };
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
        /// measured counterclockwise from the positive x axis
        #[must_use]
        #[inline]
        pub fn from_radians(radians: impl Into<f32>) -> Rotation {
            let normalized_radians = radians.into().rem_euclid(TAU);

            Rotation {
                deci_degrees: (normalized_radians * (3600. / TAU)) as u16,
            }
        }

        /// Converts this direction into radians, measured counterclockwise from the positive x axis
        #[inline]
        #[must_use]
        pub fn into_radians(self) -> f32 {
            self.deci_degrees as f32 * TAU / 3600.
        }

        /// Construct a [`Direction`](crate::orientation::Direction) from degrees, measured counterclockwise from the positive x axis
        #[must_use]
        #[inline]
        pub fn from_degrees(degrees: impl Into<f32>) -> Rotation {
            let normalized_degrees: f32 = degrees.into().rem_euclid(360.0);

            Rotation {
                deci_degrees: (normalized_degrees * 10.0) as u16,
            }
        }

        /// Converts this direction into degrees, measured counterclockwise from the positive x axis
        #[inline]
        #[must_use]
        pub fn into_degrees(self) -> f32 {
            self.deci_degrees as f32 / 10.
        }
    }

    impl Add for Rotation {
        type Output = Rotation;
        fn add(self, rhs: Self) -> Rotation {
            Rotation::new(self.deci_degrees + rhs.deci_degrees)
        }
    }

    impl Sub for Rotation {
        type Output = Rotation;
        fn sub(self, rhs: Self) -> Rotation {
            if self.deci_degrees >= rhs.deci_degrees {
                Rotation::new(self.deci_degrees - rhs.deci_degrees)
            } else {
                Rotation::new(self.deci_degrees + Rotation::FULL_CIRCLE - rhs.deci_degrees)
            }
        }
    }

    impl AddAssign for Rotation {
        fn add_assign(&mut self, rhs: Self) {
            self.deci_degrees = (self.deci_degrees + rhs.deci_degrees) % Rotation::FULL_CIRCLE;
        }
    }

    impl SubAssign for Rotation {
        fn sub_assign(&mut self, rhs: Self) {
            // Be sure to avoid overflow when subtracting
            if self.deci_degrees >= rhs.deci_degrees {
                self.deci_degrees = self.deci_degrees - rhs.deci_degrees;
            } else {
                self.deci_degrees = Rotation::FULL_CIRCLE - (rhs.deci_degrees - self.deci_degrees);
            }
        }
    }

    impl Neg for Rotation {
        type Output = Rotation;
        fn neg(self) -> Rotation {
            Rotation {
                deci_degrees: Rotation::FULL_CIRCLE - self.deci_degrees,
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
        /// as it is consistent with the default [`Rotation`]
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
            // the correct decidegree. 32-bit floating point math rounds to the wrong decidegree,
            // which usually isn't a big deal, but can result in unexpected surprises when people
            // are dealing only with cardinal directions. The underlying problem is that f32 values
            // for 1.0 and -1.0 can't be represented exactly, so our unit vectors start with an
            // approximate value and both `atan2` above and `from_radians` below magnify the
            // imprecision. So, we cheat.
            const APPROX_SOUTH: f32 = -1.5707964;
            const APPROX_NORTHWEST: f32 = 2.3561945;
            if radians == APPROX_NORTHWEST {
                Rotation::new(1350)
            } else if radians == APPROX_SOUTH {
                Rotation::new(2700)
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
            match vec2.try_normalize() {
                Some(unit_vector) => Ok(Direction { unit_vector }),
                None => Err(NearlySingularConversion),
            }
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
            match quaternion.mul_vec3(Vec3::X).truncate().try_normalize() {
                Some(unit_vector) => Direction { unit_vector },
                None => Default::default(),
            }
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
            Rotation::new(900),
            "we want north to end up exact in decidegrees"
        );
        let northeast_rot: Rotation = Direction::NORTHEAST.into();
        assert_eq!(
            northeast_rot,
            Rotation::new(450),
            "we want northeast to end up exact in decidegrees"
        );
        let northwest_rot: Rotation = Direction::NORTHWEST.into();
        assert_eq!(
            northwest_rot,
            Rotation::new(1350),
            "we want northwest to end up exact in decidegrees"
        );
        let south_rot: Rotation = Direction::SOUTH.into();
        assert_eq!(
            south_rot,
            Rotation::new(2700),
            "we want south to end up exact in decidegrees"
        );
        let southeast_rot: Rotation = Direction::SOUTHEAST.into();
        assert_eq!(
            southeast_rot,
            Rotation::new(3150),
            "we want southeast to end up exact in decidegrees"
        );
        let southwest_rot: Rotation = Direction::SOUTHWEST.into();
        assert_eq!(
            southwest_rot,
            Rotation::new(2250),
            "we want southwest to end up exact in decidegrees"
        );
        let east_rot: Rotation = Direction::EAST.into();
        assert_eq!(
            east_rot,
            Rotation::new(0),
            "we want east to end up exact in decidegrees"
        );
        let west_rot: Rotation = Direction::WEST.into();
        assert_eq!(
            west_rot,
            Rotation::new(1800),
            "we want west to end up exact in decidegrees"
        );
    }
}
