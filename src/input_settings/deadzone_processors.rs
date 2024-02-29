//! Utilities for deadzone handling in input processing.
//!
//! This module provides functionality for defining deadzones in inputs.
//! Deadzone is an area around the input's zero positions where the values are considered neutral or ignored.
//!
//! ## One-Dimensional Deadzones
//!
//! The [`Deadzone1`] enum provides settings for one-dimensional deadzones in single-axis inputs:
//! - [`Deadzone1::None`]: No deadzone is performed on the input values
//! - [`Deadzone1::Symmetric`]: Deadzone with symmetric bounds
//!
//! ## Two-Dimensional Deadzones
//!
//! The [`Deadzone2`] enum provides settings for two-dimensional deadzones in dual-axis inputs:
//! - [`Deadzone2::None`]: No deadzone is performed on the input values
//! - [`Deadzone2::Circle`]: Deadzone with a circular-shaped area
//! - [`Deadzone2::Square`]: Deadzone with a cross-shaped area
//! - [`Deadzone2::RoundedSquare`]: Deadzone with a cross-shaped area and the rounded corners

use bevy::math::Vec2;
use bevy::prelude::Reflect;
use bevy::utils::FloatOrd;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::ops::Neg;

/// Default upper bound for the deadzone.
pub const DEFAULT_DEADZONE_UPPER: f32 = 0.1;

/// Default upper bound for the livezone.
pub const DEFAULT_LIVEZONE_UPPER: f32 = 1.0;

/// One-dimensional deadzones in single-axis inputs.
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub enum Deadzone1 {
    /// No deadzone is applied.
    None,

    /// A deadzone with symmetric bounds.
    ///
    /// The input values are categorized into five ranges:
    /// - `[f32::MIN, -1.0)`: Treated as `-1.0`
    /// - `[-1.0, -threshold)`: Normalized into the range `[-1.0, 0.0)`
    /// - `[-threshold, threshold]`: Treated as `0.0`
    /// - `(threshold, 1.0]`: Normalized into the range `(0.0, 1.0]`
    /// - `(1.0, f32::MAX]`: Treated as `1.0`
    Symmetric {
        /// The upper bound of the deadzone for the absolute value of input values.
        ///
        /// This value must be non-negative, and its negation represents the lower bound of the deadzone.
        /// Values within the range `[-threshold, threshold]` are treated as `0.0`
        threshold: f32,

        /// The cached width of the deadzone-excluded range `(threshold, 1.0]`.
        livezone_width: f32,

        /// The cached reciprocal of the `livezone_width`,
        /// improving performance by eliminating the need for division during computation.
        recip_livezone_width: f32,
    },
}

impl Deadzone1 {
    /// Default [`Deadzone1::Symmetric`].
    ///
    /// This deadzone excludes input values within the range `[-0.1, 0.1]`.
    pub const SYMMETRIC_DEFAULT: Self = Self::Symmetric {
        threshold: DEFAULT_DEADZONE_UPPER,
        livezone_width: DEFAULT_LIVEZONE_UPPER - DEFAULT_DEADZONE_UPPER,
        recip_livezone_width: 1.0 / (DEFAULT_LIVEZONE_UPPER - DEFAULT_DEADZONE_UPPER),
    };

    /// Creates a new [`Deadzone1::Symmetric`] to filter input values within the range `[-threshold, threshold]`.
    ///
    /// If the `threshold` is less than or equal to `0.0`, returns the constant [`Deadzone1::None`].
    ///
    /// # Arguments
    ///
    /// - `threshold`: Lower bound for the absolute value of input values, clamped to the range `[0.0, 1.0]`
    #[must_use]
    pub fn symmetric(threshold: f32) -> Self {
        if threshold <= 0.0 {
            return Self::None;
        }

        let threshold = threshold.min(DEFAULT_LIVEZONE_UPPER);
        let livezone_width = DEFAULT_LIVEZONE_UPPER - threshold;
        Self::Symmetric {
            threshold,
            livezone_width,
            recip_livezone_width: livezone_width.recip(),
        }
    }

    /// Returns the deadzone-adjusted `input_value`.
    #[must_use]
    pub fn value(&self, input_value: f32) -> f32 {
        match self {
            Deadzone1::None => input_value,
            Deadzone1::Symmetric {
                threshold,
                livezone_width,
                recip_livezone_width,
            } => Self::value_symmetric(
                input_value,
                *threshold,
                *livezone_width,
                *recip_livezone_width,
            ),
        }
    }

