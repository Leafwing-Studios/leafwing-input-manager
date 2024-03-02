//! [`SingleAxisSettings`] and its processing pipeline.
//!
//! Single-axis inputs typically represent input values along a single dimension,
//! such as horizontal movement on a joystick or slider.
//!
//! This module provides the [`SingleAxisSettings`] struct,
//! which defines the processing pipeline and configuration for single-axis inputs.

use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use bevy::prelude::Reflect;
use bevy::utils::FloatOrd;
use serde::{Deserialize, Serialize};

use super::common_processors::*;
use super::deadzones::*;

/// Defines the processing pipeline and configuration for single-axis inputs.
///
/// Single-axis inputs are processed through a series of steps to achieve desired behavior.
/// This structure defines these steps and their configuration.
///
/// # Processing Steps
///
/// The processing pipeline transforms raw input values into usable output values
/// through a series of configurable steps:
///
/// **Note**: Limiting and Normalization aren't used by default.
///
/// 1. **Raw Input Scaling**: Adjusts raw input values based on a specified scale factor.
///    - Positive values scale input:
///      - `0.0`: Disregards input changes (effectively disables the input)
///      - `(0.0, 1.0)`: Reduces input influence
///      - `1.0`: No adjustment (default)
///      - `(1.0, infinity)`: Amplifies input influence
///    - Negative values invert and scale input (magnitude follows the same rules as positive values)
/// 2. **Raw Input Limiting (Optional)**: Clamps raw input values to a specified range.
/// 3. **Normalization (Optional)**: Maps input values into a specified range.
/// 4. **Deadzone (Optional)**: Defines regions where input values are considered neutral.
/// 5. **Processed Input Scaling**: Adjusts processed input values based on a specified scale factor.
///    It follows the same rules as raw input scaling.
/// 6. **Processed Input Limiting (Optional)**: Clamps processed input values to a specified range.
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub struct SingleAxisSettings {
    /// Scales raw input values.
    ///
    /// Using a single `f32` here for both sensitivity and inversion
    /// avoids separate fields and branching logic.
    ///
    /// Combines sensitivity (absolute value) and inversion (sign):
    /// - Positive values scale the input:
    ///   - `0.0`: Disregards input changes (effectively disables the input)
    ///   - `(0.0, 1.0)`: Reduces input influence
    ///   - `1.0`: No adjustment (default)
    ///   - `(1.0, infinity)`: Amplifies input influence
    /// - Negative values invert and scale the input (magnitude follows the same rules as positive values)
    raw_scale: f32,

    /// (Optional) Defines the range for clamping raw input values.
    raw_limit: ValueLimit,

    /// (Optional) Maps input values into a specified range.
    normalizer: ValueNormalizer,

    /// (Optional) Defines regions where input values are considered neutral.
    deadzone: Deadzone1,

    /// Scales processed input values.
    ///
    /// Using a single `f32` here for both sensitivity and inversion
    /// avoids separate fields and branching logic.
    ///
    /// Combines sensitivity (absolute value) and inversion (sign):
    /// - Positive values scale the input:
    ///   - `0.0`: Disregards input changes (effectively disables the input)
    ///   - `(0.0, 1.0)`: Reduces input influence
    ///   - `1.0`: No adjustment (default)
    ///   - `(1.0, infinity)`: Amplifies input influence
    /// - Negative values invert and scale the input (magnitude follows the same rules as positive values)
    processed_scale: f32,

    /// (Optional) Defines the range for clamping processed input values.
    processed_limit: ValueLimit,
}

impl SingleAxisSettings {
    /// - Scaling: `1.0` (default)
    /// - Deadzone: No deadzone is applied
    pub const EMPTY: Self = Self {
        raw_scale: 1.0,
        raw_limit: ValueLimit::None,
        normalizer: ValueNormalizer::None,
        deadzone: Deadzone1::None,
        processed_scale: 1.0,
        processed_limit: ValueLimit::None,
    };

    /// - Scaling: `1.0` (default)
    /// - Deadzone: Excludes input values within the range `[-0.1, 0.1]`
    /// - Output: Values within the livezone are normalized into the range `[-1.0, 1.0]`
    pub const DEFAULT: Self = Self::EMPTY.with_deadzone(Deadzone1::DEFAULT_SYMMETRIC);

