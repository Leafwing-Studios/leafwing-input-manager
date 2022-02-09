//! Missing Bevy primitives for working with 2D directions

use bevy::math::{const_vec2, Vec2};
use core::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};
pub use direction::Direction;
pub use rotation::Rotation;

mod rotation {
    use super::*;

    /// A discretized 2-dimensional rotation
    ///
    /// Internally, these are stored in hundredths of a degree, and so can be cleanly added and reversed
    /// without accumulating error.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Rotation {
        /// Hundredth of a degree, measured clockwise from midnight (x=0, y=1)
        ///
        /// 36000 make up a full circle.
        centi_degrees: u16,
    }

    // Constants
    impl Rotation {
        /// The number of centi-degrees that make up a full circle
        pub const FULL_CIRCLE: u16 = 36000;

        /// The direction that points straight up
        pub const NORTH: Rotation = Rotation { centi_degrees: 0 };

        /// The direction that points straight right
        pub const EAST: Rotation = Rotation {
            centi_degrees: 9000,
        };
        /// The direction that points straight down
        pub const SOUTH: Rotation = Rotation {
            centi_degrees: 18000,
        };
        /// The direction that points straight left
        pub const WEST: Rotation = Rotation {
            centi_degrees: 27000,
        };

        /// The direction that points halfway between up and right
        pub const NORTHEAST: Rotation = Rotation {
            centi_degrees: 4500,
        };
        /// The direction that points halfway between down and right
        pub const SOUTHEAST: Rotation = Rotation {
            centi_degrees: 13500,
        };
        /// The direction that points halfway between down and left
        pub const SOUTHWEST: Rotation = Rotation {
            centi_degrees: 22500,
        };
        /// The direction that points halfway between left and up
        pub const NORTHWEST: Rotation = Rotation {
            centi_degrees: 31500,
        };
    }

    // Conversion methods
    impl Rotation {
        /// Constructs a [`Direction`] from an (x,y) Euclidean coordinate
        ///
        /// If both x and y are nearly 0 (the magnitude is less than [`EPSILON`](f32::EPSILON)), None will be returned instead.
        #[must_use]
        #[inline]
        pub fn from_xy(xy: Vec2) -> Option<Rotation> {
            if xy.length_squared() < f32::EPSILON * f32::EPSILON {
                return None;
            } else {
                let radians = f32::atan2(xy.y, xy.x);
                Some(Rotation::from_radians(radians))
            }
        }

        /// Converts this direction into an (x, y) pair with magnitude 1
        pub fn into_xy(self) -> Vec2 {
            let radians = self.into_radians();
            Vec2::new(radians.cos(), radians.sin())
        }

        /// Construct a [`Direction`] from radians, measured clockwise from midnight
        #[must_use]
        #[inline]
        pub fn from_radians(radians: impl Into<f32>) -> Rotation {
            use std::f32::consts::TAU;

            let normalized_radians: f32 = radians.into().div_euclid(TAU);

            Rotation {
                centi_degrees: (normalized_radians * 36000. / TAU) as u16,
            }
        }

        /// Converts this direction into radians, measured clockwise from midnight
        pub fn into_radians(self) -> f32 {
            self.centi_degrees as f32 * std::f32::consts::TAU / 36000.
        }

        /// Construct a [`Direction`] from degrees, measured clockwise from midnight
        #[must_use]
        #[inline]
        pub fn from_degrees(degrees: impl Into<f32>) -> Rotation {
            let normalized_degrees: f32 = degrees.into().div_euclid(360.0);

            Rotation {
                centi_degrees: (normalized_degrees * 100.0) as u16,
            }
        }

        /// Converts this direction into degrees, measured clockwise from midnight
        pub fn into_degrees(self) -> f32 {
            self.centi_degrees as f32 / 100.
        }
    }
}

mod direction {
    use bevy::math::Vec3;
    use std::f32::consts::SQRT_2;

    use super::*;

    /// A unit direction vector
    ///
    /// Its magnitude is always either zero or  one.
    #[derive(Clone, Copy, Debug, PartialEq, Default)]
    pub struct Direction {
        unit_vector: Vec2,
    }

    impl Direction {
        /// Creates a new [`Direction`] from a [`Vec2`]
        ///
        /// The [`Vec2`] will be normalized, or if it is near zero, [`Direction::NEUTRAL`] will be returned instead
        #[inline]
        pub fn new(vec2: Vec2) -> Self {
            Self {
                unit_vector: vec2.normalize_or_zero(),
            }
        }

        /// The neutral direction, which does not point anywhere
        ///
        /// This is the only constructable value with a magnitude other than 1.
        pub const NEUTRAL: Direction = Direction {
            unit_vector: Vec2::ZERO,
        };

        /// The direction that points straight up
        pub const NORTH: Direction = Direction {
            unit_vector: const_vec2!([0.0, 1.0]),
        };
        /// The direction that points straight right
        pub const EAST: Direction = Direction {
            unit_vector: const_vec2!([1.0, 0.0]),
        };
        /// The direction that points straight down
        pub const SOUTH: Direction = Direction {
            unit_vector: const_vec2!([0.0, -1.0]),
        };
        /// The direction that points straight left
        pub const WEST: Direction = Direction {
            unit_vector: const_vec2!([-1.0, 0.0]),
        };

        /// The direction that points halfway between up and right
        pub const NORTHEAST: Direction = Direction {
            unit_vector: const_vec2!([SQRT_2, SQRT_2]),
        };
        /// The direction that points halfway between down and right
        pub const SOUTHEAST: Direction = Direction {
            unit_vector: const_vec2!([SQRT_2, -SQRT_2]),
        };
        /// The direction that points halfway between down and left
        pub const SOUTHWEST: Direction = Direction {
            unit_vector: const_vec2!([-SQRT_2, -SQRT_2]),
        };
        /// The direction that points halfway between left and up
        pub const NORTHWEST: Direction = Direction {
            unit_vector: const_vec2!([-SQRT_2, SQRT_2]),
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
