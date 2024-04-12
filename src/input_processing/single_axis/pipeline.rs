//! Processing pipeline for single-axis inputs.

use crate::input_processing::AxisProcessor;
use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};

/// A dynamic sequence container of [`AxisProcessor`]s designed for processing input values.
///
/// # Warning
///
/// This flexibility may hinder compiler optimizations such as inlining or dead code elimination.
/// For performance-critical scenarios, consider creating your own processors for improved performance.
///
/// # Examples
///
/// ```rust
/// use leafwing_input_manager::prelude::*;
///
/// // Just a heads up, the default pipeline won't tweak values.
/// let pipeline = AxisProcessingPipeline::default();
/// assert_eq!(pipeline.process(1.5), 1.5);
///
/// // You can link up a sequence of processors to make a pipeline.
/// let mut pipeline = AxisProcessingPipeline::default()
///     .with(AxisSensitivity(2.0))
///     .with(AxisInverted);
///
/// // Now it doubles and flips values.
/// assert_eq!(pipeline.process(1.5), -3.0);
///
/// // You can also add a processor just like you would do with a Vec.
/// pipeline.push(AxisSensitivity(1.5));
///
/// // Now it triples and inverts values.
/// assert_eq!(pipeline.process(1.5), -4.5);
///
/// // Plus, you can switch out a processor at a specific index.
/// pipeline.set(2, AxisSensitivity(-2.0));
///
/// // Now it multiplies values by -4 and inverts the result.
/// assert_eq!(pipeline.process(1.5), 6.0);
///
/// // If needed, you can remove all processors.
/// pipeline.clear();
///
/// // Now it just leaves values as is.
/// assert_eq!(pipeline.process(1.5), 1.5);
/// ```
#[must_use]
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub struct AxisProcessingPipeline(pub(crate) Vec<Box<dyn AxisProcessor>>);

#[typetag::serde]
impl AxisProcessor for AxisProcessingPipeline {
    /// Computes the result by passing the `input_value` through this pipeline.
    #[must_use]
    #[inline]
    fn process(&self, input_value: f32) -> f32 {
        self.0
            .iter()
            .fold(input_value, |value, next| next.process(value))
    }
}

impl AxisProcessingPipeline {
    /// Appends the given [`AxisProcessor`] into this pipeline and returns `self`.
    #[inline]
    pub fn with(mut self, processor: impl AxisProcessor) -> Self {
        self.push(processor);
        self
    }

    /// Appends the given [`AxisProcessor`] into this pipeline.
    #[inline]
    pub fn push(&mut self, processor: impl AxisProcessor) {
        self.0.push(Box::new(processor));
    }

    /// Replaces the processor at the `index` with the given [`AxisProcessor`].
    #[inline]
    pub fn set(&mut self, index: usize, processor: impl AxisProcessor) {
        self.0[index] = Box::new(processor);
    }

    /// Removes all processors in this pipeline.
    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }
}

/// A trait for appending an [`AxisProcessor`] as the next processing step in the pipeline,
/// enabling further processing of input values.
pub trait WithAxisProcessor {
    /// Appends an [`AxisProcessor`] as the next processing step in the pipeline.
    fn with_processor(self, processor: impl AxisProcessor) -> Self;
}

impl WithAxisProcessor for Box<dyn AxisProcessor> {
    /// Creates a new boxed [`AxisProcessingPipeline`] with the existing steps
    /// and appends the given [`AxisProcessor`].
    fn with_processor(self, processor: impl AxisProcessor) -> Self {
        let pipeline = match Reflect::as_any(&*self).downcast_ref::<AxisProcessingPipeline>() {
            Some(pipeline) => pipeline.clone(),
            None => AxisProcessingPipeline(vec![self]),
        };
        Box::new(pipeline.with(processor))
    }
}

#[cfg(test)]
mod tests {
    use crate::input_processing::*;

    #[test]
    fn test_axis_processing_pipeline() {
        // Chain processors to make a new pipeline.
        let mut pipeline = AxisProcessingPipeline::default()
            .with(AxisSensitivity(4.0))
            .with(AxisInverted)
            .with(AxisSensitivity(4.0));

        pipeline.push(AxisInverted);

        pipeline.set(2, AxisSensitivity(6.0));

        // This pipeline now scales input values by a factor of 24.0
        assert_eq!(pipeline.process(2.0), 48.0);
        assert_eq!(pipeline.process(-3.0), -72.0);

        // Now it just leaves values as is.
        pipeline.clear();
        assert_eq!(pipeline, AxisProcessingPipeline::default());
        assert_eq!(pipeline.process(4.0), 4.0);
    }

    #[test]
    fn test_with_axis_processor() {
        let first = AxisSensitivity(2.0);
        let first_boxed: Box<dyn AxisProcessor> = Box::new(first);

        let second = AxisSensitivity(3.0);
        let merged_second = first_boxed.with_processor(second);
        let expected = AxisProcessingPipeline::default().with(first).with(second);
        let expected_boxed: Box<dyn AxisProcessor> = Box::new(expected);
        assert_eq!(merged_second, expected_boxed);

        let third = AxisSensitivity(4.0);
        let merged_third = merged_second.with_processor(third);
        let expected = AxisProcessingPipeline::default()
            .with(first)
            .with(second)
            .with(third);
        let expected_boxed: Box<dyn AxisProcessor> = Box::new(expected);
        assert_eq!(merged_third, expected_boxed);
    }
}