    /// Returns the adjusted `input_value` after applying the [`Deadzone1::Symmetric`].
    ///
    /// The `input_value` is categorized into five ranges:
    /// - `[f32::MIN, -1.0)`: Treated as `-1.0`
    /// - `[-1.0, -threshold)`: Normalized into the range `[-1.0, 0.0)`
    /// - `[-threshold, threshold]`: Treated as `0.0`
    /// - `(threshold, 1.0]`: Normalized into the range `(0.0, 1.0]`
    /// - `(1.0, f32::MAX]`: Treated as `1.0`
    fn value_symmetric(
        input_value: f32,
        threshold: f32,
        livezone_width: f32,
        recip_livezone_width: f32,
    ) -> f32 {
        let alive_value = input_value.abs() - threshold;
        if alive_value <= f32::EPSILON {
            0.0
        } else if livezone_width - alive_value <= f32::EPSILON {
            input_value.signum() * DEFAULT_LIVEZONE_UPPER
        } else {
            input_value.signum() * alive_value * recip_livezone_width
        }
    }
}

/// Two-dimensional deadzones in dual-axis inputs.
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub enum Deadzone2 {
    /// No deadzone is applied.
    None,

    /// A deadzone with a circular-shaped area.
    ///
    /// This deadzone forms a circular area at the origin defined by its radii along each axis.
    ///
    /// Both xy values are categorized into two regions:
    /// - Values within the circle are treated as `Vec2::ZERO`
    /// - Values outside the circle are normalized into the range `[-1.0, 1.0]` on each axis
    ///
    /// # Advantages
    /// - Allows for smooth transitions into and out of the deadzone area
    ///
    /// # Disadvantages
    /// - May not be suitable for all input devices or game mechanics
    Circle {
        /// Radius for the deadzone along the x-axis.
        radius_x: f32,
        /// Radius for the deadzone along the y-axis.
        radius_y: f32,
    },

    /// A deadzone with a cross-shaped area.
    ///
    /// This deadzone consists of two [`Deadzone1`]s intersecting at their centers,
    /// providing independent deadzone areas for each axis.
    ///
    /// Both xy values are categorized into four regions:
    /// - Values within the center square are treated as `Vec2::ZERO`
    /// - Values within the `deadzone_x` rectangle (excluding the rounded square)
    ///     have their x value treated as `0.0`, and their y value is normalized into the range `[-1.0, 1.0]`
    /// - Values within the `deadzone_y` rectangle (excluding the rounded square)
    ///     have their y value treated as `0.0`, and their x value is normalized into the range `[-1.0, 1.0]`
    /// - Values outside these shapes are normalized into the range `[-1.0, 1.0]` on each axis
    ///
    /// # Advantages
    /// - Provides independent deadzone control for each axis
    /// - Creates a "snapping" effect, which may be desirable for certain input devices or game mechanics
    ///
    /// # Disadvantages
    /// - May result in less smooth transitions compared to [`Deadzone2::Circle`]
    Square {
        /// The deadzone along the x-axis.
        deadzone_x: Deadzone1,
        /// The deadzone along the y-axis.
        deadzone_y: Deadzone1,
    },

    /// A deadzone with a cross-shaped area and the rounded corners.
    ///
    /// This deadzone consists of a rounded square and two rectangles intersecting at their centers,
    /// providing smooth transitions into and out of the deadzone area.
    ///
    /// Both xy values are categorized into four regions:
    /// - Values within the rounded square are treated as `Vec2::ZERO`
    /// - Values within the `[-min_x, min_x]` rectangle (excluding the rounded square)
    ///     have their x value treated as `0.0`, and their y value is normalized into the range `[-1.0, 1.0]`
    /// - Values within the `[-min_y, min_y]` rectangle (excluding the rounded square)
    ///     have their y value treated as `0.0`, and their x value is normalized into the range `[-1.0, 1.0]`
    /// - Values outside these shapes are normalized into the range `[-1.0, 1.0]` on each axis
    ///
    /// # Advantages
    /// - Combines the advantages of [`Deadzone2::Circle`] and [`Deadzone2::Square`]
    ///
    /// # Disadvantages
    /// - Requires computational resources similar to calculating both of them simultaneously
    RoundedSquare {
        /// Lower bound for the absolute value of input values along the x-axis.
        threshold_x: f32,
        /// Lower bound for the absolute value of input values along the y-axis.
        threshold_y: f32,
        /// Radius for the rounded corners along the x-axis.
        radius_x: f32,
        /// Radius for the rounded corners along the y-axis.
        radius_y: f32,
    },
}

