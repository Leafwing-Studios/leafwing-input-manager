//! Utilities for configuring settings related to single-axis inputs.
//!
//! Single-axis inputs typically represent input values along a single dimension,
//! such as horizontal movement on a joystick or slider.
//!
//! # Settings
//!
//! The [`SingleAxisSettings`] struct defines settings for processing single-axis input values.
//!
//! # Processing Procedure
//!
//! The general processing procedure for single-axis inputs involves several steps:
//!
//! 1. **Inversion**: Determines whether the input direction is reversed.
//! 2. **Sensitivity**: Controls the responsiveness of the input.
//!    Sensitivity values must be non-negative:
//!    - `1.0`: No adjustment to the value
//!    - `0.0`: Disregards input changes
//!    - `(1.0, f32::MAX]`: Amplify input changes
//!    - `(0.0, 1.0)`: Reduce input changes
//! 3. **Input [`ValueLimit`]**: Limits the input values to a specified range before further processing.
//! 4. **[`ValueNormalizer`]**: Ensures that input values fall within a standardized range before further processing.
//! 5. **[`Deadzone1`]**: Specifies the ranges where input values are considered neutral or ignored.
//! 6. **Scaling**: Adjusts processed input values according to a specified scale factor.
//! 7. **Output [`ValueLimit`]**: Limits the processed input values to a specified range.
//!
//! Each of these steps can be configured using the respective settings
//! provided by the [`SingleAxisSettings`] struct.

use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use bevy::prelude::Reflect;
use bevy::utils::FloatOrd;
use serde::{Deserialize, Serialize};

use super::common_processors::*;
use super::deadzone_processors::*;

/// Settings for single-axis inputs using input processors.
///
/// # Processing Procedure
///
/// 1. **Inversion**: Determines whether the input direction is reversed.
/// 2. **Sensitivity**: Controls the responsiveness of the input.
///    Sensitivity values must be non-negative:
///    - `1.0`: No adjustment to the value
///    - `0.0`: Disregards input changes
///    - `(1.0, f32::MAX]`: Amplify input changes
///    - `(0.0, 1.0)`: Reduce input changes
/// 3. **Input [`ValueLimit`]**: Limits the input values to a specified range before further processing.
/// 4. **[`ValueNormalizer`]**: Ensures that input values fall within a standardized range before further processing.
/// 5. **[`Deadzone1`]**: Specifies the ranges where input values are considered neutral or ignored.
/// 6. **Scaling**: Adjusts processed input values according to a specified scale factor.
/// 7. **Output [`ValueLimit`]**: Limits the processed input values to a specified range.
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub struct SingleAxisSettings {
    /// The sensitivity and inversion of the input.
    ///
    /// Using a single `f32` here for both sensitivity and inversion
    /// improves performance by eliminating the need for separate fields and branching logic.
    ///
    /// The absolution value determines the sensitivity of the input:
    /// - `1.0`: No adjustment to the value
    /// - `0.0`: Disregards input changes
    /// - `(1.0, f32::MAX]`: Amplify input changes
    /// - `(0.0, 1.0)`: Reduce input changes
    ///
    /// The sign indicates the inversion:
    /// - Positive values indicate no inversion.
    /// - Negative values indicate inversion.
    input_multiplier: f32,

    /// The value limits for clamping the input values to a specified range before further processing.
    input_limit: ValueLimit,

    /// The normalizer ensuring that input values fall within a specific range before further processing.
    normalizer: ValueNormalizer,

    /// The deadzone settings for the input.
    deadzone: Deadzone1,

    /// The scale factor for adjusting processed input values.
    output_scale: f32,

    /// The value limits for clamping the processed input values to a specified range.
    output_limit: ValueLimit,
}

impl SingleAxisSettings {
    /// - Sensitivity: `1.0`
    /// - Deadzone: Excludes near-zero input values within the range `[-0.1, 0.1]`
    /// - Output: Livezone values are normalized into the range `[-1.0, 1.0]`
    pub const DEFAULT: Self = Self {
        input_multiplier: 1.0,
        input_limit: ValueLimit::None,
        normalizer: ValueNormalizer::None,
        deadzone: Deadzone1::SYMMETRIC_DEFAULT,
        output_scale: 1.0,
        output_limit: ValueLimit::None,
    };

    /// - Sensitivity: `1.0`
    /// - Deadzone: None
    pub const NO_DEADZONE: Self = Self {
        input_multiplier: 1.0,
        input_limit: ValueLimit::None,
        normalizer: ValueNormalizer::None,
        deadzone: Deadzone1::None,
        output_scale: 1.0,
        output_limit: ValueLimit::None,
    };

