//! Processors for single-axis input values

use std::hash::{Hash, Hasher};

use bevy::{math::FloatOrd, prelude::Reflect};
use serde::{Deserialize, Serialize};

pub use self::custom::*;
pub use self::range::*;

mod custom;
mod range;

/// A processor for single-axis input values,
/// accepting a `f32` input and producing a `f32` output.
#[must_use]
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub enum AxisProcessor {
    /// Converts input values into three discrete values,
    /// similar to [`f32::signum()`] but returning `0.0` for zero values.
    ///
    /// ```rust
    /// use leafwing_input_manager::prelude::*;
    ///
    /// // 1.0 for positive values
    /// assert_eq!(AxisProcessor::Digital.process(2.5), 1.0);
    /// assert_eq!(AxisProcessor::Digital.process(0.5), 1.0);
    ///
    /// // 0.0 for zero values
    /// assert_eq!(AxisProcessor::Digital.process(0.0), 0.0);
    /// assert_eq!(AxisProcessor::Digital.process(-0.0), 0.0);
    ///
    /// // -1.0 for negative values
    /// assert_eq!(AxisProcessor::Digital.process(-0.5), -1.0);
    /// assert_eq!(AxisProcessor::Digital.process(-2.5), -1.0);
    /// ```
    Digital,

    /// Flips the sign of input values, resulting in a directional reversal of control.
    ///
    /// ```rust
    /// use leafwing_input_manager::prelude::*;
    ///
    /// assert_eq!(AxisProcessor::Inverted.process(2.5), -2.5);
    /// assert_eq!(AxisProcessor::Inverted.process(-2.5), 2.5);
    /// ```
    Inverted,

    /// Scales input values using a specified multiplier to fine-tune the responsiveness of control.
    ///
    /// ```rust
    /// use leafwing_input_manager::prelude::*;
    ///
    /// // Doubled!
    /// assert_eq!(AxisProcessor::Sensitivity(2.0).process(2.0), 4.0);
    ///
    /// // Halved!
    /// assert_eq!(AxisProcessor::Sensitivity(0.5).process(2.0), 1.0);
    ///
    /// // Negated and halved!
    /// assert_eq!(AxisProcessor::Sensitivity(-0.5).process(2.0), -1.0);
    /// ```
    Sensitivity(f32),

    /// A wrapper around [`AxisBounds`] to represent value bounds.
    ValueBounds(AxisBounds),

    /// A wrapper around [`AxisExclusion`] to represent unscaled deadzone.
    Exclusion(AxisExclusion),

    /// A wrapper around [`AxisDeadZone`] to represent scaled deadzone.
    DeadZone(AxisDeadZone),

    /// A user-defined processor that implements [`CustomAxisProcessor`].
    Custom(Box<dyn CustomAxisProcessor>),
}

impl AxisProcessor {
    /// Computes the result by processing the `input_value`.
    #[must_use]
    #[inline]
    pub fn process(&self, input_value: f32) -> f32 {
        match self {
            Self::Digital => {
                if input_value == 0.0 {
                    0.0
                } else {
                    input_value.signum()
                }
            }
            Self::Inverted => -input_value,
            Self::Sensitivity(sensitivity) => sensitivity * input_value,
            Self::ValueBounds(bounds) => bounds.clamp(input_value),
            Self::Exclusion(exclusion) => exclusion.exclude(input_value),
            Self::DeadZone(deadzone) => deadzone.normalize(input_value),
            Self::Custom(processor) => processor.process(input_value),
        }
    }
}

impl Eq for AxisProcessor {}

impl Hash for AxisProcessor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Self::Digital => {}
            Self::Inverted => {}
            Self::Sensitivity(sensitivity) => FloatOrd(*sensitivity).hash(state),
            Self::ValueBounds(bounds) => bounds.hash(state),
            Self::Exclusion(exclusion) => exclusion.hash(state),
            Self::DeadZone(deadzone) => deadzone.hash(state),
            Self::Custom(processor) => processor.hash(state),
        }
    }
}

/// Provides methods for configuring and manipulating the processing pipeline for single-axis input.
pub trait WithAxisProcessingPipelineExt: Sized {
    /// Resets the processing pipeline, removing any currently applied processors.
    fn reset_processing_pipeline(self) -> Self;

