//! Processors for dual-axis input values

use std::hash::{Hash, Hasher};

use bevy::{
    math::FloatOrd,
    prelude::{BVec2, Reflect, Vec2},
};
use serde::{Deserialize, Serialize};

use crate::input_processing::AxisProcessor;

pub use self::circle::*;
pub use self::custom::*;
pub use self::range::*;

mod circle;
mod custom;
mod range;

/// A processor for dual-axis input values,
/// accepting a [`Vec2`] input and producing a [`Vec2`] output.
#[must_use]
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum DualAxisProcessor {
    /// Converts input values into three discrete values along each axis,
    /// similar to [`Vec2::signum()`] but returning `0.0` for zero values.
    ///
    /// ```rust
    /// use bevy::prelude::*;
    /// use leafwing_input_manager::prelude::*;
    ///
    /// // 1.0 for positive values
    /// assert_eq!(DualAxisProcessor::Digital.process(Vec2::splat(2.5)), Vec2::ONE);
    /// assert_eq!(DualAxisProcessor::Digital.process(Vec2::splat(0.5)), Vec2::ONE);
    ///
    /// // 0.0 for zero values
    /// assert_eq!(DualAxisProcessor::Digital.process(Vec2::ZERO), Vec2::ZERO);
    /// assert_eq!(DualAxisProcessor::Digital.process(-Vec2::ZERO), Vec2::ZERO);
    ///
    /// // -1.0 for negative values
    /// assert_eq!(DualAxisProcessor::Digital.process(Vec2::splat(-0.5)), Vec2::NEG_ONE);
    /// assert_eq!(DualAxisProcessor::Digital.process(Vec2::splat(-2.5)), Vec2::NEG_ONE);
    ///
    /// // Mixed digital values
    /// assert_eq!(DualAxisProcessor::Digital.process(Vec2::new(0.5, -0.5)), Vec2::new(1.0, -1.0));
    /// assert_eq!(DualAxisProcessor::Digital.process(Vec2::new(-0.5, 0.5)), Vec2::new(-1.0, 1.0));
    /// ```
    Digital,

    /// A wrapper around [`DualAxisInverted`] to represent inversion.
    Inverted(DualAxisInverted),

    /// A wrapper around [`DualAxisSensitivity`] to represent sensitivity.
    Sensitivity(DualAxisSensitivity),

    /// A wrapper around [`DualAxisBounds`] to represent value bounds.
    ValueBounds(DualAxisBounds),

    /// A wrapper around [`DualAxisExclusion`] to represent unscaled deadzone.
    Exclusion(DualAxisExclusion),

    /// A wrapper around [`DualAxisDeadZone`] to represent scaled deadzone.
    DeadZone(DualAxisDeadZone),

    /// A wrapper around [`CircleBounds`] to represent circular value bounds.
    CircleBounds(CircleBounds),

    /// A wrapper around [`CircleExclusion`] to represent unscaled deadzone.
    CircleExclusion(CircleExclusion),

    /// A wrapper around [`CircleDeadZone`] to represent scaled deadzone.
    CircleDeadZone(CircleDeadZone),

    /// A user-defined processor that implements [`CustomDualAxisProcessor`].
    Custom(Box<dyn CustomDualAxisProcessor>),
}

impl DualAxisProcessor {
    /// Computes the result by processing the `input_value`.
    #[must_use]
    #[inline]
    pub fn process(&self, input_value: Vec2) -> Vec2 {
        match self {
            Self::Digital => Vec2::new(
                AxisProcessor::Digital.process(input_value.x),
                AxisProcessor::Digital.process(input_value.y),
            ),
            Self::Inverted(inversion) => inversion.invert(input_value),
            Self::Sensitivity(sensitivity) => sensitivity.scale(input_value),
            Self::ValueBounds(bounds) => bounds.clamp(input_value),
            Self::Exclusion(exclusion) => exclusion.exclude(input_value),
            Self::DeadZone(deadzone) => deadzone.normalize(input_value),
            Self::CircleBounds(bounds) => bounds.clamp(input_value),
            Self::CircleExclusion(exclusion) => exclusion.exclude(input_value),
            Self::CircleDeadZone(deadzone) => deadzone.normalize(input_value),
            Self::Custom(processor) => processor.process(input_value),
        }
    }
}

