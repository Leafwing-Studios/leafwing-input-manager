//! Input processors for dual-axis values

use std::fmt::Debug;
use std::hash::Hash;

use bevy::prelude::*;
use dyn_clone::DynClone;
use dyn_eq::DynEq;
use dyn_hash::DynHash;

use super::single_axis::*;

pub use self::bounds::*;
pub use self::deadzone::*;

mod bounds;
mod deadzone;

// region processor trait

/// A trait for defining processors applied to input values on the X and Y axes.
/// These processors accept a [`Vec2`] input and produce a [`Vec2`] output.
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
/// /// Doubles the input, takes its absolute value,
/// /// and discards results that meet the specified condition on the X-axis.
/// // If your processor includes fields not implemented Eq and Hash,
/// // implementation is necessary as shown below.
/// // Otherwise, you can derive Eq and Hash directly.
/// #[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
/// pub struct DoubleAbsoluteValueThenRejectX(f32);
///
/// // Add this attribute for ensuring proper serialization and deserialization.
/// #[serde_trait_object]
/// impl DualAxisProcessor for DoubleAbsoluteValueThenRejectX {
///     fn process(&self, input_value: Vec2) -> Vec2 {
///         // Implement the logic just like you would in a normal function.
///
///         // You can use other processors within this function.
///         let value = AxisSensitivity(2.0).extend_dual().process(input_value);
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
/// // Others are left unchanged.
/// assert_eq!(processor.process(Vec2::splat(6.0)), Vec2::splat(12.0));
/// assert_eq!(processor.process(Vec2::splat(4.0)), Vec2::splat(8.0));
/// assert_eq!(processor.process(Vec2::splat(0.0)), Vec2::splat(0.0));
/// assert_eq!(processor.process(Vec2::splat(-4.0)), Vec2::splat(8.0));
/// assert_eq!(processor.process(Vec2::splat(-6.0)), Vec2::splat(12.0));
/// ```
#[typetag::serde(tag = "type")]
pub trait DualAxisProcessor: Send + Sync + Debug + DynClone + DynEq + DynHash + Reflect {
    /// Processes the `input_value` and returns the result.
    fn process(&self, input_value: Vec2) -> Vec2;
}

dyn_clone::clone_trait_object!(DualAxisProcessor);
dyn_eq::eq_trait_object!(DualAxisProcessor);
dyn_hash::hash_trait_object!(DualAxisProcessor);
crate::__reflect_box_dyn_trait_object!(DualAxisProcessor);

// endregion processor trait

// region pipeline

/// Defines an optimized pipeline that sequentially processes input values
/// using a chain of specified [`DualAxisProcessor`]s with inlined logic.
///
/// # Examples
///
/// ```rust
/// use bevy::prelude::*;
/// use leafwing_input_manager::prelude::*;
/// use serde::{Serialize, Deserialize};
///
/// define_dual_axis_processing_pipeline!(
///     // The name of the new pipeline.
///     name: InvertedThenDouble,
///     // Processors used in the pipeline.
///     processors: [AxisInverted.extend_dual(), AxisSensitivity(2.0).extend_dual()]
/// );
///
/// // This new pipeline is just a unit struct with inlined logic.
/// let processor = InvertedThenDouble;
///
/// // Now you can use it!
/// assert_eq!(processor.process(Vec2::splat(2.0)), Vec2::splat(-4.0));
/// assert_eq!(processor.process(Vec2::splat(-1.0)), Vec2::splat(2.0));
/// ```
#[macro_export]
macro_rules! define_dual_axis_processing_pipeline {
    (name: $Pipeline:ident, processors: [$($processor:expr),* $(,)?]) => {
        $crate::define_input_processing_pipeline!(
            name: $Pipeline,
            value_type: Vec2,
            processor_type: DualAxisProcessor,
            processors: [$($processor,)*]
        );
    };
}

crate::define_dynamic_input_processing_pipeline!(
    name: DualAxisProcessingPipeline,
    value_type: Vec2,
    processor_type: DualAxisProcessor
);

/// A trait for processors that can be merged with other [`DualAxisProcessor`]s.
pub trait MergeDualAxisProcessor {
    /// Merges this processor with another [`DualAxisProcessor`].
    fn merge_processor(self, processor: impl DualAxisProcessor) -> Self;
}

