//! Processors for input values
//!
//! This module simplifies input handling in your application by providing processors
//! for refining and manipulating values before reaching the application logic.
//!
//! # Processor Traits
//!
//! The foundation of this module lies in these core traits.
//!
//! - [`AxisProcessor`]: Handles `f32` values for single-axis inputs.
//! - [`DualAxisProcessor`]: Handles [`Vec2`](bevy::prelude::Vec2) values for dual-axis inputs.
//!
//! Need something specific? You can also create your own processors by implementing these traits for specific needs.
//!
//! Feel free to suggest additions to the built-in processors if you have a common use case!
//!
//! # Built-in Processors
//!
//! ## Pipelines
//!
//! Pipelines are dynamic sequence containers of processors,
//! transforming input values by passing them through each processor in the pipeline,
//! allowing to create complex processing workflows by combining simpler steps.
//!
//! - [`AxisProcessingPipeline`]: Transforms single-axis input values with a sequence of [`AxisProcessor`]s.
//! - [`DualAxisProcessingPipeline`]: Transforms dual-axis input values with a sequence of [`DualAxisProcessor`]s.
//!
//! While pipelines offer flexibility in dynamic managing processing steps,
//! they may hinder compiler optimizations such as inlining or dead code elimination.
//! For performance-critical scenarios, consider creating you own processors for improved performance.
//!
//! ## Inversion
//!
//! Inversion flips the sign of input values, resulting in a directional reversal of control.
//! For example, positive values become negative, and up becomes down.
//!
//! - [`AxisInverted`]: Single-axis inversion.
//! - [`DualAxisInverted`]: Dual-axis inversion.
//!
//! ## Sensitivity
//!
//! Sensitivity scales input values with a specified multiplier (doubling, halving, etc.),
//! allowing fine-tuning the responsiveness of controls.
//!
//! - [`AxisSensitivity`]: Single-axis scaling.
//! - [`DualAxisSensitivity`]: Dual-axis scaling.
//!
//! ## Value Bounds
//!
//! Value bounds define an acceptable range for input values,
//! clamping out-of-bounds inputs to the nearest valid value and leaving others as is
//! to avoid unexpected behavior caused by extreme inputs.
//!
//! - [`AxisBounds`]: A min-max range for valid single-axis inputs.
//! - [`DualAxisBounds`]: A square-shaped region for valid dual-axis inputs, with independent min-max ranges for each axis.
//! - [`CircleBounds`]: A circular region for valid dual-axis inputs, with a radius defining the maximum magnitude.
//!
//! ## Dead Zones
//!
//! ### Unscaled Versions
//!
//! Unscaled dead zones specify regions where input values within the regions
//! are considered excluded from further processing and treated as zeros,
//! helping filter out minor fluctuations and unintended movements.
//!
//! - [`AxisExclusion`]: A min-max range for excluding single-axis input values.
//! - [`DualAxisExclusion`]: A cross-shaped region for excluding dual-axis inputs, with independent min-max ranges for each axis.
//! - [`CircleExclusion`]: A circular region for excluding dual-axis inputs, with a radius defining the maximum excluded magnitude.
//!
//! ### Scaled Versions
//!
//! Scaled dead zones transform input values by restricting values within the default bounds,
//! and then scaling non-excluded values linearly into the "live zone",
//! the remaining region within the bounds after dead zone exclusion.
//!
//! - [`AxisDeadZone`]: A scaled version of [`AxisExclusion`] with the bounds set to [`AxisBounds::magnitude(1.0)`](AxisBounds::default).
//! - [`DualAxisDeadZone`]: A scaled version of [`DualAxisExclusion`] with the bounds set to [`DualAxisBounds::magnitude_all(1.0)`](DualAxisBounds::default).
//! - [`CircleDeadZone`]: A scaled version of [`CircleExclusion`] with the bounds set to [`CircleBounds::magnitude(1.0)`](CircleBounds::default).

pub use self::dual_axis::*;
pub use self::single_axis::*;

pub mod dual_axis;
pub mod single_axis;
