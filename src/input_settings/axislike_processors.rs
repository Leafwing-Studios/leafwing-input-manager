//! Utilities for processing axis-like inputs.
//!
//! This module provides structures and functions for processing axis-like inputs in applications or games.
//! Axis-like inputs typically involve values representing movement or direction along one or two axes, such as joystick movements or mouse positions.
//!
//! ## Clamp Processors
//!
//! The [`InputClamp`] enum defines various types of input clamping:
//! - [`InputClamp::None`]: No clamping is applied
//! - [`InputClamp::AtLeast`]: Clamps the input value to be at least the specified minimum value
//! - [`InputClamp::AtMost`]: Clamps the input value to be at most the specified maximum value
//! - [`InputClamp::Range`]: Clamps the input value to be within the specified range
//!
//! ## Normalization Processors
//!
//! The [`InputNormalizer`] enum defines various types of input normalization:
//! - [`InputNormalizer::None`]: No normalization is applied
//! - [`InputNormalizer::MinMax`]: A min-max normalizer to map input values to a specified output range
//!
//! ## Deadzone Processors
//!
//! - The [`SingleAxisDeadzone`] enum provides settings for deadzones in single-axis inputs:
//!   - [`SingleAxisDeadzone::None`]: No deadzone is applied
//!   - [`SingleAxisDeadzone::Symmetric`]: Deadzone with a symmetric bound
//! - The [`DualAxisDeadzone`] enum provides settings for deadzones in dual-axis inputs:
//!   - [`DualAxisDeadzone::None`]: No deadzone is applied
//!   - [`DualAxisDeadzone::Circle`]: Deadzone with a circular-shaped area
//!   - [`DualAxisDeadzone::Square`]: Deadzone with a cross-shaped area
//!   - [`DualAxisDeadzone::RoundedSquare`]: Deadzone with a cross-shaped area and four rounded corners

// region Clamp Processors ------------------------

use std::hash::{Hash, Hasher};
use std::ops::Range;

use bevy::math::Vec2;
use bevy::prelude::Reflect;
use bevy::utils::FloatOrd;
use serde::{Deserialize, Serialize};

/// Various types of input clamping.
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub enum InputClamp {
    /// No clamping is applied.
    None,

    /// Clamps the input value to be at least the specified minimum value.
    AtLeast(f32),

    /// Clamps the input value to be at most the specified maximum value.
    AtMost(f32),

    /// Clamps the input value to be within the specified range.
    Range(f32, f32),
}

impl InputClamp {
    /// Clamps the input value based on the specified clamping type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use leafwing_input_manager::prelude::InputClamp;
    ///
    /// let clamp = InputClamp::AtMost(2.0);
    ///
    /// assert_eq!(clamp.clamp(5.0), 2.0);
    /// assert_eq!(clamp.clamp(0.0), 0.0);
    /// assert_eq!(clamp.clamp(-5.0), -5.0);
    /// ```
    #[must_use]
    #[inline]
    pub fn clamp(&self, input_value: f32) -> f32 {
        match self {
            Self::None => input_value,
            Self::AtLeast(min) => input_value.max(*min),
            Self::AtMost(max) => input_value.min(*max),
            Self::Range(min, max) => input_value.clamp(*min, *max),
        }
    }
}

// endregion Clamp Processors ------------------------

// region Normalization Processors ------------------------

/// Various types of input normalization.
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub enum InputNormalizer {
    /// No normalization is applied.
    None,

    /// A min-max normalizer to map input values to a specified output range.
    MinMax {
        /// The minimum value of the input range.
        input_min: f32,

        /// The width of the range `(input_max, input_min)`,
        input_range_width: f32,

        /// The reciprocal of the `input_range_width`,
        /// pre-calculated to avoid division during computation.
        recip_input_range_width: f32,

        /// The minimum value of the output range.
        output_min: f32,

        /// The width of the output range.
        output_range_width: f32,
    },
}

