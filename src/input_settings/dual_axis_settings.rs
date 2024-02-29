//! Utilities for configuring settings related to dual-axis inputs.
//!
//! Dual-axis inputs typically represent input values in two dimensions,
//! such as joystick movements in both horizontal and vertical directions.
//!
//! # Settings
//!
//! The [`DualAxisSettings`] struct defines settings for processing dual-axis input values.
//! It provides similar customization options as [`SingleAxisSettings`](crate::prelude::SingleAxisSettings)
//! but separately for each axis, allowing more fine-grained control over input processing.
//!
//! # Processing Procedure
//!
//! The general processing procedure for dual-axis inputs involves several steps:
//!
//! 1. **Inversion**: Determines whether the input direction is reversed on each axis.
//! 2. **Sensitivity**: Controls the responsiveness of the input on each axis.
//!    Sensitivity values must be non-negative:
//!    - `1.0`: No adjustment to the value
//!    - `0.0`: Disregards input changes
//!    - `(1.0, f32::MAX]`: Amplify input changes
//!    - `(0.0, 1.0)`: Reduce input changes
//! 3. **Input [`ValueLimit`]**: Limits the input values to a specified range on each axis before further processing.
//! 4. **[`ValueNormalizer`]**: Ensures that input values fall within a specific range on each axis before further processing.
//! 5. **[`Deadzone2`]**: Specifies the shapes where input values are considered neutral or ignored on each axis.
//! 6. **Scaling**: Adjusts processed input values according to a specified scale factor on each axis.
//! 7. **Output [`ValueLimit`]**: Limits the output values to a specified range on each axis.
//!
//! Each of these steps can be configured using the respective settings
//! provided by the [`DualAxisSettings`] struct.

use std::hash::{Hash, Hasher};

use bevy::prelude::{Reflect, Vec2};
use bevy::utils::FloatOrd;
use serde::{Deserialize, Serialize};

use super::common_processors::*;
use super::deadzone_processors::*;

/// Settings for dual-axis inputs using input processors.
///
/// # Processing Procedure
///
/// 1. **Inversion**: Determines whether the input direction is reversed on each axis.
/// 2. **Sensitivity**: Controls the responsiveness of the input on each axis.
///    Sensitivity values must be non-negative:
///    - `1.0`: No adjustment to the value
///    - `0.0`: Disregards input changes
///    - `(1.0, f32::MAX]`: Amplify input changes
///    - `(0.0, 1.0)`: Reduce input changes
/// 3. **Input [`ValueLimit`]**: Limits the input values to a specified range on each axis before further processing.
/// 4. **[`ValueNormalizer`]**: Ensures that input values fall within a specific range on each axis before further processing.
/// 5. **[`Deadzone2`]**: Specifies the shapes where input values are considered neutral or ignored on each axis.
/// 6. **Scaling**: Adjusts processed input values according to a specified scale factor on each axis.
/// 7. **Output [`ValueLimit`]**: Limits the output values to a specified range on each axis.
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub struct DualAxisSettings {
    /// The sensitivity and inversion factor of the input.
    ///
    /// Using a `Vec2` here for both sensitivity and inversion
    /// improves performance by eliminating the need for separate fields and branching logic.
    ///
    /// For each axis, the absolution value determines the sensitivity of the input:
    /// - `1.0`: No adjustment to the value
    /// - `0.0`: Disregards input changes
    /// - `(1.0, f32::MAX]`: Amplify input changes
    /// - `(0.0, 1.0)`: Reduce input changes
    ///
    /// For each axis, the sign indicates the direction of inversion:
    /// - Positive values indicate no inversion.
    /// - Negative values indicate inversion.
    input_multipliers: Vec2,

    /// The input clamps limiting the input values to a specified range.
    input_limits: [ValueLimit; 2],

    /// The input normalizers ensuring that input values fall within a standardized range before further processing.
    normalizers: [ValueNormalizer; 2],

    /// The deadzone settings for the input.
    deadzone: Deadzone2,

    /// The scale factors for adjusting processed input values.
    output_scales: Vec2,

    /// The output clamps limiting the output values to a specified range.
    output_limits: [ValueLimit; 2],
}