    /// Creates a new [`SingleAxisSettings`] with the given `sensitivity`.
    ///
    /// If the given `sensitivity` value is negative,
    /// it'll be converted to its absolute value.
    #[must_use]
    pub fn with_sensitivity(&self, sensitivity: f32) -> Self {
        Self {
            input_multiplier: sensitivity.abs(),
            input_limit: self.input_limit,
            normalizer: self.normalizer,
            deadzone: self.deadzone,
            output_scale: self.output_scale,
            output_limit: self.output_limit,
        }
    }

    /// Returns a new [`SingleAxisSettings`] with inversion applied.
    #[must_use]
    pub fn with_inverted(&self) -> Self {
        Self {
            input_multiplier: -self.input_multiplier,
            input_limit: self.input_limit,
            normalizer: self.normalizer,
            deadzone: self.deadzone,
            output_scale: self.output_scale,
            output_limit: self.output_limit,
        }
    }

    /// Creates a new [`SingleAxisSettings`] with the given `limit` for input values before further processing.
    #[must_use]
    pub fn with_input_limit(&self, limit: ValueLimit) -> Self {
        Self {
            input_multiplier: self.input_multiplier,
            input_limit: limit,
            normalizer: self.normalizer,
            deadzone: self.deadzone,
            output_scale: self.output_scale,
            output_limit: self.output_limit,
        }
    }

    /// Creates a new [`SingleAxisSettings`] with the given `normalizer` for input values before further processing.
    #[must_use]
    pub fn with_normalizer(&self, normalizer: ValueNormalizer) -> Self {
        Self {
            input_multiplier: self.input_multiplier,
            input_limit: self.input_limit,
            normalizer,
            deadzone: self.deadzone,
            output_scale: self.output_scale,
            output_limit: self.output_limit,
        }
    }

    /// Creates a new [`SingleAxisSettings`] with a deadzone having symmetric `threshold`s.
    #[must_use]
    pub fn with_symmetric_deadzone(&self, threshold: f32) -> Self {
        Self {
            input_multiplier: self.input_multiplier,
            input_limit: self.input_limit,
            normalizer: self.normalizer,
            deadzone: Deadzone1::symmetric(threshold),
            output_scale: self.output_scale,
            output_limit: self.output_limit,
        }
    }

    /// Creates a new [`SingleAxisSettings`] with the given `scale` for output values.
    #[must_use]
    pub fn with_output_scale(&self, scale: f32) -> Self {
        Self {
            input_multiplier: self.input_multiplier,
            input_limit: self.input_limit,
            normalizer: self.normalizer,
            deadzone: self.deadzone,
            output_scale: scale,
            output_limit: self.output_limit,
        }
    }

    /// Creates a new [`SingleAxisSettings`] with the given `limit` for output values.
    #[must_use]
    pub fn with_output_limit(&self, limit: ValueLimit) -> Self {
        Self {
            input_multiplier: self.input_multiplier,
            input_limit: self.input_limit,
            normalizer: self.normalizer,
            deadzone: self.deadzone,
            output_scale: self.output_scale,
            output_limit: limit,
        }
    }

    /// Returns the adjusted `input_value` after applying these settings.
    #[must_use]
    pub fn value(&self, input_value: f32) -> f32 {
        let processed_value = self.input_multiplier * input_value;
        let processed_value = self.input_limit.clamp(processed_value);
        let processed_value = self.normalizer.normalize(processed_value);
        let processed_value = self.deadzone.value(processed_value);
        let processed_value = self.output_scale * processed_value;
        self.output_limit.clamp(processed_value)
    }
}

impl Eq for SingleAxisSettings {}