    /// Replaces the current processing pipeline with the given [`AxisProcessor`]s.
    fn replace_processing_pipeline(
        self,
        processors: impl IntoIterator<Item = AxisProcessor>,
    ) -> Self;

    /// Appends the given [`AxisProcessor`] as the next processing step.
    fn with_processor(self, processor: impl Into<AxisProcessor>) -> Self;

    /// Appends an [`AxisProcessor::Digital`] processor as the next processing step,
    /// similar to [`f32::signum`] but returning `0.0` for zero values.
    #[inline]
    fn digital(self) -> Self {
        self.with_processor(AxisProcessor::Digital)
    }

    /// Appends an [`AxisProcessor::Inverted`] processor as the next processing step,
    /// flipping the sign of values on the axis.
    #[inline]
    fn inverted(self) -> Self {
        self.with_processor(AxisProcessor::Inverted)
    }

    /// Appends an [`AxisProcessor::Sensitivity`] processor as the next processing step,
    /// multiplying values on the axis with the given sensitivity factor.
    #[inline]
    fn sensitivity(self, sensitivity: f32) -> Self {
        self.with_processor(AxisProcessor::Sensitivity(sensitivity))
    }

    /// Appends an [`AxisBounds`] processor as the next processing step,
    /// restricting values within the range `[min, max]` on the axis.
    #[inline]
    fn with_bounds(self, min: f32, max: f32) -> Self {
        self.with_processor(AxisBounds::new(min, max))
    }

    /// Appends an [`AxisBounds`] processor as the next processing step,
    /// restricting values to a `threshold` magnitude.
    #[inline]
    fn with_bounds_symmetric(self, threshold: f32) -> Self {
        self.with_processor(AxisBounds::symmetric(threshold))
    }

    /// Appends an [`AxisDeadZone`] processor as the next processing step,
    /// excluding values within the dead zone range `[negative_max, positive_min]` on the axis,
    /// treating them as zeros, then normalizing non-excluded input values into the "live zone",
    /// the remaining range within the [`AxisBounds::magnitude(1.0)`](AxisBounds::default)
    /// after dead zone exclusion.
    #[inline]
    fn with_deadzone(self, negative_max: f32, positive_min: f32) -> Self {
        self.with_processor(AxisDeadZone::new(negative_max, positive_min))
    }

    /// Appends an [`AxisDeadZone`] processor as the next processing step,
    /// excluding values within the dead zone range `[-threshold, threshold]` on the axis,
    /// treating them as zeros, then normalizing non-excluded input values into the "live zone",
    /// the remaining range within the [`AxisBounds::magnitude(1.0)`](AxisBounds::default)
    /// after dead zone exclusion.
    #[inline]
    fn with_deadzone_symmetric(self, threshold: f32) -> Self {
        self.with_processor(AxisDeadZone::symmetric(threshold))
    }

    /// Appends an [`AxisExclusion`] processor as the next processing step,
    /// ignoring values within the dead zone range `[negative_max, positive_min]` on the axis,
    /// treating them as zeros.
    #[inline]
    fn with_deadzone_unscaled(self, negative_max: f32, positive_min: f32) -> Self {
        self.with_processor(AxisExclusion::new(negative_max, positive_min))
    }

    /// Appends an [`AxisExclusion`] processor as the next processing step,
    /// ignoring values within the dead zone range `[-threshold, threshold]` on the axis,
    /// treating them as zeros.
    #[inline]
    fn with_deadzone_symmetric_unscaled(self, threshold: f32) -> Self {
        self.with_processor(AxisExclusion::symmetric(threshold))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_axis_inversion_processor() {
        for value in -300..300 {
            let value = value as f32 * 0.01;

            assert_eq!(AxisProcessor::Inverted.process(value), -value);
            assert_eq!(AxisProcessor::Inverted.process(-value), value);
        }
    }

    #[test]
    fn test_axis_sensitivity_processor() {
        for value in -300..300 {
            let value = value as f32 * 0.01;

            for sensitivity in -300..300 {
                let sensitivity = sensitivity as f32 * 0.01;

                let processor = AxisProcessor::Sensitivity(sensitivity);
                assert_eq!(processor.process(value), sensitivity * value);
            }
        }
    }
}