impl Deadzone2 {
    /// Default [`Deadzone2::Circle`].
    ///
    /// This deadzone excludes input values within a `0.1` distance from `Vec2::ZERO`.
    pub const CIRCLE_DEFAULT: Self = Self::Circle {
        radius_x: DEFAULT_DEADZONE_UPPER,
        radius_y: DEFAULT_DEADZONE_UPPER,
    };

    /// Default [`Deadzone2::Square`].
    ///
    /// This deadzone excludes input values within the range `[-0.1, 0.1]` on each axis.
    pub const SQUARE_DEFAULT: Self = Self::Square {
        deadzone_x: Deadzone1::SYMMETRIC_DEFAULT,
        deadzone_y: Deadzone1::SYMMETRIC_DEFAULT,
    };

    /// Default [`Deadzone2::RoundedSquare`].
    ///
    /// This deadzone consists of a square with rounded corners,
    /// excluding near-zero input values within the range `[-0.1, 0.1]` on each axis,
    /// and applying rounded corners with the radius of `0.025` along each axis.
    pub const ROUNDED_SQUARE_DEFAULT: Self = Self::RoundedSquare {
        threshold_x: DEFAULT_DEADZONE_UPPER,
        threshold_y: DEFAULT_DEADZONE_UPPER,
        radius_x: 0.25 * DEFAULT_DEADZONE_UPPER,
        radius_y: 0.25 * DEFAULT_DEADZONE_UPPER,
    };

    /// Creates a new [`Deadzone2::Circle`] with the given settings.
    ///
    /// If both `radius_x` and `radius_y` are less than or equal to `0.0`,
    /// returns the constant [`Deadzone2::None`].
    ///
    /// # Arguments
    ///
    /// - `radius_x`: Radius for the deadzone along the x-axis, clamped into the range `[0.0, 1.0]`
    /// - `radius_y`: Radius for the deadzone along the y-axis, clamped into the range `[0.0, 1.0]`
    #[must_use]
    pub fn new_circle(radius_x: f32, radius_y: f32) -> Self {
        if radius_x <= 0.0 && radius_y <= 0.0 {
            return Self::None;
        }

        Self::Circle {
            // Ensure all the radii are within the range `[0.0, MAX]`
            radius_x: radius_x.clamp(0.0, DEFAULT_LIVEZONE_UPPER),
            radius_y: radius_y.clamp(0.0, DEFAULT_LIVEZONE_UPPER),
        }
    }

    /// Creates a new [`Deadzone2::Square`] with the given settings.
    ///
    /// # Arguments
    ///
    /// - `threshold_x`: Lower bound for the absolute value of input values on the x-axis,
    ///      clamped to the range `[0.0, 1.0]`
    /// - `threshold_y`: Lower bound for the absolute value of input values on the y-axis,
    ///      clamped to the range `[0.0, 1.0]`
    #[must_use]
    pub fn new_square(threshold_x: f32, threshold_y: f32) -> Self {
        Self::Square {
            deadzone_x: Deadzone1::symmetric(threshold_x),
            deadzone_y: Deadzone1::symmetric(threshold_y),
        }
    }

    /// Creates a new [`Deadzone2::RoundedSquare`] with the given settings.
    ///
    /// If both `threshold_x` and `threshold_y` are less than or equal to `0.0`,
    /// a [`Deadzone2::Circle`] with the specified radii will be created instead.
    ///
    /// # Arguments
    ///
    /// - `threshold_x`: Lower bound for the absolute value of input values along the x-axis,
    ///      clamped to the range `[0.0, 1.0]`
    /// - `threshold_y`: Lower bound for the absolute value of input values along the y-axis,
    ///      clamped to the range `[0.0, 1.0]`
    /// - `radius_x`: Radius for the rounded corners along the x-axis, clamped into the range `[0.0, 1.0]`
    /// - `radius_y`: Radius for the rounded corners along the y-axis, clamped into the range `[0.0, 1.0]`
    #[must_use]
    pub fn new_rounded_square(
        threshold_x: f32,
        threshold_y: f32,
        radius_x: f32,
        radius_y: f32,
    ) -> Self {
        if threshold_x <= 0.0 && threshold_y <= 0.0 {
            return Self::new_circle(radius_x, radius_y);
        }

        Self::RoundedSquare {
            // Ensure all the components are within the range `[0.0, MAX]`
            threshold_x: threshold_x.clamp(0.0, DEFAULT_LIVEZONE_UPPER),
            threshold_y: threshold_y.clamp(0.0, DEFAULT_LIVEZONE_UPPER),
            radius_x: radius_x.clamp(0.0, DEFAULT_LIVEZONE_UPPER),
            radius_y: radius_y.clamp(0.0, DEFAULT_LIVEZONE_UPPER),
        }
    }

