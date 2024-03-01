//! Utilities for various deadzone regions.
//!
//! This module provides functionality to define deadzones in input processing.
//!
//! Deadzone is a region around the input's zero positions where the values are considered ignored,
//! preventing unintended actions or jittery movements
//!
//! ## One-Dimensional Deadzones
//!
//! The [`Deadzone1`] enum provides options for shaping one-dimensional deadzones in single-axis inputs:
//! - [`Deadzone1::None`]: No deadzone applied
//! - [`Deadzone1::Symmetric`]: Deadzone with symmetric bounds
//!
//! ## Two-Dimensional Deadzones
//!
//! The [`Deadzone2`] enum provides options for shaping two-dimensional deadzones in dual-axis inputs:
//! - [`Deadzone2::None`]: No deadzone applied
//! - [`Deadzone2::Circle`]: Deadzone with a circular-shaped area
//! - [`Deadzone2::Square`]: Deadzone with a cross-shaped area
//! - [`Deadzone2::RoundedSquare`]: Deadzone with a cross-shaped area and the rounded corners

use bevy::math::Vec2;
use bevy::prelude::Reflect;
use bevy::utils::FloatOrd;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

/// Default upper bound for the deadzone.
pub const DEFAULT_DEADZONE_UPPER: f32 = 0.1;

/// Default upper bound for the livezone.
pub const DEFAULT_LIVEZONE_UPPER: f32 = 1.0;

/// One-dimensional deadzone configuration options for single-axis inputs.
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub enum Deadzone1 {
    /// No deadzone applied.
    None,

    /// Applies a deadzone centered at zero with symmetric bounds.
    ///
    /// Input values are categorized into the following ranges:
    /// - `[-infinity, -1.0)`: Treated as `-1.0` (clamped to minimum output)
    /// - `[-1.0, -threshold)`: Scaled linearly to the range `[-1.0, 0.0)`
    /// - `[-threshold, threshold]`: Treated as `0.0` (considered neutral)
    /// - `(threshold, 1.0]`: Scaled linearly to the range `(0.0, 1.0]`
    /// - `(1.0, infinity]`: Treated as `1.0` (clamped to maximum output)
    Symmetric {
        /// The absolute value threshold for the deadzone.
        /// Values within the range `[-threshold, threshold]` are treated as `0.0`.
        ///
        /// This value must be non-negative, and its negation represents the lower bound of the deadzone.
        threshold: f32,

        /// Pre-calculated width of the livezone `(threshold, 1.0]`
        /// avoids redundant calculations during deadzone computation.
        livezone_width: f32,

        /// Pre-calculated reciprocal of the `livezone_width`
        /// avoids division during deadzone computation.
        recip_livezone_width: f32,
    },
}

impl Deadzone1 {
    /// Default [`Deadzone1::Symmetric`].
    ///
    /// This deadzone excludes input values within the range `[-0.1, 0.1]`.
    pub const DEFAULT_SYMMETRIC: Self = Self::Symmetric {
        threshold: DEFAULT_DEADZONE_UPPER,
        livezone_width: DEFAULT_LIVEZONE_UPPER - DEFAULT_DEADZONE_UPPER,
        recip_livezone_width: 1.0 / (DEFAULT_LIVEZONE_UPPER - DEFAULT_DEADZONE_UPPER),
    };

    /// Creates a new [`Deadzone1::Symmetric`] instance
    /// to filter input values within the range `[-threshold, threshold]`.
    ///
    /// If the `threshold` is non-positive, returns [`Deadzone1::None`].
    ///
    /// # Arguments
    ///
    /// - `threshold`: Absolute value threshold for the deadzone, clamped to the range `(0.0, 1.0]`
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

    /// Applies the current deadzone to the `input_value` and returns the adjusted value.
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

