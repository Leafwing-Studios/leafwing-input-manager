//! Utilities for configuring settings related to axis-like inputs.
//!
//! Axis-like inputs typically involve values representing movement or direction along the axes,
//! such as joystick movements or mouse motions.
//!
//! # Single-Axis Settings
//!
//! The [`SingleAxisSettings`] struct defines settings for processing single-axis input values.
//!
//! # Dual-Axis Settings
//!
//! The [`DualAxisSettings`] struct defines settings for processing dual-axis input values.
//! It provides similar customization options as `SingleAxisSettings` but separately for each axis,
//! allowing more fine-grained control over input processing.
//!
//! # Processing Procedure
//!
//! The general processing procedure for all axis-like inputs involves several steps:
//!
//! 1. **Inversion**: Determines whether the input direction is reversed.
//! 2. **Sensitivity**: Controls the responsiveness of the input.
//! 3. **Input Clamping**: Limits the input values to a specified range.
//! 4. **Input Normalization**: Ensures that input values fall within a standardized range before further processing.
//! 5. **Deadzone**: Specifies the ranges where input values are considered neutral or ignored.
//! 6. **Scaling**: Adjusts processed input values according to a specified scale factor.
//! 7. **Output Clamping**: Limits the output values to a specified range.
//!
//! Each of these steps can be configured using the respective settings
//! provided by the [`SingleAxisSettings`] and [`DualAxisSettings`] structs.

use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use bevy::prelude::{Reflect, Vec2};
use bevy::utils::FloatOrd;
use serde::{Deserialize, Serialize};

use super::axislike_processors::*;

// region Axis Settings ------------------------

/// Settings for single-axis inputs using various types of axis-like input processors.
///
/// # Processing Procedure
///
/// The procedure for processing single-axis inputs is as follows:
///
/// 1. **Inversion**: Determines whether the input direction is reversed.
/// 2. **Sensitivity**: Controls the responsiveness of the input.
///    - Sensitivity values must be non-negative:
///      - `1.0`: No adjustment to the value
///      - `0.0`: Disregards input changes
///      - `(1.0, f32::MAX]`: Amplify input changes
///      - `(0.0, 1.0)`: Reduce input changes
/// 3. [`InputClamp`]: Limits the input values to a specified range.
/// 4. [`InputNormalizer`]: Ensures that input values fall within a standardized range before further processing.
/// 5. [`SingleAxisDeadzone`]: Specifies the ranges where input values are considered neutral or ignored.
/// 6. **Scaling**: Adjusts processed input values according to a specified scale factor.
/// 7. **Output Clamp**: Similar to [`InputClamp`], but limits the output values to a specified range.
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub struct SingleAxisSettings {
    /// The sensitivity and inversion of the input.
    ///
    /// Using a single `f32` here for both sensitivity and inversion
    /// improves performance by eliminating the need for separate fields and branching logic.
    ///
    /// The absolution value determines the sensitivity of the input:
    /// - `1.0` indicates no adjustment to the value
    /// - `0.0` disregards input changes
    /// - Values greater than `1.0` amplify input changes
    /// - Values between `0.0` and `1.0` reduce input changes
    ///
    /// The sign indicates the inversion:
    /// - Positive values indicate no inversion.
    /// - Negative values indicate inversion.
    multiplier: f32,

    /// The input clamp limiting the input values to a specified range.
    clamp_input: InputClamp,

    /// The input normalizer ensuring that input values fall within a specific range before further processing.
    normalizer: InputNormalizer,

    /// The deadzone settings for the input.
    deadzone: SingleAxisDeadzone,

    /// The scale factor for adjusting processed input values.
    scale_output: f32,

    /// The output clamp limiting the output values to a specified range.
    clamp_output: InputClamp,
}

impl SingleAxisSettings {
    /// - Sensitivity: `1.0`
    /// - Deadzone: Excludes near-zero input values within the range `[-0.1, 0.1]`
    /// - Output: Livezone values are normalized into the range `[-1.0, 1.0]`
    pub const DEFAULT: Self = Self {
        multiplier: 1.0,
        clamp_input: InputClamp::None,
        normalizer: InputNormalizer::None,
        deadzone: SingleAxisDeadzone::DEFAULT,
        scale_output: 1.0,
        clamp_output: InputClamp::None,
    };

