//! Processors for dual-axis input values

use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};

use bevy::prelude::*;
use bevy::utils::FloatOrd;
use dyn_clone::DynClone;
use dyn_eq::DynEq;
use dyn_hash::DynHash;
use serde::{Deserialize, Serialize};

pub use self::bounds::*;
pub use self::deadzone::*;

mod bounds;
mod deadzone;

/// A trait for processing dual-axis input values,
/// accepting a [`Vec2`] input and producing a [`Vec2`] output.
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
/// /// Doubles the input, takes its absolute value,
/// /// and discards results that meet the specified condition on the X-axis.
/// // If your processor includes fields not implemented Eq and Hash,
/// // implementation is necessary as shown below.
/// // Otherwise, you can derive Eq and Hash directly.
/// #[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
/// pub struct DoubleAbsoluteValueThenRejectX(pub f32);
///
/// // Add this attribute for ensuring proper serialization and deserialization.
/// #[serde_trait_object]
/// impl DualAxisProcessor for DoubleAbsoluteValueThenRejectX {
///     fn process(&self, input_value: Vec2) -> Vec2 {
///         // Implement the logic just like you would in a normal function.
///
///         // You can use other processors within this function.
///         let value = DualAxisSensitivity::all(2.0).process(input_value);
///
///         let value = value.abs();
///         let new_x = if value.x == self.0 {
///             0.0
///         } else {
///             value.x
///         };
///         Vec2::new(new_x, value.y)
///     }
/// }
///
/// // Unfortunately, manual implementation is required due to the float field.
/// impl Eq for DoubleAbsoluteValueThenRejectX {}
/// impl Hash for DoubleAbsoluteValueThenRejectX {
///     fn hash<H: Hasher>(&self, state: &mut H) {
///         // Encapsulate the float field for hashing.
///         FloatOrd(self.0).hash(state);
///     }
/// }
///
/// // Now you can use it!
/// let processor = DoubleAbsoluteValueThenRejectX(4.0);
///
/// // Rejected X!
/// assert_eq!(processor.process(Vec2::splat(2.0)), Vec2::new(0.0, 4.0));
/// assert_eq!(processor.process(Vec2::splat(-2.0)), Vec2::new(0.0, 4.0));
///
/// // Others are just doubled absolute value.
/// assert_eq!(processor.process(Vec2::splat(6.0)), Vec2::splat(12.0));
/// assert_eq!(processor.process(Vec2::splat(4.0)), Vec2::splat(8.0));
/// assert_eq!(processor.process(Vec2::splat(0.0)), Vec2::splat(0.0));
/// assert_eq!(processor.process(Vec2::splat(-4.0)), Vec2::splat(8.0));
/// assert_eq!(processor.process(Vec2::splat(-6.0)), Vec2::splat(12.0));
/// ```
#[typetag::serde(tag = "type")]
pub trait DualAxisProcessor: Send + Sync + Debug + DynClone + DynEq + DynHash + Reflect {
    /// Computes the result by processing the `input_value`.
    fn process(&self, input_value: Vec2) -> Vec2;
}

dyn_clone::clone_trait_object!(DualAxisProcessor);
dyn_eq::eq_trait_object!(DualAxisProcessor);
dyn_hash::hash_trait_object!(DualAxisProcessor);
crate::__reflect_trait_object!(DualAxisProcessor);

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
///     .with(DualAxisSensitivity::all(2.0))
///     .with(DualAxisInverted::ALL);
///
/// // Now it doubles and flips values.
/// assert_eq!(pipeline.process(input_value), -2.0 * input_value);
///
/// // You can also add processors just like you would do with a Vec.
/// pipeline.push(DualAxisSensitivity::only_x(1.5));
///
/// // Now it flips values and multiplies the results by [3, 2]
/// assert_eq!(pipeline.process(input_value), Vec2::new(-3.0, -2.0) * input_value);
///
/// // Plus, you can switch out a processor at a specific index.
/// pipeline.set(0, DualAxisSensitivity::all(3.0));
///
/// // Now it flips values and multiplies the results by [4.5, 3]
/// assert_eq!(pipeline.process(input_value), Vec2::new(-4.5, -3.0) * input_value);
///
/// // If needed, you can clear out all processors.
/// pipeline.clear();
///
/// // Now it just leaves values as is.
/// assert_eq!(pipeline.process(input_value), input_value);
/// ```
#[must_use]
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub struct DualAxisProcessingPipeline(pub(crate) Vec<Box<dyn DualAxisProcessor>>);

#[typetag::serde]
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
pub trait WithNextDualAxisProcessor {
    /// Appends a [`DualAxisProcessor`] as the next processing step in the pipeline.
    fn with_processor(self, processor: impl DualAxisProcessor) -> Self;
}

