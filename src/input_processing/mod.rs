//! Utilities for processing input values.
//!
//! - The [`single_axis`] module provides the utilities for processing input values on an axis.
//! - The [`dual_axis`] module provides the utilities for processing input values on the XY axes.

pub use self::dual_axis::*;
pub use self::single_axis::*;

pub mod dual_axis;
pub mod single_axis;

// region pipeline

/// Defines a new pipeline with the name specified by the `Pipeline` parameter
/// that processes `InputValueType` values sequentially
/// using a chain of specified `processors` implementing `ProcessorTrait`.
/// This is helpful for certain optimizations such as inlining or dead code elimination,
/// ensuring the performance and efficiency of the final product.
///
/// # Examples
///
/// ```rust
/// use bevy::prelude::*;
/// use leafwing_input_manager::prelude::*;
/// use serde::{Serialize, Deserialize};
///
/// define_input_processing_pipeline!(
///     // The name of the new pipeline.
///     name: InvertedThenDouble,
///     // The type of input values to process.
///     value_type: f32,
///     // Processor trait constraint.
///     processor_type: AxisProcessor,
///     // Processors used in the pipeline.
///     processors: [AxisInverted, AxisSensitivity(2.0)]
/// );
///
/// // This new pipeline is just a unit struct with inlined logic.
/// let processor = InvertedThenDouble;
///
/// // Now you can use it!
/// assert_eq!(processor.process(2.0), -4.0);
/// assert_eq!(processor.process(-1.0), 2.0);
/// ```
#[macro_export]
macro_rules! define_input_processing_pipeline {
    (
        name: $Pipeline:ident,
        value_type: $InputValueType:ty,
        processor_type: $ProcessorTrait:ident,
        processors: [$($processor:expr),* $(,)?]
    ) => {
        #[doc = concat!("The [`", stringify!($ProcessorTrait), "`] for sequential processing input values by passing them through a sequence of `[", __stringify_expressions!($($processor),*), "]`.")]
        #[must_use]
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
        pub struct $Pipeline;

        #[typetag::serde]
        impl $ProcessorTrait for $Pipeline {
            /// Processes input values through this pipeline and returns the result.
            #[must_use]
            #[inline]
            fn process(&self, input_value: $InputValueType) -> $InputValueType {
                $(let input_value = $processor.process(input_value);)*
                input_value
            }
        }

        impl ::core::fmt::Debug for $Pipeline {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct(stringify!($Pipeline))
                    .field("processors", &concat!("[", $crate::__stringify_expressions!($($processor),*), "]"))
                    .finish()
            }
        }
    };
}

/// Defines a new pipeline with the name specified by the `Pipeline` parameter,
/// processing `InputValueType` values sequentially.
/// The pipeline wraps a [`Vec`] containing instances that implement the `ProcessorTrait`,
/// enabling dynamic modification, beneficial during debugging and code refinement.
///
/// # Warning
///
/// This flexibility may hinder compiler optimizations such as inlining or dead code elimination.
/// For production-ready solutions, consider creating your own pipelines
/// with optimized logic for improved performance.
///
/// # Examples
///
/// ```rust
/// use bevy::prelude::*;
/// use leafwing_input_manager::prelude::*;
/// use serde::{Serialize, Deserialize};
///
/// define_dynamic_input_processing_pipeline!(
///     // The name of the new pipeline.
///     name: MyDynamicProcessingPipeline,
///     // The type of input values to process.
///     value_type: f32,
///     // Processor trait constraint.
///     processor_type: AxisProcessor
/// );
///
/// // Create a mutable empty pipeline.
/// let mut pipeline = MyDynamicProcessingPipeline::new();
///
/// // Chain processors using the with() function.
/// pipeline = pipeline.with(AxisSensitivity(2.0))
///     .with(AxisSensitivity(3.0))
///     .with(AxisInverted);
///
/// pipeline.set(1, AxisSensitivity(4.0));
///
/// // Now you get a pipeline with a sensitivity of 8.0 and inversion.
/// assert_eq!(pipeline.process(4.0), -32.0);
/// assert_eq!(pipeline.process(-2.0), 16.0);
/// ```
#[macro_export]
macro_rules! define_dynamic_input_processing_pipeline {
    (name: $Pipeline:ident, value_type: $InputValueType:ty, processor_type: $ProcessorTrait:ident) => {
        #[doc = concat!("The [`", stringify!($ProcessorTrait), "`] that wraps a [`Vec`] containing boxed processors for sequential processing input values.")]
        /// This allows for dynamic modification, beneficial during debugging and code refinement.
        ///
        /// # Warning
        ///
        /// This flexibility may hinder compiler optimizations such as inlining or dead code elimination.
        /// For production-ready solutions, consider creating your own pipelines
        /// with optimized logic for improved performance.
        #[must_use]
        #[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
        pub struct $Pipeline(Vec<Box<dyn $ProcessorTrait>>);

        #[typetag::serde]
        impl $ProcessorTrait for $Pipeline {
            /// Processes input values through this dynamic pipeline and returns the result.
            #[must_use]
            #[inline]
            fn process(&self, input_value: $InputValueType) -> $InputValueType {
                self.0.iter().fold(input_value, |value, next| next.process(value))
            }
        }

        impl $Pipeline {
            /// Creates an empty pipeline.
            #[inline]
            pub fn new() -> Self {
                Self(Vec::new())
            }

            /// Appends the given `processor` into this pipeline and returns `self`.
            #[inline]
            pub fn with(mut self, processor: impl $ProcessorTrait) -> Self {
                self.0.push(Box::new(processor));
                self
            }

            /// Replaces the processor at the `index` with the given `processor`.
            pub fn set(&mut self, index: usize, processor: impl $ProcessorTrait) {
                self.0[index] = Box::new(processor);
            }
        }
    };
}

// endregion pipeline