    /// - Sensitivity: `1.0`
    /// - Deadzone: Only excludes the zeroes
    /// - Output: Livezone values are normalized into the range `[-1.0, 1.0]`
    pub const NO_DEADZONE: Self = Self {
        multiplier: 1.0,
        clamp_input: InputClamp::None,
        normalizer: InputNormalizer::None,
        deadzone: SingleAxisDeadzone::NONE,
        scale_output: 1.0,
        clamp_output: InputClamp::None,
    };

    /// Creates a new [`SingleAxisSettings`] only with the given `sensitivity`.
    ///
    /// If the given `sensitivity` value is negative,
    /// it'll be converted to its absolute value.
    #[must_use]
    pub fn with_sensitivity(sensitivity: f32) -> Self {
        Self {
            multiplier: sensitivity.abs(),
            clamp_input: InputClamp::None,
            normalizer: InputNormalizer::None,
            deadzone: SingleAxisDeadzone::NONE,
            scale_output: 1.0,
            clamp_output: InputClamp::None,
        }
    }

    /// Returns a new [`SingleAxisSettings`] with inversion applied.
    #[must_use]
    pub fn with_inverted(self) -> Self {
        Self {
            multiplier: -self.multiplier,
            clamp_input: self.clamp_input,
            normalizer: self.normalizer,
            deadzone: self.deadzone,
            scale_output: self.scale_output,
            clamp_output: self.clamp_output,
        }
    }

    /// Creates a new [`SingleAxisSettings`] with the given `clamp` for input values before further processing.
    #[must_use]
    pub fn with_input_clamp(&self, clamp: InputClamp) -> Self {
        Self {
            multiplier: self.multiplier,
            clamp_input: clamp,
            normalizer: self.normalizer,
            deadzone: self.deadzone,
            scale_output: self.scale_output,
            clamp_output: self.clamp_output,
        }
    }

    /// Creates a new [`SingleAxisSettings`] with the given `normalizer` before further processing.
    #[must_use]
    pub fn with_normalizer(&self, normalizer: InputNormalizer) -> Self {
        Self {
            multiplier: self.multiplier,
            clamp_input: self.clamp_input,
            normalizer,
            deadzone: self.deadzone,
            scale_output: self.scale_output,
            clamp_output: self.clamp_output,
        }
    }

    /// Creates a new [`SingleAxisSettings`] with the given `deadzone`.
    #[must_use]
    pub fn with_deadzone(&self, deadzone: SingleAxisDeadzone) -> Self {
        Self {
            multiplier: self.multiplier,
            clamp_input: self.clamp_input,
            normalizer: self.normalizer,
            deadzone,
            scale_output: self.scale_output,
            clamp_output: self.clamp_output,
        }
    }

    /// Creates a new [`SingleAxisSettings`] with the given `scale` for output values.
    #[must_use]
    pub fn with_output_scale(&self, scale: f32) -> Self {
        Self {
            multiplier: self.multiplier,
            clamp_input: self.clamp_input,
            normalizer: self.normalizer,
            deadzone: self.deadzone,
            scale_output: scale,
            clamp_output: self.clamp_output,
        }
    }

    /// Creates a new [`SingleAxisSettings`] with the given `clamp` for output values.
    #[must_use]
    pub fn with_output_clamp(&self, clamp: InputClamp) -> Self {
        Self {
            multiplier: self.multiplier,
            clamp_input: self.clamp_input,
            normalizer: self.normalizer,
            deadzone: self.deadzone,
            scale_output: self.scale_output,
            clamp_output: clamp,
        }
    }

    /// Returns the adjusted input value after applying these settings.
    #[must_use]
    pub fn value(&self, input_value: f32) -> f32 {
        let processed_value = self.multiplier * input_value;
        let processed_value = self.clamp_input.clamp(processed_value);
        let processed_value = self.normalizer.normalize(processed_value);
        let processed_value = self.deadzone.value(processed_value);
        let processed_value = self.scale_output * processed_value;
        self.clamp_output.clamp(processed_value)
    }
}