impl DualAxisSettings {
    /// - Sensitivity: `1.0` on both axes
    /// - Deadzone: Excludes near-zero input values within a distance of `0.1` from `Vec2::ZERO`
    /// - Output: Livezone values are normalized into the range `[-1.0, 1.0]` on each axis
    pub const CIRCLE_DEFAULT: DualAxisSettings = DualAxisSettings {
        input_multipliers: Vec2::ONE,
        input_limits: [ValueLimit::None, ValueLimit::None],
        normalizers: [ValueNormalizer::None, ValueNormalizer::None],
        deadzone: Deadzone2::CIRCLE_DEFAULT,
        output_scales: Vec2::ONE,
        output_limits: [ValueLimit::None, ValueLimit::None],
    };

    /// - Sensitivity: `1.0` on both axes
    /// - Deadzone: Excludes near-zero input values within the range `[-0.1, 0.1]` on each axis
    /// - Output: Livezone values are normalized into the range `[-1.0, 1.0]` on each axis
    pub const SQUARE_DEFAULT: DualAxisSettings = DualAxisSettings {
        input_multipliers: Vec2::ONE,
        input_limits: [ValueLimit::None, ValueLimit::None],
        normalizers: [ValueNormalizer::None, ValueNormalizer::None],
        deadzone: Deadzone2::SQUARE_DEFAULT,
        output_scales: Vec2::ONE,
        output_limits: [ValueLimit::None, ValueLimit::None],
    };

    /// - Sensitivity: `1.0` on both axes
    /// - Deadzone: Excluding near-zero input values within the range `[-0.1, 0.1]` on each axis,
    ///     and applying rounded corners with the radius of `0.025` along each axis
    /// - Output: Livezone values are normalized into the range `[-1.0, 1.0]` on each axis
    pub const ROUNDED_SQUARE_DEFAULT: DualAxisSettings = DualAxisSettings {
        input_multipliers: Vec2::ONE,
        input_limits: [ValueLimit::None, ValueLimit::None],
        normalizers: [ValueNormalizer::None, ValueNormalizer::None],
        deadzone: Deadzone2::ROUNDED_SQUARE_DEFAULT,
        output_scales: Vec2::ONE,
        output_limits: [ValueLimit::None, ValueLimit::None],
    };

    /// - Sensitivity: `1.0` on both axes
    /// - Deadzone: None
    pub const NO_DEADZONE: DualAxisSettings = DualAxisSettings {
        input_multipliers: Vec2::ONE,
        input_limits: [ValueLimit::None, ValueLimit::None],
        normalizers: [ValueNormalizer::None, ValueNormalizer::None],
        deadzone: Deadzone2::None,
        output_scales: Vec2::ONE,
        output_limits: [ValueLimit::None, ValueLimit::None],
    };

    /// Creates a new [`DualAxisSettings`] only with the given `sensitivity`.
    ///
    /// If the given `sensitivity` values are negative,
    /// they'll be converted to their absolute value.
    #[must_use]
    pub fn with_sensitivity(&self, sensitivity: Vec2) -> Self {
        Self {
            input_multipliers: sensitivity.abs(),
            input_limits: self.input_limits,
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            output_scales: self.output_scales,
            output_limits: self.output_limits,
        }
    }

    /// Returns a new [`DualAxisSettings`] with inversion applied.
    #[must_use]
    pub fn with_inverted(&self) -> Self {
        Self {
            input_multipliers: -self.input_multipliers,
            input_limits: self.input_limits,
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            output_scales: self.output_scales,
            output_limits: self.output_limits,
        }
    }