impl InputNormalizer {
    /// Creates a new [`InputNormalizer::MinMax`] with the output range of `[0.0, 1.0]` and the specified input range.
    ///
    /// # Arguments
    ///
    /// - `input_range`: The range of input values
    ///
    /// # Examples
    ///
    /// ```rust
    /// use leafwing_input_manager::prelude::InputNormalizer;
    ///
    /// let normalizer = InputNormalizer::standard_min_max(0.0..100.0);
    ///
    /// assert_eq!(normalizer.normalize(500.0), 1.0);
    /// assert_eq!(normalizer.normalize(100.0), 1.0);
    /// assert_eq!(normalizer.normalize(75.0), 0.75);
    /// assert_eq!(normalizer.normalize(50.0), 0.5);
    /// assert_eq!(normalizer.normalize(25.0), 0.25);
    /// assert_eq!(normalizer.normalize(0.0), 0.0);
    /// assert_eq!(normalizer.normalize(-500.0), 0.0);
    /// ```
    pub fn standard_min_max(input_range: Range<f32>) -> Self {
        Self::custom_min_max(input_range, 0.0..1.0)
    }

    /// Creates a new [`InputNormalizer::MinMax`] with the output range of `[-1.0, 1.0]` and the specified input range.
    ///
    /// # Arguments
    ///
    /// - `input_range`: The range of input values
    ///
    /// # Examples
    ///
    /// ```rust
    /// use leafwing_input_manager::prelude::InputNormalizer;
    ///
    /// let normalizer = InputNormalizer::symmetric_min_max(0.0..100.0);
    ///
    /// assert_eq!(normalizer.normalize(500.0), 1.0);
    /// assert_eq!(normalizer.normalize(100.0), 1.0);
    /// assert_eq!(normalizer.normalize(75.0), 0.5);
    /// assert_eq!(normalizer.normalize(50.0), 0.0);
    /// assert_eq!(normalizer.normalize(25.0), -0.5);
    /// assert_eq!(normalizer.normalize(0.0), -1.0);
    /// assert_eq!(normalizer.normalize(-500.0), -1.0);
    /// ```
    pub fn symmetric_min_max(input_range: Range<f32>) -> Self {
        Self::custom_min_max(input_range, -1.0..1.0)
    }

    /// Creates a new [`InputNormalizer::MinMax`] with the specified input and output ranges.
    ///
    /// # Arguments
    ///
    /// - `input_range`: The range of input values
    /// - `output_range`: The range of output values
    ///
    /// # Examples
    ///
    /// ```rust
    /// use leafwing_input_manager::prelude::InputNormalizer;
    ///
    /// let normalizer = InputNormalizer::custom_min_max(0.0..100.0, -4.0..4.0);
    ///
    /// assert_eq!(normalizer.normalize(500.0), 4.0);
    /// assert_eq!(normalizer.normalize(100.0), 4.0);
    /// assert_eq!(normalizer.normalize(75.0), 2.0);
    /// assert_eq!(normalizer.normalize(50.0), 0.0);
    /// assert_eq!(normalizer.normalize(25.0), -2.0);
    /// assert_eq!(normalizer.normalize(0.0), -4.0);
    /// assert_eq!(normalizer.normalize(-500.0), -4.0);
    /// ```
    pub fn custom_min_max(input_range: Range<f32>, output_range: Range<f32>) -> Self {
        let (input_min, input_max) = (input_range.start, input_range.end);
        let (output_min, output_max) = (output_range.start, output_range.end);

        let input_range_width = input_max - input_min;
        Self::MinMax {
            input_min,
            input_range_width,
            recip_input_range_width: input_range_width.recip(),
            output_min,
            output_range_width: output_max - output_min,
        }
    }