    /// Creates a new [`SingleAxisSettings`] instance with the provided `scale` for raw input values.
    ///
    /// # Arguments
    ///
    /// - `scale`: The scale factor to adjust raw input values:
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
    /// use leafwing_input_manager::prelude::SingleAxisSettings;
    ///
    /// // Create a new SingleAxisSettings with scale 0.5 for raw input
    /// let base = SingleAxisSettings::EMPTY.with_raw_scale(0.5);
    ///
    /// // Increase the factor
    /// let more_sensitive = base.with_raw_scale(2.0);
    ///
    /// // Further increase the factor
    /// let ultra_sensitive = more_sensitive.with_raw_scale(100.0);
    /// ```
    #[must_use]
    pub fn with_raw_scale(&self, scale: f32) -> Self {
        Self {
            raw_scale: scale,
            raw_limit: self.raw_limit,
            normalizer: self.normalizer,
            deadzone: self.deadzone,
            processed_scale: self.processed_scale,
            processed_limit: self.processed_limit,
        }
    }

    /// Creates a new [`SingleAxisSettings`] instance with inversion applied to raw input values.
    ///
    /// # Example
    ///
    /// ```rust
    /// use leafwing_input_manager::prelude::SingleAxisSettings;
    ///
    /// let original = SingleAxisSettings::EMPTY;
    ///
    /// // Invert the input direction
    /// let inverted = original.with_inverted();
    ///
    /// // Revert the settings to the original
    /// let reverted = inverted.with_inverted();
    /// ```
    #[must_use]
    pub fn with_inverted(&self) -> Self {
        Self {
            raw_scale: -self.raw_scale,
            raw_limit: self.raw_limit,
            normalizer: self.normalizer,
            deadzone: self.deadzone,
            processed_scale: self.processed_scale,
            processed_limit: self.processed_limit,
        }
    }

    /// Creates a new [`SingleAxisSettings`] instance with the provided `limit` for raw input values.
    ///
    /// # Arguments
    ///
    /// - `limit`: The limit for raw input values.
    ///
    /// See [`ValueLimit`] documentation for available limit options.
    ///
    /// # Example
    ///
    /// ```rust
    /// use leafwing_input_manager::prelude::{SingleAxisSettings, ValueLimit};
    ///
    /// // Create a new SingleAxisSettings with a non-negative limit
    /// let non_negative = SingleAxisSettings::EMPTY.with_raw_limit(ValueLimit::AtLeast(0.0));
    /// ```
    #[must_use]
    pub const fn with_raw_limit(&self, limit: ValueLimit) -> Self {
        Self {
            raw_scale: self.raw_scale,
            raw_limit: limit,
            normalizer: self.normalizer,
            deadzone: self.deadzone,
            processed_scale: self.processed_scale,
            processed_limit: self.processed_limit,
        }
    }

    /// Creates a new [`SingleAxisSettings`] instance with the provided `normalizer`.
    ///
    /// # Arguments
    ///
    /// - `normalizer`: The normalizer that maps input values into a specified range.
    ///
    /// See [`ValueNormalizer`] documentation for available normalization options.
    ///
    /// # Example
    ///
    /// ```rust
    /// use leafwing_input_manager::prelude::{SingleAxisSettings, ValueNormalizer};
    ///
    /// // Create a new SingleAxisSettings with a standard min-max normalizer
    ///let standard_min_max_settings = SingleAxisSettings::EMPTY
    ///     .with_normalizer(ValueNormalizer::standard_min_max(-100.0..100.0));
    /// ```
    #[must_use]
    pub const fn with_normalizer(&self, normalizer: ValueNormalizer) -> Self {
        Self {
            raw_scale: self.raw_scale,
            raw_limit: self.raw_limit,
            normalizer,
            deadzone: self.deadzone,
            processed_scale: self.processed_scale,
            processed_limit: self.processed_limit,
        }
    }