/// Provides methods for configuring and manipulating the processing pipeline for dual-axis input.
pub trait WithDualAxisProcessingPipelineExt: Sized {
    /// Resets the processing pipeline, removing any currently applied processors.
    fn reset_processing_pipeline(self) -> Self;

    /// Replaces the current processing pipeline with the given [`DualAxisProcessor`]s.
    fn replace_processing_pipeline(
        self,
        processors: impl IntoIterator<Item = DualAxisProcessor>,
    ) -> Self;

    /// Appends the given [`DualAxisProcessor`] as the next processing step.
    fn with_processor(self, processor: impl Into<DualAxisProcessor>) -> Self;

    /// Appends an [`DualAxisProcessor::Digital`] processor as the next processing step,
    /// similar to [`Vec2::signum`] but returning `0.0` for zero values.
    #[inline]
    fn digital(self) -> Self {
        self.with_processor(DualAxisProcessor::Digital)
    }

    /// Appends a [`DualAxisInverted::ALL`] processor as the next processing step,
    /// flipping the sign of values on both axes.
    #[inline]
    fn inverted(self) -> Self {
        self.with_processor(DualAxisInverted::ALL)
    }

    /// Appends a [`DualAxisInverted::ONLY_X`] processor as the next processing step,
    /// only flipping the sign of the X-axis values.
    #[inline]
    fn inverted_x(self) -> Self {
        self.with_processor(DualAxisInverted::ONLY_X)
    }

    /// Appends a [`DualAxisInverted::ONLY_Y`] processor as the next processing step,
    /// only flipping the sign of the Y-axis values.
    #[inline]
    fn inverted_y(self) -> Self {
        self.with_processor(DualAxisInverted::ONLY_Y)
    }

    /// Appends a [`DualAxisSensitivity`] processor as the next processing step,
    /// multiplying values on both axes with the given sensitivity factor.
    #[inline]
    fn sensitivity(self, sensitivity: f32) -> Self {
        self.with_processor(DualAxisSensitivity::all(sensitivity))
    }

    /// Appends a [`DualAxisSensitivity`] processor as the next processing step,
    /// only multiplying the X-axis values with the given sensitivity factor.
    #[inline]
    fn sensitivity_x(self, sensitivity: f32) -> Self {
        self.with_processor(DualAxisSensitivity::only_x(sensitivity))
    }

    /// Appends a [`DualAxisSensitivity`] processor as the next processing step,
    /// only multiplying the Y-axis values with the given sensitivity factor.
    #[inline]
    fn sensitivity_y(self, sensitivity: f32) -> Self {
        self.with_processor(DualAxisSensitivity::only_y(sensitivity))
    }

    /// Appends a [`DualAxisBounds`] processor as the next processing step,
    /// restricting values within the same range `[min, max]` on both axes.
    #[inline]
    fn with_bounds(self, min: f32, max: f32) -> Self {
        self.with_processor(DualAxisBounds::all(min, max))
    }

    /// Appends a [`DualAxisBounds`] processor as the next processing step,
    /// restricting values within the same range `[-threshold, threshold]` on both axes.
    #[inline]
    fn with_bounds_symmetric(self, threshold: f32) -> Self {
        self.with_processor(DualAxisBounds::symmetric_all(threshold))
    }

    /// Appends a [`DualAxisBounds`] processor as the next processing step,
    /// only restricting values within the range `[min, max]` on the X-axis.
    #[inline]
    fn with_bounds_x(self, min: f32, max: f32) -> Self {
        self.with_processor(DualAxisBounds::only_x(min, max))
    }

    /// Appends a [`DualAxisBounds`] processor as the next processing step,
    /// restricting values within the range `[-threshold, threshold]` on the X-axis.
    #[inline]
    fn with_bounds_x_symmetric(self, threshold: f32) -> Self {
        self.with_processor(DualAxisBounds::symmetric_all(threshold))
    }