    /// Returns the normalized input value.
    ///
    /// # Examples
    ///
    /// ```
    /// use leafwing_input_manager::prelude::InputNormalizer;
    ///
    /// let normalizer = InputNormalizer::symmetric_min_max(0.0..100.0);
    ///
    /// assert_eq!(normalizer.normalize(-500.0), -1.0);
    /// assert_eq!(normalizer.normalize(0.0), -1.0);
    /// assert_eq!(normalizer.normalize(25.0), -0.5);
    /// assert_eq!(normalizer.normalize(50.0), 0.0);
    /// assert_eq!(normalizer.normalize(75.0), 0.5);
    /// assert_eq!(normalizer.normalize(100.0), 1.0);
    /// assert_eq!(normalizer.normalize(500.0), 1.0);
    /// ```
    #[must_use]
    pub fn normalize(&self, input_value: f32) -> f32 {
        match self {
            InputNormalizer::None => input_value,
            InputNormalizer::MinMax {
                input_min,
                input_range_width,
                recip_input_range_width,
                output_min,
                output_range_width,
            } => {
                // Using `clamp` here helps optimizations like `minss` and `maxss` when supported,
                // potentially reducing branching logic
                let clamped_value = (input_value - input_min).clamp(0.0, *input_range_width);
                let scaled_value = clamped_value * recip_input_range_width;
                scaled_value.mul_add(*output_range_width, *output_min)
            }
        }
    }
}

// endregion Normalization Processors ------------------------

// region Deadzone Processors ------------------------

/// Default maximum limit for deadzone.
pub const DEFAULT_DEADZONE_MAX: f32 = 0.1;

/// Default maximum limit for livezone.
pub const DEFAULT_LIVEZONE_MAX: f32 = 1.0;

/// Various deadzones in single-axis inputs.
///
/// The input values are categorized into five ranges:
/// - `[f32::MIN, -1.0]`: Treated as `-1.0`
/// - `(-1.0, -min)`: Normalized into the range `(-1.0, 0.0)`
/// - `[-min, min]`: Treated as `0.0`
/// - `(min, 1.0)`: Normalized into the range `(0.0, 1.0)`
/// - `[1.0, f32::MAX]`: Treated as `1.0`
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub enum SingleAxisDeadzone {
    /// No deadzone is applied.
    None,

    /// Deadzone with a symmetric bound.
    ///
    /// The input values are categorized into five ranges:
    /// - `[f32::MIN, -1.0]`: Treated as `-1.0`
    /// - `(-1.0, -min)`: Normalized into the range `(-1.0, 0.0)`
    /// - `[-min, min]`: Treated as `0.0`
    /// - `(min, 1.0)`: Normalized into the range `(0.0, 1.0)`
    /// - `[1.0, f32::MAX]`: Treated as `1.0`
    Symmetric {
        /// The minimum limit for input values.
        ///
        /// Values within the range `[-min, min]` are treated as `0.0`
        min: f32,

        /// The cached width of the deadzone-excluded range `(min, 1.0]`.
        livezone_width: f32,

        /// The cached reciprocal of the `livezone_width`,
        /// improving performance by eliminating the need for division during computation.
        recip_livezone_width: f32,
    },
}

impl SingleAxisDeadzone {
    /// Default [`SingleAxisDeadzone`].
    ///
    /// This deadzone excludes near-zero input values within the range `[-0.1, 0.1]`.
    pub const DEFAULT: Self = Self::Symmetric {
        min: DEFAULT_DEADZONE_MAX,
        livezone_width: DEFAULT_LIVEZONE_MAX - DEFAULT_DEADZONE_MAX,
        recip_livezone_width: 1.0 / (DEFAULT_LIVEZONE_MAX - DEFAULT_DEADZONE_MAX),
    };

    /// Creates a new [`SingleAxisDeadzone::Symmetric`] to filter input values within the range `[-min, min]`.
    ///
    /// If the `min` is less than or equal to `0.0`, returns the constant [`SingleAxisDeadzone::None`].
    ///
    /// # Arguments
    ///
    /// - `min`: The minimum limit for the absolute values, clamped to the range `[0.0, 1.0]`
    #[must_use]
    pub fn symmetric(min: f32) -> Self {
        if min <= 0.0 {
            return Self::None;
        }

        let min = min.min(DEFAULT_LIVEZONE_MAX);
        let livezone_width = DEFAULT_LIVEZONE_MAX - min;
        Self::Symmetric {
            min,
            livezone_width,
            recip_livezone_width: livezone_width.recip(),
        }
    }