/// Settings for dual-axis inputs using various types of axis-like input processors.
///
/// # Processing Procedure
///
/// The procedure for processing dual-axis inputs is as follows:
///
/// 1. **Inversion**: Determines whether the input direction is reversed on each axis.
/// 2. **Sensitivity**: Controls the responsiveness of the input on each axis.
///    - Sensitivity values must be non-negative:
///      - `1.0`: No adjustment to the value
///      - `0.0`: Disregards input changes
///      - `(1.0, f32::MAX]`: Amplify input changes
///      - `(0.0, 1.0)`: Reduce input changes
/// 3. [`InputClamp`]: Limits the input values to a specified range on each axis.
/// 4. [`InputNormalizer`]: Ensures that input values fall within a specific range on each axis before further processing.
/// 5. [`DualAxisDeadzone`]: Specifies the shapes where input values are considered neutral or ignored on each axis.
/// 6. **Scaling**: Adjusts processed input values on each axis according to a specified scale factor.
/// 7. **Output Clamp**: Similar to [`InputClamp`], but limits the output values to a specified range on each axis.
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub struct DualAxisSettings {
    /// The sensitivity and inversion factor of the input.
    ///
    /// Using a `Vec2` here for both sensitivity and inversion
    /// improves performance by eliminating the need for separate fields and branching logic.
    ///
    /// For each axis, the absolution value determines the sensitivity of the input:
    /// - `1.0` indicates no adjustment to the value
    /// - `0.0` disregards input changes
    /// - Values greater than `1.0` amplify input changes
    /// - Values between `0.0` and `1.0` reduce input changes
    ///
    /// For each axis, the sign indicates the direction of inversion:
    /// - Positive values indicate no inversion.
    /// - Negative values indicate inversion.
    multipliers: Vec2,

    /// The input clamps limiting the input values to a specified range.
    clamps_input: [InputClamp; 2],

    /// The input normalizers ensuring that input values fall within a standardized range before further processing.
    normalizers: [InputNormalizer; 2],

    /// The deadzone settings for the input.
    deadzone: DualAxisDeadzone,

    /// The scale factors for adjusting processed input values.
    scales_output: Vec2,

    /// The output clamps limiting the output values to a specified range.
    clamps_output: [InputClamp; 2],
}

impl DualAxisSettings {
    /// - Sensitivity: `1.0` on both axes
    /// - Deadzone: Excludes near-zero input values within a distance of `0.1` from `Vec2::ZERO`
    /// - Output: Livezone values are normalized into the range `[-1.0, 1.0]` on each axis
    pub const CIRCLE_DEFAULT: DualAxisSettings = DualAxisSettings {
        multipliers: Vec2::ONE,
        clamps_input: [InputClamp::None, InputClamp::None],
        normalizers: [InputNormalizer::None, InputNormalizer::None],
        deadzone: DualAxisDeadzone::CIRCLE_DEFAULT,
        scales_output: Vec2::ONE,
        clamps_output: [InputClamp::None, InputClamp::None],
    };

    /// - Sensitivity: `1.0` on both axes
    /// - Deadzone: Excludes near-zero input values within the range `[-0.1, 0.1]` on each axis
    /// - Output: Livezone values are normalized into the range `[-1.0, 1.0]` on each axis
    pub const SQUARE_DEFAULT: DualAxisSettings = DualAxisSettings {
        multipliers: Vec2::ONE,
        clamps_input: [InputClamp::None, InputClamp::None],
        normalizers: [InputNormalizer::None, InputNormalizer::None],
        deadzone: DualAxisDeadzone::SQUARE_DEFAULT,
        scales_output: Vec2::ONE,
        clamps_output: [InputClamp::None, InputClamp::None],
    };

    /// - Sensitivity: `1.0` on both axes
    /// - Deadzone: Excluding near-zero input values within the range `[-0.1, 0.1]` on each axis,
    ///     and applying rounded corners with the radius of `0.025` along each axis
    /// - Output: Livezone values are normalized into the range `[-1.0, 1.0]` on each axis
    pub const ROUNDED_SQUARE_DEFAULT: DualAxisSettings = DualAxisSettings {
        multipliers: Vec2::ONE,
        clamps_input: [InputClamp::None, InputClamp::None],
        normalizers: [InputNormalizer::None, InputNormalizer::None],
        deadzone: DualAxisDeadzone::ROUNDED_SQUARE_DEFAULT,
        scales_output: Vec2::ONE,
        clamps_output: [InputClamp::None, InputClamp::None],
    };

