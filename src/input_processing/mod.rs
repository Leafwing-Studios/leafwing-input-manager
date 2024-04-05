//! Input Processors
//!
//! This module simplifies handling input values in your application by providing processors
//! for refining and manipulating them before reaching the application logic.
//!
//! # Processor Traits
//!
//! The foundation of this module lies in these core traits.
//!
//! - [`AxisProcessor`]: Handles single-axis input values.
//! - [`DualAxisProcessor`]: Handles dual-axis input values (X and Y axes).
//!
//! Need something specific? You can also create your own processors by implementing these traits for specific needs.
//!
//! Feel free to suggest additions to the built-in processors if you have a common use case!
//!
//! # Built-in Processors
//!
//! ## Inversion
//!
//! Inversion reverses the control, such as positive value becomes negative or up becomes down.
//!
//! - [`AxisInverted`]: Single-axis inversion.
//! - [`DualAxisInverted`]: Dual-axis inversion.
//!
//! ## Sensitivity
//!
//! Sensitivity adjusts control responsiveness by scaling input values with a multiplier (doubling, halving, etc.).
//!
//! - [`AxisSensitivity`]: Single-axis scaling.
//! - [`DualAxisSensitivity`]: Dual-axis scaling.
//!
//! ## Value Bounds
//!
//! Value bounds define the boundaries for constraining input values,
//! preventing unexpected behavior that might occur outside these bounds.
//! Values exceeding the bounds are clamped to fit within the specified range.
//!
//! - [`AxisBounds`]: Restricts single-axis input values to a range.
//! - [`DualAxisBounds`]: Restricts dual-axis input values to a range along each axis.
//! - [`CircleBounds`]: Limits dual-axis input values to a maximum magnitude.
//!
//! ## Deadzones
//!
//! ### Unscaled Versions
//!
//! Unscaled deadzones specify ranges where near-zero input values are excluded, treating them as zero.
//!
//! - [`AxisExclusion`]: Excludes single-axis input values within a specified range.
//! - [`DualAxisExclusion`]: Excludes dual-axis input values within a specified range along each axis.
//! - [`CircleExclusion`]: Excludes dual-axis input values below a specified magnitude threshold.
//!
//! ### Scaled Versions
//!
//! Scaled deadzones process input values by clamping them to fit within the default value bounds,
//! considering a specified exclusion range, and scaling unchanged values linearly in between.
//!
//! - [`AxisDeadzone`]: Normalizes single-axis input values based on [`AxisBounds::default`] and a specified [`AxisExclusion`].
//! - [`DualAxisDeadzone`]: Normalizes dual-axis input values based on [`DualAxisBounds::default`] and a specified [`DualAxisExclusion`].
//! - [`CircleDeadzone`]: Normalizes dual-axis input values based on [`CircleBounds::default`] and a specified [`CircleExclusion`].
//!
//! ## Composite Processors
//!
//! Pipelines are dynamic sequence containers of processors, allowing you to create custom processing steps.
//!
//! - [`AxisProcessingPipeline`]: Applies a sequence of [`AxisProcessor`]s to process single-axis input values.
//! - [`DualAxisProcessingPipeline`]: Applies a sequence of [`DualAxisProcessor`]s to process dual-axis input values.
//!
//! While pipelines offer flexibility in defining custom processing steps,
//! for performance-critical scenarios, consider using the macros below for optimized implementations.
//!
//! - [`define_axis_processing_pipeline`](crate::define_axis_processing_pipeline):
//!     Generates an [`AxisProcessor`] with fixed processing steps to process single-axis input values.
//! - [`define_dual_axis_processing_pipeline`](crate::define_dual_axis_processing_pipeline):
//!     Generates a [`DualAxisProcessor`] with fixed processing steps to process dual-axis input values.
//!
//! Choose between pipelines and macros based on your specific processing needs and performance requirements.

pub use self::dual_axis::*;
pub use self::single_axis::*;

pub mod dual_axis;
pub mod single_axis;

// region pipeline

/// Defines a new pipeline called the `name` that processes `value_type` values sequentially
/// through a chain of specified `processors` implementing the `processor_type`.
/// This enhances performance by enabling optimizations like inlining and dead code elimination.
///
/// # Examples
///
/// ```rust
/// use bevy::prelude::*;
/// use leafwing_input_manager::prelude::*;
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
        #[derive(Clone, Copy, PartialEq, Eq, Hash, ::bevy::reflect::Reflect, ::serde::Serialize, ::serde::Deserialize)]
        pub struct $Pipeline;

        #[$crate::prelude::serde_trait_object]
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
/// let mut pipeline = MyDynamicProcessingPipeline::default();
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
        #[doc = concat!("A dynamic sequence container of [`", stringify!($ProcessorTrait), "`] designed for processing input values.")]
        ///
        /// # Warning
        ///
        /// This flexibility may hinder compiler optimizations such as inlining or dead code elimination.
        /// For production-ready solutions, consider creating your own pipelines
        /// with optimized logic for improved performance.
        #[must_use]
        #[derive(Debug, Default, Clone, PartialEq, Eq, Hash, ::bevy::reflect::Reflect, ::serde::Serialize, ::serde::Deserialize)]
        pub struct $Pipeline(Vec<Box<dyn $ProcessorTrait>>);

        #[$crate::prelude::serde_trait_object]
        impl $ProcessorTrait for $Pipeline {
            /// Processes input values through this dynamic pipeline and returns the result.
            #[must_use]
            #[inline]
            fn process(&self, input_value: $InputValueType) -> $InputValueType {
                self.0.iter().fold(input_value, |value, next| next.process(value))
            }
        }

        impl $Pipeline {
            /// Appends the given `processor` into this pipeline and returns `self`.
            #[inline]
            pub fn with(mut self, processor: impl $ProcessorTrait) -> Self {
                self.0.push(Box::new(processor));
                self
            }

            /// Replaces the processor at the `index` with the given `processor`.
            #[inline]
            pub fn set(&mut self, index: usize, processor: impl $ProcessorTrait) {
                self.0[index] = Box::new(processor);
            }

            /// Removes all processors in this pipeline.
            #[inline]
            pub fn clear(&mut self) {
                self.0.clear();
            }
        }
    };
}

// endregion pipeline
