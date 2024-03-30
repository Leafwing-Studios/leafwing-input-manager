//! Input processors for a single axis
//!
//! This module provides [`AxisProcessor`]s to refine and manipulate input values
//! along a single axis before they reach your game logic.
//!
//! # Using Processors
//!
//! This module offers several built-in processors to handle common tasks.
//!
//! - [`AxisInverted`]: Flips the sign of input values (e.g., up becomes down).
//! - [`AxisSensitivity`]: Adjusts control responsiveness by scaling input values with a multiplier.
//! - [`AxisBounds`]: Defines valid limits for input values to prevent unexpected behavior.
//! - [`AxisExclusion`]: Specifies a range where input values are ignored (treated as zero).
//! - [`AxisDeadzone`]: Linearly scales input values into the livezone ranges defined
//!     by a specified [`AxisExclusion`].
//!
//! Need something specific? You can also create your own processors
//! by implementing the [`AxisProcessor`] trait for specific needs.
//!
//! Feel free to suggest additions to the built-in processors if you have a common use case!
//!
//! # Custom Processing Pipelines
//!
//! To construct custom processing pipelines for complex tasks,
//! begin by creating an empty [`AxisProcessingPipeline`].
//! Then, add desired processing steps (of type [`AxisProcessor`]) to the pipeline.
//!
//! Keep in mind that while this approach offers flexibility, it may limit compiler optimizations.
//! For performance-critical production environments, opt for the `define_axis_processing_pipeline` macro.
//! This macro generates an optimized pipeline with inlined logic for all specified processors.
//!
//! All pipelines have implemented the [`AxisProcessor`] trait,
//! allowing you to use them directly as a single [`AxisProcessor`].

use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use bevy::reflect::Reflect;
use bevy::utils::FloatOrd;
use dyn_clone::DynClone;
use dyn_eq::DynEq;
use dyn_hash::DynHash;
use serde::{Deserialize, Serialize};

// region processor trait

/// A trait for defining processors applied to input values on an axis,
/// taking a `f32` value and return the processed `f32` result.
///
/// Implementors of this trait are responsible for providing the specific processing logic.
///
/// # Examples
///
/// ```rust
/// use std::hash::{Hash, Hasher};
/// use bevy::prelude::*;
/// use bevy::utils::FloatOrd;
/// use serde::{Deserialize, Serialize};
/// use leafwing_input_manager::prelude::*;
///
/// /// Doubles the input, takes the absolute value,
/// /// and discards results that meet the specified condition.
/// // If your processor includes fields not implemented Eq and Hash,
/// // implementation is necessary as shown below.
/// // Otherwise, you can derive Eq and Hash directly.
/// #[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
/// pub struct DoubleAbsoluteValueThenIgnored(f32);
///
/// // Add this attribute for ensuring proper serialization and deserialization.
/// #[processor_serde]
/// impl AxisProcessor for DoubleAbsoluteValueThenIgnored {
///     fn process(&self, input_value: f32) -> f32 {
///         // Implement the logic just like you would in a normal function.
///
///         // You can use other processors within this function.
///         let value = AxisSensitivity(2.0).process(input_value);
///
///         let value = value.abs();
///         if value == self.0 {
///             0.0
///         } else {
///             value
///         }
///     }
/// }
///
/// // Unfortunately, manual implementation is required due to the float field.
/// impl Eq for DoubleAbsoluteValueThenIgnored {}
/// impl Hash for DoubleAbsoluteValueThenIgnored {
///     fn hash<H: Hasher>(&self, state: &mut H) {
///         // Encapsulate the float field for hashing.
///         FloatOrd(self.0).hash(state);
///     }
/// }
///
/// // Now you can use it!
/// let processor = DoubleAbsoluteValueThenIgnored(4.0);
///
/// // Rejected!
/// assert_eq!(processor.process(2.0), 0.0);
/// assert_eq!(processor.process(-2.0), 0.0);
///
/// // Others are left unchanged.
/// assert_eq!(processor.process(6.0), 12.0);
/// assert_eq!(processor.process(4.0), 8.0);
/// assert_eq!(processor.process(0.0), 0.0);
/// assert_eq!(processor.process(-4.0), 8.0);
/// assert_eq!(processor.process(-6.0), 12.0);
/// ```
#[typetag::serde(tag = "type")]
pub trait AxisProcessor: Send + Sync + Debug + DynClone + DynEq + DynHash + Reflect {
    /// Processes the `input_value` and returns the result.
    fn process(&self, input_value: f32) -> f32;
}