    /// Returns the deadzone-adjusted `input_value`.
    #[must_use]
    pub fn value(&self, input_value: f32) -> f32 {
        match self {
            SingleAxisDeadzone::None => input_value,
            SingleAxisDeadzone::Symmetric {
                min,
                livezone_width,
                recip_livezone_width,
            } => Self::value_symmetric(input_value, min, livezone_width, recip_livezone_width),
        }
    }

    /// Returns the adjusted `input_value` after applying the [`SingleAxisDeadzone::Symmetric`].
    ///
    /// The `input_value` is categorized into five ranges:
    /// - `[f32::MIN, -1.0]`: Treated as `-1.0`
    /// - `(-1.0, -min)`: Normalized into the range `(-1.0, 0.0)`
    /// - `[-min, min]`: Treated as `0.0`
    /// - `(min, 1.0)`: Normalized into the range `(0.0, 1.0)`
    /// - `[1.0, f32::MAX]`: Treated as `1.0`
    fn value_symmetric(
        input_value: f32,
        min: &f32,
        livezone_width: &f32,
        recip_livezone_width: &f32,
    ) -> f32 {
        let distance_to_min = input_value.abs() - min;
        if distance_to_min <= f32::EPSILON {
            0.0
        } else if livezone_width - distance_to_min <= f32::EPSILON {
            DEFAULT_LIVEZONE_MAX * input_value.signum()
        } else {
            distance_to_min * recip_livezone_width * input_value.signum()
        }
    }
}

/// Various deadzones in dual-axis inputs.
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub enum DualAxisDeadzone {
    /// No deadzone is applied.
    None,

    /// Deadzone with a circular-shaped area.
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
        /// The radius for the deadzone along the x-axis.
        radius_x: f32,
        /// The radius for the deadzone along the y-axis.
        radius_y: f32,
    },

    /// Deadzone with a cross-shaped area.
    ///
    /// This deadzone consists of two [`SingleAxisDeadzone`]s intersecting at their centers,
    /// providing independent deadzone areas for each axis.
    ///
    /// Both xy values are categorized into four regions:
    /// - Values within the center square are treated as `Vec2::ZERO`
    /// - Values within the `deadzone_x` rectangle (excluding the rounded square)
    ///     have their x value set to `0.0`, and their y value is normalized
    /// - Values within the `deadzone_y` rectangle (excluding the rounded square)
    ///     have their y value set to `0.0`, and their x value is normalized
    /// - Values outside these shapes are normalized into the range `[-1.0, 1.0]`
    ///
    /// # Advantages
    /// - Provides independent deadzone control for each axis
    /// - Creates a "snapping" effect, which may be desirable for certain input devices or game mechanics
    ///
    /// # Disadvantages
    /// - May result in less smooth transitions compared to [`DualAxisDeadzone::Circle`]
    Square {
        /// The deadzone along the x-axis.
        deadzone_x: SingleAxisDeadzone,
        /// The deadzone along the y-axis.
        deadzone_y: SingleAxisDeadzone,
    },

    /// Deadzone with a cross-shaped area and four rounded corners.
    ///
    /// This deadzone consists of a rounded square and two rectangles intersecting at their centers,
    /// providing smooth transitions into and out of the deadzone area.
    ///
    /// Both xy values are categorized into four regions:
    /// - Values within the rounded square are treated as `Vec2::ZERO`
    /// - Values within the `[-min_x, min_x]` rectangle (excluding the rounded square)
    ///     have their x value set to `0.0`, and their y value is normalized
    /// - Values within the `[-min_y, min_y]` rectangle (excluding the rounded square)
    ///     have their y value set to `0.0`, and their x value is normalized
    /// - Values outside these shapes are normalized into the range `[-1.0, 1.0]`
    ///
    /// # Advantages
    /// - Combines the advantages of [`DualAxisDeadzone::Circle`] and [`DualAxisDeadzone::Square`]
    ///
    /// # Disadvantages
    /// - Requires computational resources similar to calculating both of them at the same time
    RoundedSquare {
        /// The minimum limit for input values along the x-axis.
        min_x: f32,
        /// The minimum limit for input values along the y-axis.
        min_y: f32,
        /// The radius for the rounded corners along the x-axis.
        radius_x: f32,
        /// The radius for the rounded corners along the y-axis.
        radius_y: f32,
    },
}