    /// Creates a new [`SingleAxisSettings`] instance with the provided `deadzone`.
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
    /// use leafwing_input_manager::prelude::{Deadzone1, SingleAxisSettings};
    ///
    /// // Create a new SingleAxisSettings with a deadzone that has symmetric thresholds set to 0.5
    ///let settings = SingleAxisSettings::EMPTY.with_deadzone(Deadzone1::symmetric(0.5));
    /// ```
    #[must_use]
    pub const fn with_deadzone(&self, deadzone: Deadzone1) -> Self {
        Self {
            raw_scale: self.raw_scale,
            raw_limit: self.raw_limit,
            normalizer: self.normalizer,
            deadzone,
            processed_scale: self.processed_scale,
            processed_limit: self.processed_limit,
        }
    }

    /// Creates a new [`SingleAxisSettings`] instance with the provided `scale` for processed input values.
    ///
    /// # Arguments
    ///
    /// - `scale`: The scale factor to adjust processed input values:
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
    /// use leafwing_input_manager::prelude::SingleAxisSettings;
    ///
    /// // Create a new SingleAxisSettings with scale 0.5 for raw input
    /// let settings = SingleAxisSettings::EMPTY.with_processed_scale(0.5);
    ///
    /// // Increase the scale factor of the existing settings
    /// let more_sensitive = settings.with_processed_scale(2.0);
    ///
    /// // Further increase the scale factor
    /// let ultra_sensitive = more_sensitive.with_processed_scale(100.0);
    /// ```
    #[must_use]
    pub const fn with_processed_scale(&self, scale: f32) -> Self {
        Self {
            raw_scale: self.raw_scale,
            raw_limit: self.raw_limit,
            normalizer: self.normalizer,
            deadzone: self.deadzone,
            processed_scale: scale,
            processed_limit: self.processed_limit,
        }
    }