impl WithNextDualAxisProcessor for Box<dyn DualAxisProcessor> {
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

/// Flips the sign of dual-axis input values, resulting in a directional reversal of control.
///
/// ```rust
/// use bevy::prelude::*;
/// use leafwing_input_manager::prelude::*;
///
/// let value = Vec2::new(1.5, 2.0);
/// let Vec2 { x, y } = value;
///
/// assert_eq!(DualAxisInverted::ALL.process(value), -value);
/// assert_eq!(DualAxisInverted::ALL.process(-value), value);
///
/// assert_eq!(DualAxisInverted::ONLY_X.process(value), Vec2::new(-x, y));
/// assert_eq!(DualAxisInverted::ONLY_X.process(-value), Vec2::new(x, -y));
///
/// assert_eq!(DualAxisInverted::ONLY_Y.process(value), Vec2::new(x, -y));
/// assert_eq!(DualAxisInverted::ONLY_Y.process(-value), Vec2::new(-x, y));
#[derive(Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct DualAxisInverted(Vec2);

#[typetag::serde]
impl DualAxisProcessor for DualAxisInverted {
    /// Multiples the `input_value` by the specified inversion vector.
    #[must_use]
    #[inline]
    fn process(&self, input_value: Vec2) -> Vec2 {
        self.0 * input_value
    }
}

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
        self.0.cmpne(Vec2::NEG_ONE)
    }
}

impl Eq for DualAxisInverted {}

impl Hash for DualAxisInverted {
    fn hash<H: Hasher>(&self, state: &mut H) {
        FloatOrd(self.0.x).hash(state);
        FloatOrd(self.0.y).hash(state);
    }
}

impl Debug for DualAxisInverted {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "DualAxisInverted({:?})", self.inverted())
    }
}

/// Scales dual-axis input values using a specified multiplier,
/// allowing fine-tuning the responsiveness of controls.
///
/// # Examples
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
/// assert_eq!(neg_x_half_y.process(value).x, -x);
/// assert_eq!(neg_x_half_y.process(value).y, 0.5 * y);
///
/// // Doubled X and doubled Y
/// let double = DualAxisSensitivity::all(2.0);
/// assert_eq!(double.process(value), 2.0 * value);
///
/// // Halved X
/// let half_x = DualAxisSensitivity::only_x(0.5);
/// assert_eq!(half_x.process(value).x, 0.5 * x);
/// assert_eq!(half_x.process(value).y, y);
///
/// // Negated and doubled Y
/// let neg_double_y = DualAxisSensitivity::only_y(-2.0);
/// assert_eq!(neg_double_y.process(value).x, x);
/// assert_eq!(neg_double_y.process(value).y, -2.0 * y);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct DualAxisSensitivity(pub(crate) Vec2);

#[typetag::serde]
impl DualAxisProcessor for DualAxisSensitivity {
    /// Multiples the `input_value` by the specified sensitivity vector.
    #[must_use]
    #[inline]
    fn process(&self, input_value: Vec2) -> Vec2 {
        self.0 * input_value
    }
}

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
    fn test_merge_axis_processor() {
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

    #[test]
    fn test_dual_axis_inverted() {
        for x in -300..300 {
            let x = x as f32 * 0.01;

            for y in -300..300 {
                let y = y as f32 * 0.01;
                let value = Vec2::new(x, y);

                assert_eq!(DualAxisInverted::ALL.process(value), -value);
                assert_eq!(DualAxisInverted::ALL.process(-value), value);

                assert_eq!(DualAxisInverted::ONLY_X.process(value), Vec2::new(-x, y));
                assert_eq!(DualAxisInverted::ONLY_X.process(-value), Vec2::new(x, -y));

                assert_eq!(DualAxisInverted::ONLY_Y.process(value), Vec2::new(x, -y));
                assert_eq!(DualAxisInverted::ONLY_Y.process(-value), Vec2::new(-x, y));
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
                assert_eq!(all.sensitivities(), Vec2::splat(sensitivity));
                assert_eq!(all.process(value), sensitivity * value);

                let only_x = DualAxisSensitivity::only_x(sensitivity);
                assert_eq!(only_x.sensitivities(), Vec2::new(sensitivity, 1.0));
                assert_eq!(only_x.process(value).x, x * sensitivity);
                assert_eq!(only_x.process(value).y, y);

                let only_y = DualAxisSensitivity::only_y(sensitivity);
                assert_eq!(only_y.sensitivities(), Vec2::new(1.0, sensitivity));
                assert_eq!(only_y.process(value).x, x);
                assert_eq!(only_y.process(value).y, y * sensitivity);

                let sensitivity2 = y;
                let separate = DualAxisSensitivity::new(sensitivity, sensitivity2);
                assert_eq!(
                    separate.sensitivities(),
                    Vec2::new(sensitivity, sensitivity2)
                );
                assert_eq!(separate.process(value).x, x * sensitivity);
                assert_eq!(separate.process(value).y, y * sensitivity2);
            }
        }
    }
}