    /// Returns a new [`DualAxisSettings`] with inversion applied on the x-axis.
    #[must_use]
    pub fn with_inverted_x(&self) -> Self {
        Self {
            input_multipliers: Vec2::new(-self.input_multipliers.x, self.input_multipliers.y),
            input_limits: self.input_limits,
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            output_scales: self.output_scales,
            output_limits: self.output_limits,
        }
    }

    /// Returns a new [`DualAxisSettings`] with inversion applied on the y-axis.
    #[must_use]
    pub fn with_inverted_y(&self) -> Self {
        Self {
            input_multipliers: Vec2::new(self.input_multipliers.x, -self.input_multipliers.y),
            input_limits: self.input_limits,
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            output_scales: self.output_scales,
            output_limits: self.output_limits,
        }
    }

    /// Creates a new [`DualAxisSettings`] with the given `limit` for input values on the x-axis before further processing.
    #[must_use]
    pub fn with_input_limit_x(&self, limit: ValueLimit) -> Self {
        Self {
            input_multipliers: self.input_multipliers,
            input_limits: [limit, self.input_limits[1]],
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            output_scales: self.output_scales,
            output_limits: self.output_limits,
        }
    }

    /// Creates a new [`DualAxisSettings`] with the given `limit` for input values on the y-axis before further processing.
    #[must_use]
    pub fn with_input_limit_y(&self, limit: ValueLimit) -> Self {
        Self {
            input_multipliers: self.input_multipliers,
            input_limits: [self.input_limits[0], limit],
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            output_scales: self.output_scales,
            output_limits: self.output_limits,
        }
    }

    /// Creates a new [`DualAxisSettings`] with the given `normalizer` on the x-axis before further processing.
    #[must_use]
    pub fn with_normalizer_x(&self, normalizer: ValueNormalizer) -> Self {
        Self {
            input_multipliers: self.input_multipliers,
            input_limits: self.input_limits,
            normalizers: [normalizer, self.normalizers[1]],
            deadzone: self.deadzone,
            output_scales: self.output_scales,
            output_limits: self.output_limits,
        }
    }

    /// Creates a new [`DualAxisSettings`] with the given `normalizer` on the y-axis before further processing.
    #[must_use]
    pub fn with_normalizer_y(&self, normalizer: ValueNormalizer) -> Self {
        Self {
            input_multipliers: self.input_multipliers,
            input_limits: self.input_limits,
            normalizers: [self.normalizers[0], normalizer],
            deadzone: self.deadzone,
            output_scales: self.output_scales,
            output_limits: self.output_limits,
        }
    }

    /// Creates a new [`DualAxisSettings`] with the given `deadzone`.
    #[must_use]
    pub fn with_deadzone(&self, deadzone: Deadzone2) -> Self {
        Self {
            input_multipliers: self.input_multipliers,
            input_limits: self.input_limits,
            normalizers: self.normalizers,
            deadzone,
            output_scales: self.output_scales,
            output_limits: self.output_limits,
        }
    }

    /// Creates a new [`DualAxisSettings`] with the given `scales` for output values on both axes.
    #[must_use]
    pub fn with_output_scales(&self, scales: Vec2) -> Self {
        Self {
            input_multipliers: self.input_multipliers,
            input_limits: self.input_limits,
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            output_scales: scales,
            output_limits: self.output_limits,
        }
    }

    /// Creates a new [`DualAxisSettings`] with the given `scale` for output values on the x-axis.
    #[must_use]
    pub fn with_output_scale_x(&self, scale: f32) -> Self {
        Self {
            input_multipliers: self.input_multipliers,
            input_limits: self.input_limits,
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            output_scales: Vec2::new(scale, self.output_scales.y),
            output_limits: self.output_limits,
        }
    }

    /// Creates a new [`DualAxisSettings`] with the given `scale` for output values on the y-axis.
    #[must_use]
    pub fn with_output_scale_y(&self, scale: f32) -> Self {
        Self {
            input_multipliers: self.input_multipliers,
            input_limits: self.input_limits,
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            output_scales: Vec2::new(self.output_scales.x, scale),
            output_limits: self.output_limits,
        }
    }