impl Hash for SingleAxisSettings {
    fn hash<H: Hasher>(&self, state: &mut H) {
        FloatOrd(self.input_multiplier).hash(state);
        self.input_limit.hash(state);
        self.normalizer.hash(state);
        self.deadzone.hash(state);
        FloatOrd(self.output_scale).hash(state);
        self.output_limit.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // region single-axis setting consts -------------

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

    #[test]
    fn test_single_axis_settings_no_deadzone() {
        let settings = SingleAxisSettings::NO_DEADZONE;

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

    // endregion single-axis setting consts -------------

    // region single-axis sensitivity -------------

    #[test]
    fn test_single_axis_sensitivity() {
        let ratio = 0.5;
        let custom = SingleAxisSettings::NO_DEADZONE.with_sensitivity(ratio);
        let normal = SingleAxisSettings::NO_DEADZONE.with_sensitivity(1.0);

        let normal_value = |value: f32| ratio * normal.value(value);

        assert_eq!(normal_value(1.0), custom.value(1.0));
        assert_eq!(normal_value(0.0), custom.value(0.0));
        assert_eq!(normal_value(-1.0), custom.value(-1.0));
    }

    #[test]
    fn test_single_axis_negative_sensitivity() {
        let ratio = -0.5;
        let custom = SingleAxisSettings::NO_DEADZONE.with_sensitivity(ratio);
        let normal = SingleAxisSettings::NO_DEADZONE.with_sensitivity(1.0);

        let normal_value = |value: f32| ratio.abs() * normal.value(value);

        assert_eq!(normal_value(1.0), custom.value(1.0));
        assert_eq!(normal_value(0.0), custom.value(0.0));
        assert_eq!(normal_value(-1.0), custom.value(-1.0));
    }

    #[test]
    fn test_single_axis_zero_sensitivity() {
        let settings = SingleAxisSettings::NO_DEADZONE.with_sensitivity(0.0);

        assert_eq!(0.0, settings.value(1.0));
        assert_eq!(0.0, settings.value(0.0));
        assert_eq!(0.0, settings.value(-1.0));
    }

    // endregion single-axis sensitivity -------------

    // region single-axis inversion -------------

    #[test]
    fn test_single_axis_inversion() {
        let settings = SingleAxisSettings::NO_DEADZONE;

        assert_eq!(1.0, settings.value(1.0));
        assert_eq!(0.5, settings.value(0.5));
        assert_eq!(0.0, settings.value(0.0));
        assert_eq!(-0.5, settings.value(-0.5));
        assert_eq!(-1.0, settings.value(-1.0));

        let settings = settings.with_inverted();

        assert_eq!(-1.0, settings.value(1.0));
        assert_eq!(-0.5, settings.value(0.5));
        assert_eq!(0.0, settings.value(0.0));
        assert_eq!(0.5, settings.value(-0.5));
        assert_eq!(1.0, settings.value(-1.0));

        let settings = settings.with_inverted();

        assert_eq!(1.0, settings.value(1.0));
        assert_eq!(0.5, settings.value(0.5));
        assert_eq!(0.0, settings.value(0.0));
        assert_eq!(-0.5, settings.value(-0.5));
        assert_eq!(-1.0, settings.value(-1.0));
    }

    // endregion single-axis inversion -------------

    // region single-axis input limit -------------

    #[test]
    fn test_single_axis_input_limit() {
        let settings = SingleAxisSettings::NO_DEADZONE.with_input_limit(ValueLimit::AtLeast(0.5));

        assert_eq!(1.0, settings.value(1.0));
        assert_eq!(0.5, settings.value(0.5));
        assert_eq!(0.5, settings.value(0.0));
        assert_eq!(0.5, settings.value(-0.5));
        assert_eq!(0.5, settings.value(-1.0));
    }

    // endregion single-axis input limit -------------

    // region single-axis normalization -------------

    #[test]
    fn test_single_axis_normalization() {
        let settings = SingleAxisSettings::NO_DEADZONE
            .with_normalizer(ValueNormalizer::standard_min_max(0.0..1.0));

        assert_eq!(1.0, settings.value(1.0));
        assert_eq!(0.5, settings.value(0.5));
        assert_eq!(0.0, settings.value(0.0));
        assert_eq!(0.0, settings.value(-0.5));
        assert_eq!(0.0, settings.value(-1.0));
    }

    // endregion single-axis normalization -------------

    // region single-axis deadzone -------------

    #[test]
    fn test_single_axis_deadzone() {
        let settings = SingleAxisSettings::NO_DEADZONE.with_symmetric_deadzone(0.2);

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

    // endregion single-axis deadzone -------------

    // region single-axis output scale -------------

    #[test]
    fn test_single_axis_output_scale() {
        let settings = SingleAxisSettings::NO_DEADZONE.with_output_scale(5.0);

        assert_eq!(5.0, settings.value(1.0));
        assert_eq!(2.5, settings.value(0.5));
        assert_eq!(0.0, settings.value(0.0));
        assert_eq!(-2.5, settings.value(-0.5));
        assert_eq!(-5.0, settings.value(-1.0));
    }

    // endregion single-axis output scale -------------

    // region single-axis output limit -------------

    #[test]
    fn test_single_axis_output_limit() {
        let settings = SingleAxisSettings::NO_DEADZONE
            .with_output_scale(5.0)
            .with_output_limit(ValueLimit::AtLeast(0.5));

        assert_eq!(5.0, settings.value(1.0));
        assert_eq!(2.5, settings.value(0.5));
        assert_eq!(0.5, settings.value(0.0));
        assert_eq!(0.5, settings.value(-0.5));
        assert_eq!(0.5, settings.value(-1.0));
    }

    // endregion single-axis clamp limit -------------
}