    /// Returns the deadzone-adjusted `input_value`.
    #[must_use]
    pub fn value(&self, input_value: Vec2) -> Vec2 {
        match self {
            Self::None => input_value,
            Self::Circle { radius_x, radius_y } => {
                Self::value_circle(input_value, *radius_x, *radius_y)
            }
            Self::Square {
                deadzone_x,
                deadzone_y,
            } => Vec2::new(
                deadzone_x.value(input_value.x),
                deadzone_y.value(input_value.y),
            ),
            Self::RoundedSquare {
                threshold_x,
                threshold_y,
                radius_x,
                radius_y,
            } => Self::value_rounded_square(
                input_value,
                *threshold_x,
                *threshold_y,
                *radius_x,
                *radius_y,
            ),
        }
    }

    /// Returns the adjusted `input_value` after applying the [`Deadzone2::Circle`].
    ///
    /// Both xy values are categorized into two regions:
    /// - Values within the circle are treated as `Vec2::ZERO`
    /// - Values outside the circle are normalized into the range `[-1.0, 1.0]`
    #[inline]
    fn value_circle(input_value: Vec2, radius_x: f32, radius_y: f32) -> Vec2 {
        let Vec2 { x, y } = input_value;

        // Calculate the real threshold for the xy values
        let angle = x.atan2(y);
        let threshold_x = radius_x * angle.sin().abs();
        let threshold_y = radius_y * angle.cos().abs();

        // Normalize the xy values
        let new_x = Self::normalize_input(x, threshold_x);
        let new_y = Self::normalize_input(y, threshold_y);

        Vec2::new(new_x, new_y)
    }

    /// Returns the adjusted `input_value` after applying the [`Deadzone2::RoundedSquare`].
    ///
    /// Both xy values are categorized into four regions:
    /// - Values within the rounded square are treated as `Vec2::ZERO`
    /// - Values within the `[-threshold_x, threshold_x]` rectangle (excluding the rounded square)
    ///     have their x value set to `0.0`, and their y value is normalized
    /// - Values within the `[-threshold_y, threshold_y]` rectangle (excluding the rounded square)
    ///     have their y value set to `0.0`, and their x value is normalized
    /// - Values outside these shapes are normalized into the range `[-1.0, 1.0]`
    #[inline]
    fn value_rounded_square(
        input_value: Vec2,
        threshold_x: f32,
        threshold_y: f32,
        radius_x: f32,
        radius_y: f32,
    ) -> Vec2 {
        let Vec2 { x, y } = input_value;

        // Calculate the real threshold for the xy values
        let angle = (x.abs() - threshold_x).atan2(y.abs() - threshold_y);
        let real_min_x = deadzone_min(threshold_x, angle.sin(), radius_x, y.abs() > threshold_y);
        let real_min_y = deadzone_min(threshold_y, angle.cos(), radius_y, x.abs() > threshold_x);
        fn deadzone_min(value_min: f32, angle: f32, radius: f32, nearer_corner: bool) -> f32 {
            if nearer_corner {
                radius.mul_add(angle.abs(), value_min)
            } else {
                radius + value_min
            }
        }

        // Normalize the xy values
        let new_x = Self::normalize_input(x, real_min_x);
        let new_y = Self::normalize_input(y, real_min_y);

        Vec2::new(new_x, new_y)
    }

    /// Normalizes the given `input_value` into the livezone range `(threshold, 1.0]`
    fn normalize_input(input_value: f32, threshold: f32) -> f32 {
        let livezone_width = DEFAULT_LIVEZONE_UPPER - threshold;
        let alive_value = input_value.abs() - threshold;
        if alive_value <= f32::EPSILON {
            0.0
        } else if livezone_width - alive_value <= f32::EPSILON {
            1.0 * input_value.signum()
        } else {
            alive_value / livezone_width * input_value.signum()
        }
    }
}

// -------------------------
// Unfortunately, Rust doesn't let us automatically derive `Eq` and `Hash` for `f32`.
// It's like teaching a fish to ride a bike – a bit nonsensical!
// But if that fish really wants to pedal, we'll make it work.
// So here we are, showing Rust who's boss!
// -------------------------

impl Eq for Deadzone1 {}