impl MergeDualAxisProcessor for Box<dyn DualAxisProcessor> {
    fn merge_processor(self, processor: impl DualAxisProcessor) -> Self {
        let pipeline = match Reflect::as_any(&*self).downcast_ref::<DualAxisProcessingPipeline>() {
            Some(pipeline) => pipeline.clone(),
            None => DualAxisProcessingPipeline(vec![self]),
        };
        Box::new(pipeline.with(processor))
    }
}

// endregion pipeline

// region macros

/// Generates a [`DualAxisProcessor`] enum representing different ways of applying the `operation`
/// to input values on the X and Y axes using the specified `AxisProcessor`.
///
/// # Examples
///
/// ```ignore
/// use bevy::prelude::*;
/// use leafwing_input_manager::prelude::*;
/// use serde::{Serialize, Deserialize};
///
/// // Create a DualAxisProcessor using AxisProcessor that defined as unit struct.
/// define_dual_axis_processor!(
///     // The name of your processor.
///     name: MyDualAxisInverted,
///     // The operation name you want to add into the doc.
///     perform: "my operation",
///     // Specifies the unit struct used as AxisProcessor.
///     unit_processor_type: MyAxisProcessor
///     // Optional: Uncomment the next line to add additional description if needed.
///     // info: "the additional description of `MyDualAxisProcessor`"
/// );
///
/// // Create a DualAxisProcessor using `AxisProcessor stored in its variants.
/// define_dual_axis_processor!(
///     name: MyDualAxisProcessor,
///     perform: "my operation",
///     // The only thing to change.
///     stored_processor_type: MyAxisProcessor
///     // Add additional descriptions if needed.
/// );
/// ```
#[macro_export]
macro_rules! define_dual_axis_processor {
    // This branch defines a DualAxisProcessor that performs the operation
    // using the AxisProcessor directly during processing
    (
        name: $DualAxisProcessor:ident,
        perform: $operation:literal,
        unit_processor_type: $AxisProcessor:ident
    ) => {
        $crate::define_dual_axis_processor!(
            name: $DualAxisProcessor,
            perform: $operation,
            unit_processor_type: $AxisProcessor,
            info: ""
        );
    };

    // This branch defines a DualAxisProcessor that performs the operation
    // using the AxisProcessor directly during processing
    // with additional description in the documentation
    (
        name: $DualAxisProcessor:ident,
        perform: $operation:literal,
        unit_processor_type: $AxisProcessor:ident,
        info: $information:literal
    ) => {
        #[doc = concat!("Applies ", $operation, " to input values on the X and Y axes using the specified [`", stringify!($AxisProcessor), "`] processors.")]
        ///
        #[doc = $information]
        ///
        #[doc = concat!("In simple terms, this processor is just the dual-axis version of [`", stringify!($AxisProcessor), "`]. ")]
        /// Please refer to its documentation for detailed examples and usage guidelines.
        ///
        /// # Variants
        ///
        #[doc = concat!("- [`", stringify!($DualAxisProcessor), "::All`]: Applies ", $operation, " to all axes.")]
        #[doc = concat!("- [`", stringify!($DualAxisProcessor), "::OnlyX`]: Applies ", $operation, " only to the X-axis.")]
        #[doc = concat!("- [`", stringify!($DualAxisProcessor), "::OnlyY`]: Applies ", $operation, " only to the Y-axis.")]
        ///
        /// # Notes
        ///
        #[doc = concat!("Helpers like [`", stringify!($AxisProcessor), "::extend_dual()`] and its peers can be used to create an instance of [`", stringify!($DualAxisProcessor), "`].")]
        #[derive(Clone, Copy, PartialEq, Eq, Hash, ::bevy::reflect::Reflect, ::serde::Serialize, ::serde::Deserialize)]
        #[must_use]
        pub enum $DualAxisProcessor {
            #[doc = concat!("Applies ", $operation, " to all axes.")]
            All,

            #[doc = concat!("Applies ", $operation, " only to the X-axis.")]
            OnlyX,

            #[doc = concat!("Applies ", $operation, " only to the Y-axis.")]
            OnlyY,
        }

        #[$crate::prelude::serde_trait_object]
        impl $crate::prelude::DualAxisProcessor for $DualAxisProcessor {
            #[doc = concat!("Applies ", $operation, " to the `input_value` and returns the result.")]
            #[must_use]
            #[inline]
            fn process(&self, input_value: Vec2) -> Vec2 {
                let Vec2 { x, y } = input_value;
                match self {
                    Self::OnlyX => {
                        let new_x = $crate::prelude::AxisProcessor::process(&$AxisProcessor, x);
                        Vec2::new(new_x, y)
                    }
                    Self::OnlyY => {
                        let new_y = $crate::prelude::AxisProcessor::process(&$AxisProcessor, y);
                        Vec2::new(x, new_y)
                    }
                    Self::All => Vec2::new(
                        $crate::prelude::AxisProcessor::process(&$AxisProcessor, x),
                        $crate::prelude::AxisProcessor::process(&$AxisProcessor, y),
                    ),
                }
            }
        }

        impl $DualAxisProcessor {
            #[doc = concat!("Checks if ", $operation, " is applied to the X-axis.")]
            #[must_use]
            #[inline]
            pub fn applied_x(&self) -> bool {
                *self != Self::OnlyY
            }

            #[doc = concat!("Checks if ", $operation, " is applied to the Y-axis.")]
            #[must_use]
            #[inline]
            pub fn applied_y(&self) -> bool {
                *self != Self::OnlyX
            }
        }

        // Conversion of $axis_processor into $dual_axis_processor
        impl $AxisProcessor {
            #[doc = concat!("Creates a new [`", stringify!($DualAxisProcessor), "::All`] that applies ", $operation, " to all axes.")]
            #[inline]
            pub fn extend_dual(self) -> $DualAxisProcessor {
                $DualAxisProcessor::All
            }

            #[doc = concat!("Creates a new [`", stringify!($DualAxisProcessor), "::OnlyX`] that applies ", $operation, " only to the X-axis.")]
            #[inline]
            pub fn extend_dual_only_x(self) -> $DualAxisProcessor {
                $DualAxisProcessor::OnlyX
            }

            #[doc = concat!("Creates a new [`", stringify!($DualAxisProcessor), "::OnlyY`] that applies ", $operation, " only to the Y-axis.")]
            #[inline]
            pub fn extend_dual_only_y(self) -> $DualAxisProcessor {
                $DualAxisProcessor::OnlyY
            }
        }

        impl ::core::fmt::Debug for $DualAxisProcessor {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match self {
                    Self::All => f.write_str(concat!("DualAxis::All(", stringify!($AxisProcessor), ")")),
                    Self::OnlyX => f.write_str(concat!("DualAxis::OnlyX(", stringify!($AxisProcessor), ")")),
                    Self::OnlyY => f.write_str(concat!("DualAxis::OnlyY(", stringify!($AxisProcessor), ")")),
                }
            }
        }
    };

    // This branch defines a DualAxisProcessor that performs the operation
    // using the specified AxisProcessor stored in the variants.
    (
        name: $DualAxisProcessor:ident,
        perform: $operation:literal,
        stored_processor_type: $AxisProcessor:ident
    ) => {
        $crate::define_dual_axis_processor!(
            name: $DualAxisProcessor,
            perform: $operation,
            stored_processor_type: $AxisProcessor,
            info: ""
        );
    };

    // This branch defines a DualAxisProcessor that performs the operation
    // using the specified AxisProcessor stored in the variants
    // with additional description in the documentation.
    (
        name: $DualAxisProcessor:ident,
        perform: $operation:literal,
        stored_processor_type: $AxisProcessor:ident,
        info: $information:literal
    ) => {
        #[doc = concat!("Applies ", $operation, " to input values on the X and Y axes using the specified [`", stringify!($AxisProcessor), "`] processors.")]
        ///
        #[doc = $information]
        ///
        #[doc = concat!("In simple terms, this processor is just the dual-axis version of [`", stringify!($AxisProcessor), "`].")]
        /// Please refer to its documentation for detailed examples and usage guidelines.
        ///
        /// # Variants
        ///
        #[doc = concat!("- [`", stringify!($DualAxisProcessor), "::All`]: Applies a specified [`", stringify!($AxisProcessor), "`] processor to both the X and Y axes.")]
        #[doc = concat!("- [`", stringify!($DualAxisProcessor), "::Separate`]: Applies two [`", stringify!($AxisProcessor), "`] processors, one for the X-axis and one for the Y-axis.")]
        #[doc = concat!("- [`", stringify!($DualAxisProcessor), "::OnlyX`]: Applies a specified [`", stringify!($AxisProcessor), "`] processors only to the X-axis.")]
        #[doc = concat!("- [`", stringify!($DualAxisProcessor), "::OnlyY`]: Applies a specified [`", stringify!($AxisProcessor), "`] processors only to the Y-axis.")]
        ///
        /// # Notes
        ///
        #[doc = concat!("Helpers like [`", stringify!($AxisProcessor), "::extend_dual()`] and its peers can be used to create an instance of [`", stringify!($DualAxisProcessor), "`].")]
        #[derive(Clone, Copy, PartialEq, Eq, Hash, ::bevy::reflect::Reflect, ::serde::Serialize, ::serde::Deserialize)]
        #[must_use]
        pub enum $DualAxisProcessor {
            #[doc = concat!("Applies a specified [`", stringify!($AxisProcessor), "`] processor to both the X and Y axes.")]
            All($AxisProcessor),

            #[doc = concat!("Applies two [`", stringify!($AxisProcessor), "`] processors, one for the X-axis and one for the Y-axis.")]
            Separate($AxisProcessor, $AxisProcessor),

            #[doc = concat!("Applies a specified [`", stringify!($AxisProcessor), "`] processors only to the X-axis.")]
            OnlyX($AxisProcessor),

            #[doc = concat!("Applies a specified [`", stringify!($AxisProcessor), "`] processors only to the Y-axis.")]
            OnlyY($AxisProcessor),
        }

        #[$crate::prelude::serde_trait_object]
        impl $crate::prelude::DualAxisProcessor for $DualAxisProcessor {
            #[doc = concat!("Applies ", $operation, " to the `input_value` and returns the result.")]
            #[must_use]
            #[inline]
            fn process(&self, input_value: Vec2) -> Vec2 {
                let Vec2 { x, y } = input_value;
                match self {
                    Self::OnlyX(processor) => {
                        let new_x = $crate::prelude::AxisProcessor::process(processor, x);
                        Vec2::new(new_x, y)
                    }
                    Self::OnlyY(processor) => {
                        let new_y = $crate::prelude::AxisProcessor::process(processor, y);
                        Vec2::new(x, new_y)
                    }
                    Self::All(processor) => Vec2::new(
                        $crate::prelude::AxisProcessor::process(processor, x),
                        $crate::prelude::AxisProcessor::process(processor, y),
                    ),
                    Self::Separate(processor_x, processor_y) => Vec2::new(
                        $crate::prelude::AxisProcessor::process(processor_x, x),
                        $crate::prelude::AxisProcessor::process(processor_y, y),
                    ),
                }
            }
        }

        impl $DualAxisProcessor {
            /// Returns the processor for the X-axis inputs, if exists.
            #[must_use]
            #[inline]
            pub fn x(&self) -> Option<$AxisProcessor> {
                match self {
                    Self::All(processor) => Some(*processor),
                    Self::Separate(processor_x, _) => Some(*processor_x),
                    Self::OnlyX(processor) => Some(*processor),
                    Self::OnlyY(_) => None,
                }
            }

            /// Returns the processor for the Y-axis inputs, if exists.
            #[must_use]
            #[inline]
            pub fn y(&self) -> Option<$AxisProcessor> {
                match self {
                    Self::All(processor) => Some(*processor),
                    Self::Separate(_, processor_y) => Some(*processor_y),
                    Self::OnlyX(_) => None,
                    Self::OnlyY(processor) => Some(*processor),
                }
            }

            /// Checks if there is an associated processor for the X-axis inputs.
            #[must_use]
            #[inline]
            pub fn applied_x(&self) -> bool {
                self.x().is_some()
            }

            /// Checks if there is an associated processor for the Y-axis inputs.
            #[must_use]
            #[inline]
            pub fn applied_y(&self) -> bool {
                self.y().is_some()
            }
        }

        // Conversion of $axis_processor into $dual_axis_processor
        impl $AxisProcessor {
            #[doc = concat!("Creates a new [`", stringify!($DualAxisProcessor), "::All`] that applies `self` to all axes.")]
            #[inline]
            pub fn extend_dual(self) -> $DualAxisProcessor {
                $DualAxisProcessor::All(self)
            }

            #[doc = concat!("Creates a new [`", stringify!($DualAxisProcessor), "::OnlyX`] that applies `self` only to the X-axis.")]
            #[inline]
            pub fn extend_dual_only_x(self) -> $DualAxisProcessor {
                $DualAxisProcessor::OnlyX(self)
            }

            #[doc = concat!("Creates a new [`", stringify!($DualAxisProcessor), "::OnlyY`] that applies `self` only to the Y-axis.")]
            #[inline]
            pub fn extend_dual_only_y(self) -> $DualAxisProcessor {
                $DualAxisProcessor::OnlyY(self)
            }

            #[doc = concat!("Creates a new [`", stringify!($DualAxisProcessor), "::Separate`] that applies `self` to the Y-axis with the given `x` processor to the X-axis.")]
            #[inline]
            pub fn extend_dual_with_x(self, x: Self) -> $DualAxisProcessor {
                $DualAxisProcessor::Separate(x, self)
            }

            #[doc = concat!("Creates a new [`", stringify!($DualAxisProcessor), "::Separate`] that applies `self` to the X-axis with the given `y` processor to the Y-axis.")]
            #[inline]
            pub fn extend_dual_with_y(self, y: Self) -> $DualAxisProcessor {
                $DualAxisProcessor::Separate(self, y)
            }
        }

        impl ::core::fmt::Debug for $DualAxisProcessor {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match self {
                    Self::All(processor) => f.write_fmt(format_args!("DualAxis::All({processor:?})")),
                    Self::Separate(processor_x, processor_y) => {
                        f.write_fmt(format_args!("DualAxis::Separate({processor_x:?}, {processor_y:?})"))
                    }
                    Self::OnlyX(processor) => f.write_fmt(format_args!("DualAxis::OnlyX({processor:?})")),
                    Self::OnlyY(processor) => f.write_fmt(format_args!("DualAxis::OnlyY({processor:?})")),
                }
            }
        }
    };
}

