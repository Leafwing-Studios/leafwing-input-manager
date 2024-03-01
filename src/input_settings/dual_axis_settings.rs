//! Settings for dual-axis input.
//!
//! Dual-axis inputs typically represent input values in two dimensions,
//! such as joystick movements in both horizontal and vertical directions.
//!
//! This module provides the [`DualAxisSettings`] struct,
//! which defines the processing pipeline and configuration for dual-axis inputs.
//! It provides similar customization options as [`SingleAxisSettings`](crate::prelude::SingleAxisSettings)
//! but separately for each axis, allowing more fine-grained control over input processing.

use std::hash::{Hash, Hasher};

use bevy::prelude::{Reflect, Vec2};
use bevy::utils::FloatOrd;
use serde::{Deserialize, Serialize};

use super::common_processors::*;
use super::deadzones::*;

/// Defines the processing pipeline and configuration for dual-axis inputs.
///
/// Dual-axis inputs are processed through a series of steps to achieve desired behavior.
/// This structure defines these steps and their configuration.
///
/// # Processing Steps
///
/// The processing pipeline transforms raw input values into usable output values
/// through a series of configurable steps:
///
/// **Note**: Limiting and Normalization aren't used by default.
///
/// 1. **Raw Input Scaling**: Adjusts raw input values based on a specified scale factor on each axis.
///    - Positive values scale input on each axis:
///      - `0.0`: Disregards input changes (effectively disables the input)
///      - `(0.0, 1.0)`: Reduces input influence
///      - `1.0`: No adjustment (default)
///      - `(1.0, infinity)`: Amplifies input influence
///    - Negative values invert and scale input on each axis (magnitude follows the same rules as positive values)
/// 2. **Raw Input Limiting (Optional)**: Clamps raw input values to a specified range on each axis.
/// 3. **Normalization (Optional)**: Maps input values into a specified range on each axis.
/// 4. **Deadzone (Optional)**: Defines regions where input values are considered neutral on each axis.
/// 5. **Processed Input Scaling**: Adjusts processed input values based on a specified scale factor on each axis.
///    It follows the same rules as raw input scaling.
/// 6. **Processed Input Limiting (Optional)**: Clamps processed input values to a specified range on each axis.
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub struct DualAxisSettings {
    /// Scales raw input values on each axis.
    ///
    /// Using a `Vec2` here for both sensitivity and inversion
    /// avoids separate fields and branching logic.
    ///
    /// Combines sensitivity (absolute value) and inversion (sign):
    /// - Positive values scale input on each axis:
    ///   - `0.0`: Disregards input changes (effectively disables the input)
    ///   - `(0.0, 1.0)`: Reduces input influence
    ///   - `1.0`: No adjustment (default)
    ///   - `(1.0, infinity)`: Amplifies input influence
    /// - Negative values invert and scale input on each axis (magnitude follows the same rules as positive values)
    raw_scales: Vec2,

    /// (Optional) Defines the range for clamping raw input values on each axis.
    raw_limits: [ValueLimit; 2],

    /// (Optional) Maps input values into a specified range on each axis.
    normalizers: [ValueNormalizer; 2],

    /// (Optional) Defines regions where input values are considered neutral on each axis.
    deadzone: Deadzone2,

    /// Scales processed input values on each axis.
    ///
    /// Using a `Vec2` here for both sensitivity and inversion
    /// avoids separate fields and branching logic.
    ///
    /// Combines sensitivity (absolute value) and inversion (sign):
    /// - Positive values scale input on each axis:
    ///   - `0.0`: Disregards input changes (effectively disables the input)
    ///   - `(0.0, 1.0)`: Reduces input influence
    ///   - `1.0`: No adjustment (default)
    ///   - `(1.0, infinity)`: Amplifies input influence
    /// - Negative values invert and scale input on each axis (magnitude follows the same rules as positive values)
    processed_scales: Vec2,

    /// (Optional) Defines the range for clamping processed input values on each axis.
    processed_limits: [ValueLimit; 2],
}

impl DualAxisSettings {
    /// - Scaling: `1.0` on both axes (default)
    /// - Deadzone: No deadzone is applied
    pub const EMPTY: DualAxisSettings = DualAxisSettings {
        raw_scales: Vec2::ONE,
        raw_limits: [ValueLimit::None, ValueLimit::None],
        normalizers: [ValueNormalizer::None, ValueNormalizer::None],
        deadzone: Deadzone2::None,
        processed_scales: Vec2::ONE,
        processed_limits: [ValueLimit::None, ValueLimit::None],
    };

    /// - Scaling: `1.0` on both axes (default)
    /// - Deadzone: Excludes input values within a distance of `0.1` from `Vec2::ZERO`
    /// - Output: Values within the livezone are normalized into the range `[-1.0, 1.0]` on each axis
    pub const DEFAULT_CIRCLE_DEADZONE: DualAxisSettings = DualAxisSettings {
        raw_scales: Vec2::ONE,
        raw_limits: [ValueLimit::None, ValueLimit::None],
        normalizers: [ValueNormalizer::None, ValueNormalizer::None],
        deadzone: Deadzone2::CIRCLE_DEFAULT,
        processed_scales: Vec2::ONE,
        processed_limits: [ValueLimit::None, ValueLimit::None],
    };

    /// - Scaling: `1.0` on both axes (default)
    /// - Deadzone: Excludes input values within the range `[-0.1, 0.1]` on each axis
    /// - Output: Values within the livezone are normalized into the range `[-1.0, 1.0]` on each axis
    pub const DEFAULT_SQUARE_DEADZONE: DualAxisSettings = DualAxisSettings {
        raw_scales: Vec2::ONE,
        raw_limits: [ValueLimit::None, ValueLimit::None],
        normalizers: [ValueNormalizer::None, ValueNormalizer::None],
        deadzone: Deadzone2::SQUARE_DEFAULT,
        processed_scales: Vec2::ONE,
        processed_limits: [ValueLimit::None, ValueLimit::None],
    };

    /// - Scaling: `1.0` on both axes (default)
    /// - Deadzone: Excluding input values within the range `[-0.1, 0.1]` on each axis,
    ///     and applying rounded corners with the radius of `0.025` along each axis
    /// - Output: Values within the livezone are normalized into the range `[-1.0, 1.0]` on each axis
    pub const DEFAULT_ROUNDED_SQUARE_DEADZONE: DualAxisSettings = DualAxisSettings {
        raw_scales: Vec2::ONE,
        raw_limits: [ValueLimit::None, ValueLimit::None],
        normalizers: [ValueNormalizer::None, ValueNormalizer::None],
        deadzone: Deadzone2::ROUNDED_SQUARE_DEFAULT,
        processed_scales: Vec2::ONE,
        processed_limits: [ValueLimit::None, ValueLimit::None],
    };