impl DualAxisDeadzone {
    /// Default [`DualAxisDeadzone::Circle`].
    ///
    /// This deadzone excludes near-zero input values within a distance of `0.1` from `Vec2::ZERO`.
    pub const CIRCLE_DEFAULT: Self = Self::Circle {
        radius_x: DEFAULT_DEADZONE_MAX,
        radius_y: DEFAULT_DEADZONE_MAX,
    };

    /// Default [`DualAxisDeadzone::Square`].
    ///
    /// This deadzone excludes near-zero input values within the range `[-0.1, 0.1]` on each axis.
    pub const SQUARE_DEFAULT: Self = Self::Square {
        deadzone_x: SingleAxisDeadzone::DEFAULT,
        deadzone_y: SingleAxisDeadzone::DEFAULT,
    };

    /// Default [`DualAxisDeadzone::RoundedSquare`].
    ///
    /// This deadzone consists of a square with rounded corners,
    /// excluding near-zero input values within the range `[-0.1, 0.1]` on each axis,
    /// and applying rounded corners with the radius of `0.025` along each axis.
    pub const ROUNDED_SQUARE_DEFAULT: Self = Self::RoundedSquare {
        min_x: DEFAULT_DEADZONE_MAX,
        min_y: DEFAULT_DEADZONE_MAX,
        radius_x: 0.25 * DEFAULT_DEADZONE_MAX,
        radius_y: 0.25 * DEFAULT_DEADZONE_MAX,
    };

    /// Creates a new [`DualAxisDeadzone::Circle`] with the given settings.
    ///
    /// If both `radius_x` and `radius_y` are less than or equal to `0.0`,
    /// returns the constant [`DualAxisDeadzone::None`].
    ///
    /// # Arguments
    ///
    /// - `radius_x`: The radius along the x-axis for the deadzone, clamped into the range `[0.0, 1.0]`
    /// - `radius_y`: The radius along the y-axis for the deadzone, clamped into the range `[0.0, 1.0]`
    #[must_use]
    pub fn new_circle(radius_x: f32, radius_y: f32) -> Self {
        if radius_x <= 0.0 && radius_y <= 0.0 {
            return Self::None;
        }

        Self::Circle {
            // Ensure all the radii are within the range `[0.0, MAX]`
            radius_x: radius_x.clamp(0.0, DEFAULT_LIVEZONE_MAX),
            radius_y: radius_y.clamp(0.0, DEFAULT_LIVEZONE_MAX),
        }
    }

    /// Creates a new [`DualAxisDeadzone::Square`] with the given settings.
    ///
    /// # Arguments
    ///
    /// - `min_x`: The minimum limit for the absolute values on the x-axis, clamped to the range `[0.0, 1.0]`
    /// - `min_y`: The minimum limit for the absolute values on the y-axis, clamped to the range `[0.0, 1.0]`
    #[must_use]
    pub fn new_square(min_x: f32, min_y: f32) -> Self {
        Self::Square {
            deadzone_x: SingleAxisDeadzone::symmetric(min_x),
            deadzone_y: SingleAxisDeadzone::symmetric(min_y),
        }
    }

