//! Processors for single-axis input values

use std::hash::{Hash, Hasher};
use std::sync::Arc;

use bevy::prelude::Reflect;
use bevy::utils::FloatOrd;
use serde::{Deserialize, Serialize};

pub use self::custom::*;
pub use self::range::*;

mod custom;
mod range;

/// A processor for single-axis input values,
/// accepting a `f32` input and producing a `f32` output.
#[must_use]
#[non_exhaustive]
#[derive(Default, Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub enum AxisProcessor {
    /// No processor is applied.
    #[default]
    None,

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

    /// Processes input values sequentially through a sequence of [`AxisProcessor`]s.
    ///
    /// For a straightforward creation of a [`AxisProcessor::Pipeline`],
    /// you can use [`AxisProcessor::with_processor`] or [`FromIterator<AxisProcessor>::from_iter`] methods.
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use leafwing_input_manager::prelude::*;
    ///
    /// let expected = AxisProcessor::Pipeline(vec![
    ///     Arc::new(AxisProcessor::Inverted),
    ///     Arc::new(AxisProcessor::Sensitivity(2.0)),
    /// ]);
    ///
    /// assert_eq!(
    ///     expected,
    ///     AxisProcessor::Inverted.with_processor(AxisProcessor::Sensitivity(2.0))
    /// );
    ///
    /// assert_eq!(
    ///     expected,
    ///     AxisProcessor::from_iter([
    ///         AxisProcessor::Inverted,
    ///         AxisProcessor::Sensitivity(2.0),
    ///     ])
    /// );
    /// ```
    Pipeline(Vec<Arc<AxisProcessor>>),

    /// A user-defined processor that implements [`CustomAxisProcessor`].
    Custom(Box<dyn CustomAxisProcessor>),
}

impl AxisProcessor {
    /// Computes the result by processing the `input_value`.
    #[must_use]
    #[inline]
    pub fn process(&self, input_value: f32) -> f32 {
        match self {
            Self::None => input_value,
            Self::Inverted => -input_value,
            Self::Sensitivity(sensitivity) => sensitivity * input_value,
            Self::ValueBounds(bounds) => bounds.clamp(input_value),
            Self::Exclusion(exclusion) => exclusion.exclude(input_value),
            Self::DeadZone(deadzone) => deadzone.normalize(input_value),
            Self::Pipeline(sequence) => sequence
                .iter()
                .fold(input_value, |value, next| next.process(value)),
            Self::Custom(processor) => processor.process(input_value),
        }
    }

    /// Appends the given `next_processor` as the next processing step.
    ///
    /// - If either processor is [`AxisProcessor::None`], returns the other.
    /// - If the current processor is [`AxisProcessor::Pipeline`], pushes the other into it.
    /// - If the given processor is [`AxisProcessor::Pipeline`], prepends the current one into it.
    /// - If both processors are [`AxisProcessor::Pipeline`], merges the two pipelines.
    /// - If neither processor is [`AxisProcessor::None`] nor a pipeline,
    ///     creates a new pipeline containing them.
    #[inline]
    pub fn with_processor(self, next_processor: impl Into<AxisProcessor>) -> Self {
        let other = next_processor.into();
        match (self.clone(), other.clone()) {
            (_, Self::None) => self,
            (Self::None, _) => other,
            (Self::Pipeline(mut self_seq), Self::Pipeline(mut next_seq)) => {
                self_seq.append(&mut next_seq);
                Self::Pipeline(self_seq)
            }
            (Self::Pipeline(mut self_seq), _) => {
                self_seq.push(Arc::new(other));
                Self::Pipeline(self_seq)
            }
            (_, Self::Pipeline(mut next_seq)) => {
                next_seq.insert(0, Arc::new(self));
                Self::Pipeline(next_seq)
            }
            (_, _) => Self::Pipeline(vec![Arc::new(self), Arc::new(other)]),
        }
    }
}

impl FromIterator<AxisProcessor> for AxisProcessor {
    fn from_iter<T: IntoIterator<Item = AxisProcessor>>(iter: T) -> Self {
        Self::Pipeline(iter.into_iter().map(Arc::new).collect())
    }
}

impl Eq for AxisProcessor {}