    /// Creates a new [`DualAxisSettings`] instance with the provided `scales` for raw input values on each axis.
    ///
    /// # Arguments
    ///
    /// - `scale`: The scale factor to adjust raw input values on each axis:
    ///   - Positive values scale the input:
    ///     - `0.0`: Disregards input changes (effectively disables the input)
    ///     - `(0.0, 1.0)`: Reduces input influence
    ///     - `1.0`: No adjustment (default)
    ///     - `(1.0, infinity)`: Amplifies input influence
    ///   - Negative values invert and scale the input (magnitude follows the same rules as positive values)
    ///
    /// # Example
    ///
    /// ```rust
    /// use bevy::math::Vec2;
    /// use leafwing_input_manager::prelude::DualAxisSettings;
    ///
    /// // Create a new DualAxisSettings with scale 0.5 for raw input on each axis
    /// let base = DualAxisSettings::EMPTY.with_raw_scales(Vec2::splat(0.5));
    ///
    /// // Increase the factors on each axis
    /// let more_sensitive = base.with_raw_scales(Vec2::splat(2.0));
    ///
    /// // Further increase the factors on each axis
    /// let ultra_sensitive = more_sensitive.with_raw_scale(Vec2::splat(100.0));
    /// ```
    #[must_use]
    pub const fn with_raw_scales(&self, scales: Vec2) -> Self {
        Self {
            raw_scales: scales,
            raw_limits: self.raw_limits,
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            processed_scales: self.processed_scales,
            processed_limits: self.processed_limits,
        }
    }

    /// Creates a new [`DualAxisSettings`] instance with the provided `scale` for raw input values on the x-axis.
    ///
    /// # Arguments
    ///
    /// - `scale`: The scale factor to adjust raw input values on the x-axis:
    ///   - Positive values scale input:
    ///     - `0.0`: Disregards input changes (effectively disables the input)
    ///     - `(0.0, 1.0)`: Reduces input influence
    ///     - `1.0`: No adjustment (default)
    ///     - `(1.0, infinity)`: Amplifies input influence
    ///   - Negative values invert and scale input (magnitude follows the same rules as positive values)
    ///
    /// # Example
    ///
    /// ```rust
    /// use bevy::math::Vec2;
    /// use leafwing_input_manager::prelude::DualAxisSettings;
    ///
    /// // Create a new DualAxisSettings with scale 0.5 for raw input on the x-axis
    /// let settings = DualAxisSettings::EMPTY.with_raw_scale_x(0.5);
    ///
    /// // Increase the factor on the x-axis
    /// let more_sensitive = settings.with_raw_scale_x(2.0);
    ///
    /// // Further increase the factor on the x-axis
    /// let ultra_sensitive = more_sensitive.with_raw_scale_x(100.0);
    /// ```
    #[must_use]
    pub const fn with_raw_scale_x(&self, scale: f32) -> Self {
        Self {
            raw_scales: Vec2::new(scale, self.raw_scales.y),
            raw_limits: self.raw_limits,
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            processed_scales: self.processed_scales,
            processed_limits: self.processed_limits,
        }
    }

    /// Creates a new [`DualAxisSettings`] instance with the provided `scale` for raw input values on the y-axis.
    ///
    /// # Arguments
    ///
    /// - `scale`: The scale factor to adjust raw input values on the y-axis.
    ///   - Positive values scale input:
    ///     - `0.0`: Disregards input changes (effectively disables the input)
    ///     - `(0.0, 1.0)`: Reduces input influence
    ///     - `1.0`: No adjustment (default)
    ///     - `(1.0, infinity)`: Amplifies input influence
    ///   - Negative values invert and scale input (magnitude follows the same rules as positive values)
    ///
    /// # Example
    ///
    /// ```rust
    /// use bevy::math::Vec2;
    /// use leafwing_input_manager::prelude::DualAxisSettings;
    ///
    /// // Create a new DualAxisSettings with scale 0.5 for raw input on the y-axis
    /// let settings = DualAxisSettings::EMPTY.with_raw_scale_y(0.5);
    ///
    /// // Increase the factor on the y-axis
    /// let more_sensitive = settings.with_raw_scale_y(2.0);
    ///
    /// // Further increase the factor on the y-axis
    /// let ultra_sensitive = more_sensitive.with_raw_scale_y(100.0);
    /// ```
    #[must_use]
    pub const fn with_raw_scale_y(&self, scale: f32) -> Self {
        Self {
            raw_scales: Vec2::new(self.raw_scales.x, scale),
            raw_limits: self.raw_limits,
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            processed_scales: self.processed_scales,
            processed_limits: self.processed_limits,
        }
    }

    /// Creates a new [`DualAxisSettings`] instance with inversion applied to raw input values on each axis.
    ///
    /// # Example
    ///
    /// ```rust
    /// use leafwing_input_manager::prelude::DualAxisSettings;
    ///
    /// let original = DualAxisSettings::EMPTY;
    ///
    /// // Invert the input direction on each axis
    /// let inverted = original.with_inverted();
    ///
    /// // Revert the settings to the original
    /// let reverted = inverted.with_inverted();
    /// ```
    #[must_use]
    pub fn with_inverted(&self) -> Self {
        Self {
            raw_scales: -self.raw_scales,
            raw_limits: self.raw_limits,
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            processed_scales: self.processed_scales,
            processed_limits: self.processed_limits,
        }
    }

    /// Creates a new [`DualAxisSettings`] instance with inversion applied to raw input values on the x-axis.
    ///
    /// # Example
    ///
    /// ```rust
    /// use leafwing_input_manager::prelude::DualAxisSettings;
    ///
    /// let original = DualAxisSettings::EMPTY;
    ///
    /// // Invert the input direction on the x-axis
    /// let inverted_x = original.with_inverted_x();
    ///
    /// // Revert the settings to the original
    /// let reverted = inverted_x.with_inverted_x();
    /// ```
    #[must_use]
    pub fn with_inverted_x(&self) -> Self {
        Self {
            raw_scales: Vec2::new(-self.raw_scales.x, self.raw_scales.y),
            raw_limits: self.raw_limits,
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            processed_scales: self.processed_scales,
            processed_limits: self.processed_limits,
        }
    }

    /// Creates a new [`DualAxisSettings`] instance with inversion applied to raw input values on the y-axis.
    ///
    /// # Example
    ///
    /// ```rust
    /// use leafwing_input_manager::prelude::DualAxisSettings;
    ///
    /// let original = DualAxisSettings::EMPTY;
    ///
    /// // Invert the input direction on the y-axis
    /// let inverted_y = original.with_inverted_y();
    ///
    /// // Revert the settings to the original
    /// let reverted = inverted_y.with_inverted_y();
    /// ```
    #[must_use]
    pub fn with_inverted_y(&self) -> Self {
        Self {
            raw_scales: Vec2::new(self.raw_scales.x, -self.raw_scales.y),
            raw_limits: self.raw_limits,
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            processed_scales: self.processed_scales,
            processed_limits: self.processed_limits,
        }
    }