    /// Appends a [`DualAxisBounds`] processor as the next processing step,
    /// only restricting values within the range `[min, max]` on the Y-axis.
    #[inline]
    fn with_bounds_y(self, min: f32, max: f32) -> Self {
        self.with_processor(DualAxisBounds::only_y(min, max))
    }

    /// Appends a [`DualAxisBounds`] processor as the next processing step,
    /// restricting values within the range `[-threshold, threshold]` on the Y-axis.
    #[inline]
    fn with_bounds_y_symmetric(self, threshold: f32) -> Self {
        self.with_processor(DualAxisBounds::symmetric_all(threshold))
    }

    /// Appends a [`CircleBounds`] processor as the next processing step,
    /// restricting values to a `max` magnitude.
    #[inline]
    fn with_circle_bounds(self, max: f32) -> Self {
        self.with_processor(CircleBounds::new(max))
    }

    /// Appends a [`DualAxisDeadZone`] processor as the next processing step,
    /// excluding values within the dead zone range `[min, max]` on both axes,
    /// treating them as zeros, then normalizing non-excluded input values into the "live zone",
    /// the remaining range within the [`DualAxisBounds::symmetric_all(1.0)`](DualAxisBounds::default)
    /// after dead zone exclusion.
    #[inline]
    fn with_deadzone(self, min: f32, max: f32) -> Self {
        self.with_processor(DualAxisDeadZone::all(min, max))
    }

    /// Appends a [`DualAxisDeadZone`] processor as the next processing step,
    /// excluding values within the dead zone range `[-threshold, threshold]` on both axes,
    /// treating them as zeros, then normalizing non-excluded input values into the "live zone",
    /// the remaining range within the [`DualAxisBounds::symmetric_all(1.0)`](DualAxisBounds::default)
    /// after dead zone exclusion.
    #[inline]
    fn with_deadzone_symmetric(self, threshold: f32) -> Self {
        self.with_processor(DualAxisDeadZone::symmetric_all(threshold))
    }

    /// Appends a [`DualAxisDeadZone`] processor as the next processing step,
    /// excluding values within the dead zone range `[min, max]` on the X-axis,
    /// treating them as zeros, then normalizing non-excluded input values into the "live zone",
    /// the remaining range within the [`DualAxisBounds::symmetric_all(1.0)`](DualAxisBounds::default)
    /// after dead zone exclusion.
    #[inline]
    fn with_deadzone_x(self, min: f32, max: f32) -> Self {
        self.with_processor(DualAxisDeadZone::only_x(min, max))
    }

    /// Appends a [`DualAxisDeadZone`] processor as the next processing step,
    /// excluding values within the dead zone range `[-threshold, threshold]` on the X-axis,
    /// treating them as zeros, then normalizing non-excluded input values into the "live zone",
    /// the remaining range within the [`DualAxisBounds::symmetric_all(1.0)`](DualAxisBounds::default)
    /// after dead zone exclusion.
    #[inline]
    fn with_deadzone_x_symmetric(self, threshold: f32) -> Self {
        self.with_processor(DualAxisDeadZone::symmetric_only_x(threshold))
    }

    /// Appends a [`DualAxisDeadZone`] processor as the next processing step,
    /// excluding values within the dead zone range `[min, max]` on the Y-axis,
    /// treating them as zeros, then normalizing non-excluded input values into the "live zone",
    /// the remaining range within the [`DualAxisBounds::symmetric_all(1.0)`](DualAxisBounds::default)
    /// after dead zone exclusion.
    #[inline]
    fn with_deadzone_y(self, min: f32, max: f32) -> Self {
        self.with_processor(DualAxisDeadZone::only_y(min, max))
    }

    /// Appends a [`DualAxisDeadZone`] processor as the next processing step,
    /// excluding values within the deadzone range `[-threshold, threshold]` on the Y-axis,
    /// treating them as zeros, then normalizing non-excluded input values into the "live zone",
    /// the remaining range within the [`DualAxisBounds::symmetric_all(1.0)`](DualAxisBounds::default)
    /// after dead zone exclusion.
    #[inline]
    fn with_deadzone_y_symmetric(self, threshold: f32) -> Self {
        self.with_processor(DualAxisDeadZone::symmetric_only_y(threshold))
    }