// endregion macros

// region inversion

crate::define_dual_axis_processor!(
    name: DualAxisInverted,
    perform: "inversion",
    unit_processor_type: AxisInverted
);

// endregion inversion

// region sensitivity

crate::define_dual_axis_processor!(
    name: DualAxisSensitivity,
    perform: "sensitivity scaling",
    stored_processor_type: AxisSensitivity
);

impl DualAxisSensitivity {
    /// Creates a new [`DualAxisSensitivity::All`] with the specified factor.
    #[inline]
    pub fn new(sensitivity: f32) -> Self {
        Self::All(AxisSensitivity(sensitivity))
    }

    /// Creates a new [`DualAxisSensitivity::Separate`] with the specified factors.
    #[inline]
    pub fn separate(sensitivity_x: f32, sensitivity_y: f32) -> Self {
        Self::Separate(
            AxisSensitivity(sensitivity_x),
            AxisSensitivity(sensitivity_y),
        )
    }

    /// Creates a new [`DualAxisSensitivity::OnlyX`] with the specified factor.
    #[inline]
    pub fn only_x(sensitivity: f32) -> Self {
        Self::OnlyX(AxisSensitivity(sensitivity))
    }

    /// Creates a new [`DualAxisSensitivity::OnlyY`] with the specified factor.
    #[inline]
    pub fn only_y(sensitivity: f32) -> Self {
        Self::OnlyY(AxisSensitivity(sensitivity))
    }
}