dyn_clone::clone_trait_object!(AxisProcessor);
dyn_eq::eq_trait_object!(AxisProcessor);
dyn_hash::hash_trait_object!(AxisProcessor);
crate::__reflect_box_dyn_trait_object!(AxisProcessor);

// endregion processor trait

// region pipeline

/// Defines an optimized pipeline that sequentially processes input values
/// using a chain of specified [`AxisProcessor`]s with inlined logic.
///
/// # Examples
///
/// ```rust
/// use bevy::prelude::*;
/// use leafwing_input_manager::prelude::*;
/// use serde::{Serialize, Deserialize};
///
/// define_axis_processing_pipeline!(
///     // The name of the new pipeline.
///     name: InvertedThenDouble,
///     // Processors used in the pipeline.
///     processors: [AxisInverted, AxisSensitivity(2.0)]
/// );
///
/// // This new pipeline is just a unit struct with inlined logic.
/// let pipeline = InvertedThenDouble;
///
/// // Now you can use it!
/// assert_eq!(pipeline.process(2.0), -4.0);
/// assert_eq!(pipeline.process(-1.0), 2.0);
/// ```
#[macro_export]
macro_rules! define_axis_processing_pipeline {
    (name: $Pipeline:ident, processors: [$($processor:expr),* $(,)?]) => {
        $crate::define_input_processing_pipeline!(
            name: $Pipeline,
            value_type: f32,
            processor_type: AxisProcessor,
            processors: [$($processor,)*]
        );
    };
}

crate::define_dynamic_input_processing_pipeline!(
    name: AxisProcessingPipeline,
    value_type: f32,
    processor_type: AxisProcessor
);

// endregion pipeline

// region inversion

/// Flips the sign of input values on an axis.
///
/// This is useful for reversing controls, such as reversing camera movement direction.
///
/// # Examples
///
/// ```rust
/// use leafwing_input_manager::prelude::*;
///
/// // Positive values become negative.
/// assert_eq!(AxisInverted.process(2.0), -2.0);
///
/// // And vice versa; in other words, negative values become positive.
/// assert_eq!(AxisInverted.process(-1.0), 1.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct AxisInverted;

#[typetag::serde]
impl AxisProcessor for AxisInverted {
    /// Returns the opposite value of the `input_value`.
    #[must_use]
    #[inline]
    fn process(&self, input_value: f32) -> f32 {
        -input_value
    }
}

// endregion inversion

// region sensitivity

/// Scales input values on an axis by a specified `sensitivity` multiplier.
///
/// This processor allows fine-tuning the responsiveness of controls based on user preference or game mechanics.
///
/// # Examples
///
/// ```rust
/// use leafwing_input_manager::prelude::*;
///
/// // Doubled.
/// assert_eq!(AxisSensitivity(2.0).process(2.0), 4.0);
///
/// // Halved.
/// assert_eq!(AxisSensitivity(0.5).process(2.0), 1.0);
///
/// // Inverted and halved.
/// assert_eq!(AxisSensitivity(-0.5).process(2.0), -1.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct AxisSensitivity(pub f32);

#[typetag::serde]
impl AxisProcessor for AxisSensitivity {
    /// Scales the input value by the specified `sensitivity` multiplier.
    #[must_use]
    #[inline]
    fn process(&self, input_value: f32) -> f32 {
        self.0 * input_value
    }
}

impl Eq for AxisSensitivity {}

impl Hash for AxisSensitivity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        FloatOrd(self.0).hash(state);
    }
}

// endregion sensitivity

// region bounds