    /// Appends a [`CircleDeadZone`] processor as the next processing step,
    /// ignoring values below a `min` magnitude, treating them as zeros,
    /// then normalizing non-excluded input values into the "live zone",
    /// the remaining range within the [`CircleBounds::new(1.0)`](CircleBounds::default)
    /// after dead zone exclusion.
    #[inline]
    fn with_circle_deadzone(self, min: f32) -> Self {
        self.with_processor(CircleDeadZone::new(min))
    }

    /// Appends a [`DualAxisExclusion`] processor as the next processing step,
    /// ignoring values within the dead zone range `[min, max]` on both axes,
    /// treating them as zeros.
    #[inline]
    fn with_deadzone_unscaled(self, min: f32, max: f32) -> Self {
        self.with_processor(DualAxisExclusion::all(min, max))
    }

    /// Appends a [`DualAxisExclusion`] processor as the next processing step,
    /// ignoring values within the dead zone range `[-threshold, threshold]` on both axes,
    /// treating them as zeros.
    #[inline]
    fn with_deadzone_symmetric_unscaled(self, threshold: f32) -> Self {
        self.with_processor(DualAxisExclusion::symmetric_all(threshold))
    }

    /// Appends a [`DualAxisExclusion`] processor as the next processing step,
    /// only ignoring values within the dead zone range `[min, max]` on the X-axis,
    /// treating them as zeros.
    #[inline]
    fn with_deadzone_x_unscaled(self, min: f32, max: f32) -> Self {
        self.with_processor(DualAxisExclusion::only_x(min, max))
    }

    /// Appends a [`DualAxisExclusion`] processor as the next processing step,
    /// only ignoring values within the dead zone range `[-threshold, threshold]` on the X-axis,
    /// treating them as zeros.
    #[inline]
    fn with_deadzone_x_symmetric_unscaled(self, threshold: f32) -> Self {
        self.with_processor(DualAxisExclusion::symmetric_only_x(threshold))
    }

    /// Appends a [`DualAxisExclusion`] processor as the next processing step,
    /// only ignoring values within the dead zone range `[min, max]` on the Y-axis,
    /// treating them as zeros.
    #[inline]
    fn with_deadzone_y_unscaled(self, min: f32, max: f32) -> Self {
        self.with_processor(DualAxisExclusion::only_y(min, max))
    }

    /// Appends a [`DualAxisExclusion`] processor as the next processing step,
    /// only ignoring values within the dead zone range `[-threshold, threshold]` on the Y-axis,
    /// treating them as zeros.
    #[inline]
    fn with_deadzone_y_symmetric_unscaled(self, threshold: f32) -> Self {
        self.with_processor(DualAxisExclusion::symmetric_only_y(threshold))
    }

    /// Appends a [`CircleExclusion`] processor as the next processing step,
    /// ignoring values below a `min` magnitude, treating them as zeros.
    #[inline]
    fn with_circle_deadzone_unscaled(self, min: f32) -> Self {
        self.with_processor(CircleExclusion::new(min))
    }
}

/// Flips the sign of dual-axis input values, resulting in a directional reversal of control.
///
/// ```rust
/// use bevy::prelude::*;
/// use leafwing_input_manager::prelude::*;
///
/// let value = Vec2::new(1.5, 2.0);
/// let Vec2 { x, y } = value;
///
/// assert_eq!(DualAxisInverted::ALL.invert(value), -value);
/// assert_eq!(DualAxisInverted::ALL.invert(-value), value);
///
/// assert_eq!(DualAxisInverted::ONLY_X.invert(value), Vec2::new(-x, y));
/// assert_eq!(DualAxisInverted::ONLY_X.invert(-value), Vec2::new(x, -y));
///
/// assert_eq!(DualAxisInverted::ONLY_Y.invert(value), Vec2::new(x, -y));
/// assert_eq!(DualAxisInverted::ONLY_Y.invert(-value), Vec2::new(-x, y));
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct DualAxisInverted(Vec2);