    /// - Sensitivity: `1.0` on both axes
    /// - Deadzone: Only excludes the zeroes
    /// - Output: Livezone values are normalized into the range `[-1.0, 1.0]` on each axis
    pub const NO_DEADZONE: DualAxisSettings = DualAxisSettings {
        multipliers: Vec2::ONE,
        clamps_input: [InputClamp::None, InputClamp::None],
        normalizers: [InputNormalizer::None, InputNormalizer::None],
        deadzone: DualAxisDeadzone::NONE,
        scales_output: Vec2::ONE,
        clamps_output: [InputClamp::None, InputClamp::None],
    };

    /// Creates a new [`DualAxisSettings`] only with the given `sensitivity`.
    ///
    /// If the given `sensitivity` values are negative,
    /// they'll be converted to their absolute value.
    #[must_use]
    pub fn with_sensitivity(sensitivity: Vec2) -> Self {
        Self {
            multipliers: sensitivity.abs(),
            clamps_input: [InputClamp::None, InputClamp::None],
            normalizers: [InputNormalizer::None, InputNormalizer::None],
            deadzone: DualAxisDeadzone::NONE,
            scales_output: Vec2::ONE,
            clamps_output: [InputClamp::None, InputClamp::None],
        }
    }

    /// Returns a new [`DualAxisSettings`] with inversion applied.
    #[must_use]
    pub fn with_inverted(self) -> Self {
        Self {
            multipliers: -self.multipliers,
            clamps_input: self.clamps_input,
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            scales_output: self.scales_output,
            clamps_output: self.clamps_output,
        }
    }

    /// Returns a new [`DualAxisSettings`] with inversion applied on the x-axis.
    #[must_use]
    pub fn with_inverted_x(self) -> Self {
        Self {
            multipliers: Vec2::new(-self.multipliers.x, self.multipliers.y),
            clamps_input: self.clamps_input,
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            scales_output: self.scales_output,
            clamps_output: self.clamps_output,
        }
    }

    /// Returns a new [`DualAxisSettings`] with inversion applied on the y-axis.
    #[must_use]
    pub fn with_inverted_y(self) -> Self {
        Self {
            multipliers: Vec2::new(self.multipliers.x, -self.multipliers.y),
            clamps_input: self.clamps_input,
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            scales_output: self.scales_output,
            clamps_output: self.clamps_output,
        }
    }

    /// Creates a new [`DualAxisSettings`] with the given `clamp` for input values on the x-axis before further processing.
    #[must_use]
    pub fn with_input_clamp_x(&self, clamp: InputClamp) -> Self {
        Self {
            multipliers: self.multipliers,
            clamps_input: [clamp, self.clamps_input[1]],
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            scales_output: self.scales_output,
            clamps_output: self.clamps_output,
        }
    }

    /// Creates a new [`DualAxisSettings`] with the given `clamp` for input values on the y-axis before further processing.
    #[must_use]
    pub fn with_input_clamp_y(&self, clamp: InputClamp) -> Self {
        Self {
            multipliers: self.multipliers,
            clamps_input: [self.clamps_input[0], clamp],
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            scales_output: self.scales_output,
            clamps_output: self.clamps_output,
        }
    }

    /// Creates a new [`DualAxisSettings`] with the given `normalizer` on the x-axis before further processing.
    #[must_use]
    pub fn with_normalizer_x(&self, normalizer: InputNormalizer) -> Self {
        Self {
            multipliers: self.multipliers,
            clamps_input: self.clamps_input,
            normalizers: [normalizer, self.normalizers[1]],
            deadzone: self.deadzone,
            scales_output: self.scales_output,
            clamps_output: self.clamps_output,
        }
    }

    /// Creates a new [`DualAxisSettings`] with the given `normalizer` on the x-axis before further processing.
    #[must_use]
    pub fn with_normalizer_y(&self, normalizer: InputNormalizer) -> Self {
        Self {
            multipliers: self.multipliers,
            clamps_input: self.clamps_input,
            normalizers: [self.normalizers[0], normalizer],
            deadzone: self.deadzone,
            scales_output: self.scales_output,
            clamps_output: self.clamps_output,
        }
    }