    /// Creates a new [`DualAxisSettings`] instance with the provided `limit` for raw input values on the x-axis.
    ///
    /// # Arguments
    ///
    /// - `limit`: The limit for raw input values on the x-axis.
    ///
    /// See [`ValueLimit`] documentation for available limit options.
    ///
    /// # Example
    ///
    /// ```rust
    /// use leafwing_input_manager::prelude::{DualAxisSettings, ValueLimit};
    ///
    /// // Create a new DualAxisSettings with a non-negative limit on the x-axis
    /// let non_negative = DualAxisSettings::EMPTY.with_raw_limit_x(ValueLimit::AtLeast(0.0));
    /// ```
    #[must_use]
    pub fn with_raw_limit_x(&self, limit: ValueLimit) -> Self {
        Self {
            raw_scales: self.raw_scales,
            raw_limits: [limit, self.raw_limits[1]],
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            processed_scales: self.processed_scales,
            processed_limits: self.processed_limits,
        }
    }

    /// Creates a new [`DualAxisSettings`] instance with the provided `limit` for raw input values on the y-axis.
    ///
    /// # Arguments
    ///
    /// - `limit`: The limit for raw input values on the y-axis.
    ///
    /// See [`ValueLimit`] documentation for available limit options.
    ///
    /// # Example
    ///
    /// ```rust
    /// use leafwing_input_manager::prelude::{DualAxisSettings, ValueLimit};
    ///
    /// // Create a new DualAxisSettings with a non-negative limit on the x-axis
    /// let non_negative = DualAxisSettings::EMPTY.with_raw_limit_x(ValueLimit::AtLeast(0.0));
    /// ```
    #[must_use]
    pub fn with_raw_limit_y(&self, limit: ValueLimit) -> Self {
        Self {
            raw_scales: self.raw_scales,
            raw_limits: [self.raw_limits[0], limit],
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            processed_scales: self.processed_scales,
            processed_limits: self.processed_limits,
        }
    }

    /// Creates a new [`DualAxisSettings`] instance with the provided `normalizer` on the x-axis.
    ///
    /// # Arguments
    ///
    /// - `normalizer`: The normalizer that maps input values on the x-axis into a specified range.
    ///
    /// See [`ValueNormalizer`] documentation for available normalization options.
    ///
    /// # Example
    ///
    /// ```rust
    /// use leafwing_input_manager::prelude::{DualAxisSettings, ValueNormalizer};
    ///
    /// // Create a new DualAxisSettings with a standard min-max normalizer on the x-axis
    ///let standard_min_max_settings = DualAxisSettings::EMPTY
    ///     .with_normalizer_x(ValueNormalizer::standard_min_max(-100.0..100.0));
    /// ```
    #[must_use]
    pub fn with_normalizer_x(&self, normalizer: ValueNormalizer) -> Self {
        Self {
            raw_scales: self.raw_scales,
            raw_limits: self.raw_limits,
            normalizers: [normalizer, self.normalizers[1]],
            deadzone: self.deadzone,
            processed_scales: self.processed_scales,
            processed_limits: self.processed_limits,
        }
    }

    /// Creates a new [`DualAxisSettings`] instance with the provided `normalizer` on the y-axis.
    ///
    /// # Arguments
    ///
    /// - `normalizer`: The normalizer that maps input values on the y-axis into a specified range.
    ///
    /// See [`ValueNormalizer`] documentation for available normalization options.
    ///
    /// # Example
    ///
    /// ```rust
    /// use leafwing_input_manager::prelude::{DualAxisSettings, ValueNormalizer};
    ///
    /// // Create a new DualAxisSettings with a standard min-max normalizer on the y-axis
    ///let standard_min_max_settings = DualAxisSettings::EMPTY
    ///     .with_normalizer_y(ValueNormalizer::standard_min_max(-100.0..100.0));
    /// ```
    #[must_use]
    pub fn with_normalizer_y(&self, normalizer: ValueNormalizer) -> Self {
        Self {
            raw_scales: self.raw_scales,
            raw_limits: self.raw_limits,
            normalizers: [self.normalizers[0], normalizer],
            deadzone: self.deadzone,
            processed_scales: self.processed_scales,
            processed_limits: self.processed_limits,
        }
    }

    /// Creates a new [`DualAxisSettings`] instance with the provided `deadzone`.
    ///
    /// # Arguments
    ///
    /// - `deadzone`: The deadzone that defines regions where input values are considered neutral.
    ///
    /// See [`Deadzone1`] documentation for available deadzone options.
    ///
    /// # Example
    ///
    /// ```rust
    /// use leafwing_input_manager::prelude::{Deadzone2, DualAxisSettings};
    ///
    /// // Create a new DualAxisSettings with a circular deadzone defined by radii 0.2 and 0.3
    ///let settings = DualAxisSettings::EMPTY.with_deadzone(Deadzone2::new_circle(0.2, 0.3));
    /// ```
    #[must_use]
    pub fn with_deadzone(&self, deadzone: Deadzone2) -> Self {
        Self {
            raw_scales: self.raw_scales,
            raw_limits: self.raw_limits,
            normalizers: self.normalizers,
            deadzone,
            processed_scales: self.processed_scales,
            processed_limits: self.processed_limits,
        }
    }

    /// Creates a new [`DualAxisSettings`] instance with the provided `scales` for processed input values on both axes.
    ///
    /// # Arguments
    ///
    /// - `scale`: The scale factor to adjust processed input values on each axis:
    ///   - Positive values scale the input:
    ///     - `0.0`: Disregards input changes (effectively disables the input)
    ///     - `(0.0, 1.0)`: Reduces input influence
    ///     - `1.0`: No adjustment (default)
    ///     - `(1.0, infinity)`: Amplifies input influence
    ///   - Negative values invert and scale the input (magnitude follows the same rules as positive values)
    ///
    /// # Example
    ///
    /// ```rust
    /// use bevy::math::Vec2;
    /// use leafwing_input_manager::prelude::DualAxisSettings;
    ///
    /// // Create a new DualAxisSettings with scale 0.5 for processed input on each axis
    /// let base = DualAxisSettings::EMPTY.with_processed_scales(Vec2::splat(0.5));
    ///
    /// // Increase the factors on each axis
    /// let more_sensitive = base.with_processed_scales(Vec2::splat(2.0));
    ///
    /// // Further increase the factors on each axis
    /// let ultra_sensitive = more_sensitive.with_processed_scale(Vec2::splat(100.0));
    /// ```
    #[must_use]
    pub fn with_processed_scales(&self, scales: Vec2) -> Self {
        Self {
            raw_scales: self.raw_scales,
            raw_limits: self.raw_limits,
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            processed_scales: scales,
            processed_limits: self.processed_limits,
        }
    }