    /// Creates a new [`DualAxisDeadzone::RoundedSquare`] with the given settings.
    ///
    /// If both `min_x` and `min_y` are less than or equal to `0.0`,
    /// a [`DualAxisDeadzone::Circle`] with the radii will be created instead.
    ///
    /// # Arguments
    ///
    /// - `min_x`: The minimum limit for input values along the x-axis
    /// - `min_y`: The minimum limit for input values along the y-axis
    /// - `radius_x`: The radius along the x-axis for the rounded corners, clamped into the range `[0.0, 1.0]`
    /// - `radius_x`: The radius along the y-axis for the rounded corners, clamped into the range `[0.0, 1.0]`
    #[must_use]
    pub fn new_rounded_square(min_x: f32, min_y: f32, radius_x: f32, radius_y: f32) -> Self {
        if min_x <= 0.0 && min_y <= 0.0 {
            return Self::new_circle(radius_x, radius_y);
        }

        Self::RoundedSquare {
            // Ensure all the components are within the range `[0.0, MAX]`
            min_x: min_x.clamp(0.0, DEFAULT_LIVEZONE_MAX),
            min_y: min_y.clamp(0.0, DEFAULT_LIVEZONE_MAX),
            radius_x: radius_x.clamp(0.0, DEFAULT_LIVEZONE_MAX),
            radius_y: radius_y.clamp(0.0, DEFAULT_LIVEZONE_MAX),
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
                min_x,
                min_y,
                radius_x,
                radius_y,
            } => Self::value_rounded_square(input_value, *min_x, *min_y, *radius_x, *radius_y),
        }
    }

    /// Returns the adjusted `input_value` after applying the [`DualAxisDeadzone::Circle`].
    ///
    /// Both xy values are categorized into two regions:
    /// - Values within the circle are treated as `Vec2::ZERO`
    /// - Values outside the circle are normalized into the range `[-1.0, 1.0]`
    #[inline]
    fn value_circle(input_value: Vec2, radius_x: f32, radius_y: f32) -> Vec2 {
        let Vec2 { x, y } = input_value;

        // Calculate the xy values of the closest point on the circle to the `input_value`
        let angle = x.atan2(y);
        let closest_x = radius_x * angle.sin();
        let closest_y = radius_y * angle.cos();

        // Normalize the xy values
        let new_x = Self::normalize_input(x, closest_x.abs());
        let new_y = Self::normalize_input(y, closest_y.abs());

        Vec2::new(new_x, new_y)
    }

    /// Returns the adjusted `input_value` after applying the [`DualAxisDeadzone::RoundedSquare`].
    ///
    /// Both xy values are categorized into four regions:
    /// - Values within the rounded square are treated as `Vec2::ZERO`
    /// - Values within the `[-min_x, min_x]` rectangle (excluding the rounded square)
    ///     have their x value set to `0.0`, and their y value is normalized
    /// - Values within the `[-min_y, min_y]` rectangle (excluding the rounded square)
    ///     have their y value set to `0.0`, and their x value is normalized
    /// - Values outside these shapes are normalized into the range `[-1.0, 1.0]`
    #[inline]
    fn value_rounded_square(
        input_value: Vec2,
        min_x: f32,
        min_y: f32,
        radius_x: f32,
        radius_y: f32,
    ) -> Vec2 {
        let Vec2 { x, y } = input_value;

        // Calculate the real minimum bound for the xy values
        let angle = (x.abs() - min_x).atan2(y.abs() - min_y);
        let real_min_x = deadzone_min(min_x, angle.sin(), radius_x, y.abs() > min_y);
        let real_min_y = deadzone_min(min_y, angle.cos(), radius_y, x.abs() > min_x);
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

    /// Normalizes the given `input` into the livezone range `(deadzone_min, 1.0]`
    fn normalize_input(input: f32, deadzone_max: f32) -> f32 {
        let livezone_width = DEFAULT_LIVEZONE_MAX - deadzone_max;
        let distance_to_min = input.abs() - deadzone_max;
        if distance_to_min <= f32::EPSILON {
            0.0
        } else if livezone_width - distance_to_min <= f32::EPSILON {
            1.0 * input.signum()
        } else {
            distance_to_min / livezone_width * input.signum()
        }
    }
}

// endregion Deadzone Processors ------------------------