    /// Creates a new [`DualAxisSettings`] with the given `deadzone`.
    #[must_use]
    pub fn with_deadzone(&self, deadzone: DualAxisDeadzone) -> Self {
        Self {
            multipliers: self.multipliers,
            clamps_input: self.clamps_input,
            normalizers: self.normalizers,
            deadzone,
            scales_output: self.scales_output,
            clamps_output: self.clamps_output,
        }
    }

    /// Creates a new [`DualAxisSettings`] with the given `scale` for output values on the x-axis.
    #[must_use]
    pub fn with_output_scales(&self, scales: Vec2) -> Self {
        Self {
            multipliers: self.multipliers,
            clamps_input: self.clamps_input,
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            scales_output: scales,
            clamps_output: self.clamps_output,
        }
    }

    /// Creates a new [`DualAxisSettings`] with the given `scale` for output values on the x-axis.
    #[must_use]
    pub fn with_output_scale_x(&self, scale: f32) -> Self {
        Self {
            multipliers: self.multipliers,
            clamps_input: self.clamps_input,
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            scales_output: Vec2::new(scale, self.scales_output.y),
            clamps_output: self.clamps_output,
        }
    }

    /// Creates a new [`DualAxisSettings`] with the given `scale` for output values on the y-axis.
    #[must_use]
    pub fn with_output_scale_y(&self, scale: f32) -> Self {
        Self {
            multipliers: self.multipliers,
            clamps_input: self.clamps_input,
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            scales_output: Vec2::new(self.scales_output.x, scale),
            clamps_output: self.clamps_output,
        }
    }

    /// Creates a new [`DualAxisSettings`] with the given `clamp` for output values on the x-axis.
    #[must_use]
    pub fn with_output_clamp_x(&self, clamp: InputClamp) -> Self {
        Self {
            multipliers: self.multipliers,
            clamps_input: self.clamps_input,
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            scales_output: self.scales_output,
            clamps_output: [clamp, self.clamps_output[1]],
        }
    }

    /// Creates a new [`DualAxisSettings`] with the given `clamp` for output values on the y-axis.
    #[must_use]
    pub fn with_output_clamp_y(&self, clamp: InputClamp) -> Self {
        Self {
            multipliers: self.multipliers,
            clamps_input: self.clamps_input,
            normalizers: self.normalizers,
            deadzone: self.deadzone,
            scales_output: self.scales_output,
            clamps_output: [self.clamps_output[0], clamp],
        }
    }

    /// Returns the adjusted `input_value` after applying these settings.
    #[must_use]
    #[inline]
    pub fn value(&self, input_value: Vec2) -> Vec2 {
        let processed_value = self.multipliers * input_value;
        let processed_value = Vec2::new(
            self.clamps_input[0].clamp(processed_value.x),
            self.clamps_input[1].clamp(processed_value.y),
        );
        let processed_value = Vec2::new(
            self.normalizers[0].normalize(processed_value.x),
            self.normalizers[1].normalize(processed_value.y),
        );
        let processed_value = self.deadzone.value(processed_value);
        let processed_value = self.scales_output * processed_value;
        Vec2::new(
            self.clamps_output[0].clamp(processed_value.x),
            self.clamps_output[1].clamp(processed_value.y),
        )
    }
}

// endregion Axis Settings ------------------------

// -------------------------
// Unfortunately, Rust doesn't let us automatically derive `Eq` and `Hash` for `f32`.
// It's like teaching a fish to ride a bike – a bit nonsensical!
// But if that fish really wants to pedal, we'll make it work.
// So here we are, showing Rust who's boss!
// -------------------------

impl Eq for SingleAxisSettings {}

impl Hash for SingleAxisSettings {
    fn hash<H: Hasher>(&self, state: &mut H) {
        FloatOrd(self.multiplier).hash(state);
        self.clamp_input.hash(state);
        self.normalizer.hash(state);
        self.deadzone.hash(state);
        FloatOrd(self.scale_output).hash(state);
        self.clamp_output.hash(state);
    }
}

impl Eq for DualAxisSettings {}

