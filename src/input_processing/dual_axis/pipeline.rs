//! Processing pipeline for dual-axis inputs.

use bevy::prelude::{Reflect, Vec2};
use leafwing_input_manager_macros::serde_typetag;
use serde::{Deserialize, Serialize};

use super::DualAxisProcessor;
use crate as leafwing_input_manager;

/// A dynamic sequence container of [`DualAxisProcessor`]s designed for processing input values.
///
/// # Warning
///
/// This flexibility may hinder compiler optimizations such as inlining or dead code elimination.
/// For performance-critical scenarios, consider creating your own processors for improved performance.
///
/// # Examples
///
/// ```rust
/// use bevy::prelude::*;
/// use leafwing_input_manager::prelude::*;
///
/// let input_value = Vec2::splat(1.5);
///
/// // Just a heads up, the default pipeline won't tweak values.
/// let pipeline = DualAxisProcessingPipeline::default();
/// assert_eq!(pipeline.process(input_value), input_value);
///
/// // You can link up a sequence of processors to make a pipeline.
/// let mut pipeline = DualAxisProcessingPipeline::default()
///     .with(DualAxisInverted::ALL)
///     .with(DualAxisSensitivity::all(2.0));
///
/// // Now it inverts and doubles values.
/// assert_eq!(pipeline.process(input_value), -2.0 * input_value);
///
/// // You can also add a processor just like you would do with a Vec.
/// pipeline.push(DualAxisSensitivity::only_x(1.5));
///
/// // Now it inverts values and multiplies the results by [3.0, 2.0]
/// assert_eq!(pipeline.process(input_value), Vec2::new(-3.0, -2.0) * input_value);
///
/// // Plus, you can switch out a processor at a specific index.
/// pipeline.set(1, DualAxisSensitivity::all(3.0));
///
/// // Now it inverts values and multiplies the results by [4.5, 3.0]
/// assert_eq!(pipeline.process(input_value), Vec2::new(-4.5, -3.0) * input_value);
///
/// // If needed, you can remove all processors.
/// pipeline.clear();
///
/// // Now it just leaves values as is.
/// assert_eq!(pipeline.process(input_value), input_value);
/// ```
#[must_use]
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub struct DualAxisProcessingPipeline(pub(crate) Vec<Box<dyn DualAxisProcessor>>);

#[serde_typetag]
impl DualAxisProcessor for DualAxisProcessingPipeline {
    /// Computes the result by passing the `input_value` through this pipeline.
    #[must_use]
    #[inline]
    fn process(&self, input_value: Vec2) -> Vec2 {
        self.0
            .iter()
            .fold(input_value, |value, next| next.process(value))
    }
}

impl DualAxisProcessingPipeline {
    /// Appends the given [`DualAxisProcessor`] into this pipeline and returns `self`.
    #[inline]
    pub fn with(mut self, processor: impl DualAxisProcessor) -> Self {
        self.push(processor);
        self
    }

    /// Appends the given [`DualAxisProcessor`] into this pipeline.
    #[inline]
    pub fn push(&mut self, processor: impl DualAxisProcessor) {
        self.0.push(Box::new(processor));
    }

    /// Replaces the processor at the `index` with the given [`DualAxisProcessor`].
    #[inline]
    pub fn set(&mut self, index: usize, processor: impl DualAxisProcessor) {
        self.0[index] = Box::new(processor);
    }

    /// Removes all processors in this pipeline.
    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }
}

/// A trait for appending a [`DualAxisProcessor`] as the next processing step in the pipeline,
/// enabling further processing of input values.
pub trait WithDualAxisProcessor {
    /// Appends a [`DualAxisProcessor`] as the next processing step in the pipeline.
    fn with_processor(self, processor: impl DualAxisProcessor) -> Self;
}

impl WithDualAxisProcessor for Box<dyn DualAxisProcessor> {
    /// Creates a new boxed [`DualAxisProcessingPipeline`] with the existing steps
    /// and appends the given [`DualAxisProcessor`].
    fn with_processor(self, processor: impl DualAxisProcessor) -> Self {
        let pipeline = match Reflect::as_any(&*self).downcast_ref::<DualAxisProcessingPipeline>() {
            Some(pipeline) => pipeline.clone(),
            None => DualAxisProcessingPipeline(vec![self]),
        };
        Box::new(pipeline.with(processor))
    }
}

#[cfg(test)]
mod tests {
    use crate::input_processing::*;
    use bevy::prelude::Vec2;

    #[test]
    fn test_dual_axis_processing_pipeline() {
        // Add processors to a new pipeline.
        let mut pipeline = DualAxisProcessingPipeline::default()
            .with(DualAxisSensitivity::all(4.0))
            .with(DualAxisInverted::ALL)
            .with(DualAxisInverted::ALL);

        pipeline.push(DualAxisSensitivity::all(3.0));

        pipeline.set(3, DualAxisSensitivity::all(6.0));

        // This pipeline now scales input values by a factor of 24.0
        assert_eq!(pipeline.process(Vec2::splat(2.0)), Vec2::splat(48.0));
        assert_eq!(pipeline.process(Vec2::splat(-3.0)), Vec2::splat(-72.0));

        // Now it just leaves values as is.
        pipeline.clear();
        assert_eq!(pipeline, DualAxisProcessingPipeline::default());
        assert_eq!(pipeline.process(Vec2::splat(2.0)), Vec2::splat(2.0));
    }

    #[test]
    fn test_with_axis_processor() {
        let first = DualAxisSensitivity::all(2.0);
        let first_boxed: Box<dyn DualAxisProcessor> = Box::new(first);

        let second = DualAxisSensitivity::all(3.0);
        let merged_second = first_boxed.with_processor(second);
        let expected = DualAxisProcessingPipeline::default()
            .with(first)
            .with(second);
        let expected_boxed: Box<dyn DualAxisProcessor> = Box::new(expected);
        assert_eq!(merged_second, expected_boxed);

        let third = DualAxisSensitivity::all(4.0);
        let merged_third = merged_second.with_processor(third);
        let expected = DualAxisProcessingPipeline::default()
            .with(first)
            .with(second)
            .with(third);
        let expected_boxed: Box<dyn DualAxisProcessor> = Box::new(expected);
        assert_eq!(merged_third, expected_boxed);
    }
}
