use bevy::prelude::{Reflect, TypePath};
use serde::{Deserialize, Serialize};

use super::InputProcessor;

/// A dynamic sequence container of [`InputProcessor`]s designed for processing input values.
///
/// # Warning
///
/// This flexibility may hinder compiler optimizations such as inlining or dead code elimination.
/// For performance-critical scenarios, consider creating your own processors for improved performance.
#[must_use]
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub struct InputProcessingPipeline<T: TypePath>(pub(crate) Vec<Box<dyn InputProcessor<T>>>);

impl<T: TypePath> InputProcessor<T> for InputProcessingPipeline<T> {
    /// Computes the result by passing the `input_value` through this pipeline.
    #[must_use]
    #[inline]
    fn process(&self, input_value: T) -> T {
        self.0
            .iter()
            .fold(input_value, |value, next| next.process(value))
    }
}

impl<T: TypePath> InputProcessingPipeline<T> {
    /// Appends the given [`InputProcessor`] into this pipeline and returns `self`.
    #[inline]
    pub fn with(mut self, processor: impl InputProcessor<T>) -> Self {
        self.push(processor);
        self
    }

    /// Appends the given [`InputProcessor`] into this pipeline.
    #[inline]
    pub fn push(&mut self, processor: impl InputProcessor<T>) {
        self.0.push(Box::new(processor));
    }

    /// Replaces the processor at the `index` with the given [`InputProcessor`].
    #[inline]
    pub fn set(&mut self, index: usize, processor: impl InputProcessor<T>) {
        self.0[index] = Box::new(processor);
    }

    /// Removes all processors in this pipeline.
    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }
}

/// A trait for appending an [`InputProcessor`] as the next processing step in the pipeline,
/// enabling further processing of input values.
pub trait WithNextInputProcessor<T: TypePath> {
    /// Appends an [`AxisProcessor`] as the next processing step in the pipeline.
    fn with_processor(self, processor: impl InputProcessor<T>) -> Self;
}

impl<T: TypePath> WithNextInputProcessor<T> for Box<dyn InputProcessor<T>> {
    /// Creates a new boxed [`crate::input_processing::AxisProcessingPipeline`] with the existing steps
    /// and appends the given [`AxisProcessor`].
    fn with_processor(self, processor: impl InputProcessor<T>) -> Self {
        let pipeline = match Reflect::as_any(&*self).downcast_ref::<InputProcessingPipeline<T>>() {
            Some(pipeline) => pipeline.clone(),
            None => InputProcessingPipeline(vec![self]),
        };
        Box::new(pipeline.with(processor))
    }
}