impl DualAxisInverted {
    /// The [`DualAxisInverted`] that inverts both axes.
    pub const ALL: Self = Self(Vec2::NEG_ONE);

    /// The [`DualAxisInverted`] that only inverts the X-axis inputs.
    pub const ONLY_X: Self = Self(Vec2::new(-1.0, 1.0));

    /// The [`DualAxisInverted`] that only inverts the Y-axis inputs.
    pub const ONLY_Y: Self = Self(Vec2::new(1.0, -1.0));

    /// Are inputs inverted on both axes?
    #[must_use]
    #[inline]
    pub fn inverted(&self) -> BVec2 {
        self.0.cmpeq(Vec2::NEG_ONE)
    }

    /// Multiples the `input_value` by the specified inversion vector.
    #[must_use]
    #[inline]
    pub fn invert(&self, input_value: Vec2) -> Vec2 {
        self.0 * input_value
    }
}

impl From<DualAxisInverted> for DualAxisProcessor {
    fn from(value: DualAxisInverted) -> Self {
        Self::Inverted(value)
    }
}

impl Eq for DualAxisInverted {}

impl Hash for DualAxisInverted {
    fn hash<H: Hasher>(&self, state: &mut H) {
        FloatOrd(self.0.x).hash(state);
        FloatOrd(self.0.y).hash(state);
    }
}

/// Scales dual-axis input values using a specified multiplier to fine-tune the responsiveness of control.
///
/// ```rust
/// use bevy::prelude::*;
/// use leafwing_input_manager::prelude::*;
///
/// let value = Vec2::new(1.5, 2.5);
/// let Vec2 { x, y } = value;
///
/// // Negated X and halved Y
/// let neg_x_half_y = DualAxisSensitivity::new(-1.0, 0.5);
/// assert_eq!(neg_x_half_y.scale(value).x, -x);
/// assert_eq!(neg_x_half_y.scale(value).y, 0.5 * y);
///
/// // Doubled X and doubled Y
/// let double = DualAxisSensitivity::all(2.0);
/// assert_eq!(double.scale(value), 2.0 * value);
///
/// // Halved X
/// let half_x = DualAxisSensitivity::only_x(0.5);
/// assert_eq!(half_x.scale(value).x, 0.5 * x);
/// assert_eq!(half_x.scale(value).y, y);
///
/// // Negated and doubled Y
/// let neg_double_y = DualAxisSensitivity::only_y(-2.0);
/// assert_eq!(neg_double_y.scale(value).x, x);
/// assert_eq!(neg_double_y.scale(value).y, -2.0 * y);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct DualAxisSensitivity(pub(crate) Vec2);

impl DualAxisSensitivity {
    /// Creates a [`DualAxisSensitivity`] with the given values for each axis separately.
    #[inline]
    pub const fn new(sensitivity_x: f32, sensitivity_y: f32) -> Self {
        Self(Vec2::new(sensitivity_x, sensitivity_y))
    }

    /// Creates a [`DualAxisSensitivity`] with the same value for both axes.
    #[inline]
    pub const fn all(sensitivity: f32) -> Self {
        Self::new(sensitivity, sensitivity)
    }

    /// Creates a [`DualAxisSensitivity`] that only affects the X-axis using the given value.
    #[inline]
    pub const fn only_x(sensitivity: f32) -> Self {
        Self::new(sensitivity, 1.0)
    }

    /// Creates a [`DualAxisSensitivity`] that only affects the Y-axis using the given value.
    #[inline]
    pub const fn only_y(sensitivity: f32) -> Self {
        Self::new(1.0, sensitivity)
    }

    /// Returns the sensitivity values.
    #[must_use]
    #[inline]
    pub fn sensitivities(&self) -> Vec2 {
        self.0
    }

    /// Multiples the `input_value` by the specified sensitivity vector.
    #[must_use]
    #[inline]
    pub fn scale(&self, input_value: Vec2) -> Vec2 {
        self.0 * input_value
    }
}

