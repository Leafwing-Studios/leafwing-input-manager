//! Processors for dual-axis input values

use std::fmt::Debug;

use bevy::prelude::*;
use dyn_clone::DynClone;
use dyn_eq::DynEq;
use dyn_hash::DynHash;

pub use self::circle::*;
pub use self::modifier::*;
pub use self::pipeline::*;
pub use self::range::*;

mod circle;
mod modifier;
mod pipeline;
mod range;

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
/// #[typetag::serde]
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