/// Specifies the bounds (`[min, max]`) for limiting valid input values.
///
/// This processor ensures all processed values stay within the bounds,
/// preventing unexpected behavior from extreme inputs.
///
/// # Examples
///
/// ```rust
/// use leafwing_input_manager::prelude::*;
///
/// // Set the bounds to [-2, 3].
/// let bounds = AxisBounds::new(-2.0, 3.0);
///
/// // These values are within the bounds.
/// let values = [3.0, 1.0, 0.5, -0.5, -1.0, -2.0];
/// for value in values {
///     assert!(bounds.contains(value));
///
///     // So the value should be left unchanged after processing.
///     assert_eq!(bounds.process(value), value);
/// }
///
/// // These values are out of the bounds.
/// let values = [500.0, 5.0, 3.1, -2.1, -5.0, -500.0];
/// for value in values {
///     assert!(!bounds.contains(value));
///
///     // So the value should be clamped to the bounds.
///     let processed = bounds.process(value);
///     assert!(processed == bounds.min() || processed == bounds.max());
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct AxisBounds {
    /// The minimum value of valid inputs.
    pub(crate) min: f32,

    /// The maximum value of valid inputs.
    pub(crate) max: f32,
}

#[typetag::serde]
impl AxisProcessor for AxisBounds {
    /// Clamps input values that are out of the bounds to fit within them.
    #[must_use]
    #[inline(always)]
    fn process(&self, input_value: f32) -> f32 {
        // std clamp() will panic if either bound set to `NaN`.
        input_value.max(self.min).min(self.max)
    }
}

impl Default for AxisBounds {
    /// Creates a new [`AxisBounds`] with the bounds set to `[-1.0, 1.0]`.
    #[inline]
    fn default() -> Self {
        Self::symmetric(1.0)
    }
}

impl AxisBounds {
    /// Creates a new [`AxisBounds`] with the bounds set to `[min, max]`.
    ///
    /// # Requirements
    ///
    /// - `min` <= `max`.
    ///
    /// # Panics
    ///
    /// Panics if any of the requirements isn't met.
    #[inline]
    pub fn new(min: f32, max: f32) -> Self {
        assert!(min <= max);
        Self { min, max }
    }

    /// Creates a new [`AxisBounds`] with the bounds set to `[-threshold, threshold]`.
    ///
    /// # Requirements
    ///
    /// - `threshold` >= `0.0`.
    ///
    /// # Panics
    ///
    /// Panics if any of the requirements isn't met.
    #[inline]
    pub fn symmetric(threshold: f32) -> Self {
        Self::new(-threshold, threshold)
    }

    /// Creates a new [`AxisBounds`] with the minimum bound set to `min`.
    #[inline]
    pub fn at_least(min: f32) -> Self {
        Self::new(min, f32::MAX)
    }

    /// Creates a new [`AxisBounds`] with the maximum bound set to `max`.
    #[inline]
    pub fn at_most(max: f32) -> Self {
        Self::new(f32::MIN, max)
    }

    /// Creates an [`AxisBounds`] with unlimited bounds.
    pub fn full_range() -> Self {
        Self::new(f32::MIN, f32::MAX)
    }

    /// Returns the minimum bound.
    #[must_use]
    #[inline]
    pub fn min(&self) -> f32 {
        self.min
    }

    /// Returns the maximum bounds.
    #[must_use]
    #[inline]
    pub fn max(&self) -> f32 {
        self.max
    }

    /// Returns the minimum and maximum bounds.
    #[must_use]
    #[inline]
    pub fn min_max(&self) -> (f32, f32) {
        (self.min, self.max)
    }

    /// Returns the center of these bounds.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use leafwing_input_manager::prelude::*;
    ///
    /// let bounds = AxisBounds::new(0.0, 0.2);
    ///
    /// assert_eq!(bounds.center(), 0.1);
    /// ```
    #[must_use]
    #[inline]
    pub fn center(&self) -> f32 {
        (self.min + self.max) * 0.5
    }

    /// Checks whether the `input_value` is a valid input within the bounds.
    #[must_use]
    #[inline]
    pub fn contains(&self, input_value: f32) -> bool {
        self.min <= input_value && input_value <= self.max
    }
}

impl Eq for AxisBounds {}

impl Hash for AxisBounds {
    fn hash<H: Hasher>(&self, state: &mut H) {
        FloatOrd(self.min).hash(state);
        FloatOrd(self.max).hash(state);
    }
}

// endregion bounds

// region exclusion