    /// Creates a new [`DualAxisSettings`] instance with the provided `scale` for processed input values on the x-axis.
    ///
    /// # Arguments
    ///
    /// - `scale`: The scale factor to adjust processed input values on the x-axis:
    ///   - Positive values scale the input:
    ///     - `0.0`: Disregards input changes (effectively disables the input)
    ///     - `(0.0, 1.0)`: Reduces input influence
    ///     - `1.0`: No adjustment (default)
    ///     - `(1.0, infinity)`: Amplifies input influence
    ///   - Negative values invert and scale the input (magnitude follows the same rules as positive values)
    ///
    /// # Example
    ///
    /// ```rust
    /// use bevy::math::Vec2;
    /// use leafwing_input_manager::prelude::DualAxisSettings;
    ///
    /// // Create a new DualAxisSettings with scale 0.5 for processed input on the x-axis
    /// let settings = DualAxisSettings::EMPTY.with_processed_scale_x(0.5);
    ///
    /// // Increase the factor on the x-axis
    /// let more_sensitive = settings.with_processed_scale_x(2.0);
    ///
    /// // Further increase the factor on the x-axis
    /// let ultra_sensitive = more_sensitive.with_processed_scale_x(100.0);
    /// ```
    #[must_use]
    pub fn with_processed_scale_x(&self, scale: f32) -> Self {
        Self {
            raw_scales: self.raw_scales,
            raw_limits: self.raw_limits,
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            processed_scales: Vec2::new(scale, self.processed_scales.y),
            processed_limits: self.processed_limits,
        }
    }

    /// Creates a new [`DualAxisSettings`] instance with the provided `scale` for processed input values on the y-axis.
    ///
    /// # Arguments
    ///
    /// - `scale`: The scale factor to adjust processed input values on the y-axis.
    ///   - Positive values scale input:
    ///     - `0.0`: Disregards input changes (effectively disables the input)
    ///     - `(0.0, 1.0)`: Reduces input influence
    ///     - `1.0`: No adjustment (default)
    ///     - `(1.0, infinity)`: Amplifies input influence
    ///   - Negative values invert and scale input (magnitude follows the same rules as positive values)
    ///
    /// # Example
    ///
    /// ```rust
    /// use bevy::math::Vec2;
    /// use leafwing_input_manager::prelude::DualAxisSettings;
    ///
    /// // Create a new DualAxisSettings with scale 0.5 for processed input on the y-axis
    /// let settings = DualAxisSettings::EMPTY.with_raw_scale_y(0.5);
    ///
    /// // Increase the factor on the y-axis
    /// let more_sensitive = settings.with_raw_scale_y(2.0);
    ///
    /// // Further increase the factor on the y-axis
    /// let ultra_sensitive = more_sensitive.with_raw_scale_y(100.0);
    /// ```
    #[must_use]
    pub fn with_processed_scale_y(&self, scale: f32) -> Self {
        Self {
            raw_scales: self.raw_scales,
            raw_limits: self.raw_limits,
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            processed_scales: Vec2::new(self.processed_scales.x, scale),
            processed_limits: self.processed_limits,
        }
    }

    /// Creates a new [`DualAxisSettings`] instance with the provided `limit` for processed input values on the x-axis.
    ///
    /// # Arguments
    ///
    /// - `limit`: The limit for processed input values on the x-axis.
    ///
    /// See [`ValueLimit`] documentation for available limit options.
    ///
    /// # Example
    ///
    /// ```rust
    /// use leafwing_input_manager::prelude::{DualAxisSettings, ValueLimit};
    ///
    /// // Create a new DualAxisSettings with a non-negative limit on the x-axis
    /// let non_negative = DualAxisSettings::EMPTY.with_processed_limit_x(ValueLimit::AtLeast(0.0));
    /// ```
    #[must_use]
    pub fn with_processed_limit_x(&self, clamp: ValueLimit) -> Self {
        Self {
            raw_scales: self.raw_scales,
            raw_limits: self.raw_limits,
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            processed_scales: self.processed_scales,
            processed_limits: [clamp, self.processed_limits[1]],
        }
    }

    /// Creates a new [`DualAxisSettings`] instance with the provided `limit` for processed input values on the y-axis.
    ///
    /// # Arguments
    ///
    /// - `limit`: The limit for processed input values on the y-axis.
    ///
    /// See [`ValueLimit`] documentation for available limit options.
    ///
    /// # Example
    ///
    /// ```rust
    /// use leafwing_input_manager::prelude::{DualAxisSettings, ValueLimit};
    ///
    /// // Create a new DualAxisSettings with a non-negative limit on the x-axis
    /// let non_negative = DualAxisSettings::EMPTY.with_processed_limit_x(ValueLimit::AtLeast(0.0));
    /// ```
    #[must_use]
    pub fn with_processed_limit_y(&self, clamp: ValueLimit) -> Self {
        Self {
            raw_scales: self.raw_scales,
            raw_limits: self.raw_limits,
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            processed_scales: self.processed_scales,
            processed_limits: [self.processed_limits[0], clamp],
        }
    }

    /// Computes and returns the final adjusted input value after applying the currently configured settings.
    ///
    /// # Arguments
    ///
    /// - `input_value`: The raw input value received from the input device.
    ///
    /// # Additional Notes
    ///
    /// This function doesn't modify the [`DualAxisSettings`] instance itself.
    /// It only calculates the processed value based on the provided settings and raw input value.
    ///
    /// By default, not all settings such as limiting, normalization, or deadzone are applied.
    /// They must be explicitly configured in the [`DualAxisSettings`] instance.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bevy::math::Vec2;
    /// use leafwing_input_manager::prelude::DualAxisSettings;
    ///
    /// let settings = DualAxisSettings::DEFAULT_CIRCLE_DEADZONE;
    ///
    /// let processed_value = settings.value(Vec2::splat(0.5));
    ///
    /// // Use the processed_value as you wish...
    /// ```
    #[must_use]
    #[inline]
    pub fn value(&self, input_value: Vec2) -> Vec2 {
        let Vec2 { x, y } = input_value;
        let (x, y) = (x * self.raw_scales.x, y * self.raw_scales.y);
        let (x, y) = (self.raw_limits[0].clamp(x), self.raw_limits[1].clamp(y));
        let (x, y) = (
            self.normalizers[0].normalize(x),
            self.normalizers[1].normalize(y),
        );
        let Vec2 { x, y } = self.deadzone.value(Vec2::new(x, y));
        let (x, y) = (x * self.processed_scales.x, y * self.processed_scales.y);
        Vec2::new(
            self.processed_limits[0].clamp(x),
            self.processed_limits[1].clamp(y),
        )
    }
}