impl From<DualAxisSensitivity> for DualAxisProcessor {
    fn from(value: DualAxisSensitivity) -> Self {
        Self::Sensitivity(value)
    }
}

impl Eq for DualAxisSensitivity {}

impl Hash for DualAxisSensitivity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        FloatOrd(self.0.x).hash(state);
        FloatOrd(self.0.y).hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dual_axis_inverted() {
        let all = DualAxisInverted::ALL;
        assert_eq!(all.inverted(), BVec2::TRUE);

        let only_x = DualAxisInverted::ONLY_X;
        assert_eq!(only_x.inverted(), BVec2::new(true, false));

        let only_y = DualAxisInverted::ONLY_Y;
        assert_eq!(only_y.inverted(), BVec2::new(false, true));

        for x in -300..300 {
            let x = x as f32 * 0.01;

            for y in -300..300 {
                let y = y as f32 * 0.01;
                let value = Vec2::new(x, y);

                let processor = DualAxisProcessor::Inverted(all);
                assert_eq!(DualAxisProcessor::from(all), processor);
                assert_eq!(processor.process(value), all.invert(value));
                assert_eq!(all.invert(value), -value);
                assert_eq!(all.invert(-value), value);

                let processor = DualAxisProcessor::Inverted(only_x);
                assert_eq!(DualAxisProcessor::from(only_x), processor);
                assert_eq!(processor.process(value), only_x.invert(value));
                assert_eq!(only_x.invert(value), Vec2::new(-x, y));
                assert_eq!(only_x.invert(-value), Vec2::new(x, -y));

                let processor = DualAxisProcessor::Inverted(only_y);
                assert_eq!(DualAxisProcessor::from(only_y), processor);
                assert_eq!(processor.process(value), only_y.invert(value));
                assert_eq!(only_y.invert(value), Vec2::new(x, -y));
                assert_eq!(only_y.invert(-value), Vec2::new(-x, y));
            }
        }
    }

    #[test]
    fn test_dual_axis_sensitivity() {
        for x in -300..300 {
            let x = x as f32 * 0.01;

            for y in -300..300 {
                let y = y as f32 * 0.01;
                let value = Vec2::new(x, y);

                let sensitivity = x;

                let all = DualAxisSensitivity::all(sensitivity);
                let processor = DualAxisProcessor::Sensitivity(all);
                assert_eq!(DualAxisProcessor::from(all), processor);
                assert_eq!(processor.process(value), all.scale(value));
                assert_eq!(all.sensitivities(), Vec2::splat(sensitivity));
                assert_eq!(all.scale(value), sensitivity * value);

                let only_x = DualAxisSensitivity::only_x(sensitivity);
                let processor = DualAxisProcessor::Sensitivity(only_x);
                assert_eq!(DualAxisProcessor::from(only_x), processor);
                assert_eq!(processor.process(value), only_x.scale(value));
                assert_eq!(only_x.sensitivities(), Vec2::new(sensitivity, 1.0));
                assert_eq!(only_x.scale(value).x, x * sensitivity);
                assert_eq!(only_x.scale(value).y, y);

                let only_y = DualAxisSensitivity::only_y(sensitivity);
                let processor = DualAxisProcessor::Sensitivity(only_y);
                assert_eq!(DualAxisProcessor::from(only_y), processor);
                assert_eq!(processor.process(value), only_y.scale(value));
                assert_eq!(only_y.sensitivities(), Vec2::new(1.0, sensitivity));
                assert_eq!(only_y.scale(value).x, x);
                assert_eq!(only_y.scale(value).y, y * sensitivity);

                let sensitivity2 = y;
                let separate = DualAxisSensitivity::new(sensitivity, sensitivity2);
                let processor = DualAxisProcessor::Sensitivity(separate);
                assert_eq!(DualAxisProcessor::from(separate), processor);
                assert_eq!(processor.process(value), separate.scale(value));
                assert_eq!(
                    separate.sensitivities(),
                    Vec2::new(sensitivity, sensitivity2)
                );
                assert_eq!(separate.scale(value).x, x * sensitivity);
                assert_eq!(separate.scale(value).y, y * sensitivity2);
            }
        }
    }
}