/// Defines an exclusion range (`[negative_max, positive_min]`)
/// where near-zero input values are ignored (treated as `0.0`).
///
/// In simple terms, this exclusion behaves like a deadzone without normalization.
/// This processor helps filter out minor fluctuations and unintended movements.
///
/// # Examples
///
/// ```rust
/// use leafwing_input_manager::prelude::*;
///
/// // Exclude values from -2 to 3.
/// let exclusion = AxisExclusion::new(-2.0, 3.0);
///
/// // These values are within the range.
/// let values = [3.0, 1.0, 0.5, -0.5, -1.0, -2.0];
/// for value in values {
///     assert!(exclusion.contains(value));
///
///     // So the value should be treated as zeros.
///     assert_eq!(exclusion.process(value), 0.0);
/// }
///
/// // These values are out of the range.
/// let values = [500.0, 5.0, 3.1, -2.1, -5.0, -500.0];
/// for value in values {
///     assert!(!exclusion.contains(value));
///
///     // So the value should be left unchanged after processing.
///     assert_eq!(exclusion.process(value), value);
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct AxisExclusion {
    /// The maximum negative value affected by the deadzone.
    pub(crate) negative_max: f32,

    /// The minimum positive value affected by the deadzone.
    pub(crate) positive_min: f32,
}

#[typetag::serde]
impl AxisProcessor for AxisExclusion {
    /// Excludes input values within the range.
    #[must_use]
    #[inline(always)]
    fn process(&self, input_value: f32) -> f32 {
        if self.contains(input_value) {
            0.0
        } else {
            input_value
        }
    }
}

impl Default for AxisExclusion {
    /// Creates a default [`AxisExclusion`], excluding input values within the range `[-0.1, 0.1]`.
    #[inline]
    fn default() -> Self {
        Self::new(-0.1, 0.1)
    }
}

impl AxisExclusion {
    /// Creates a new [`AxisExclusion`] for input values within `[negative_max, positive_min]`.
    ///
    /// # Requirements
    ///
    /// - `negative_max` <= `0.0` <= `positive_min`.
    ///
    /// # Panics
    ///
    /// Panics if any of the requirements isn't met.
    #[inline]
    pub fn new(negative_max: f32, positive_min: f32) -> Self {
        assert!(negative_max <= 0.0);
        assert!(positive_min >= 0.0);
        Self {
            negative_max,
            positive_min,
        }
    }

    /// Creates a new [`AxisExclusion`] for input values within `[-threshold, threshold]`.
    ///
    /// # Requirements
    ///
    /// - `threshold` >= `0.0`.
    ///
    /// # Panics
    ///
    /// Panics if any of the requirements isn't met.
    #[inline]
    pub fn symmetric(threshold: f32) -> Self {
        Self::new(-threshold, threshold)
    }

    /// Returns the minimum bound.
    #[must_use]
    #[inline]
    pub fn min(&self) -> f32 {
        self.negative_max
    }

    /// Returns the maximum bounds.
    #[must_use]
    #[inline]
    pub fn max(&self) -> f32 {
        self.positive_min
    }

    /// Returns the minimum and maximum bounds.
    #[must_use]
    #[inline]
    pub fn min_max(&self) -> (f32, f32) {
        (self.negative_max, self.positive_min)
    }

    /// Returns the center of this deadzone range.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use leafwing_input_manager::prelude::*;
    ///
    /// let deadzone = AxisExclusion::new(0.0, 0.2);
    ///
    /// assert_eq!(deadzone.center(), 0.1);
    /// ```
    #[must_use]
    #[inline]
    pub fn center(&self) -> f32 {
        (self.negative_max + self.positive_min) * 0.5
    }

    /// Checks whether the `input_value` should be excluded.
    #[must_use]
    #[inline]
    pub fn contains(&self, input_value: f32) -> bool {
        self.negative_max <= input_value && input_value <= self.positive_min
    }

    /// Creates a new [`AxisDeadzone`] that normalizes input values
    /// within the livezone regions defined by the `self`.
    pub fn normalized(&self) -> AxisDeadzone {
        AxisDeadzone::new(*self)
    }
}

impl Eq for AxisExclusion {}

impl Hash for AxisExclusion {
    fn hash<H: Hasher>(&self, state: &mut H) {
        FloatOrd(self.negative_max).hash(state);
        FloatOrd(self.positive_min).hash(state);
    }
}

