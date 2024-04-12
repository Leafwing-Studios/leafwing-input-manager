//! Processors for single-axis input values

use std::fmt::Debug;

use bevy::prelude::*;
use dyn_clone::DynClone;
use dyn_eq::DynEq;
use dyn_hash::DynHash;

pub use self::modifier::*;
pub use self::pipeline::*;
pub use self::range::*;

mod modifier;
mod pipeline;
mod range;

/// A trait for processing single-axis input values,
/// accepting a `f32` input and producing a `f32` output.
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
/// pub struct DoubleAbsoluteValueThenIgnored(pub f32);
///
/// // Add this attribute for ensuring proper serialization and deserialization.
/// #[typetag::serde]
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
/// // Others are just doubled absolute value.
/// assert_eq!(processor.process(6.0), 12.0);
/// assert_eq!(processor.process(4.0), 8.0);
/// assert_eq!(processor.process(0.0), 0.0);
/// assert_eq!(processor.process(-4.0), 8.0);
/// assert_eq!(processor.process(-6.0), 12.0);
/// ```
#[typetag::serde(tag = "type")]
pub trait AxisProcessor: Send + Sync + Debug + DynClone + DynEq + DynHash + Reflect {
    /// Computes the result by processing the `input_value`.
    fn process(&self, input_value: f32) -> f32;
}

dyn_clone::clone_trait_object!(AxisProcessor);
dyn_eq::eq_trait_object!(AxisProcessor);
dyn_hash::hash_trait_object!(AxisProcessor);
crate::__reflect_trait_object!(AxisProcessor);