// endregion sensitivity

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dual_axis_processing_pipeline() {
        // Add processors to a new pipeline.
        let mut pipeline = DualAxisProcessingPipeline::default()
            .with(AxisSensitivity(4.0).extend_dual())
            .with(AxisInverted.extend_dual())
            .with(AxisInverted.extend_dual())
            .with(AxisSensitivity(4.0).extend_dual());

        // Replace the processor at index 3.
        pipeline.set(3, AxisSensitivity(6.0).extend_dual());

        // This pipeline now scales input values by a factor of 24.0
        assert_eq!(pipeline.process(Vec2::splat(2.0)), Vec2::splat(48.0));
        assert_eq!(pipeline.process(Vec2::splat(-3.0)), Vec2::splat(-72.0));
    }

    #[test]
    fn test_inlined_dual_axis_processing_pipeline() {
        // Define an optimized pipeline.
        define_dual_axis_processing_pipeline!(
            name: InvertedThenDouble,
            processors: [
                AxisInverted.extend_dual(),
                AxisSensitivity(2.0).extend_dual(),
            ]
        );

        // This pipeline now inverts and scales input values by a factor of 2.0
        let pipeline = InvertedThenDouble;
        assert_eq!(pipeline.process(Vec2::splat(2.0)), Vec2::splat(-4.0));
        assert_eq!(pipeline.process(Vec2::splat(-4.0)), Vec2::splat(8.0));
    }

    #[test]
    fn test_merge_axis_processor() {
        let first = AxisSensitivity(2.0).extend_dual();
        let first_boxed: Box<dyn DualAxisProcessor> = Box::new(first);

        let second = AxisSensitivity(3.0).extend_dual();
        let merged_second = first_boxed.merge_processor(second);
        let expected = DualAxisProcessingPipeline::default()
            .with(first)
            .with(second);
        let expected_boxed: Box<dyn DualAxisProcessor> = Box::new(expected);
        assert_eq!(merged_second, expected_boxed);

        let third = AxisSensitivity(4.0).extend_dual();
        let merged_third = merged_second.merge_processor(third);
        let expected = DualAxisProcessingPipeline::default()
            .with(first)
            .with(second)
            .with(third);
        let expected_boxed: Box<dyn DualAxisProcessor> = Box::new(expected);
        assert_eq!(merged_third, expected_boxed);
    }

    #[test]
    fn test_dual_axis_inverted() {
        let all = DualAxisInverted::All;
        let only_x = DualAxisInverted::OnlyX;
        let only_y = DualAxisInverted::OnlyY;

        // These should be identical.
        assert_eq!(all, AxisInverted.extend_dual());
        assert_eq!(only_x, AxisInverted.extend_dual_only_x());
        assert_eq!(only_y, AxisInverted.extend_dual_only_y());

        // Check if applied.
        assert!(all.applied_x());
        assert!(all.applied_y());
        assert!(only_x.applied_x());
        assert!(!only_x.applied_y());
        assert!(!only_y.applied_x());
        assert!(only_y.applied_y());

        // And they can invert the direction.
        assert_eq!(all.process(Vec2::ONE), Vec2::NEG_ONE);
        assert_eq!(only_x.process(Vec2::ONE), Vec2::new(-1.0, 1.0));
        assert_eq!(only_y.process(Vec2::ONE), Vec2::new(1.0, -1.0));
    }

    #[test]
    fn test_dual_axis_sensitivity() {
        let axis_x = AxisSensitivity(4.0);
        let axis_y = AxisSensitivity(5.0);

        // These should be identical.
        let all = DualAxisSensitivity::All(axis_x);
        assert_eq!(all, DualAxisSensitivity::new(4.0));
        assert_eq!(all, axis_x.extend_dual());

        let separate = DualAxisSensitivity::Separate(axis_x, axis_y);
        assert_eq!(separate, DualAxisSensitivity::separate(4.0, 5.0));
        assert_eq!(separate, axis_x.extend_dual_with_y(axis_y));
        assert_eq!(separate, axis_y.extend_dual_with_x(axis_x));

        let only_x = DualAxisSensitivity::OnlyX(axis_x);
        assert_eq!(only_x, DualAxisSensitivity::only_x(4.0));
        assert_eq!(only_x, axis_x.extend_dual_only_x());

        let only_y = DualAxisSensitivity::OnlyY(axis_y);
        assert_eq!(only_y, DualAxisSensitivity::only_y(5.0));
        assert_eq!(only_y, axis_y.extend_dual_only_y());

        // The DualAxisSensitivity is just a dual-axis version of AxisSensitivity.
        let value = Vec2::new(2.0, 3.0);
        assert_eq!(axis_x.process(value.x), 8.0);
        assert_eq!(axis_y.process(value.y), 15.0);
        assert_eq!(all.process(value).x, axis_x.process(value.x));
        assert_eq!(all.process(value).y, axis_x.process(value.y));
        assert_eq!(separate.process(value).x, axis_x.process(value.x));
        assert_eq!(separate.process(value).y, axis_y.process(value.y));
        assert_eq!(only_x.process(value).x, axis_x.process(value.x));
        assert_eq!(only_y.process(value).y, axis_y.process(value.y));
    }
}