impl Hash for AxisProcessor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Self::None => {}
            Self::Inverted => {}
            Self::Sensitivity(sensitivity) => FloatOrd(*sensitivity).hash(state),
            Self::ValueBounds(bounds) => bounds.hash(state),
            Self::Exclusion(exclusion) => exclusion.hash(state),
            Self::DeadZone(deadzone) => deadzone.hash(state),
            Self::Pipeline(sequence) => sequence.hash(state),
            Self::Custom(processor) => processor.hash(state),
        }
    }
}

/// Provides methods for configuring and manipulating the processing pipeline for single-axis input.
pub trait WithAxisProcessor: Sized {
    /// Remove the current used [`AxisProcessor`].
    fn no_processor(self) -> Self;

    /// Replaces the current pipeline with the specified [`AxisProcessor`].
    fn replace_processor(self, processor: impl Into<AxisProcessor>) -> Self;

    /// Appends the given [`AxisProcessor`] as the next processing step.
    fn with_processor(self, processor: impl Into<AxisProcessor>) -> Self;

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
    fn filter(self, min: f32, max: f32) -> Self {
        self.with_processor(AxisBounds::new(min, max))
    }

    /// Appends an [`AxisBounds`] processor as the next processing step,
    /// restricting values to a `max` magnitude.
    #[inline]
    fn filter_magnitude(self, max: f32) -> Self {
        self.with_processor(AxisBounds::magnitude(max))
    }

    /// Appends an [`AxisDeadZone`] processor as the next processing step,
    /// excluding values within the dead zone range `[negative_max, positive_min]` on the axis,
    /// treating them as zeros, then normalizing non-excluded input values into the "live zone",
    /// the remaining range within the [`AxisBounds::magnitude(1.0)`](AxisBounds::default)
    /// after dead zone exclusion.
    #[inline]
    fn deadzone(self, negative_max: f32, positive_min: f32) -> Self {
        self.with_processor(AxisDeadZone::new(negative_max, positive_min))
    }

    /// Appends an [`AxisDeadZone`] processor as the next processing step,
    /// excluding values below a `min` magnitude, treating them as zeros
    /// then normalizing non-excluded input values into the "live zone",
    /// the remaining range within the [`AxisBounds::magnitude(1.0)`](AxisBounds::default)
    /// after dead zone exclusion.
    #[inline]
    fn deadzone_magnitude(self, min: f32) -> Self {
        self.with_processor(AxisDeadZone::magnitude(min))
    }

    /// Appends an [`AxisExclusion`] processor as the next processing step,
    /// ignoring values within the dead zone range `[negative_max, positive_min]` on the axis,
    /// treating them as zeros.
    #[inline]
    fn deadzone_unscaled(self, negative_max: f32, positive_min: f32) -> Self {
        self.with_processor(AxisExclusion::new(negative_max, positive_min))
    }

    /// Appends an [`AxisExclusion`] processor as the next processing step,
    /// ignoring values below a `min` magnitude, treating them as zeros.
    #[inline]
    fn deadzone_magnitude_unscaled(self, min: f32) -> Self {
        self.with_processor(AxisExclusion::magnitude(min))
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

    #[test]
    fn test_axis_processor_pipeline() {
        let pipeline = AxisProcessor::Pipeline(vec![
            Arc::new(AxisProcessor::Inverted),
            Arc::new(AxisProcessor::Sensitivity(2.0)),
        ]);

        for value in -300..300 {
            let value = value as f32 * 0.01;

            assert_eq!(pipeline.process(value), value * -2.0);
        }
    }

    #[test]
    fn test_axis_processor_from_iter() {
        assert_eq!(
            AxisProcessor::from_iter([]),
            AxisProcessor::Pipeline(vec![])
        );

        assert_eq!(
            AxisProcessor::from_iter([AxisProcessor::Inverted]),
            AxisProcessor::Pipeline(vec![Arc::new(AxisProcessor::Inverted)]),
        );

        assert_eq!(
            AxisProcessor::from_iter([AxisProcessor::Inverted, AxisProcessor::Sensitivity(2.0)]),
            AxisProcessor::Pipeline(vec![
                Arc::new(AxisProcessor::Inverted),
                Arc::new(AxisProcessor::Sensitivity(2.0)),
            ])
        );
    }
}