    /// Creates a new [`SingleAxisSettings`] instance with the provided `limit` for processed input values.
    ///
    /// # Arguments
    ///
    /// - `limit`: The limit for processed input values.
    ///
    /// See [`ValueLimit`] documentation for available limit options.
    ///
    /// # Example
    ///
    /// ```rust
    /// use leafwing_input_manager::prelude::{SingleAxisSettings, ValueLimit};
    ///
    /// // Create a new SingleAxisSettings with a non-negative limit
    /// let non_negative = SingleAxisSettings::EMPTY.with_processed_limit(ValueLimit::AtLeast(0.0));
    /// ```
    #[must_use]
    pub const fn with_processed_limit(&self, limit: ValueLimit) -> Self {
        Self {
            raw_scale: self.raw_scale,
            raw_limit: self.raw_limit,
            normalizer: self.normalizer,
            deadzone: self.deadzone,
            processed_scale: self.processed_scale,
            processed_limit: limit,
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
    /// This function doesn't modify the [`SingleAxisSettings`] instance itself.
    /// It only calculates the processed value based on the provided settings and raw input value.
    ///
    /// By default, not all settings such as limiting, normalization, or deadzone are applied.
    /// They must be explicitly configured in the [`SingleAxisSettings`] instance.
    ///
    /// # Example
    ///
    /// ```rust
    /// use leafwing_input_manager::prelude::SingleAxisSettings;
    ///
    /// let settings = SingleAxisSettings::DEFAULT;
    ///
    /// let processed_value = settings.value(0.5);
    ///
    /// // Use the processed_value as you wish...
    /// ```
    #[must_use]
    pub fn value(&self, input_value: f32) -> f32 {
        let value = self.raw_scale * input_value;
        let value = self.raw_limit.clamp(value);
        let value = self.normalizer.normalize(value);
        let value = self.deadzone.value(value);
        let value = self.processed_scale * value;
        self.processed_limit.clamp(value)
    }
}

impl Eq for SingleAxisSettings {}

impl Hash for SingleAxisSettings {
    fn hash<H: Hasher>(&self, state: &mut H) {
        FloatOrd(self.raw_scale).hash(state);
        self.raw_limit.hash(state);
        self.normalizer.hash(state);
        self.deadzone.hash(state);
        FloatOrd(self.processed_scale).hash(state);
        self.processed_limit.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // region consts -------------

    #[test]
    fn test_single_axis_settings_empty() {
        let settings = SingleAxisSettings::EMPTY;

        // No output clamp
        assert_eq!(5.0, settings.value(5.0));
        assert_eq!(-5.0, settings.value(-5.0));

        // No inversion
        assert_eq!(1.0, settings.value(1.0));
        assert_eq!(0.0, settings.value(0.0));
        assert_eq!(-1.0, settings.value(-1.0));

        // No deadzone
        assert_eq!(0.1, settings.value(0.1));
        assert_eq!(0.01, settings.value(0.01));
        assert_eq!(-0.01, settings.value(-0.01));
        assert_eq!(-0.1, settings.value(-0.1));

        // No livezone normalization
        assert_eq!(0.75, settings.value(0.75));
        assert_eq!(0.5, settings.value(0.5));
        assert_eq!(0.25, settings.value(0.25));
        assert_eq!(0.11, settings.value(0.11));
        assert_eq!(-0.11, settings.value(-0.11));
        assert_eq!(-0.25, settings.value(-0.25));
        assert_eq!(-0.5, settings.value(-0.5));
        assert_eq!(-0.75, settings.value(-0.75));
    }

    #[test]
    fn test_single_axis_settings_default() {
        let settings = SingleAxisSettings::DEFAULT;

        // Output clamp
        assert_eq!(1.0, settings.value(5.0));
        assert_eq!(-1.0, settings.value(-5.0));

        // No inversion
        assert_eq!(1.0, settings.value(1.0));
        assert_eq!(0.0, settings.value(0.0));
        assert_eq!(-1.0, settings.value(-1.0));

        // Deadzone
        assert_eq!(0.0, settings.value(0.1));
        assert_eq!(0.0, settings.value(0.01));
        assert_eq!(0.0, settings.value(-0.01));
        assert_eq!(0.0, settings.value(-0.1));

        // Livezone normalization
        let livezone_0_75 = 0.7222222;
        assert_eq!(livezone_0_75, settings.value(0.75));
        assert_eq!(-livezone_0_75, settings.value(-0.75));

        let livezone_0_5 = 0.44444448;
        assert_eq!(livezone_0_5, settings.value(0.5));
        assert_eq!(-livezone_0_5, settings.value(-0.5));

        let livezone_0_25 = 0.16666669;
        assert_eq!(livezone_0_25, settings.value(0.25));
        assert_eq!(-livezone_0_25, settings.value(-0.25));

        let livezone_0_11 = 0.0111111095;
        assert_eq!(livezone_0_11, settings.value(0.11));
        assert_eq!(-livezone_0_11, settings.value(-0.11));
    }

    // endregion consts -------------

    // region raw input scaling -------------

    #[test]
    fn test_single_axis_raw_scale() {
        let original = SingleAxisSettings::EMPTY.with_raw_scale(1.0);
        let ratio = 0.5;
        let changed = original.with_raw_scale(ratio);

        let expected_value = |value: f32| ratio * original.value(value);

        assert_eq!(expected_value(1.0), changed.value(1.0));
        assert_eq!(expected_value(0.0), changed.value(0.0));
        assert_eq!(expected_value(-1.0), changed.value(-1.0));
    }

    #[test]
    fn test_single_axis_negative_raw_scale() {
        let original = SingleAxisSettings::EMPTY.with_raw_scale(1.0);
        let ratio = -0.5;
        let changed = original.with_raw_scale(ratio);

        let expected_value = |value: f32| ratio * original.value(value);

        assert_eq!(expected_value(1.0), changed.value(1.0));
        assert_eq!(expected_value(0.0), changed.value(0.0));
        assert_eq!(expected_value(-1.0), changed.value(-1.0));
    }

    #[test]
    fn test_single_axis_zero_raw_scale() {
        let settings = SingleAxisSettings::EMPTY.with_raw_scale(0.0);

        assert_eq!(0.0, settings.value(1.0));
        assert_eq!(0.0, settings.value(0.0));
        assert_eq!(0.0, settings.value(-1.0));
    }

    // endregion raw input scaling -------------

    // region inversion -------------

    #[test]
    fn test_single_axis_inversion() {
        let original = SingleAxisSettings::EMPTY;

        assert_eq!(1.0, original.value(1.0));
        assert_eq!(0.5, original.value(0.5));
        assert_eq!(0.0, original.value(0.0));
        assert_eq!(-0.5, original.value(-0.5));
        assert_eq!(-1.0, original.value(-1.0));

        let inverted = original.with_inverted();

        assert_eq!(-1.0, inverted.value(1.0));
        assert_eq!(-0.5, inverted.value(0.5));
        assert_eq!(0.0, inverted.value(0.0));
        assert_eq!(0.5, inverted.value(-0.5));
        assert_eq!(1.0, inverted.value(-1.0));

        let reverted = inverted.with_inverted();

        assert_eq!(1.0, reverted.value(1.0));
        assert_eq!(0.5, reverted.value(0.5));
        assert_eq!(0.0, reverted.value(0.0));
        assert_eq!(-0.5, reverted.value(-0.5));
        assert_eq!(-1.0, reverted.value(-1.0));
    }

    // endregion inversion -------------

    // region raw input limiting -------------

    #[test]
    fn test_single_axis_raw_limit() {
        let settings = SingleAxisSettings::EMPTY.with_raw_limit(ValueLimit::AtLeast(0.5));

        assert_eq!(1.0, settings.value(1.0));
        assert_eq!(0.5, settings.value(0.5));
        assert_eq!(0.5, settings.value(0.0));
        assert_eq!(0.5, settings.value(-0.5));
        assert_eq!(0.5, settings.value(-1.0));
    }

    // endregion raw input limiting -------------

    // region normalization -------------

    #[test]
    fn test_single_axis_normalization() {
        let settings =
            SingleAxisSettings::EMPTY.with_normalizer(ValueNormalizer::standard_min_max(0.0..1.0));

        assert_eq!(1.0, settings.value(1.0));
        assert_eq!(0.5, settings.value(0.5));
        assert_eq!(0.0, settings.value(0.0));
        assert_eq!(0.0, settings.value(-0.5));
        assert_eq!(0.0, settings.value(-1.0));
    }

    // endregion normalization -------------

    // region deadzone -------------

    #[test]
    fn test_single_axis_deadzone() {
        let deadzone = Deadzone1::symmetric(0.2);
        let settings = SingleAxisSettings::EMPTY.with_deadzone(deadzone);

        // Deadzone
        assert_eq!(0.0, settings.value(0.2));
        assert_eq!(0.0, settings.value(0.1));
        assert_eq!(0.0, settings.value(0.0));
        assert_eq!(0.0, settings.value(-0.1));
        assert_eq!(0.0, settings.value(-0.2));

        // livezone normalization
        assert_eq!(1.0, settings.value(1.0));
        assert_eq!(-1.0, settings.value(-1.0));

        assert_eq!(0.75, settings.value(0.8));
        assert_eq!(-0.75, settings.value(-0.8));

        let livezone_0_6 = 0.50000006;
        assert_eq!(livezone_0_6, settings.value(0.6));
        assert_eq!(-livezone_0_6, settings.value(-0.6));

        assert_eq!(0.25, settings.value(0.4));
        assert_eq!(-0.25, settings.value(-0.4));

        let livezone_0_3 = 0.12500001;
        assert_eq!(livezone_0_3, settings.value(0.3));
        assert_eq!(-livezone_0_3, settings.value(-0.3));
    }

    // endregion deadzone -------------

    // region processed input scale -------------

    #[test]
    fn test_single_axis_processed_scale() {
        let settings = SingleAxisSettings::EMPTY.with_processed_scale(5.0);

        assert_eq!(5.0, settings.value(1.0));
        assert_eq!(2.5, settings.value(0.5));
        assert_eq!(0.0, settings.value(0.0));
        assert_eq!(-2.5, settings.value(-0.5));
        assert_eq!(-5.0, settings.value(-1.0));
    }

    // endregion processed input scale -------------

    // region processed input limit -------------

    #[test]
    fn test_single_axis_processed_limit() {
        let settings = SingleAxisSettings::EMPTY
            .with_processed_scale(5.0)
            .with_processed_limit(ValueLimit::AtLeast(0.5));

        assert_eq!(5.0, settings.value(1.0));
        assert_eq!(2.5, settings.value(0.5));
        assert_eq!(0.5, settings.value(0.0));
        assert_eq!(0.5, settings.value(-0.5));
        assert_eq!(0.5, settings.value(-1.0));
    }

    // endregion processed input limit -------------
}