impl Hash for Deadzone1 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Deadzone1::None => {
                0.hash(state);
            }
            Deadzone1::Symmetric {
                threshold: min,
                livezone_width,
                recip_livezone_width,
            } => {
                1.hash(state);
                FloatOrd(*min).hash(state);
                FloatOrd(*livezone_width).hash(state);
                FloatOrd(*recip_livezone_width).hash(state);
            }
        }
    }
}

impl Eq for Deadzone2 {}

impl Hash for Deadzone2 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Deadzone2::None => {
                0.hash(state);
            }
            Deadzone2::Circle { radius_x, radius_y } => {
                1.hash(state);
                FloatOrd(*radius_x).hash(state);
                FloatOrd(*radius_y).hash(state);
            }
            Deadzone2::Square {
                deadzone_x,
                deadzone_y,
            } => {
                2.hash(state);
                deadzone_x.hash(state);
                deadzone_y.hash(state);
            }
            Deadzone2::RoundedSquare {
                threshold_x: min_x,
                threshold_y: min_y,
                radius_x,
                radius_y,
            } => {
                3.hash(state);
                FloatOrd(*min_x).hash(state);
                FloatOrd(*min_y).hash(state);
                FloatOrd(*radius_x).hash(state);
                FloatOrd(*radius_y).hash(state);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    mod one_dimensional {
        use crate::prelude::Deadzone1;

        #[test]
        fn test_deadzone1_none() {
            let deadzone = Deadzone1::None;

            // No deadzone and livezone normalization
            assert_eq!(5.0, deadzone.value(5.0));
            assert_eq!(1.0, deadzone.value(1.0));
            assert_eq!(0.5, deadzone.value(0.5));
            assert_eq!(0.0, deadzone.value(0.0));
            assert_eq!(-0.5, deadzone.value(-0.5));
            assert_eq!(-1.0, deadzone.value(-1.0));
            assert_eq!(-5.0, deadzone.value(-5.0));
        }

        #[test]
        fn test_deadzone1_default() {
            let deadzone = Deadzone1::SYMMETRIC_DEFAULT;

            // Deadzone
            assert_eq!(0.0, deadzone.value(0.1));
            assert_eq!(0.0, deadzone.value(0.01));
            assert_eq!(0.0, deadzone.value(-0.01));
            assert_eq!(0.0, deadzone.value(-0.1));

            // Livezone normalization
            let livezone_0_75 = 0.7222222;
            assert_eq!(livezone_0_75, deadzone.value(0.75));
            assert_eq!(-livezone_0_75, deadzone.value(-0.75));

            let livezone_0_5 = 0.44444448;
            assert_eq!(livezone_0_5, deadzone.value(0.5));
            assert_eq!(-livezone_0_5, deadzone.value(-0.5));

            let livezone_0_25 = 0.16666669;
            assert_eq!(livezone_0_25, deadzone.value(0.25));
            assert_eq!(-livezone_0_25, deadzone.value(-0.25));

            let livezone_0_11 = 0.0111111095;
            assert_eq!(livezone_0_11, deadzone.value(0.11));
            assert_eq!(-livezone_0_11, deadzone.value(-0.11));
        }

        #[test]
        fn test_deadzone1_custom() {
            let deadzone = Deadzone1::symmetric(0.2);

            // Deadzone
            assert_eq!(0.0, deadzone.value(0.2));
            assert_eq!(0.0, deadzone.value(0.1));
            assert_eq!(0.0, deadzone.value(0.0));
            assert_eq!(0.0, deadzone.value(-0.1));
            assert_eq!(0.0, deadzone.value(-0.2));

            // livezone normalization
            assert_eq!(1.0, deadzone.value(1.0));
            assert_eq!(-1.0, deadzone.value(-1.0));

            assert_eq!(0.75, deadzone.value(0.8));
            assert_eq!(-0.75, deadzone.value(-0.8));

            let livezone_0_6 = 0.50000006;
            assert_eq!(livezone_0_6, deadzone.value(0.6));
            assert_eq!(-livezone_0_6, deadzone.value(-0.6));

            assert_eq!(0.25, deadzone.value(0.4));
            assert_eq!(-0.25, deadzone.value(-0.4));

            let livezone_0_3 = 0.12500001;
            assert_eq!(livezone_0_3, deadzone.value(0.3));
            assert_eq!(-livezone_0_3, deadzone.value(-0.3));
        }
    }

    mod two_dimensional {
        use crate::prelude::Deadzone2;
        use bevy::math::{vec2, Vec2};

        #[test]
        fn test_deadzone2_none() {
            let deadzone = Deadzone2::None;

            // No deadzone and livezone normalization
            assert_eq!(Vec2::splat(5.0), deadzone.value(Vec2::splat(5.0)));
            assert_eq!(Vec2::splat(1.0), deadzone.value(Vec2::splat(1.0)));
            assert_eq!(Vec2::splat(0.5), deadzone.value(Vec2::splat(0.5)));
            assert_eq!(Vec2::splat(0.25), deadzone.value(Vec2::splat(0.25)));
            assert_eq!(Vec2::splat(0.1), deadzone.value(Vec2::splat(0.1)));
            assert_eq!(Vec2::splat(0.01), deadzone.value(Vec2::splat(0.01)));
            assert_eq!(Vec2::splat(-0.01), deadzone.value(Vec2::splat(-0.01)));
            assert_eq!(Vec2::splat(-0.1), deadzone.value(Vec2::splat(-0.1)));
            assert_eq!(Vec2::splat(-0.25), deadzone.value(Vec2::splat(-0.25)));
            assert_eq!(Vec2::splat(-0.5), deadzone.value(Vec2::splat(-0.5)));
            assert_eq!(Vec2::splat(-1.0), deadzone.value(Vec2::splat(-1.0)));
            assert_eq!(Vec2::splat(-5.0), deadzone.value(Vec2::splat(-5.0)));
        }

        #[test]
        fn test_deadzone2_circle_default() {
            let deadzone = Deadzone2::CIRCLE_DEFAULT;

            // Deadzone
            assert_eq!(Vec2::ZERO, deadzone.value(vec2(0.1, 0.0)));
            assert_eq!(Vec2::ZERO, deadzone.value(vec2(0.0, 0.1)));
            assert_eq!(Vec2::ZERO, deadzone.value(vec2(-0.1, 0.0)));
            assert_eq!(Vec2::ZERO, deadzone.value(vec2(0.0, -0.1)));
            assert_eq!(Vec2::ZERO, deadzone.value(Vec2::splat(0.05)));
            assert_eq!(Vec2::ZERO, deadzone.value(Vec2::splat(-0.05)));

            // Livezone normalization
            let livezone_0_75 = Vec2::splat(0.73097724);
            assert_eq!(livezone_0_75, deadzone.value(Vec2::splat(0.75)));
            assert_eq!(-livezone_0_75, deadzone.value(Vec2::splat(-0.75)));

            let livezone_0_5 = Vec2::splat(0.4619544);
            assert_eq!(livezone_0_5, deadzone.value(Vec2::splat(0.5)));
            assert_eq!(-livezone_0_5, deadzone.value(Vec2::splat(-0.5)));

            let livezone_0_25 = Vec2::splat(0.19293164);
            assert_eq!(livezone_0_25, deadzone.value(Vec2::splat(0.25)));
            assert_eq!(-livezone_0_25, deadzone.value(Vec2::splat(-0.25)));

            let livezone_0_1 = Vec2::splat(0.03151798);
            assert_eq!(livezone_0_1, deadzone.value(Vec2::splat(0.1)));
            assert_eq!(-livezone_0_1, deadzone.value(Vec2::splat(-0.1)));
        }

        #[test]
        fn test_deadzone2_circle_custom() {
            let deadzone = Deadzone2::new_circle(0.3, 0.35);

            // Deadzone
            assert_eq!(Vec2::ZERO, deadzone.value(Vec2::splat(0.2)));
            assert_eq!(Vec2::ZERO, deadzone.value(Vec2::splat(0.1)));
            assert_eq!(Vec2::ZERO, deadzone.value(Vec2::ZERO));
            assert_eq!(Vec2::ZERO, deadzone.value(Vec2::splat(-0.1)));
            assert_eq!(Vec2::ZERO, deadzone.value(Vec2::splat(-0.2)));

            // Livezone normalization
            assert_eq!(Vec2::ONE, deadzone.value(Vec2::ONE));
            assert_eq!(Vec2::NEG_ONE, deadzone.value(Vec2::NEG_ONE));

            let livezone_0_8 = vec2(0.7461504, 0.7342237);
            assert_eq!(livezone_0_8, deadzone.value(Vec2::splat(0.8)));
            assert_eq!(-livezone_0_8, deadzone.value(Vec2::splat(-0.8)));

            let livezone_0_6 = vec2(0.49230075, 0.4684475);
            assert_eq!(livezone_0_6, deadzone.value(Vec2::splat(0.6)));
            assert_eq!(-livezone_0_6, deadzone.value(Vec2::splat(-0.6)));

            let livezone_0_4 = vec2(0.23845108, 0.2026712);
            assert_eq!(livezone_0_4, deadzone.value(Vec2::splat(0.4)));
            assert_eq!(-livezone_0_4, deadzone.value(Vec2::splat(-0.4)));

            let livezone_0_3 = vec2(0.11152627, 0.06978308);
            assert_eq!(livezone_0_3, deadzone.value(Vec2::splat(0.3)));
            assert_eq!(-livezone_0_3, deadzone.value(Vec2::splat(-0.3)));

            let livezone_0_25 = vec2(0.048063844, 0.0033389921);
            assert_eq!(livezone_0_25, deadzone.value(Vec2::splat(0.25)));
            assert_eq!(-livezone_0_25, deadzone.value(Vec2::splat(-0.25)));
        }

        #[test]
        fn test_deadzone2_square_default() {
            let deadzone = Deadzone2::SQUARE_DEFAULT;

            // Deadzone
            assert_eq!(Vec2::ZERO, deadzone.value(Vec2::splat(0.1)));
            assert_eq!(Vec2::ZERO, deadzone.value(Vec2::splat(-0.1)));
            assert_eq!(Vec2::ZERO, deadzone.value(vec2(0.1, 0.0)));
            assert_eq!(Vec2::ZERO, deadzone.value(vec2(0.0, 0.1)));
            assert_eq!(Vec2::ZERO, deadzone.value(vec2(-0.1, 0.0)));
            assert_eq!(Vec2::ZERO, deadzone.value(vec2(0.0, -0.1)));
            assert_eq!(Vec2::ZERO, deadzone.value(Vec2::splat(0.05)));
            assert_eq!(Vec2::ZERO, deadzone.value(Vec2::splat(-0.05)));

            // Livezone normalization
            let livezone_0_75 = Vec2::splat(0.7222222);
            assert_eq!(livezone_0_75, deadzone.value(Vec2::splat(0.75)));
            assert_eq!(-livezone_0_75, deadzone.value(Vec2::splat(-0.75)));

            let livezone_0_5 = Vec2::splat(0.44444448);
            assert_eq!(livezone_0_5, deadzone.value(Vec2::splat(0.5)));
            assert_eq!(-livezone_0_5, deadzone.value(Vec2::splat(-0.5)));

            let livezone_0_25 = Vec2::splat(0.16666669);
            assert_eq!(livezone_0_25, deadzone.value(Vec2::splat(0.25)));
            assert_eq!(-livezone_0_25, deadzone.value(Vec2::splat(-0.25)));

            let livezone_0_25 = Vec2::splat(0.027777778);
            assert_eq!(livezone_0_25, deadzone.value(Vec2::splat(0.125)));
            assert_eq!(-livezone_0_25, deadzone.value(Vec2::splat(-0.125)));
        }

        #[test]
        fn test_deadzone2_square_custom() {
            let deadzone = Deadzone2::new_square(0.25, 0.3);

            // Deadzone
            assert_eq!(Vec2::ZERO, deadzone.value(Vec2::splat(0.25)));
            assert_eq!(Vec2::ZERO, deadzone.value(Vec2::splat(-0.25)));
            assert_eq!(Vec2::ZERO, deadzone.value(Vec2::splat(0.1)));
            assert_eq!(Vec2::ZERO, deadzone.value(Vec2::splat(-0.1)));
            assert_eq!(Vec2::ZERO, deadzone.value(vec2(0.1, 0.0)));
            assert_eq!(Vec2::ZERO, deadzone.value(vec2(0.0, 0.1)));
            assert_eq!(Vec2::ZERO, deadzone.value(vec2(-0.1, 0.0)));
            assert_eq!(Vec2::ZERO, deadzone.value(vec2(0.0, -0.1)));
            assert_eq!(Vec2::ZERO, deadzone.value(Vec2::splat(0.05)));
            assert_eq!(Vec2::ZERO, deadzone.value(Vec2::splat(-0.05)));

            // Livezone normalization
            let livezone_0_75 = vec2(0.6666667, 0.64285713);
            assert_eq!(livezone_0_75, deadzone.value(Vec2::splat(0.75)));
            assert_eq!(-livezone_0_75, deadzone.value(Vec2::splat(-0.75)));

            let livezone_0_5 = vec2(0.33333334, 0.28571427);
            assert_eq!(livezone_0_5, deadzone.value(Vec2::splat(0.5)));
            assert_eq!(-livezone_0_5, deadzone.value(Vec2::splat(-0.5)));

            let livezone_0_3 = vec2(0.066666685, 0.0);
            assert_eq!(livezone_0_3, deadzone.value(Vec2::splat(0.3)));
            assert_eq!(-livezone_0_3, deadzone.value(Vec2::splat(-0.3)));
        }

        #[test]
        fn test_deadzone2_rounded_square_default() {
            let deadzone = Deadzone2::ROUNDED_SQUARE_DEFAULT;

            // Deadzone
            assert_eq!(Vec2::ZERO, deadzone.value(Vec2::splat(0.1)));
            assert_eq!(Vec2::ZERO, deadzone.value(Vec2::splat(-0.1)));
            assert_eq!(Vec2::ZERO, deadzone.value(vec2(0.1, 0.0)));
            assert_eq!(Vec2::ZERO, deadzone.value(vec2(0.0, 0.1)));
            assert_eq!(Vec2::ZERO, deadzone.value(vec2(-0.1, 0.0)));
            assert_eq!(Vec2::ZERO, deadzone.value(vec2(0.0, -0.1)));
            assert_eq!(Vec2::ZERO, deadzone.value(Vec2::splat(0.05)));
            assert_eq!(Vec2::ZERO, deadzone.value(Vec2::splat(-0.05)));

            // Livezone normalization
            let livezone_0_75 = Vec2::splat(0.7166568);
            assert_eq!(livezone_0_75, deadzone.value(Vec2::splat(0.75)));
            assert_eq!(-livezone_0_75, deadzone.value(Vec2::splat(-0.75)));

            let livezone_0_5 = Vec2::splat(0.43331367);
            assert_eq!(livezone_0_5, deadzone.value(Vec2::splat(0.5)));
            assert_eq!(-livezone_0_5, deadzone.value(Vec2::splat(-0.5)));

            let livezone_0_25 = Vec2::splat(0.14997052);
            assert_eq!(livezone_0_25, deadzone.value(Vec2::splat(0.25)));
            assert_eq!(-livezone_0_25, deadzone.value(Vec2::splat(-0.25)));

            let livezone_0_125 = Vec2::splat(0.008298924);
            assert_eq!(livezone_0_125, deadzone.value(Vec2::splat(0.125)));
            assert_eq!(-livezone_0_125, deadzone.value(Vec2::splat(-0.125)));
        }

        #[test]
        fn test_deadzone2_rounded_square_custom() {
            let deadzone = Deadzone2::new_rounded_square(0.15, 0.16, 0.05, 0.06);

            // Deadzone
            assert_eq!(Vec2::ZERO, deadzone.value(Vec2::splat(0.1)));
            assert_eq!(Vec2::ZERO, deadzone.value(Vec2::splat(-0.1)));
            assert_eq!(Vec2::ZERO, deadzone.value(vec2(0.1, 0.0)));
            assert_eq!(Vec2::ZERO, deadzone.value(vec2(0.0, 0.1)));
            assert_eq!(Vec2::ZERO, deadzone.value(vec2(-0.1, 0.0)));
            assert_eq!(Vec2::ZERO, deadzone.value(vec2(0.0, -0.1)));
            assert_eq!(Vec2::ZERO, deadzone.value(Vec2::splat(0.05)));
            assert_eq!(Vec2::ZERO, deadzone.value(Vec2::splat(-0.05)));

            // Livezone normalization
            let livezone_0_75 = vec2(0.6930062, 0.6866899);
            assert_eq!(livezone_0_75, deadzone.value(Vec2::splat(0.75)));
            assert_eq!(-livezone_0_75, deadzone.value(Vec2::splat(-0.75)));

            let livezone_0_5 = vec2(0.385852, 0.373585);
            assert_eq!(livezone_0_5, deadzone.value(Vec2::splat(0.5)));
            assert_eq!(-livezone_0_5, deadzone.value(Vec2::splat(-0.5)));

            let livezone_0_25 = vec2(0.07730384, 0.06233839);
            assert_eq!(livezone_0_25, deadzone.value(Vec2::splat(0.25)));
            assert_eq!(-livezone_0_25, deadzone.value(Vec2::splat(-0.25)));

            let livezone_0_2 = vec2(0.013510657, 0.0031379922);
            assert_eq!(livezone_0_2, deadzone.value(Vec2::splat(0.2)));
            assert_eq!(-livezone_0_2, deadzone.value(Vec2::splat(-0.2)));
        }
    }
}