// endregion exclusion

// region deadzone

/// Defines a deadzone that normalizes input values by clamping values to `[-1.0, 1.0]`,
/// excluding values within a specified exclusion range,
/// and scaling unchanged values linearly in between.
///
/// # Warning
///
/// - Using an `exclusion` exceeding all bounds will exclude all input values.
///
/// # Examples
///
/// ```rust
/// use leafwing_input_manager::prelude::*;
///
/// // Create a deadzone with a specified exclusion range.
/// let exclusion = AxisExclusion::new(-0.2, 0.3);
/// let deadzone = AxisDeadzone::new(exclusion);
///
/// // Another way to create an AxisDeadzone.
/// let alternative = exclusion.normalized();
/// assert_eq!(alternative, deadzone);
///
/// // The bounds after normalization.
/// let bounds = AxisBounds::default();
///
/// // These values are within the exclusion.
/// let values = [-0.2, 0.05, 0.3];
/// for value in values {
///     assert!(exclusion.contains(value));
///
///     // So the value should be treated as zero.
///     assert_eq!(deadzone.process(value), 0.0);
/// }
///
/// // These values are out of the bounds.
/// let values = [-5.0, -2.0, 2.0, 5.0];
/// for value in values {
///     assert!(!bounds.contains(value));
///
///     // So the value should be clamped to the bounds.
///     let result = deadzone.process(value);
///     assert!(result == bounds.min() || result == bounds.max());
/// }
///
/// // These values are within the lower livezone.
/// let values = [-0.9, -0.7, -0.5, -0.3];
/// for value in values {
///     assert!(!exclusion.contains(value));
///     assert!(bounds.contains(value));
///
///     // So the value should be normalized to fit the range.
///     let (value_min, value_max) = deadzone.livezone_lower_min_max();
///     let expected = (value - value_max) / (value_max - value_min);
///     assert!(deadzone.process(value) - expected <= f32::EPSILON);
/// }
///
/// // These values are within the upper livezone.
/// let values = [0.9, 0.7, 0.5, 0.35];
/// for value in values {
///     assert!(!exclusion.contains(value));
///     assert!(bounds.contains(value));
///
///     // So the value should be normalized to fit the range.
///     let (value_min, value_max) = deadzone.livezone_upper_min_max();
///     let expected = (value - value_min) / (value_max - value_min);
///     assert!(deadzone.process(value) - expected <= f32::EPSILON);
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct AxisDeadzone {
    /// The exclusion used for normalization.
    pub(crate) exclusion: AxisExclusion,

    /// Pre-calculated reciprocal of the lower livezone width,
    /// preventing division during normalization.
    pub(crate) livezone_lower_recip: f32,

    /// Pre-calculated reciprocal of the upper livezone width,
    /// preventing division during normalization.
    pub(crate) livezone_upper_recip: f32,
}

#[typetag::serde]
impl AxisProcessor for AxisDeadzone {
    /// Processes the `input_value` by clamping values to `[-1.0, 1.0]`,
    /// excluding those within the deadzone, and scaling unchanged values linearly in between.
    #[must_use]
    fn process(&self, input_value: f32) -> f32 {
        if input_value <= 0.0 {
            let (bound, deadzone) = self.livezone_lower_min_max();
            let clamped_input = input_value.max(bound);
            let distance_to_deadzone = (clamped_input - deadzone).min(0.0);
            distance_to_deadzone * self.livezone_lower_recip
        } else {
            let (deadzone, bound) = self.livezone_upper_min_max();
            let clamped_input = input_value.min(bound);
            let distance_to_deadzone = (clamped_input - deadzone).max(0.0);
            distance_to_deadzone * self.livezone_upper_recip
        }
    }
}

impl Default for AxisDeadzone {
    /// Creates a [`AxisDeadzone`] that normalizes input values by clamping values to `[-1.0, 1.0]`,
    /// excluding values within `[-0.1, 0.1]`, and scaling unchanged values linearly in between.
    #[inline]
    fn default() -> Self {
        Self::new(AxisExclusion::default())
    }
}

