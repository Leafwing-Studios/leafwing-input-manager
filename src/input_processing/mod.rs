//! Processors for input values
//!
//! This module simplifies input handling in your application by providing processors
//! for refining and manipulating values before reaching the application logic.
//!
//! # Processor Enums
//!
//! The foundation of this module lies in these enums.
//!
//! - [`AxisProcessor`]: Handles `f32` values for single-axis inputs.
//! - [`DualAxisProcessor`]: Handles [`Vec2`](bevy::prelude::Vec2) values for dual-axis inputs.
//!
//! Need something specific? You can also create your own processors by implementing these traits for specific needs.
//!
//! - [`CustomAxisProcessor`]: Handles `f32` values for single-axis inputs.
//! - [`CustomDualAxisProcessor`]: Handles [`Vec2`](bevy::prelude::Vec2) values for dual-axis inputs.
//!
//! Feel free to suggest additions to the built-in processors if you have a common use case!
//!
//! # Built-in Processors
//!
//! ## Sequential Processors
//!
//! Sequential processors handle input values sequentially by chaining together two processors,
//! one for the current step and the other for the next step.
//!
//! - [`AxisProcessor::Sequential`]: Transforms single-axis input values with two [`AxisProcessor`]s.
//! - [`DualAxisProcessor::Sequential`]: Transforms dual-axis input values with two [`DualAxisProcessor`]s.
//!
//! You can also use `with_processor` method to create a sequential processor from another processor.
//!
//! ## Inversion
//!
//! Inversion flips the sign of input values, resulting in a directional reversal of control.
//! For example, positive values become negative, and up becomes down.
//!
//! - [`AxisProcessor::Inverted`]: Single-axis inversion.
//! - [`DualAxisInverted`]: Dual-axis inversion, implemented [`Into<DualAxisProcessor>`].
//!
//! ## Sensitivity
//!
//! Sensitivity scales input values with a specified multiplier (doubling, halving, etc.),
//! allowing fine-tuning the responsiveness of controls.
//!
//! - [`AxisProcessor::Sensitivity`]: Single-axis scaling.
//! - [`DualAxisSensitivity`]: Dual-axis scaling, implemented [`Into<DualAxisProcessor>`].
//!
//! ## Value Bounds
//!
//! Value bounds define an acceptable range for input values,
//! clamping out-of-bounds inputs to the nearest valid value and leaving others as is
//! to avoid unexpected behavior caused by extreme inputs.
//!
//! - [`AxisBounds`]: A min-max range for valid single-axis inputs, implemented [`Into<AxisProcessor>`].
//! - [`DualAxisBounds`]: A square-shaped region for valid dual-axis inputs,
//!     with independent min-max ranges for each axis, implemented [`Into<DualAxisProcessor>`].
//! - [`CircleBounds`]: A circular region for valid dual-axis inputs,
//!     with a radius defining the maximum magnitude, implemented [`Into<DualAxisProcessor>`].
//!
//! ## Dead Zones
//!
//! ### Unscaled Versions
//!
//! Unscaled dead zones specify regions where input values within the regions
//! are considered excluded from further processing and treated as zeros,
//! helping filter out minor fluctuations and unintended movements.
//!
//! - [`AxisExclusion`]: A min-max range for excluding single-axis input values,
//!     implemented `Into<AxisProcessor>`.
//! - [`DualAxisExclusion`]: A cross-shaped region for excluding dual-axis inputs,
//!     with independent min-max ranges for each axis, implemented [`Into<DualAxisProcessor>`].
//! - [`CircleExclusion`]: A circular region for excluding dual-axis inputs,
//!     with a radius defining the maximum excluded magnitude, implemented [`Into<DualAxisProcessor>`].
//!
//! ### Scaled Versions
//!
//! Scaled dead zones transform input values by restricting values within the default bounds,
//! and then scaling non-excluded values linearly into the "live zone",
//! the remaining region within the bounds after dead zone exclusion.
//!
//! - [`AxisDeadZone`]: A scaled version of [`AxisExclusion`] with the bounds
//!     set to [`AxisBounds::magnitude(1.0)`](AxisBounds::default), implemented [`Into<AxisProcessor>`].
//! - [`DualAxisDeadZone`]: A scaled version of [`DualAxisExclusion`] with the bounds
//!     set to [`DualAxisBounds::magnitude_all(1.0)`](DualAxisBounds::default), implemented [`Into<DualAxisProcessor>`].
//! - [`CircleDeadZone`]: A scaled version of [`CircleExclusion`] with the bounds
//!     set to [`CircleBounds::new(1.0)`](CircleBounds::default), implemented [`Into<DualAxisProcessor>`].

pub use self::dual_axis::*;
pub use self::single_axis::*;

pub mod dual_axis;
pub mod single_axis;