// -------------------------
// Unfortunately, Rust doesn't let us automatically derive `Eq` and `Hash` for `f32`.
// It's like teaching a fish to ride a bike – a bit nonsensical!
// But if that fish really wants to pedal, we'll make it work.
// So here we are, showing Rust who's boss!
// -------------------------

impl Eq for InputClamp {}

impl Hash for InputClamp {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            InputClamp::None => {
                0.hash(state);
            }
            InputClamp::AtLeast(min) => {
                1.hash(state);
                FloatOrd(*min).hash(state);
            }
            InputClamp::AtMost(max) => {
                2.hash(state);
                FloatOrd(*max).hash(state);
            }
            InputClamp::Range(min, max) => {
                3.hash(state);
                FloatOrd(*min).hash(state);
                FloatOrd(*max).hash(state);
            }
        }
    }
}

impl Eq for InputNormalizer {}

impl Hash for InputNormalizer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            InputNormalizer::None => {
                0.hash(state);
            }
            InputNormalizer::MinMax {
                input_min,
                input_range_width,
                recip_input_range_width,
                output_min,
                output_range_width,
            } => {
                1.hash(state);
                FloatOrd(*input_min).hash(state);
                FloatOrd(*input_range_width).hash(state);
                FloatOrd(*recip_input_range_width).hash(state);
                FloatOrd(*output_min).hash(state);
                FloatOrd(*output_range_width).hash(state);
            }
        }
    }
}

impl Eq for SingleAxisDeadzone {}