impl AxisDeadzone {
    /// Creates a [`AxisDeadzone`] that normalizes input values by clamping values to `[-1.0, 1.0]`,
    /// excluding values within the given `exclusion` range,
    /// and scaling unchanged values linearly in between.
    ///
    /// # Warning
    ///
    /// - Using an `exclusion` exceeding all bounds will exclude all input values.
    #[inline]
    pub fn new(exclusion: AxisExclusion) -> Self {
        let (exclusion_min, exclusion_max) = exclusion.min_max();
        let (bound_min, bound_max) = AxisBounds::default().min_max();
        Self {
            exclusion,
            livezone_lower_recip: (exclusion_min - bound_min).recip(),
            livezone_upper_recip: (bound_max - exclusion_max).recip(),
        }
    }

    /// Returns the [`AxisExclusion`] used by this normalizer.
    pub fn exclusion(&self) -> AxisExclusion {
        self.exclusion
    }

    /// Returns the [`AxisBounds`] used by this normalizer.
    pub fn bounds(&self) -> AxisBounds {
        AxisBounds::default()
    }

    /// Returns the minimum and maximum bounds of the lower livezone range used by this normalizer.
    ///
    /// In simple terms, this returns `(bounds.min, exclusion.min)`.
    pub fn livezone_lower_min_max(&self) -> (f32, f32) {
        (self.bounds().min(), self.exclusion.min())
    }

    /// Returns the minimum and maximum bounds of the upper livezone range used by this normalizer.
    ///
    /// In simple terms, this returns `(exclusion.max, bounds.max)`.
    pub fn livezone_upper_min_max(&self) -> (f32, f32) {
        (self.exclusion.max(), self.bounds().max())
    }
}

impl Eq for AxisDeadzone {}

impl Hash for AxisDeadzone {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.exclusion.hash(state);
    }
}