    /// Creates a new [`DualAxisSettings`] with the given `limit` for output values on the x-axis.
    #[must_use]
    pub fn with_output_limit_x(&self, clamp: ValueLimit) -> Self {
        Self {
            input_multipliers: self.input_multipliers,
            input_limits: self.input_limits,
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            output_scales: self.output_scales,
            output_limits: [clamp, self.output_limits[1]],
        }
    }

    /// Creates a new [`DualAxisSettings`] with the given `limit` for output values on the y-axis.
    #[must_use]
    pub fn with_output_limit_y(&self, clamp: ValueLimit) -> Self {
        Self {
            input_multipliers: self.input_multipliers,
            input_limits: self.input_limits,
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            output_scales: self.output_scales,
            output_limits: [self.output_limits[0], clamp],
        }
    }

    /// Returns the adjusted `input_value` after applying these settings.
    #[must_use]
    #[inline]
    pub fn value(&self, input_value: Vec2) -> Vec2 {
        let processed_value = self.input_multipliers * input_value;
        let processed_value = Vec2::new(
            self.input_limits[0].clamp(processed_value.x),
            self.input_limits[1].clamp(processed_value.y),
        );
        let processed_value = Vec2::new(
            self.normalizers[0].normalize(processed_value.x),
            self.normalizers[1].normalize(processed_value.y),
        );
        let processed_value = self.deadzone.value(processed_value);
        let processed_value = self.output_scales * processed_value;
        Vec2::new(
            self.output_limits[0].clamp(processed_value.x),
            self.output_limits[1].clamp(processed_value.y),
        )
    }
}

impl Eq for DualAxisSettings {}