impl Hash for SingleAxisDeadzone {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            SingleAxisDeadzone::None => {
                0.hash(state);
            }
            SingleAxisDeadzone::Symmetric {
                min,
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

impl Eq for DualAxisDeadzone {}

impl Hash for DualAxisDeadzone {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            DualAxisDeadzone::None => {
                0.hash(state);
            }
            DualAxisDeadzone::Circle { radius_x, radius_y } => {
                1.hash(state);
                FloatOrd(*radius_x).hash(state);
                FloatOrd(*radius_y).hash(state);
            }
            DualAxisDeadzone::Square {
                deadzone_x,
                deadzone_y,
            } => {
                2.hash(state);
                deadzone_x.hash(state);
                deadzone_y.hash(state);
            }
            DualAxisDeadzone::RoundedSquare {
                min_x,
                min_y,
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
    use super::*;

    mod clamp {
        use super::*;

        #[test]
        fn test_none_clamp() {
            let clamp = InputClamp::None;
            assert_eq!(5.0, clamp.clamp(5.0));
            assert_eq!(0.0, clamp.clamp(0.0));
            assert_eq!(-5.0, clamp.clamp(-5.0));
        }

        #[test]
        fn test_at_least_clamp() {
            let clamp = InputClamp::AtLeast(2.0);
            assert_eq!(5.0, clamp.clamp(5.0));
            assert_eq!(2.0, clamp.clamp(2.0));
            assert_eq!(2.0, clamp.clamp(0.0));
            assert_eq!(2.0, clamp.clamp(-2.0));
            assert_eq!(2.0, clamp.clamp(-5.0));
        }

        #[test]
        fn test_at_most_clamp() {
            let clamp = InputClamp::AtMost(2.0);
            assert_eq!(2.0, clamp.clamp(5.0));
            assert_eq!(2.0, clamp.clamp(2.0));
            assert_eq!(0.0, clamp.clamp(0.0));
            assert_eq!(-2.0, clamp.clamp(-2.0));
            assert_eq!(-5.0, clamp.clamp(-5.0));
        }

        #[test]
        fn test_range_clamp() {
            let clamp = InputClamp::Range(1.0, 2.0);
            assert_eq!(2.0, clamp.clamp(5.0));
            assert_eq!(2.0, clamp.clamp(2.0));
            assert_eq!(1.0, clamp.clamp(0.0));
            assert_eq!(1.0, clamp.clamp(-2.0));
            assert_eq!(1.0, clamp.clamp(-5.0));
        }
    }

    mod nomalizer {
        use super::*;

        #[test]
        fn test_standard_min_max_normalizer() {
            let normalizer = InputNormalizer::standard_min_max(0.0..100.0);
            assert_eq!(1.0, normalizer.normalize(500.0));
            assert_eq!(1.0, normalizer.normalize(100.0));
            assert_eq!(0.75, normalizer.normalize(75.0));
            assert_eq!(0.5, normalizer.normalize(50.0));
            assert_eq!(0.25, normalizer.normalize(25.0));
            assert_eq!(0.0, normalizer.normalize(0.0));
            assert_eq!(0.0, normalizer.normalize(-100.0));
            assert_eq!(0.0, normalizer.normalize(-500.0));
        }

        #[test]
        fn test_symmetric_min_max_normalizer() {
            let normalizer = InputNormalizer::symmetric_min_max(0.0..100.0);
            assert_eq!(1.0, normalizer.normalize(500.0));
            assert_eq!(1.0, normalizer.normalize(100.0));
            assert_eq!(0.5, normalizer.normalize(75.0));
            assert_eq!(0.0, normalizer.normalize(50.0));
            assert_eq!(-0.5, normalizer.normalize(25.0));
            assert_eq!(-1.0, normalizer.normalize(0.0));
            assert_eq!(-1.0, normalizer.normalize(-100.0));
            assert_eq!(-1.0, normalizer.normalize(-500.0));
        }

        #[test]
        fn test_custom_min_max_normalizer() {
            let normalizer = InputNormalizer::custom_min_max(0.0..100.0, -4.0..4.0);
            assert_eq!(4.0, normalizer.normalize(500.0));
            assert_eq!(4.0, normalizer.normalize(100.0));
            assert_eq!(2.0, normalizer.normalize(75.0));
            assert_eq!(0.0, normalizer.normalize(50.0));
            assert_eq!(-2.0, normalizer.normalize(25.0));
            assert_eq!(-4.0, normalizer.normalize(0.0));
            assert_eq!(-4.0, normalizer.normalize(-100.0));
            assert_eq!(-4.0, normalizer.normalize(-500.0));
        }
    }

    mod single_axis_deadzone {
        use super::*;

        #[test]
        fn test_single_axis_deadzone_none() {
            let deadzone = SingleAxisDeadzone::None;

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
        fn test_single_axis_deadzone_default() {
            let deadzone = SingleAxisDeadzone::DEFAULT;

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
        fn test_single_axis_deadzone_custom() {
            let deadzone = SingleAxisDeadzone::symmetric(0.2);

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

    mod dual_axis_deadzone {
        use super::*;
        use bevy::math::vec2;

        #[test]
        fn test_dual_axis_deadzone_none() {
            let deadzone = DualAxisDeadzone::None;

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
        fn test_dual_axis_deadzone_circle_default() {
            let deadzone = DualAxisDeadzone::CIRCLE_DEFAULT;

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
        fn test_dual_axis_deadzone_circle_custom() {
            let deadzone = DualAxisDeadzone::new_circle(0.3, 0.35);

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
        }

        #[test]
        fn test_dual_axis_deadzone_square_default() {
            let deadzone = DualAxisDeadzone::SQUARE_DEFAULT;

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
        fn test_dual_axis_deadzone_square_custom() {
            let deadzone = DualAxisDeadzone::new_square(0.25, 0.3);

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
        }

        #[test]
        fn test_dual_axis_deadzone_rounded_square_default() {
            let deadzone = DualAxisDeadzone::ROUNDED_SQUARE_DEFAULT;

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

            let livezone_0_25 = Vec2::splat(0.008298924);
            assert_eq!(livezone_0_25, deadzone.value(Vec2::splat(0.125)));
            assert_eq!(-livezone_0_25, deadzone.value(Vec2::splat(-0.125)));
        }

        #[test]
        fn test_dual_axis_deadzone_rounded_square_custom() {
            let deadzone = DualAxisDeadzone::new_rounded_square(0.15, 0.16, 0.05, 0.06);

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
        }
    }
}