// endregion deadzone

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use bevy::prelude::*;
    use serde::{Deserialize, Serialize};

    #[test]
    fn test_axis_processing_pipeline() {
        // Add processors to a new pipeline.
        let mut pipeline = AxisProcessingPipeline::default()
            .with(AxisSensitivity(4.0))
            .with(AxisInverted)
            .with(AxisInverted)
            .with(AxisSensitivity(4.0));

        // Replace the 3rd processor.
        pipeline.set(3, AxisSensitivity(6.0));

        // This pipeline now scales input values by a factor of 24.0
        assert_eq!(pipeline.process(2.0), 48.0);
        assert_eq!(pipeline.process(-3.0), -72.0);
    }

    #[test]
    fn test_inlined_axis_processing_pipeline() {
        // Define an optimized pipeline.
        define_axis_processing_pipeline!(
            name: InvertedThenDouble,
            processors: [AxisInverted, AxisSensitivity(2.0)]
        );

        // This pipeline now inverts and scales input values by a factor of 2.0
        let pipeline = InvertedThenDouble;
        assert_eq!(pipeline.process(2.0), -4.0);
        assert_eq!(pipeline.process(-1.0), 2.0);
    }

    #[test]
    fn test_axis_inverted() {
        let values = [-2.0, -4.0, -9.0, 0.0, 1.0, 40.0, 0.5, -0.2];
        for value in values {
            assert_eq!(AxisInverted.process(value), -value);
        }
    }

    #[test]
    fn test_axis_sensitivity() {
        let values = [-2.0, -4.0, -9.0, 0.0, 1.0, 40.0, 0.5, -0.2];
        for value in values {
            for sensitivity in values {
                assert_eq!(
                    AxisSensitivity(sensitivity).process(value),
                    sensitivity * value
                );
            }
        }
    }

    #[test]
    fn test_axis_value_bounds_constructors() {
        // -1 to 1
        let bounds = AxisBounds::default();
        assert_eq!(bounds.min(), -1.0);
        assert_eq!(bounds.max(), 1.0);

        // -2 to 3
        let bounds = AxisBounds::new(-2.0, 3.0);
        assert_eq!(bounds.min(), -2.0);
        assert_eq!(bounds.max(), 3.0);

        // -4 to 4
        let bounds = AxisBounds::symmetric(4.0);
        assert_eq!(bounds.min(), -4.0);
        assert_eq!(bounds.max(), 4.0);

        // -10 to f32::MAX
        let bounds = AxisBounds::at_least(-10.0);
        assert_eq!(bounds.min(), -10.0);
        assert_eq!(bounds.max(), f32::MAX);

        // f32::MIN to 15
        let bounds = AxisBounds::at_most(15.0);
        assert_eq!(bounds.min(), f32::MIN);
        assert_eq!(bounds.max(), 15.0);

        // f32::MIN to f32::MAX
        let bounds = AxisBounds::full_range();
        assert_eq!(bounds.min(), f32::MIN);
        assert_eq!(bounds.max(), f32::MAX);
    }

    #[test]
    fn test_axis_value_bounds_behavior() {
        let bounds = AxisBounds::new(-2.0, 3.0);

        // Getters.
        assert_eq!(bounds.min(), -2.0);
        assert_eq!(bounds.max(), 3.0);

        let (bound_min, bound_max) = bounds.min_max();
        assert_eq!(bound_min, bounds.min());
        assert_eq!(bound_max, bounds.max());
        assert_eq!(bounds.center(), (bound_min + bound_max) / 2.0);

        // bounds.min <= value <= bounds.center
        let values = [-0.5, -1.0, -2.0];
        for value in values {
            assert!(value >= bounds.min());
            assert!(value <= bounds.center());

            // So the value is within the bounds.
            assert!(bounds.contains(value));

            // So the value should be left unchanged after processing.
            let result = bounds.process(value);
            assert_eq!(result, value);
        }

        // bounds.center <= value <= bounds.max
        let values = [3.0, 1.0, 0.5];
        for value in values {
            assert!(value <= bounds.max());
            assert!(value >= bounds.center());

            // So the value is within the bounds.
            assert!(bounds.contains(value));

            // So the value should be left unchanged after processing.
            let result = bounds.process(value);
            assert_eq!(result, value);
        }

        // value < bounds.min
        let values = [-2.1, -5.0, -500.0];
        for value in values {
            assert!(value < bounds.min());

            // So the value is out of the bounds.
            assert!(!bounds.contains(value));

            // So the value should be clamped to the minimum bound.
            assert_eq!(bounds.process(value), bounds.min());
        }

        // value > bounds.max
        let values = [500.0, 5.0, 3.1];
        for value in values {
            assert!(value > bounds.max());

            // So the value is out of the bounds.
            assert!(!bounds.contains(value));

            // So the value should be clamped to the maximum bound.
            assert_eq!(bounds.process(value), bounds.max());
        }
    }

    #[test]
    fn test_axis_exclusion_constructors() {
        // -0.1 to 0.1
        let exclusion = AxisExclusion::default();
        assert_eq!(exclusion.min(), -0.1);
        assert_eq!(exclusion.max(), 0.1);

        // -2 to 3
        let exclusion = AxisExclusion::new(-2.0, 3.0);
        assert_eq!(exclusion.min(), -2.0);
        assert_eq!(exclusion.max(), 3.0);

        // -4 to 4
        let exclusion = AxisExclusion::symmetric(4.0);
        assert_eq!(exclusion.min(), -4.0);
        assert_eq!(exclusion.max(), 4.0);
    }

    #[test]
    fn test_axis_exclusion_behavior() {
        let exclusion = AxisExclusion::new(-2.0, 3.0);

        // Getters.
        assert_eq!(exclusion.min(), -2.0);
        assert_eq!(exclusion.max(), 3.0);

        let (min, max) = exclusion.min_max();
        assert_eq!(min, exclusion.min());
        assert_eq!(max, exclusion.max());
        assert_eq!(exclusion.center(), (min + max) / 2.0);

        // exclusion.min <= value <= exclusion.max
        let values = [3.0, 1.0, 0.5, -0.5, -1.0, -2.0];
        for value in values {
            assert!(value >= exclusion.min());
            assert!(value <= exclusion.max());

            // So the value should be excluded
            assert!(exclusion.contains(value));

            // So the value should be treated as zero.
            assert_eq!(exclusion.process(value), 0.0);
        }

        // value < exclusion.min
        let values = [-2.1, -5.0, -500.0];
        for value in values {
            assert!(value < exclusion.min());

            // So the value shouldn't be excluded
            assert!(!exclusion.contains(value));

            // So the value should be left unchanged after processing.
            assert_eq!(exclusion.process(value), value);
        }

        // value > exclusion.max
        let values = [500.0, 5.0, 3.1];
        for value in values {
            assert!(value > exclusion.max());

            // So the value shouldn't be excluded
            assert!(!exclusion.contains(value));

            // So the value should be left unchanged after processing.
            assert_eq!(exclusion.process(value), value);
        }
    }

    #[test]
    fn test_axis_deadzone() {
        let exclusion = AxisExclusion::new(-0.2, 0.3);
        let deadzone = AxisDeadzone::new(exclusion);
        assert_eq!(deadzone, exclusion.normalized());

        // The bounds after normalization.
        let bounds = AxisBounds::default();

        // Getters.
        assert_eq!(deadzone.exclusion(), exclusion);
        assert_eq!(deadzone.bounds(), bounds);
        assert_eq!(bounds.min(), -1.0);
        assert_eq!(exclusion.min(), -0.2);
        assert_eq!(exclusion.max(), 0.3);
        assert_eq!(bounds.max(), 1.0);

        let (lower_min, lower_max) = deadzone.livezone_lower_min_max();
        let (upper_min, upper_max) = deadzone.livezone_upper_min_max();
        assert_eq!(lower_min, bounds.min());
        assert_eq!(lower_max, exclusion.min());
        assert_eq!(upper_min, exclusion.max());
        assert_eq!(upper_max, bounds.max());

        // Inner factors.
        let expected_lower_recip = (lower_max - lower_min).recip();
        assert_eq!(deadzone.livezone_lower_recip, expected_lower_recip);
        let expected_upper_recip = (upper_max - upper_min).recip();
        assert_eq!(deadzone.livezone_upper_recip, expected_upper_recip);

        // exclusion.min <= value <= exclusion.max
        let values = [-0.2, 0.05, 0.25];
        for value in values {
            assert!(value >= exclusion.min());
            assert!(value <= exclusion.max());

            // So the value should be excluded.
            assert!(exclusion.contains(value));

            // So the value should be treated as zeros.
            assert_eq!(deadzone.process(value), 0.0);
        }

        // value < bounds.min
        let values = [-5.0, -2.0];
        for value in values {
            assert!(value < bounds.min());

            // So the value is out of the bounds.
            assert!(!bounds.contains(value));

            // So the value should be clamped to the minimum bound.
            assert_eq!(deadzone.process(value), bounds.min());
        }

        // value > bounds.max
        let values = [2.0, 5.0];
        for value in values {
            assert!(value > bounds.max());

            // So the value is out of the bounds.
            assert!(!bounds.contains(value));

            // So the value should be clamped to the maximum bound.
            assert_eq!(deadzone.process(value), bounds.max());
        }

        // lower_min < value < lower_max
        let values = [-0.9, -0.7, -0.5, -0.3];
        for value in values {
            assert!(value > lower_min);
            assert!(value < lower_max);

            // So the value is within the lower livezone range.
            assert!(!exclusion.contains(value));
            assert!(bounds.contains(value));

            // So the value should be normalized to fit the range.
            let result = deadzone.process(value);
            assert!(result > lower_min);
            assert!(result < 0.0);

            // The result is scaled by the ratio of the value in the livezone range.
            let value_in_livezone = value - lower_max;
            let livezone_width = lower_max - lower_min;
            assert_eq!(result, value_in_livezone / livezone_width);
            assert_eq!(result, value_in_livezone * deadzone.livezone_lower_recip);
        }

        // upper_min < value < upper_max
        let values = [0.9, 0.7, 0.5, 0.4];
        for value in values {
            assert!(value > upper_min);
            assert!(value < upper_max);

            // So the value is within the upper livezone range.
            assert!(!exclusion.contains(value));
            assert!(bounds.contains(value));

            // So the value should be normalized to fit the range.
            let result = deadzone.process(value);
            assert!(result > 0.0);
            assert!(result < upper_max);

            // The result is scaled by the ratio of the value in the livezone range.
            let value_in_livezone = value - upper_min;
            let livezone_width = upper_max - upper_min;
            let delta = result - value_in_livezone / livezone_width;
            assert!(delta.abs() <= f32::EPSILON);
        }
    }
}