impl Hash for DualAxisSettings {
    fn hash<H: Hasher>(&self, state: &mut H) {
        FloatOrd(self.input_multipliers.x).hash(state);
        FloatOrd(self.input_multipliers.y).hash(state);
        self.input_limits[0].hash(state);
        self.input_limits[1].hash(state);
        self.normalizers[0].hash(state);
        self.normalizers[1].hash(state);
        self.deadzone.hash(state);
        FloatOrd(self.output_scales.x).hash(state);
        FloatOrd(self.output_scales.y).hash(state);
        self.output_limits[0].hash(state);
        self.output_limits[1].hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::{vec2, Vec2};

    // region dual-axis setting consts -------------

    #[test]
    fn test_dual_axis_settings_circle_default() {
        let settings = DualAxisSettings::CIRCLE_DEFAULT;

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
    fn test_dual_axis_settings_square_default() {
        let settings = DualAxisSettings::SQUARE_DEFAULT;

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
    fn test_dual_axis_settings_rounded_square_default() {
        let settings = DualAxisSettings::ROUNDED_SQUARE_DEFAULT;

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

    #[test]
    fn test_dual_axis_settings_no_deadzone() {
        let settings = DualAxisSettings::NO_DEADZONE;

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

    // endregion dual-axis setting consts -------------

    // region dual-axis sensitivity -------------

    #[test]
    fn test_dual_axis_sensitivity() {
        let ratio = Vec2::splat(0.5);
        let custom = DualAxisSettings::NO_DEADZONE.with_sensitivity(ratio);
        let normal = DualAxisSettings::NO_DEADZONE.with_sensitivity(Vec2::ONE);

        let normal_value = |value: Vec2| ratio * normal.value(value);

        assert_eq!(normal_value(Vec2::ONE), custom.value(Vec2::ONE));
        assert_eq!(normal_value(Vec2::ZERO), custom.value(Vec2::ZERO));
        assert_eq!(normal_value(Vec2::NEG_ONE), custom.value(Vec2::NEG_ONE));
    }

    #[test]
    fn test_dual_axis_sensitivity_x() {
        let ratio = vec2(0.5, 1.0);
        let custom = DualAxisSettings::NO_DEADZONE.with_sensitivity(ratio);
        let normal = DualAxisSettings::NO_DEADZONE.with_sensitivity(Vec2::ONE);

        let normal_value = |value: Vec2| ratio * normal.value(value);

        assert_eq!(normal_value(Vec2::ONE), custom.value(Vec2::ONE));
        assert_eq!(normal_value(Vec2::ZERO), custom.value(Vec2::ZERO));
        assert_eq!(normal_value(Vec2::NEG_ONE), custom.value(Vec2::NEG_ONE));
    }

    #[test]
    fn test_dual_axis_sensitivity_y() {
        let ratio = vec2(1.0, 0.5);
        let custom = DualAxisSettings::NO_DEADZONE.with_sensitivity(ratio);
        let normal = DualAxisSettings::NO_DEADZONE.with_sensitivity(Vec2::ONE);

        let normal_value = |value: Vec2| ratio * normal.value(value);

        assert_eq!(normal_value(Vec2::ONE), custom.value(Vec2::ONE));
        assert_eq!(normal_value(Vec2::ZERO), custom.value(Vec2::ZERO));
        assert_eq!(normal_value(Vec2::NEG_ONE), custom.value(Vec2::NEG_ONE));
    }

    #[test]
    fn test_dual_axis_negative_sensitivity() {
        let ratio = Vec2::splat(-0.5);
        let custom = DualAxisSettings::NO_DEADZONE.with_sensitivity(ratio);
        let normal = DualAxisSettings::NO_DEADZONE.with_sensitivity(Vec2::ONE);

        let normal_value = |value: Vec2| ratio.abs() * normal.value(value);

        assert_eq!(normal_value(Vec2::ONE), custom.value(Vec2::ONE));
        assert_eq!(normal_value(Vec2::ZERO), custom.value(Vec2::ZERO));
        assert_eq!(normal_value(Vec2::NEG_ONE), custom.value(Vec2::NEG_ONE));
    }

    #[test]
    fn test_dual_axis_negative_sensitivity_x() {
        let ratio = vec2(-0.5, 1.0);
        let custom = DualAxisSettings::NO_DEADZONE.with_sensitivity(ratio);
        let normal = DualAxisSettings::NO_DEADZONE.with_sensitivity(Vec2::ONE);

        let normal_value = |value: Vec2| ratio.abs() * normal.value(value);

        assert_eq!(normal_value(Vec2::ONE), custom.value(Vec2::ONE));
        assert_eq!(normal_value(Vec2::ZERO), custom.value(Vec2::ZERO));
        assert_eq!(normal_value(Vec2::NEG_ONE), custom.value(Vec2::NEG_ONE));
    }

    #[test]
    fn test_dual_axis_negative_sensitivity_y() {
        let ratio = vec2(1.0, -0.5);
        let custom = DualAxisSettings::NO_DEADZONE.with_sensitivity(ratio);
        let normal = DualAxisSettings::NO_DEADZONE.with_sensitivity(Vec2::ONE);

        let normal_value = |value: Vec2| ratio.abs() * normal.value(value);

        assert_eq!(normal_value(Vec2::ONE), custom.value(Vec2::ONE));
        assert_eq!(normal_value(Vec2::ZERO), custom.value(Vec2::ZERO));
        assert_eq!(normal_value(Vec2::NEG_ONE), custom.value(Vec2::NEG_ONE));
    }

    #[test]
    fn test_dual_axis_zero_sensitivity() {
        let settings = DualAxisSettings::NO_DEADZONE.with_sensitivity(Vec2::ZERO);

        assert_eq!(Vec2::ZERO, settings.value(Vec2::ONE));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::ZERO));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::NEG_ONE));
    }

    #[test]
    fn test_dual_axis_zero_sensitivity_x() {
        let settings = DualAxisSettings::NO_DEADZONE.with_sensitivity(Vec2::Y);

        assert_eq!(Vec2::Y, settings.value(Vec2::ONE));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::ZERO));
        assert_eq!(Vec2::NEG_Y, settings.value(Vec2::NEG_ONE));
    }

    #[test]
    fn test_dual_axis_zero_sensitivity_y() {
        let settings = DualAxisSettings::NO_DEADZONE.with_sensitivity(Vec2::X);

        assert_eq!(Vec2::X, settings.value(Vec2::ONE));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::ZERO));
        assert_eq!(Vec2::NEG_X, settings.value(Vec2::NEG_ONE));
    }

    // endregion dual-axis sensitivity -------------

    // region dual-axis inversion -------------

    #[test]
    fn test_dual_axis_inversion() {
        let settings = DualAxisSettings::NO_DEADZONE;

        assert_eq!(Vec2::ONE, settings.value(Vec2::ONE));
        assert_eq!(Vec2::splat(0.5), settings.value(Vec2::splat(0.5)));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::ZERO));
        assert_eq!(Vec2::splat(-0.5), settings.value(Vec2::splat(-0.5)));
        assert_eq!(Vec2::NEG_ONE, settings.value(Vec2::NEG_ONE));

        let settings = settings.with_inverted();

        assert_eq!(Vec2::NEG_ONE, settings.value(Vec2::ONE));
        assert_eq!(Vec2::splat(-0.5), settings.value(Vec2::splat(0.5)));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::ZERO));
        assert_eq!(Vec2::splat(0.5), settings.value(Vec2::splat(-0.5)));
        assert_eq!(Vec2::ONE, settings.value(Vec2::NEG_ONE));

        let settings = settings.with_inverted();

        assert_eq!(Vec2::ONE, settings.value(Vec2::ONE));
        assert_eq!(Vec2::splat(0.5), settings.value(Vec2::splat(0.5)));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::ZERO));
        assert_eq!(Vec2::splat(-0.5), settings.value(Vec2::splat(-0.5)));
        assert_eq!(Vec2::NEG_ONE, settings.value(Vec2::NEG_ONE));
    }

    #[test]
    fn test_dual_axis_inversion_x() {
        let settings = DualAxisSettings::NO_DEADZONE;

        assert_eq!(Vec2::ONE, settings.value(Vec2::ONE));
        assert_eq!(Vec2::splat(0.5), settings.value(Vec2::splat(0.5)));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::ZERO));
        assert_eq!(Vec2::splat(-0.5), settings.value(Vec2::splat(-0.5)));
        assert_eq!(Vec2::NEG_ONE, settings.value(Vec2::NEG_ONE));

        let settings = settings.with_inverted_x();

        assert_eq!(vec2(-1.0, 1.0), settings.value(Vec2::ONE));
        assert_eq!(vec2(-0.5, 0.5), settings.value(Vec2::splat(0.5)));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::ZERO));
        assert_eq!(vec2(0.5, -0.5), settings.value(Vec2::splat(-0.5)));
        assert_eq!(vec2(1.0, -1.0), settings.value(Vec2::NEG_ONE));

        let settings = settings.with_inverted_x();

        assert_eq!(Vec2::ONE, settings.value(Vec2::ONE));
        assert_eq!(Vec2::splat(0.5), settings.value(Vec2::splat(0.5)));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::ZERO));
        assert_eq!(Vec2::splat(-0.5), settings.value(Vec2::splat(-0.5)));
        assert_eq!(Vec2::NEG_ONE, settings.value(Vec2::NEG_ONE));
    }

    #[test]
    fn test_dual_axis_inversion_y() {
        let settings = DualAxisSettings::NO_DEADZONE;

        assert_eq!(Vec2::ONE, settings.value(Vec2::ONE));
        assert_eq!(Vec2::splat(0.5), settings.value(Vec2::splat(0.5)));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::ZERO));
        assert_eq!(Vec2::splat(-0.5), settings.value(Vec2::splat(-0.5)));
        assert_eq!(Vec2::NEG_ONE, settings.value(Vec2::NEG_ONE));

        let settings = settings.with_inverted_y();

        assert_eq!(vec2(1.0, -1.0), settings.value(Vec2::ONE));
        assert_eq!(vec2(0.5, -0.5), settings.value(Vec2::splat(0.5)));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::ZERO));
        assert_eq!(vec2(-0.5, 0.5), settings.value(Vec2::splat(-0.5)));
        assert_eq!(vec2(-1.0, 1.0), settings.value(Vec2::NEG_ONE));

        let settings = settings.with_inverted_y();

        assert_eq!(Vec2::ONE, settings.value(Vec2::ONE));
        assert_eq!(Vec2::splat(0.5), settings.value(Vec2::splat(0.5)));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::ZERO));
        assert_eq!(Vec2::splat(-0.5), settings.value(Vec2::splat(-0.5)));
        assert_eq!(Vec2::NEG_ONE, settings.value(Vec2::NEG_ONE));
    }

    // endregion dual-axis inversion -------------

    // region dual-axis input limit -------------

    #[test]
    fn test_dual_axis_input_limit_x() {
        let settings = DualAxisSettings::NO_DEADZONE.with_input_limit_x(ValueLimit::AtLeast(0.5));

        assert_eq!(Vec2::ONE, settings.value(Vec2::ONE));
        assert_eq!(Vec2::splat(0.5), settings.value(Vec2::splat(0.5)));
        assert_eq!(vec2(0.5, 0.0), settings.value(Vec2::ZERO));
        assert_eq!(vec2(0.5, -0.5), settings.value(Vec2::splat(-0.5)));
        assert_eq!(vec2(0.5, -1.0), settings.value(Vec2::NEG_ONE));
    }

    #[test]
    fn test_dual_axis_input_limit_y() {
        let settings = DualAxisSettings::NO_DEADZONE.with_input_limit_y(ValueLimit::AtLeast(0.5));

        assert_eq!(Vec2::ONE, settings.value(Vec2::ONE));
        assert_eq!(Vec2::splat(0.5), settings.value(Vec2::splat(0.5)));
        assert_eq!(vec2(0.0, 0.5), settings.value(Vec2::ZERO));
        assert_eq!(vec2(-0.5, 0.5), settings.value(Vec2::splat(-0.5)));
        assert_eq!(vec2(-1.0, 0.5), settings.value(Vec2::NEG_ONE));
    }

    // endregion dual-axis input limit -------------

    // region dual-axis normalization -------------

    #[test]
    fn test_dual_axis_normalization_x() {
        let normalizer = ValueNormalizer::standard_min_max(0.0..1.0);
        let settings = DualAxisSettings::NO_DEADZONE.with_normalizer_x(normalizer);

        assert_eq!(Vec2::ONE, settings.value(Vec2::ONE));
        assert_eq!(Vec2::splat(0.5), settings.value(Vec2::splat(0.5)));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::ZERO));
        assert_eq!(vec2(0.0, -0.5), settings.value(Vec2::splat(-0.5)));
        assert_eq!(vec2(0.0, -1.0), settings.value(Vec2::NEG_ONE));
    }

    #[test]
    fn test_dual_axis_normalization_y() {
        let normalizer = ValueNormalizer::standard_min_max(0.0..1.0);
        let settings = DualAxisSettings::NO_DEADZONE.with_normalizer_y(normalizer);

        assert_eq!(Vec2::ONE, settings.value(Vec2::ONE));
        assert_eq!(Vec2::splat(0.5), settings.value(Vec2::splat(0.5)));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::ZERO));
        assert_eq!(vec2(-0.5, 0.0), settings.value(Vec2::splat(-0.5)));
        assert_eq!(vec2(-1.0, 0.0), settings.value(Vec2::NEG_ONE));
    }

    // endregion dual-axis normalization -------------

    // region dual-axis deadzone -------------

    #[test]
    fn test_dual_axis_deadzone_circle() {
        let deadzone = Deadzone2::new_circle(0.3, 0.35);
        let settings = DualAxisSettings::NO_DEADZONE.with_deadzone(deadzone);

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

    // endregion dual-axis deadzone -------------

    // region dual-axis output scale -------------

    #[test]
    fn test_dual_axis_output_scales() {
        let settings = DualAxisSettings::NO_DEADZONE.with_output_scales(Vec2::splat(5.0));

        assert_eq!(Vec2::splat(5.0), settings.value(Vec2::ONE));
        assert_eq!(Vec2::splat(2.5), settings.value(Vec2::splat(0.5)));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::ZERO));
        assert_eq!(Vec2::splat(-2.5), settings.value(Vec2::splat(-0.5)));
        assert_eq!(Vec2::splat(-5.0), settings.value(Vec2::NEG_ONE));
    }

    #[test]
    fn test_dual_axis_output_scale_x() {
        let settings = DualAxisSettings::NO_DEADZONE.with_output_scale_x(5.0);

        assert_eq!(vec2(5.0, 1.0), settings.value(Vec2::ONE));
        assert_eq!(vec2(2.5, 0.5), settings.value(Vec2::splat(0.5)));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::ZERO));
        assert_eq!(vec2(-2.5, -0.5), settings.value(Vec2::splat(-0.5)));
        assert_eq!(vec2(-5.0, -1.0), settings.value(Vec2::NEG_ONE));
    }

    #[test]
    fn test_dual_axis_output_scale_y() {
        let settings = DualAxisSettings::NO_DEADZONE.with_output_scale_y(5.0);

        assert_eq!(vec2(1.0, 5.0), settings.value(Vec2::ONE));
        assert_eq!(vec2(0.5, 2.5), settings.value(Vec2::splat(0.5)));
        assert_eq!(Vec2::ZERO, settings.value(Vec2::ZERO));
        assert_eq!(vec2(-0.5, -2.5), settings.value(Vec2::splat(-0.5)));
        assert_eq!(vec2(-1.0, -5.0), settings.value(Vec2::NEG_ONE));
    }

    // endregion dual-axis output scale -------------

    // region dual-axis output limit -------------

    #[test]
    fn test_dual_axis_output_limit_x() {
        let settings = DualAxisSettings::NO_DEADZONE
            .with_output_scale_x(5.0)
            .with_output_limit_x(ValueLimit::AtLeast(0.5));

        assert_eq!(vec2(5.0, 1.0), settings.value(Vec2::ONE));
        assert_eq!(vec2(2.5, 0.5), settings.value(Vec2::splat(0.5)));
        assert_eq!(vec2(0.5, 0.0), settings.value(Vec2::ZERO));
        assert_eq!(vec2(0.5, -0.5), settings.value(Vec2::splat(-0.5)));
        assert_eq!(vec2(0.5, -1.0), settings.value(Vec2::NEG_ONE));
    }

    #[test]
    fn test_dual_axis_output_limit_y() {
        let settings = DualAxisSettings::NO_DEADZONE
            .with_output_scale_y(5.0)
            .with_output_limit_y(ValueLimit::AtLeast(0.5));

        assert_eq!(vec2(1.0, 5.0), settings.value(Vec2::ONE));
        assert_eq!(vec2(0.5, 2.5), settings.value(Vec2::splat(0.5)));
        assert_eq!(vec2(0.0, 0.5), settings.value(Vec2::ZERO));
        assert_eq!(vec2(-0.5, 0.5), settings.value(Vec2::splat(-0.5)));
        assert_eq!(vec2(-1.0, 0.5), settings.value(Vec2::NEG_ONE));
    }

    // endregion dual-axis output limit -------------
}