    /// Applies the current [`Deadzone1::Symmetric`] to the `input_value` and returns the adjusted value.
    ///
    /// The `input_value` is categorized into the following ranges:
    /// - `[-infinity, -1.0)`: Treated as `-1.0` (clamped to minimum output)
    /// - `[-1.0, -threshold)`: Scaled linearly to the range `[-1.0, 0.0)`
    /// - `[-threshold, threshold]`: Treated as `0.0` (considered neutral)
    /// - `(threshold, 1.0]`: Scaled linearly to the range `(0.0, 1.0]`
    /// - `(1.0, infinity]`: Treated as `1.0` (clamped to maximum output)
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

/// Two-dimensional deadzone configuration options for dual-axis inputs.
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub enum Deadzone2 {
    /// No deadzone applied.
    None,

    /// A circular deadzone centered at the origin.
    ///
    /// # Behaviors
    ///
    /// - Values within the circle are treated as `Vec2::ZERO`
    /// - Values outside the circle are scaled linearly to the range `[-1.0, 1.0]` on each axis
    Circle {
        /// Radius for the deadzone along the x-axis.
        radius_x: f32,
        /// Radius for the deadzone along the y-axis.
        radius_y: f32,
    },

    /// A cross-shaped deadzone consists of two [`Deadzone1`] zones centered at the origin, overlapping to form a central zone.
    ///
    /// ## Behavior
    ///
    /// - Values within the central square are treated as `Vec2::ZERO`
    /// - Values within the `deadzone_x` zone (excluding the central zone)
    ///     have the x value treated as `0.0`, and the y value is scaled linearly to the range `[-1.0, 1.0]`
    /// - Values within the `deadzone_y` zone (excluding the central zone)
    ///     have the y value treated as `0.0`, and the x value is scaled linearly to the range `[-1.0, 1.0]`
    /// - Values outside these shapes are scaled linearly to the range `[-1.0, 1.0]` on each axis
    Square {
        /// The deadzone along the x-axis.
        deadzone_x: Deadzone1,
        /// The deadzone along the y-axis.
        deadzone_y: Deadzone1,
    },

    /// A cross-shaped deadzone consists of two [`Deadzone1`] zones centered at the origin,
    /// overlapping to form a central square with four rounded corners.
    ///
    /// ## Behavior
    ///
    /// - Values within the central zone are treated as `Vec2::ZERO`
    /// - Values within the `deadzone_x` zone (excluding the central zone)
    ///     have the x value treated as `0.0`, and the y value is scaled linearly to the range `[-1.0, 1.0]`
    /// - Values within the `deadzone_y` zone (excluding the central zone)
    ///     have the y value treated as `0.0`, and the x value is scaled linearly to the range `[-1.0, 1.0]`
    /// - Values outside these shapes are scaled linearly to the range `[-1.0, 1.0]` on each axis
    ///
    /// # Advantages
    /// - Combines the advantages of [`Deadzone2::Circle`] and [`Deadzone2::Square`]
    ///
    /// # Disadvantages
    /// - Requires computational resources similar to calculating both of them simultaneously
    RoundedSquare {
        /// Threshold for the absolute value of input values along the x-axis.
        threshold_x: f32,
        /// Threshold for the absolute value of input values along the y-axis.
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
        deadzone_x: Deadzone1::DEFAULT_SYMMETRIC,
        deadzone_y: Deadzone1::DEFAULT_SYMMETRIC,
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

    /// Creates a new [`Deadzone2::Circle`] instance with the given settings.
    ///
    /// If both `radius_x` and `radius_y` are non-positive, returns [`Deadzone2::None`].
    ///
    /// # Arguments
    ///
    /// - `radius_x`: Radius for the deadzone along the x-axis, clamped into the range `[0.0, 1.0]`
    /// - `radius_y`: Radius for the deadzone along the y-axis, clamped into the range `[0.0, 1.0]`
    #[must_use]
    pub fn circle(radius_x: f32, radius_y: f32) -> Self {
        if radius_x <= 0.0 && radius_y <= 0.0 {
            return Self::None;
        }

        Self::Circle {
            // Ensure all the radii are within the range `[0.0, MAX]`
            radius_x: radius_x.clamp(0.0, DEFAULT_LIVEZONE_UPPER),
            radius_y: radius_y.clamp(0.0, DEFAULT_LIVEZONE_UPPER),
        }
    }

    /// Creates a new [`Deadzone2::Square`] instance with the given settings.
    ///
    /// # Arguments
    ///
    /// - `threshold_x`: Threshold for the absolute value of input values on the x-axis,
    ///      clamped to the range `[0.0, 1.0]`
    /// - `threshold_y`: Threshold for the absolute value of input values on the y-axis,
    ///      clamped to the range `[0.0, 1.0]`
    #[must_use]
    pub fn square(threshold_x: f32, threshold_y: f32) -> Self {
        Self::Square {
            deadzone_x: Deadzone1::symmetric(threshold_x),
            deadzone_y: Deadzone1::symmetric(threshold_y),
        }
    }

    /// Creates a new [`Deadzone2::RoundedSquare`] instance with the given settings.
    ///
    /// If both `threshold_x` and `threshold_y` are non-positive,
    /// a [`Deadzone2::Circle`] with the specified radii will be created instead.
    ///
    /// If both `radius_x` and `radius_y` are non-positive,
    /// a [`Deadzone2::Square`] with the specified radii will be created instead.
    ///
    /// # Arguments
    ///
    /// - `threshold_x`: Threshold for the absolute value of input values along the x-axis,
    ///      clamped to the range `[0.0, 1.0]`
    /// - `threshold_y`: Threshold for the absolute value of input values along the y-axis,
    ///      clamped to the range `[0.0, 1.0]`
    /// - `radius_x`: Radius for the rounded corners along the x-axis, clamped into the range `[0.0, 1.0]`
    /// - `radius_y`: Radius for the rounded corners along the y-axis, clamped into the range `[0.0, 1.0]`
    #[must_use]
    pub fn rounded_square(
        threshold_x: f32,
        threshold_y: f32,
        radius_x: f32,
        radius_y: f32,
    ) -> Self {
        if threshold_x <= 0.0 && threshold_y <= 0.0 {
            return Self::circle(radius_x, radius_y);
        }

        if radius_x <= 0.0 && radius_y <= 0.0 {
            return Self::square(threshold_x, threshold_y);
        }

        Self::RoundedSquare {
            // Ensure all the components are within the range `[0.0, MAX]`
            threshold_x: threshold_x.clamp(0.0, DEFAULT_LIVEZONE_UPPER),
            threshold_y: threshold_y.clamp(0.0, DEFAULT_LIVEZONE_UPPER),
            radius_x: radius_x.clamp(0.0, DEFAULT_LIVEZONE_UPPER),
            radius_y: radius_y.clamp(0.0, DEFAULT_LIVEZONE_UPPER),
        }
    }

    /// Applies the current deadzone to the `input_value` and returns the adjusted value.
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

    /// Applies the current [`Deadzone2::Circle`] to the `input_value`
    /// and returns the adjusted value.
    ///
    /// # Returns
    ///
    /// - Values within the circle are treated as `Vec2::ZERO`
    /// - Values outside the circle are scaled linearly to the range `[-1.0, 1.0]` on each axis
    #[inline]
    fn value_circle(input_value: Vec2, radius_x: f32, radius_y: f32) -> Vec2 {
        let Vec2 { x, y } = input_value;

        // Calculate the threshold for the xy values
        // which is the distance from the closest point on the circle to the `input_value`
        let angle = x.atan2(y);
        let closest_x = radius_x * angle.sin().abs();
        let closest_y = radius_y * angle.cos().abs();

        // Normalize the xy values
        let new_x = Self::normalize_input(x, closest_x);
        let new_y = Self::normalize_input(y, closest_y);

        Vec2::new(new_x, new_y)
    }

    /// Applies the current [`Deadzone2::RoundedSquare`] to the `input_value`
    /// and returns the adjusted value.
    ///
    /// # Returns
    ///
    /// - Values within the central zone are treated as `Vec2::ZERO`
    /// - Values within the `deadzone_x` zone (excluding the central zone)
    ///     have the x value treated as `0.0`, and the y value is scaled linearly to the range `[-1.0, 1.0]`
    /// - Values within the `deadzone_y` zone (excluding the central zone)
    ///     have the y value treated as `0.0`, and the x value is scaled linearly to the range `[-1.0, 1.0]`
    /// - Values outside these shapes are scaled linearly to the range `[-1.0, 1.0]` on each axis
    #[inline]
    fn value_rounded_square(
        input_value: Vec2,
        threshold_x: f32,
        threshold_y: f32,
        radius_x: f32,
        radius_y: f32,
    ) -> Vec2 {
        let Vec2 { x, y } = input_value;

        // Calculate the actual threshold for the xy values
        // which is the distance from the closest point on the square to the `input_value`
        let angle = (x.abs() - threshold_x).atan2(y.abs() - threshold_y);
        let closest_x = livezone_min(threshold_x, angle.sin(), radius_x, y.abs() > threshold_y);
        let closest_y = livezone_min(threshold_y, angle.cos(), radius_y, x.abs() > threshold_x);
        fn livezone_min(value_min: f32, angle: f32, radius: f32, nearer_corner: bool) -> f32 {
            if nearer_corner {
                radius.mul_add(angle.abs(), value_min)
            } else {
                radius + value_min
            }
        }

        // Normalize the xy values
        let new_x = Self::normalize_input(x, closest_x);
        let new_y = Self::normalize_input(y, closest_y);

        Vec2::new(new_x, new_y)
    }

    /// Normalizes the provided `input_value` into the livezone range `[threshold, 1.0]`
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
            let deadzone = Deadzone1::DEFAULT_SYMMETRIC;

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
        use bevy::math::{vec2, Vec2};

        use crate::prelude::Deadzone2;

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
            let deadzone = Deadzone2::circle(0.3, 0.35);

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
            let deadzone = Deadzone2::square(0.25, 0.3);

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
            let deadzone = Deadzone2::rounded_square(0.15, 0.16, 0.05, 0.06);

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