impl Eq for DualAxisSettings {}

impl Hash for DualAxisSettings {
    fn hash<H: Hasher>(&self, state: &mut H) {
        FloatOrd(self.raw_scales.x).hash(state);
        FloatOrd(self.raw_scales.y).hash(state);
        self.raw_limits[0].hash(state);
        self.raw_limits[1].hash(state);
        self.normalizers[0].hash(state);
        self.normalizers[1].hash(state);
        self.deadzone.hash(state);
        FloatOrd(self.processed_scales.x).hash(state);
        FloatOrd(self.processed_scales.y).hash(state);
        self.processed_limits[0].hash(state);
        self.processed_limits[1].hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::{vec2, Vec2};

    // region consts -------------

    #[test]
    fn test_dual_axis_settings_empty() {
        let settings = DualAxisSettings::EMPTY;

        // No output clamp
        assert_eq!(settings.value(Vec2::splat(5.0)), Vec2::splat(5.0));
        assert_eq!(settings.value(Vec2::splat(-5.0)), Vec2::splat(-5.0));

        // No inversion
        assert_eq!(settings.value(Vec2::ONE), Vec2::ONE);
        assert_eq!(settings.value(Vec2::ZERO), Vec2::ZERO);
        assert_eq!(settings.value(Vec2::NEG_ONE), Vec2::NEG_ONE);

        // No deadzone
        assert_eq!(Vec2::splat(0.1), settings.value(Vec2::splat(0.1)));
        assert_eq!(Vec2::splat(0.01), settings.value(Vec2::splat(0.01)));
        assert_eq!(Vec2::splat(-0.01), settings.value(Vec2::splat(-0.01)));
        assert_eq!(Vec2::splat(-0.1), settings.value(Vec2::splat(-0.1)));

        // No livezone normalization
        assert_eq!(settings.value(Vec2::splat(0.75)), Vec2::splat(0.75));
        assert_eq!(settings.value(Vec2::splat(0.5)), Vec2::splat(0.5));
        assert_eq!(settings.value(Vec2::splat(0.11)), Vec2::splat(0.11));
        assert_eq!(settings.value(Vec2::splat(-0.11)), Vec2::splat(-0.11));
        assert_eq!(settings.value(Vec2::splat(-0.5)), Vec2::splat(-0.5));
        assert_eq!(settings.value(Vec2::splat(-0.75)), Vec2::splat(-0.75));
    }

    #[test]
    fn test_dual_axis_settings_default_circle_deadzone() {
        let settings = DualAxisSettings::DEFAULT_CIRCLE_DEADZONE;

        // Output clamp
        assert_eq!(Vec2::ONE, settings.value(Vec2::splat(5.0)));
        assert_eq!(Vec2::NEG_ONE, settings.value(Vec2::splat(-5.0)));

        // No inversion
        assert_eq!(Vec2::ONE, settings.value(Vec2::ONE));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::ZERO));
        assert_eq!(Vec2::NEG_ONE, settings.value(Vec2::NEG_ONE));