impl Hash for DualAxisSettings {
    fn hash<H: Hasher>(&self, state: &mut H) {
        FloatOrd(self.multipliers.x).hash(state);
        FloatOrd(self.multipliers.y).hash(state);
        self.clamps_input[0].hash(state);
        self.clamps_input[1].hash(state);
        self.normalizers[0].hash(state);
        self.normalizers[1].hash(state);
        self.deadzone.hash(state);
        FloatOrd(self.scales_output.x).hash(state);
        FloatOrd(self.scales_output.y).hash(state);
        self.clamps_output[0].hash(state);
        self.clamps_output[1].hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod single_axis {
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

            // Output clamp
            assert_eq!(1.0, settings.value(5.0));
            assert_eq!(-1.0, settings.value(-5.0));

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
            let custom = SingleAxisSettings::with_sensitivity(ratio);
            let normal = SingleAxisSettings::with_sensitivity(1.0);

            let normal_value = |value: f32| ratio * normal.value(value);

            assert_eq!(normal_value(1.0), custom.value(1.0));
            assert_eq!(normal_value(0.0), custom.value(0.0));
            assert_eq!(normal_value(-1.0), custom.value(-1.0));
        }

        #[test]
        fn test_single_axis_negative_sensitivity() {
            let ratio = -0.5;
            let custom = SingleAxisSettings::with_sensitivity(ratio);
            let normal = SingleAxisSettings::with_sensitivity(1.0);

            let normal_value = |value: f32| ratio.abs() * normal.value(value);

            assert_eq!(normal_value(1.0), custom.value(1.0));
            assert_eq!(normal_value(0.0), custom.value(0.0));
            assert_eq!(normal_value(-1.0), custom.value(-1.0));
        }

        #[test]
        fn test_single_axis_zero_sensitivity() {
            let settings = SingleAxisSettings::with_sensitivity(0.0);

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

        // region single-axis input clamp -------------

        #[test]
        fn test_single_axis_input_clamp() {
            let settings =
                SingleAxisSettings::NO_DEADZONE.with_input_clamp(InputClamp::AtLeast(0.5));

            assert_eq!(1.0, settings.value(1.0));
            assert_eq!(0.5, settings.value(0.5));
            assert_eq!(0.5, settings.value(0.0));
            assert_eq!(0.5, settings.value(-0.5));
            assert_eq!(0.5, settings.value(-1.0));
        }

        // endregion single-axis input clamp -------------

        // region single-axis normalization -------------

        #[test]
        fn test_single_axis_normalization() {
            let settings = SingleAxisSettings::NO_DEADZONE
                .with_normalizer(InputNormalizer::standard_min_max(0.0..1.0));

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
            let deadzone = SingleAxisDeadzone::new(0.2);
            let settings = SingleAxisSettings::NO_DEADZONE.with_deadzone(deadzone);

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

        // region single-axis output clamp -------------

        #[test]
        fn test_single_axis_output_clamp() {
            let settings = SingleAxisSettings::NO_DEADZONE
                .with_output_scale(5.0)
                .with_output_clamp(InputClamp::AtLeast(0.5));

            assert_eq!(5.0, settings.value(1.0));
            assert_eq!(2.5, settings.value(0.5));
            assert_eq!(0.5, settings.value(0.0));
            assert_eq!(0.5, settings.value(-0.5));
            assert_eq!(0.5, settings.value(-1.0));
        }

        // endregion single-axis clamp output -------------
    }

    mod dual_axis {
        use super::*;
        use bevy::math::vec2;

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

            // Output clamp
            assert_eq!(settings.value(Vec2::splat(5.0)), Vec2::ONE);
            assert_eq!(settings.value(Vec2::splat(-5.0)), Vec2::NEG_ONE);

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
            let custom = DualAxisSettings::with_sensitivity(ratio);
            let normal = DualAxisSettings::with_sensitivity(Vec2::ONE);

            let normal_value = |value: Vec2| ratio * normal.value(value);

            assert_eq!(normal_value(Vec2::ONE), custom.value(Vec2::ONE));
            assert_eq!(normal_value(Vec2::ZERO), custom.value(Vec2::ZERO));
            assert_eq!(normal_value(Vec2::NEG_ONE), custom.value(Vec2::NEG_ONE));
        }

        #[test]
        fn test_dual_axis_sensitivity_x() {
            let ratio = vec2(0.5, 1.0);
            let custom = DualAxisSettings::with_sensitivity(ratio);
            let normal = DualAxisSettings::with_sensitivity(Vec2::ONE);

            let normal_value = |value: Vec2| ratio * normal.value(value);

            assert_eq!(normal_value(Vec2::ONE), custom.value(Vec2::ONE));
            assert_eq!(normal_value(Vec2::ZERO), custom.value(Vec2::ZERO));
            assert_eq!(normal_value(Vec2::NEG_ONE), custom.value(Vec2::NEG_ONE));
        }

        #[test]
        fn test_dual_axis_sensitivity_y() {
            let ratio = vec2(1.0, 0.5);
            let custom = DualAxisSettings::with_sensitivity(ratio);
            let normal = DualAxisSettings::with_sensitivity(Vec2::ONE);

            let normal_value = |value: Vec2| ratio * normal.value(value);

            assert_eq!(normal_value(Vec2::ONE), custom.value(Vec2::ONE));
            assert_eq!(normal_value(Vec2::ZERO), custom.value(Vec2::ZERO));
            assert_eq!(normal_value(Vec2::NEG_ONE), custom.value(Vec2::NEG_ONE));
        }

        #[test]
        fn test_dual_axis_negative_sensitivity() {
            let ratio = Vec2::splat(-0.5);
            let custom = DualAxisSettings::with_sensitivity(ratio);
            let normal = DualAxisSettings::with_sensitivity(Vec2::ONE);

            let normal_value = |value: Vec2| ratio.abs() * normal.value(value);

            assert_eq!(normal_value(Vec2::ONE), custom.value(Vec2::ONE));
            assert_eq!(normal_value(Vec2::ZERO), custom.value(Vec2::ZERO));
            assert_eq!(normal_value(Vec2::NEG_ONE), custom.value(Vec2::NEG_ONE));
        }

        #[test]
        fn test_dual_axis_negative_sensitivity_x() {
            let ratio = vec2(-0.5, 1.0);
            let custom = DualAxisSettings::with_sensitivity(ratio);
            let normal = DualAxisSettings::with_sensitivity(Vec2::ONE);

            let normal_value = |value: Vec2| ratio.abs() * normal.value(value);

            assert_eq!(normal_value(Vec2::ONE), custom.value(Vec2::ONE));
            assert_eq!(normal_value(Vec2::ZERO), custom.value(Vec2::ZERO));
            assert_eq!(normal_value(Vec2::NEG_ONE), custom.value(Vec2::NEG_ONE));
        }

        #[test]
        fn test_dual_axis_negative_sensitivity_y() {
            let ratio = vec2(1.0, -0.5);
            let custom = DualAxisSettings::with_sensitivity(ratio);
            let normal = DualAxisSettings::with_sensitivity(Vec2::ONE);

            let normal_value = |value: Vec2| ratio.abs() * normal.value(value);

            assert_eq!(normal_value(Vec2::ONE), custom.value(Vec2::ONE));
            assert_eq!(normal_value(Vec2::ZERO), custom.value(Vec2::ZERO));
            assert_eq!(normal_value(Vec2::NEG_ONE), custom.value(Vec2::NEG_ONE));
        }

        #[test]
        fn test_dual_axis_zero_sensitivity() {
            let settings = DualAxisSettings::with_sensitivity(Vec2::ZERO);

            assert_eq!(Vec2::ZERO, settings.value(Vec2::ONE));
            assert_eq!(Vec2::ZERO, settings.value(Vec2::ZERO));
            assert_eq!(Vec2::ZERO, settings.value(Vec2::NEG_ONE));
        }

        #[test]
        fn test_dual_axis_zero_sensitivity_x() {
            let settings = DualAxisSettings::with_sensitivity(Vec2::Y);

            assert_eq!(Vec2::Y, settings.value(Vec2::ONE));
            assert_eq!(Vec2::ZERO, settings.value(Vec2::ZERO));
            assert_eq!(Vec2::NEG_Y, settings.value(Vec2::NEG_ONE));
        }

        #[test]
        fn test_dual_axis_zero_sensitivity_y() {
            let settings = DualAxisSettings::with_sensitivity(Vec2::X);

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

        // region dual-axis input clamp -------------

        #[test]
        fn test_dual_axis_input_clamp_x() {
            let settings =
                DualAxisSettings::NO_DEADZONE.with_input_clamp_x(InputClamp::AtLeast(0.5));

            assert_eq!(Vec2::ONE, settings.value(Vec2::ONE));
            assert_eq!(Vec2::splat(0.5), settings.value(Vec2::splat(0.5)));
            assert_eq!(vec2(0.5, 0.0), settings.value(Vec2::ZERO));
            assert_eq!(vec2(0.5, -0.5), settings.value(Vec2::splat(-0.5)));
            assert_eq!(vec2(0.5, -1.0), settings.value(Vec2::NEG_ONE));
        }

        #[test]
        fn test_dual_axis_input_clamp_y() {
            let settings =
                DualAxisSettings::NO_DEADZONE.with_input_clamp_y(InputClamp::AtLeast(0.5));

            assert_eq!(Vec2::ONE, settings.value(Vec2::ONE));
            assert_eq!(Vec2::splat(0.5), settings.value(Vec2::splat(0.5)));
            assert_eq!(vec2(0.0, 0.5), settings.value(Vec2::ZERO));
            assert_eq!(vec2(-0.5, 0.5), settings.value(Vec2::splat(-0.5)));
            assert_eq!(vec2(-1.0, 0.5), settings.value(Vec2::NEG_ONE));
        }

        // endregion dual-axis input clamp -------------

        // region dual-axis normalization -------------

        #[test]
        fn test_dual_axis_normalization_x() {
            let settings = DualAxisSettings::NO_DEADZONE
                .with_normalizer_x(InputNormalizer::standard_min_max(0.0..1.0));

            assert_eq!(Vec2::ONE, settings.value(Vec2::ONE));
            assert_eq!(Vec2::splat(0.5), settings.value(Vec2::splat(0.5)));
            assert_eq!(Vec2::ZERO, settings.value(Vec2::ZERO));
            assert_eq!(vec2(0.0, -0.5), settings.value(Vec2::splat(-0.5)));
            assert_eq!(vec2(0.0, -1.0), settings.value(Vec2::NEG_ONE));
        }

        #[test]
        fn test_dual_axis_normalization_y() {
            let settings = DualAxisSettings::NO_DEADZONE
                .with_normalizer_y(InputNormalizer::standard_min_max(0.0..1.0));

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
            let deadzone = DualAxisDeadzone::new_circle(0.3, 0.35);
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

        // region dual-axis output clamp -------------

        #[test]
        fn test_dual_axis_output_clamp_x() {
            let settings = DualAxisSettings::NO_DEADZONE
                .with_output_scale_x(5.0)
                .with_output_clamp_x(InputClamp::AtLeast(0.5));

            assert_eq!(vec2(5.0, 1.0), settings.value(Vec2::ONE));
            assert_eq!(vec2(2.5, 0.5), settings.value(Vec2::splat(0.5)));
            assert_eq!(vec2(0.5, 0.0), settings.value(Vec2::ZERO));
            assert_eq!(vec2(0.5, -0.5), settings.value(Vec2::splat(-0.5)));
            assert_eq!(vec2(0.5, -1.0), settings.value(Vec2::NEG_ONE));
        }

        #[test]
        fn test_dual_axis_output_clamp_y() {
            let settings = DualAxisSettings::NO_DEADZONE
                .with_output_scale_y(5.0)
                .with_output_clamp_y(InputClamp::AtLeast(0.5));

            assert_eq!(vec2(1.0, 5.0), settings.value(Vec2::ONE));
            assert_eq!(vec2(0.5, 2.5), settings.value(Vec2::splat(0.5)));
            assert_eq!(vec2(0.0, 0.5), settings.value(Vec2::ZERO));
            assert_eq!(vec2(-0.5, 0.5), settings.value(Vec2::splat(-0.5)));
            assert_eq!(vec2(-1.0, 0.5), settings.value(Vec2::NEG_ONE));
        }

        // endregion dual-axis clamp output -------------
    }
}
