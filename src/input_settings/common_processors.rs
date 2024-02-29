//! Utilities for processing all kinds of inputs.
//!
//! This module provides the functionality for processing all kinds of inputs in applications or games.
//!
//! ## Limit Processors
//!
//! The [`ValueLimit`] enum defines various strategies of input limiting:
//! - [`ValueLimit::None`]: No limiting operation is performed on the input values
//! - [`ValueLimit::AtLeast`]: Clamps the input values to be at least the specified minimum value
//! - [`ValueLimit::AtMost`]: Clamps the input values to be at most the specified maximum value
//! - [`ValueLimit::Range`]: Clamps the input values to be within the specified range
//!
//! ## Normalization Processors
//!
//! The [`ValueNormalizer`] enum defines various strategies of input normalization:
//! - [`ValueNormalizer::None`]: No normalization is performed on the input values
//! - [`ValueNormalizer::MinMax`]: Min-max normalization mapping input values to a specified output range

use std::hash::{Hash, Hasher};
use std::ops::Range;

use bevy::prelude::Reflect;
use bevy::utils::FloatOrd;
use serde::{Deserialize, Serialize};

// region Limit Processors ------------------------

/// Various strategies of input limiting.
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub enum ValueLimit {
    /// No limiting operation is performed on the input values.
    None,

    /// Clamps the input values to be at least the specified minimum value.
    AtLeast(f32),

    /// Clamps the input values to be at most the specified maximum value.
    AtMost(f32),

    /// Clamps the input values to be within the specified range.
    ///
    /// The first value represents the minimum limit,
    /// and the second value represents the maximum limit.
    Range(f32, f32),
}

impl ValueLimit {
    /// Clamps the `input_value` based on this limiting strategy.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use leafwing_input_manager::prelude::ValueLimit;
    ///
    /// let limit = ValueLimit::AtMost(2.0);
    ///
    /// assert_eq!(limit.clamp(5.0), 2.0);
    /// assert_eq!(limit.clamp(0.0), 0.0);
    /// assert_eq!(limit.clamp(-5.0), -5.0);
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

// endregion Limit Processors ------------------------

// region Normalization Processors ------------------------

/// Various strategies of input normalization.
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub enum ValueNormalizer {
    /// No normalization is performed on the input values.
    None,

    /// Min-max normalization mapping input values to a specified output range.
    MinMax {
        /// The minimum value of the input range.
        input_min: f32,

        /// The width of the range `[input_min, input_max]`,
        input_range_width: f32,

        /// The reciprocal of the `input_range_width`,
        /// pre-calculated to avoid division during computation.
        recip_input_range_width: f32,

        /// The minimum value of the output range.
        output_min: f32,

        /// The width of the range `[output_min, output_max]`.
        output_range_width: f32,
    },
}

impl ValueNormalizer {
    /// Creates a new [`ValueNormalizer::MinMax`] with the output range of `[0.0, 1.0]` and the specified input range.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use leafwing_input_manager::prelude::ValueNormalizer;
    ///
    /// let normalizer = ValueNormalizer::standard_min_max(0.0..100.0);
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

    /// Creates a new [`ValueNormalizer::MinMax`] with the output range of `[-1.0, 1.0]` and the specified input range.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use leafwing_input_manager::prelude::ValueNormalizer;
    ///
    /// let normalizer = ValueNormalizer::symmetric_min_max(0.0..100.0);
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

    /// Creates a new [`ValueNormalizer::MinMax`] with the specified input and output ranges.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use leafwing_input_manager::prelude::ValueNormalizer;
    ///
    /// let normalizer = ValueNormalizer::custom_min_max(0.0..100.0, -4.0..4.0);
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

    /// Normalizes the `input_value`.
    ///
    /// # Examples
    ///
    /// ```
    /// use leafwing_input_manager::prelude::ValueNormalizer;
    ///
    /// let normalizer = ValueNormalizer::symmetric_min_max(0.0..100.0);
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
            ValueNormalizer::None => input_value,
            ValueNormalizer::MinMax {
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

// -------------------------
// Unfortunately, Rust doesn't let us automatically derive `Eq` and `Hash` for `f32`.
// It's like teaching a fish to ride a bike – a bit nonsensical!
// But if that fish really wants to pedal, we'll make it work.
// So here we are, showing Rust who's boss!
// -------------------------

impl Eq for ValueLimit {}

impl Hash for ValueLimit {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ValueLimit::None => {
                0.hash(state);
            }
            ValueLimit::AtLeast(min) => {
                1.hash(state);
                FloatOrd(*min).hash(state);
            }
            ValueLimit::AtMost(max) => {
                2.hash(state);
                FloatOrd(*max).hash(state);
            }
            ValueLimit::Range(min, max) => {
                3.hash(state);
                FloatOrd(*min).hash(state);
                FloatOrd(*max).hash(state);
            }
        }
    }
}

impl Eq for ValueNormalizer {}

impl Hash for ValueNormalizer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ValueNormalizer::None => {
                0.hash(state);
            }
            ValueNormalizer::MinMax {
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

#[cfg(test)]
mod tests {
    mod limit {
        use crate::prelude::common_processors::ValueLimit;

        #[test]
        fn test_limit_none() {
            let limit = ValueLimit::None;
            assert_eq!(5.0, limit.clamp(5.0));
            assert_eq!(0.0, limit.clamp(0.0));
            assert_eq!(-5.0, limit.clamp(-5.0));
        }

        #[test]
        fn test_limit_at_least() {
            let limit = ValueLimit::AtLeast(2.0);
            assert_eq!(5.0, limit.clamp(5.0));
            assert_eq!(2.0, limit.clamp(2.0));
            assert_eq!(2.0, limit.clamp(0.0));
            assert_eq!(2.0, limit.clamp(-2.0));
            assert_eq!(2.0, limit.clamp(-5.0));
        }

        #[test]
        fn test_limit_at_most() {
            let limit = ValueLimit::AtMost(2.0);
            assert_eq!(2.0, limit.clamp(5.0));
            assert_eq!(2.0, limit.clamp(2.0));
            assert_eq!(0.0, limit.clamp(0.0));
            assert_eq!(-2.0, limit.clamp(-2.0));
            assert_eq!(-5.0, limit.clamp(-5.0));
        }

        #[test]
        fn test_limit_range() {
            let limit = ValueLimit::Range(1.0, 2.0);
            assert_eq!(2.0, limit.clamp(5.0));
            assert_eq!(2.0, limit.clamp(2.0));
            assert_eq!(1.0, limit.clamp(0.0));
            assert_eq!(1.0, limit.clamp(-2.0));
            assert_eq!(1.0, limit.clamp(-5.0));
        }
    }

    mod normalizer {
        use crate::prelude::common_processors::ValueNormalizer;

        #[test]
        fn test_standard_min_max_normalizer() {
            let normalizer = ValueNormalizer::standard_min_max(0.0..100.0);
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
            let normalizer = ValueNormalizer::symmetric_min_max(0.0..100.0);
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
            let normalizer = ValueNormalizer::custom_min_max(0.0..100.0, -4.0..4.0);
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
}