        // Deadzone
        assert_eq!(Vec2::ZERO, settings.value(vec2(0.1, 0.0)));
        assert_eq!(Vec2::ZERO, settings.value(vec2(0.0, 0.1)));
        assert_eq!(Vec2::ZERO, settings.value(vec2(-0.1, 0.0)));
        assert_eq!(Vec2::ZERO, settings.value(vec2(0.0, -0.1)));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::splat(0.05)));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::splat(-0.05)));

        // Livezone normalization
        let livezone_0_75 = Vec2::splat(0.73097724);
        assert_eq!(livezone_0_75, settings.value(Vec2::splat(0.75)));
        assert_eq!(-livezone_0_75, settings.value(Vec2::splat(-0.75)));

        let livezone_0_5 = Vec2::splat(0.4619544);
        assert_eq!(livezone_0_5, settings.value(Vec2::splat(0.5)));
        assert_eq!(-livezone_0_5, settings.value(Vec2::splat(-0.5)));

        let livezone_0_25 = Vec2::splat(0.19293164);
        assert_eq!(livezone_0_25, settings.value(Vec2::splat(0.25)));
        assert_eq!(-livezone_0_25, settings.value(Vec2::splat(-0.25)));

        let livezone_0_1 = Vec2::splat(0.03151798);
        assert_eq!(livezone_0_1, settings.value(Vec2::splat(0.1)));
        assert_eq!(-livezone_0_1, settings.value(Vec2::splat(-0.1)));
    }

    #[test]
    fn test_dual_axis_settings_default_square_deadzone() {
        let settings = DualAxisSettings::DEFAULT_SQUARE_DEADZONE;

        // Output clamp
        assert_eq!(Vec2::ONE, settings.value(Vec2::splat(5.0)));
        assert_eq!(Vec2::NEG_ONE, settings.value(Vec2::splat(-5.0)));

        // No inversion
        assert_eq!(Vec2::ONE, settings.value(Vec2::ONE));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::ZERO));
        assert_eq!(Vec2::NEG_ONE, settings.value(Vec2::NEG_ONE));

        // Deadzone
        assert_eq!(Vec2::ZERO, settings.value(Vec2::splat(0.1)));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::splat(-0.1)));
        assert_eq!(Vec2::ZERO, settings.value(vec2(0.1, 0.0)));
        assert_eq!(Vec2::ZERO, settings.value(vec2(0.0, 0.1)));
        assert_eq!(Vec2::ZERO, settings.value(vec2(-0.1, 0.0)));
        assert_eq!(Vec2::ZERO, settings.value(vec2(0.0, -0.1)));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::splat(0.05)));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::splat(-0.05)));

        // Livezone normalization
        let livezone_0_75 = Vec2::splat(0.7222222);
        assert_eq!(livezone_0_75, settings.value(Vec2::splat(0.75)));
        assert_eq!(-livezone_0_75, settings.value(Vec2::splat(-0.75)));

        let livezone_0_5 = Vec2::splat(0.44444448);
        assert_eq!(livezone_0_5, settings.value(Vec2::splat(0.5)));
        assert_eq!(-livezone_0_5, settings.value(Vec2::splat(-0.5)));

        let livezone_0_25 = Vec2::splat(0.16666669);
        assert_eq!(livezone_0_25, settings.value(Vec2::splat(0.25)));
        assert_eq!(-livezone_0_25, settings.value(Vec2::splat(-0.25)));

        let livezone_0_25 = Vec2::splat(0.027777778);
        assert_eq!(livezone_0_25, settings.value(Vec2::splat(0.125)));
        assert_eq!(-livezone_0_25, settings.value(Vec2::splat(-0.125)));
    }

    #[test]
    fn test_dual_axis_settings_default_rounded_square_deadzone() {
        let settings = DualAxisSettings::DEFAULT_ROUNDED_SQUARE_DEADZONE;

        // Output clamp
        assert_eq!(Vec2::ONE, settings.value(Vec2::splat(5.0)));
        assert_eq!(Vec2::NEG_ONE, settings.value(Vec2::splat(-5.0)));

        // No inversion
        assert_eq!(Vec2::ONE, settings.value(Vec2::ONE));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::ZERO));
        assert_eq!(Vec2::NEG_ONE, settings.value(Vec2::NEG_ONE));

        // Deadzone
        assert_eq!(Vec2::ZERO, settings.value(Vec2::splat(0.1)));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::splat(-0.1)));
        assert_eq!(Vec2::ZERO, settings.value(vec2(0.1, 0.0)));
        assert_eq!(Vec2::ZERO, settings.value(vec2(0.0, 0.1)));
        assert_eq!(Vec2::ZERO, settings.value(vec2(-0.1, 0.0)));
        assert_eq!(Vec2::ZERO, settings.value(vec2(0.0, -0.1)));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::splat(0.05)));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::splat(-0.05)));

        // Livezone normalization
        let livezone_0_75 = Vec2::splat(0.7166568);
        assert_eq!(livezone_0_75, settings.value(Vec2::splat(0.75)));
        assert_eq!(-livezone_0_75, settings.value(Vec2::splat(-0.75)));

        let livezone_0_5 = Vec2::splat(0.43331367);
        assert_eq!(livezone_0_5, settings.value(Vec2::splat(0.5)));
        assert_eq!(-livezone_0_5, settings.value(Vec2::splat(-0.5)));

        let livezone_0_25 = Vec2::splat(0.14997052);
        assert_eq!(livezone_0_25, settings.value(Vec2::splat(0.25)));
        assert_eq!(-livezone_0_25, settings.value(Vec2::splat(-0.25)));

        let livezone_0_25 = Vec2::splat(0.008298924);
        assert_eq!(livezone_0_25, settings.value(Vec2::splat(0.125)));
        assert_eq!(-livezone_0_25, settings.value(Vec2::splat(-0.125)));
    }

    // endregion consts -------------

    // region raw input scaling -------------

    #[test]
    fn test_dual_axis_raw_scale() {
        let original = DualAxisSettings::EMPTY.with_raw_scales(Vec2::ONE);
        let ratio = Vec2::splat(0.5);
        let changed = original.with_raw_scales(ratio);

        let expected_value = |value: Vec2| ratio * original.value(value);

        assert_eq!(expected_value(Vec2::ONE), changed.value(Vec2::ONE));
        assert_eq!(expected_value(Vec2::ZERO), changed.value(Vec2::ZERO));
        assert_eq!(expected_value(Vec2::NEG_ONE), changed.value(Vec2::NEG_ONE));
    }

    #[test]
    fn test_dual_axis_raw_scale_x() {
        let original = DualAxisSettings::EMPTY.with_raw_scales(Vec2::ONE);
        let ratio = vec2(0.5, 1.0);
        let changed = original.with_raw_scale_x(ratio.x);

        let expected_value = |value: Vec2| ratio * original.value(value);

        assert_eq!(expected_value(Vec2::ONE), changed.value(Vec2::ONE));
        assert_eq!(expected_value(Vec2::ZERO), changed.value(Vec2::ZERO));
        assert_eq!(expected_value(Vec2::NEG_ONE), changed.value(Vec2::NEG_ONE));
    }

    #[test]
    fn test_dual_axis_raw_scale_y() {
        let original = DualAxisSettings::EMPTY.with_raw_scales(Vec2::ONE);
        let ratio = vec2(1.0, 0.5);
        let changed = original.with_raw_scale_y(ratio.y);

        let expected_value = |value: Vec2| ratio * original.value(value);

        assert_eq!(expected_value(Vec2::ONE), changed.value(Vec2::ONE));
        assert_eq!(expected_value(Vec2::ZERO), changed.value(Vec2::ZERO));
        assert_eq!(expected_value(Vec2::NEG_ONE), changed.value(Vec2::NEG_ONE));
    }

    #[test]
    fn test_dual_axis_negative_raw_scales() {
        let original = DualAxisSettings::EMPTY.with_raw_scales(Vec2::ONE);
        let ratio = Vec2::splat(-0.5);
        let changed = original.with_raw_scales(ratio);

        let expected_value = |value: Vec2| ratio * original.value(value);

        assert_eq!(expected_value(Vec2::ONE), changed.value(Vec2::ONE));
        assert_eq!(expected_value(Vec2::ZERO), changed.value(Vec2::ZERO));
        assert_eq!(expected_value(Vec2::NEG_ONE), changed.value(Vec2::NEG_ONE));
    }

    #[test]
    fn test_dual_axis_negative_raw_scale_x() {
        let original = DualAxisSettings::EMPTY.with_raw_scales(Vec2::ONE);
        let ratio = vec2(-0.5, 1.0);
        let changed = DualAxisSettings::EMPTY.with_raw_scales(ratio);

        let expected_value = |value: Vec2| ratio * original.value(value);

        assert_eq!(expected_value(Vec2::ONE), changed.value(Vec2::ONE));
        assert_eq!(expected_value(Vec2::ZERO), changed.value(Vec2::ZERO));
        assert_eq!(expected_value(Vec2::NEG_ONE), changed.value(Vec2::NEG_ONE));
    }

    #[test]
    fn test_dual_axis_negative_raw_scale_y() {
        let original = DualAxisSettings::EMPTY.with_raw_scales(Vec2::ONE);
        let ratio = vec2(1.0, -0.5);
        let changed = original.with_raw_scales(ratio);

        let expected_value = |value: Vec2| ratio * original.value(value);

        assert_eq!(expected_value(Vec2::ONE), changed.value(Vec2::ONE));
        assert_eq!(expected_value(Vec2::ZERO), changed.value(Vec2::ZERO));
        assert_eq!(expected_value(Vec2::NEG_ONE), changed.value(Vec2::NEG_ONE));
    }

    #[test]
    fn test_dual_axis_zero_raw_scale() {
        let settings = DualAxisSettings::EMPTY.with_raw_scales(Vec2::ZERO);

        assert_eq!(Vec2::ZERO, settings.value(Vec2::ONE));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::ZERO));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::NEG_ONE));
    }

    #[test]
    fn test_dual_axis_zero_raw_scale_x() {
        let settings = DualAxisSettings::EMPTY.with_raw_scales(Vec2::Y);

        assert_eq!(Vec2::Y, settings.value(Vec2::ONE));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::ZERO));
        assert_eq!(Vec2::NEG_Y, settings.value(Vec2::NEG_ONE));
    }

    #[test]
    fn test_dual_axis_zero_raw_scale_y() {
        let settings = DualAxisSettings::EMPTY.with_raw_scales(Vec2::X);

        assert_eq!(Vec2::X, settings.value(Vec2::ONE));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::ZERO));
        assert_eq!(Vec2::NEG_X, settings.value(Vec2::NEG_ONE));
    }

    // endregion raw input scaling -------------

    // region inversion -------------

    #[test]
    fn test_dual_axis_inversion() {
        let original = DualAxisSettings::EMPTY;

        assert_eq!(Vec2::ONE, original.value(Vec2::ONE));
        assert_eq!(Vec2::splat(0.5), original.value(Vec2::splat(0.5)));
        assert_eq!(Vec2::ZERO, original.value(Vec2::ZERO));
        assert_eq!(Vec2::splat(-0.5), original.value(Vec2::splat(-0.5)));
        assert_eq!(Vec2::NEG_ONE, original.value(Vec2::NEG_ONE));

        let inverted = original.with_inverted();

        assert_eq!(Vec2::ONE, inverted.value(Vec2::NEG_ONE));
        assert_eq!(Vec2::splat(-0.5), inverted.value(Vec2::splat(0.5)));
        assert_eq!(Vec2::ZERO, inverted.value(Vec2::ZERO));
        assert_eq!(Vec2::splat(0.5), inverted.value(Vec2::splat(-0.5)));
        assert_eq!(Vec2::NEG_ONE, inverted.value(Vec2::ONE));

        let reverted = inverted.with_inverted();

        assert_eq!(Vec2::ONE, reverted.value(Vec2::ONE));
        assert_eq!(Vec2::splat(0.5), reverted.value(Vec2::splat(0.5)));
        assert_eq!(Vec2::ZERO, reverted.value(Vec2::ZERO));
        assert_eq!(Vec2::splat(-0.5), reverted.value(Vec2::splat(-0.5)));
        assert_eq!(Vec2::NEG_ONE, reverted.value(Vec2::NEG_ONE));
    }

    #[test]
    fn test_dual_axis_inversion_x() {
        let original = DualAxisSettings::EMPTY;

        assert_eq!(Vec2::ONE, original.value(Vec2::ONE));
        assert_eq!(Vec2::splat(0.5), original.value(Vec2::splat(0.5)));
        assert_eq!(Vec2::ZERO, original.value(Vec2::ZERO));
        assert_eq!(Vec2::splat(-0.5), original.value(Vec2::splat(-0.5)));
        assert_eq!(Vec2::NEG_ONE, original.value(Vec2::NEG_ONE));

        let inverted_x = original.with_inverted_x();

        assert_eq!(vec2(-1.0, 1.0), inverted_x.value(Vec2::ONE));
        assert_eq!(vec2(-0.5, 0.5), inverted_x.value(Vec2::splat(0.5)));
        assert_eq!(Vec2::ZERO, inverted_x.value(Vec2::ZERO));
        assert_eq!(vec2(0.5, -0.5), inverted_x.value(Vec2::splat(-0.5)));
        assert_eq!(vec2(1.0, -1.0), inverted_x.value(Vec2::NEG_ONE));

        let reverted = inverted_x.with_inverted_x();

        assert_eq!(Vec2::ONE, reverted.value(Vec2::ONE));
        assert_eq!(Vec2::splat(0.5), reverted.value(Vec2::splat(0.5)));
        assert_eq!(Vec2::ZERO, reverted.value(Vec2::ZERO));
        assert_eq!(Vec2::splat(-0.5), reverted.value(Vec2::splat(-0.5)));
        assert_eq!(Vec2::NEG_ONE, reverted.value(Vec2::NEG_ONE));
    }

    #[test]
    fn test_dual_axis_inversion_y() {
        let original = DualAxisSettings::EMPTY;

        assert_eq!(Vec2::ONE, original.value(Vec2::ONE));
        assert_eq!(Vec2::splat(0.5), original.value(Vec2::splat(0.5)));
        assert_eq!(Vec2::ZERO, original.value(Vec2::ZERO));
        assert_eq!(Vec2::splat(-0.5), original.value(Vec2::splat(-0.5)));
        assert_eq!(Vec2::NEG_ONE, original.value(Vec2::NEG_ONE));

        let inverted_y = original.with_inverted_y();

        assert_eq!(vec2(1.0, -1.0), inverted_y.value(Vec2::ONE));
        assert_eq!(vec2(0.5, -0.5), inverted_y.value(Vec2::splat(0.5)));
        assert_eq!(Vec2::ZERO, inverted_y.value(Vec2::ZERO));
        assert_eq!(vec2(-0.5, 0.5), inverted_y.value(Vec2::splat(-0.5)));
        assert_eq!(vec2(-1.0, 1.0), inverted_y.value(Vec2::NEG_ONE));

        let reverted = inverted_y.with_inverted_y();

        assert_eq!(Vec2::ONE, reverted.value(Vec2::ONE));
        assert_eq!(Vec2::splat(0.5), reverted.value(Vec2::splat(0.5)));
        assert_eq!(Vec2::ZERO, reverted.value(Vec2::ZERO));
        assert_eq!(Vec2::splat(-0.5), reverted.value(Vec2::splat(-0.5)));
        assert_eq!(Vec2::NEG_ONE, reverted.value(Vec2::NEG_ONE));
    }

    // endregion inversion -------------

    // region raw input limiting -------------

    #[test]
    fn test_dual_axis_raw_limit_x() {
        let settings = DualAxisSettings::EMPTY.with_raw_limit_x(ValueLimit::AtLeast(0.5));

        assert_eq!(Vec2::ONE, settings.value(Vec2::ONE));
        assert_eq!(Vec2::splat(0.5), settings.value(Vec2::splat(0.5)));
        assert_eq!(vec2(0.5, 0.0), settings.value(Vec2::ZERO));
        assert_eq!(vec2(0.5, -0.5), settings.value(Vec2::splat(-0.5)));
        assert_eq!(vec2(0.5, -1.0), settings.value(Vec2::NEG_ONE));
    }

    #[test]
    fn test_dual_axis_raw_limit_y() {
        let settings = DualAxisSettings::EMPTY.with_raw_limit_y(ValueLimit::AtLeast(0.5));

        assert_eq!(Vec2::ONE, settings.value(Vec2::ONE));
        assert_eq!(Vec2::splat(0.5), settings.value(Vec2::splat(0.5)));
        assert_eq!(vec2(0.0, 0.5), settings.value(Vec2::ZERO));
        assert_eq!(vec2(-0.5, 0.5), settings.value(Vec2::splat(-0.5)));
        assert_eq!(vec2(-1.0, 0.5), settings.value(Vec2::NEG_ONE));
    }

    // endregion raw input limiting -------------

    // region normalization -------------

    #[test]
    fn test_dual_axis_normalization_x() {
        let normalizer = ValueNormalizer::standard_min_max(0.0..1.0);
        let settings = DualAxisSettings::EMPTY.with_normalizer_x(normalizer);

        assert_eq!(Vec2::ONE, settings.value(Vec2::ONE));
        assert_eq!(Vec2::splat(0.5), settings.value(Vec2::splat(0.5)));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::ZERO));
        assert_eq!(vec2(0.0, -0.5), settings.value(Vec2::splat(-0.5)));
        assert_eq!(vec2(0.0, -1.0), settings.value(Vec2::NEG_ONE));
    }

    #[test]
    fn test_dual_axis_normalization_y() {
        let normalizer = ValueNormalizer::standard_min_max(0.0..1.0);
        let settings = DualAxisSettings::EMPTY.with_normalizer_y(normalizer);

        assert_eq!(Vec2::ONE, settings.value(Vec2::ONE));
        assert_eq!(Vec2::splat(0.5), settings.value(Vec2::splat(0.5)));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::ZERO));
        assert_eq!(vec2(-0.5, 0.0), settings.value(Vec2::splat(-0.5)));
        assert_eq!(vec2(-1.0, 0.0), settings.value(Vec2::NEG_ONE));
    }

    // endregion normalization -------------

    // region deadzone -------------

    #[test]
    fn test_dual_axis_deadzone_circle() {
        let deadzone = Deadzone2::circle(0.3, 0.35);
        let settings = DualAxisSettings::EMPTY.with_deadzone(deadzone);

        // Deadzone
        assert_eq!(Vec2::ZERO, settings.value(Vec2::splat(0.2)));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::splat(0.1)));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::ZERO));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::splat(-0.1)));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::splat(-0.2)));

        // Livezone normalization
        assert_eq!(Vec2::ONE, settings.value(Vec2::ONE));
        assert_eq!(Vec2::NEG_ONE, settings.value(Vec2::NEG_ONE));

        let livezone_0_8 = vec2(0.7461504, 0.7342237);
        assert_eq!(livezone_0_8, settings.value(Vec2::splat(0.8)));
        assert_eq!(-livezone_0_8, settings.value(Vec2::splat(-0.8)));

        let livezone_0_6 = vec2(0.49230075, 0.4684475);
        assert_eq!(livezone_0_6, settings.value(Vec2::splat(0.6)));
        assert_eq!(-livezone_0_6, settings.value(Vec2::splat(-0.6)));

        let livezone_0_4 = vec2(0.23845108, 0.2026712);
        assert_eq!(livezone_0_4, settings.value(Vec2::splat(0.4)));
        assert_eq!(-livezone_0_4, settings.value(Vec2::splat(-0.4)));

        let livezone_0_3 = vec2(0.11152627, 0.06978308);
        assert_eq!(livezone_0_3, settings.value(Vec2::splat(0.3)));
        assert_eq!(-livezone_0_3, settings.value(Vec2::splat(-0.3)));
    }

    // endregion deadzone -------------

    // region processed input scaling -------------

    #[test]
    fn test_dual_axis_processed_scales() {
        let settings = DualAxisSettings::EMPTY.with_processed_scales(Vec2::splat(-5.0));

        assert_eq!(Vec2::splat(-5.0), settings.value(Vec2::ONE));
        assert_eq!(Vec2::splat(-2.5), settings.value(Vec2::splat(0.5)));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::ZERO));
        assert_eq!(Vec2::splat(2.5), settings.value(Vec2::splat(-0.5)));
        assert_eq!(Vec2::splat(5.0), settings.value(Vec2::NEG_ONE));
    }

    #[test]
    fn test_dual_axis_processed_scale_x() {
        let settings = DualAxisSettings::EMPTY.with_processed_scale_x(-5.0);

        assert_eq!(vec2(-5.0, 1.0), settings.value(Vec2::ONE));
        assert_eq!(vec2(-2.5, 0.5), settings.value(Vec2::splat(0.5)));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::ZERO));
        assert_eq!(vec2(2.5, -0.5), settings.value(Vec2::splat(-0.5)));
        assert_eq!(vec2(5.0, -1.0), settings.value(Vec2::NEG_ONE));
    }

    #[test]
    fn test_dual_axis_processed_scale_y() {
        let settings = DualAxisSettings::EMPTY.with_processed_scale_y(-5.0);

        assert_eq!(vec2(1.0, -5.0), settings.value(Vec2::ONE));
        assert_eq!(vec2(0.5, -2.5), settings.value(Vec2::splat(0.5)));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::ZERO));
        assert_eq!(vec2(-0.5, 2.5), settings.value(Vec2::splat(-0.5)));
        assert_eq!(vec2(-1.0, 5.0), settings.value(Vec2::NEG_ONE));
    }

    // endregion processed input scaling -------------

    // region processed input limiting -------------

    #[test]
    fn test_dual_axis_processed_limit_x() {
        let settings = DualAxisSettings::EMPTY
            .with_processed_scale_x(5.0)
            .with_processed_limit_x(ValueLimit::AtLeast(0.5));

        assert_eq!(vec2(5.0, 1.0), settings.value(Vec2::ONE));
        assert_eq!(vec2(2.5, 0.5), settings.value(Vec2::splat(0.5)));
        assert_eq!(vec2(0.5, 0.0), settings.value(Vec2::ZERO));
        assert_eq!(vec2(0.5, -0.5), settings.value(Vec2::splat(-0.5)));
        assert_eq!(vec2(0.5, -1.0), settings.value(Vec2::NEG_ONE));
    }

    #[test]
    fn test_dual_axis_processed_limit_y() {
        let settings = DualAxisSettings::EMPTY
            .with_processed_scale_y(5.0)
            .with_processed_limit_y(ValueLimit::AtLeast(0.5));

        assert_eq!(vec2(1.0, 5.0), settings.value(Vec2::ONE));
        assert_eq!(vec2(0.5, 2.5), settings.value(Vec2::splat(0.5)));
        assert_eq!(vec2(0.0, 0.5), settings.value(Vec2::ZERO));
        assert_eq!(vec2(-0.5, 0.5), settings.value(Vec2::splat(-0.5)));
        assert_eq!(vec2(-1.0, 0.5), settings.value(Vec2::NEG_ONE));
    }

    // endregion processed input limiting -------------
}
