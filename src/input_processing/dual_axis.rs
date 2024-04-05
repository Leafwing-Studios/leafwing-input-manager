//! Input processors for values along the X and Y axes

use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use bevy::prelude::{BVec2, Reflect, Vec2};
use bevy::utils::FloatOrd;
use dyn_clone::DynClone;
use dyn_eq::DynEq;
use dyn_hash::DynHash;
use serde::{Deserialize, Serialize};

use super::single_axis::*;

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
        #[doc = concat!("- [`", stringify!($DualAxisProcessor), "::AllAxes`]: Applies ", $operation, " to all axes.")]
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
            AllAxes,

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
                    Self::OnlyX => Vec2::new($AxisProcessor.process(x), y),
                    Self::OnlyY => Vec2::new(x, $AxisProcessor.process(y)),
                    Self::AllAxes => Vec2::new($AxisProcessor.process(x), $AxisProcessor.process(y)),
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
            #[doc = concat!("Creates a new [`", stringify!($DualAxisProcessor), "::AllAxes`] that applies ", $operation, " to all axes.")]
            #[inline]
            pub fn extend_dual(self) -> $DualAxisProcessor {
                $DualAxisProcessor::AllAxes
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
                    Self::AllAxes => f.write_str(concat!("AllAxes(", stringify!($AxisProcessor), ")")),
                    Self::OnlyX => f.write_str(concat!("OnlyX(", stringify!($AxisProcessor), ")")),
                    Self::OnlyY => f.write_str(concat!("OnlyY(", stringify!($AxisProcessor), ")")),
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
        #[doc = concat!("- [`", stringify!($DualAxisProcessor), "::AllAxes`]: Applies ", $operation, " to all axes using the same [`", stringify!($AxisProcessor), "`] processor.")]
        #[doc = concat!("- [`", stringify!($DualAxisProcessor), "::XY`]: Applies two [`", stringify!($AxisProcessor), "`] processors, one for the X-axis and one for the Y-axis.")]
        #[doc = concat!("- [`", stringify!($DualAxisProcessor), "::OnlyX`]: Applies ", $operation, " only to the X-axis using the specified [`", stringify!($AxisProcessor), "`] processor.")]
        #[doc = concat!("- [`", stringify!($DualAxisProcessor), "::OnlyY`]: Applies ", $operation, " only to the Y-axis using the specified [`", stringify!($AxisProcessor), "`] processor.")]
        ///
        /// # Notes
        ///
        #[doc = concat!("Helpers like [`", stringify!($AxisProcessor), "::extend_dual()`] and its peers can be used to create an instance of [`", stringify!($DualAxisProcessor), "`].")]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ::bevy::reflect::Reflect, ::serde::Serialize, ::serde::Deserialize)]
        #[must_use]
        pub enum $DualAxisProcessor {
            #[doc = concat!("Applies ", $operation, " to all axes using the same [`", stringify!($AxisProcessor), "`] processor.")]
            AllAxes($AxisProcessor),

            #[doc = concat!("Applies two [`", stringify!($AxisProcessor), "`] processors, one for the X-axis and one for the Y-axis.")]
            XY($AxisProcessor, $AxisProcessor),

            #[doc = concat!("Applies ", $operation, " only to the X-axis using the specified [`", stringify!($AxisProcessor), "`] processor.")]
            OnlyX($AxisProcessor),

            #[doc = concat!("Applies ", $operation, " only to the Y-axis using the specified [`", stringify!($AxisProcessor), "`] processor.")]
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
                    Self::OnlyX(p) => Vec2::new(p.process(x), y),
                    Self::OnlyY(p) => Vec2::new(x, p.process(y)),
                    Self::AllAxes(p) => Vec2::new(p.process(x), p.process(y)),
                    Self::XY(px, py) => Vec2::new(px.process(x), py.process(y)),
                }
            }
        }

        impl $DualAxisProcessor {
            /// Returns the processor for the X-axis inputs, if exists.
            #[must_use]
            #[inline]
            pub fn x(&self) -> Option<$AxisProcessor> {
                match self {
                    Self::OnlyX(p) => Some(*p),
                    Self::OnlyY(_) => None,
                    Self::AllAxes(p) => Some(*p),
                    Self::XY(px, _) => Some(*px),
                }
            }

            /// Returns the processor for the Y-axis inputs, if exists.
            #[must_use]
            #[inline]
            pub fn y(&self) -> Option<$AxisProcessor> {
                match self {
                    Self::OnlyX(_) => None,
                    Self::OnlyY(p) => Some(*p),
                    Self::AllAxes(p) => Some(*p),
                    Self::XY(_, py) => Some(*py),
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
            #[doc = concat!("Creates a new [`", stringify!($DualAxisProcessor), "::AllAxes`] that applies `self` to all axes.")]
            #[inline]
            pub fn extend_dual(self) -> $DualAxisProcessor {
                $DualAxisProcessor::AllAxes(self)
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

            #[doc = concat!("Creates a new [`", stringify!($DualAxisProcessor), "::XY`] that applies `self` to the Y-axis with the given `x` processor to the X-axis.")]
            #[inline]
            pub fn extend_dual_with_x(self, x: Self) -> $DualAxisProcessor {
                $DualAxisProcessor::XY(x, self)
            }

            #[doc = concat!("Creates a new [`", stringify!($DualAxisProcessor), "::XY`] that applies `self` to the X-axis with the given `y` processor to the Y-axis.")]
            #[inline]
            pub fn extend_dual_with_y(self, y: Self) -> $DualAxisProcessor {
                $DualAxisProcessor::XY(self, y)
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
    /// Creates a new [`DualAxisSensitivity::AllAxes`] with the specified factor.
    #[inline]
    pub fn new(sensitivity: f32) -> Self {
        Self::AllAxes(AxisSensitivity(sensitivity))
    }

    /// Creates a new [`DualAxisSensitivity::XY`] with the specified factors.
    #[inline]
    pub fn new_on_xy(sensitivity_x: f32, sensitivity_y: f32) -> Self {
        Self::XY(
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

// region bounds

/// Specifies a radial bound for input values,
/// ensuring their magnitudes smaller than a specified threshold.
///
/// # Examples
///
/// ```rust
/// use bevy::prelude::*;
/// use leafwing_input_manager::prelude::*;
///
/// // Set the maximum bound to 5 for magnitudes.
/// let bounds = CircleBounds::magnitude(5.0);
///
/// assert_eq!(bounds.radius(), 5.0);
///
/// // These values have a magnitude greater than radius.
/// let values = [Vec2::ONE * 5.0, Vec2::X * 10.0];
/// for value in values {
///     assert!(value.length() > bounds.radius());
///
///     // So the value is out of the bounds.
///     assert!(!bounds.contains(value));
///
///     // And the value should be clamped to the maximum bound.
///     let result = bounds.process(value);
///     assert_eq!(result.length(), bounds.radius());
///     assert_eq!(result.y.atan2(result.x), value.y.atan2(value.x));
/// }
///
/// // These values are within the bounds.
/// let values = [Vec2::ONE * 3.0, Vec2::X * 4.0];
/// for value in values {
///     assert!(bounds.contains(value));
///
///     // So the value should be left unchanged.
///     assert_eq!(bounds.process(value), value);
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct CircleBounds {
    /// The maximum radius of the circle.
    pub(crate) radius: f32,

    /// Pre-calculated squared `radius_max`,
    /// preventing redundant calculations.
    pub(crate) radius_squared: f32,
}

#[typetag::serde]
impl DualAxisProcessor for CircleBounds {
    /// Clamps the magnitude of `input_value` to fit within the bounds.
    #[must_use]
    #[inline]
    fn process(&self, input_value: Vec2) -> Vec2 {
        input_value.clamp_length_max(self.radius)
    }
}

impl Default for CircleBounds {
    /// Creates a new [`CircleBounds`] with the maximum bound set to `1.0`.
    #[inline]
    fn default() -> Self {
        Self::magnitude(1.0)
    }
}

impl CircleBounds {
    /// Creates a [`CircleBounds`] with the maximum bound set to `radius`.
    ///
    /// # Requirements
    ///
    /// - `radius` >= `0.0`.
    ///
    /// # Panics
    ///
    /// Panics if any of the requirements isn't met.
    #[inline]
    pub fn magnitude(radius: f32) -> Self {
        assert!(radius >= 0.0);
        Self {
            radius,
            radius_squared: radius.powi(2),
        }
    }

    /// Creates a [`CircleBounds`] with unlimited bounds.
    #[inline]
    pub fn full_range() -> Self {
        Self::magnitude(f32::MAX)
    }

    /// Returns the radius of the bounds.
    #[must_use]
    #[inline]
    pub fn radius(&self) -> f32 {
        self.radius
    }

    /// Returns the squared radius of the bounds.
    #[must_use]
    #[inline]
    pub fn radius_squared(&self) -> f32 {
        self.radius_squared
    }

    /// Checks whether the `input_value` is within this deadzone.
    #[must_use]
    #[inline]
    pub fn contains(&self, input_value: Vec2) -> bool {
        input_value.length_squared() <= self.radius_squared
    }
}

impl Eq for CircleBounds {}

impl Hash for CircleBounds {
    fn hash<H: Hasher>(&self, state: &mut H) {
        FloatOrd(self.radius).hash(state);
    }
}

define_dual_axis_processor!(
    name: SquareBounds,
    perform: "axial bounds",
    stored_processor_type: AxisBounds
);

impl Default for SquareBounds {
    /// Creates a new [`SquareBounds`] with bounds set to `[-1.0, 1.0]` on each axis.
    #[inline]
    fn default() -> Self {
        AxisBounds::default().extend_dual()
    }
}

impl SquareBounds {
    /// Checks whether the `input_value` is within the bounds along each axis.
    #[must_use]
    #[inline]
    pub fn contains(&self, input_value: Vec2) -> BVec2 {
        let Vec2 { x, y } = input_value;
        match self {
            Self::OnlyX(bounds) => BVec2::new(bounds.contains(x), true),
            Self::OnlyY(bounds) => BVec2::new(true, bounds.contains(y)),
            Self::AllAxes(bounds) => BVec2::new(bounds.contains(x), bounds.contains(y)),
            Self::XY(bounds_x, bounds_y) => BVec2::new(bounds_x.contains(x), bounds_y.contains(y)),
        }
    }
}

// endregion bounds

// region exclusion

/// Specifies a radial exclusion for input values,
/// excluding those with a magnitude less than a specified threshold.
///
/// In simple terms, this processor functions as an unscaled [`CircleDeadzone`].
/// This processor is useful for filtering out minor fluctuations and unintended movements.
///
/// # Requirements
///
/// - `radius` >= `0.0`.
///
/// # Examples
///
/// ```rust
/// use bevy::prelude::*;
/// use leafwing_input_manager::prelude::*;
///
/// // Set an exclusion with a radius of 9 for magnitudes.
/// let exclusion = CircleExclusion::new(9.0);
///
/// // These values have a magnitude within the radius.
/// let values = [Vec2::ONE, Vec2::X, Vec2::new(0.5, 3.0)];
/// for value in values {
///     assert!(value.length() <= exclusion.radius());
///
///     // So the value should be excluded.
///     assert!(exclusion.contains(value));
///
///     // So the value should be treated as zeros.
///     assert_eq!(exclusion.process(value), Vec2::ZERO);
/// }
///
/// // The values have a magnitude out of the radius.
/// let values = [Vec2::new(10.0, 12.0), Vec2::new(20.0, -5.0)];
/// for value in values {
///     assert!(value.length() > exclusion.radius());
///
///     // So the value is out of the range.
///     assert!(!exclusion.contains(value));
///
///     // So the value should be left unchanged.
///     assert_eq!(exclusion.process(value), value);
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct CircleExclusion {
    /// The radius of the circle.
    pub(crate) radius: f32,

    /// Pre-calculated squared `radius`,
    /// preventing redundant calculations.
    pub(crate) radius_squared: f32,
}

#[typetag::serde]
impl DualAxisProcessor for CircleExclusion {
    /// Excludes input values with a magnitude less than the `radius`.
    #[must_use]
    #[inline]
    fn process(&self, input_value: Vec2) -> Vec2 {
        if input_value.length_squared() <= self.radius_squared {
            Vec2::ZERO
        } else {
            input_value
        }
    }
}

impl Default for CircleExclusion {
    /// Creates a [`CircleExclusion`] with a radius of `0.1`.
    fn default() -> Self {
        Self::new(0.1)
    }
}

impl CircleExclusion {
    /// Creates a new [`CircleExclusion`] with the specified `radius`.
    ///
    /// # Requirements
    ///
    /// - `radius` >= `0.0`.
    ///
    /// # Panics
    ///
    /// Panics if any of the requirements isn't met.
    #[inline]
    pub fn new(radius: f32) -> Self {
        assert!(radius >= 0.0);
        Self {
            radius,
            radius_squared: radius.powi(2),
        }
    }

    /// Returns the radius of the circle.
    #[must_use]
    #[inline]
    pub fn radius(&self) -> f32 {
        self.radius
    }

    /// Returns the squared radius of the circle.
    #[must_use]
    #[inline]
    pub fn radius_squared(&self) -> f32 {
        self.radius_squared
    }

    /// Checks whether the `input_value` should be excluded.
    #[must_use]
    #[inline]
    pub fn contains(&self, input_value: Vec2) -> bool {
        input_value.length_squared() <= self.radius_squared
    }

    /// Creates a new [`CircleDeadzone`] that normalizes input values
    /// within the livezone regions defined by the `self`.
    #[inline]
    pub fn normalized(&self) -> CircleDeadzone {
        CircleDeadzone::new(*self)
    }
}

impl Eq for CircleExclusion {}

impl Hash for CircleExclusion {
    fn hash<H: Hasher>(&self, state: &mut H) {
        FloatOrd(self.radius).hash(state);
    }
}

define_dual_axis_processor!(
    name: SquareExclusion,
    perform: "axial exclusion ranges",
    stored_processor_type: AxisExclusion
);

impl Default for SquareExclusion {
    /// Creates a default [`SquareExclusion`], excluding input values within `[-1.0, 1.0]` on each axis.
    #[inline]
    fn default() -> Self {
        AxisExclusion::default().extend_dual()
    }
}

impl SquareExclusion {
    /// Checks whether the `input_value` should be excluded.
    #[must_use]
    #[inline]
    pub fn contains(&self, input_value: Vec2) -> BVec2 {
        let Vec2 { x, y } = input_value;
        match self {
            Self::OnlyX(exclusion) => BVec2::new(exclusion.contains(x), false),
            Self::OnlyY(exclusion) => BVec2::new(false, exclusion.contains(y)),
            Self::AllAxes(exclusion) => BVec2::new(exclusion.contains(x), exclusion.contains(y)),
            Self::XY(exclusion_x, exclusion_y) => {
                BVec2::new(exclusion_x.contains(x), exclusion_y.contains(y))
            }
        }
    }
}

// endregion exclusion

// region deadzone

/// Defines a deadzone that normalizes input values by clamping their magnitude to a maximum of `1.0`,
/// excluding values via a specified [`CircleExclusion`], and scaling unchanged values linearly in between.
///
/// It is worth considering that this normalizer reduces input values on diagonals.
/// If that is not your goal, you might want to explore alternative normalizers.
///
/// # Warning
///
/// - Using an `exclusion` exceeding all bounds will exclude all input values.
///
/// # Examples
///
/// ```rust
/// use bevy::prelude::*;
/// use leafwing_input_manager::prelude::*;
///
/// // Create a deadzone that excludes values with a magnitude less than 0.3
/// let exclusion = CircleExclusion::new(0.3);
/// let deadzone = CircleDeadzone::new(exclusion);
///
/// // Another way to create a CircleDeadzone.
/// let alternative = exclusion.normalized();
/// assert_eq!(alternative, deadzone);
///
/// // The bounds after normalization.
/// let bounds = CircleBounds::default();
///
/// // These values have a magnitude within the radius of the exclusion.
/// let values = [Vec2::new(0.0, 0.2), Vec2::new(0.1, 0.15)];
/// for value in values {
///     assert!(value.length() <= exclusion.radius());
///
///     // So the value should be excluded.
///     assert!(exclusion.contains(value));
///
///     // So the value should be treated as zeros.
///     let result = deadzone.process(value);
///     assert_eq!(result, Vec2::ZERO);
/// }
///
/// // The values have a magnitude at or exceed the maximum bound.
/// let values = [Vec2::new(2.0, 10.0), Vec2::splat(5.0)];
/// for value in values {
///     assert!(value.length() >= bounds.radius());
///
///     // So the value is out of the bounds.
///     assert!(!bounds.contains(value));
///
///     // So the value should be clamped to the maximum bound.
///     let result = deadzone.process(value);
///     assert_eq!(result.length(), bounds.radius());
///     assert_eq!(result.y.atan2(result.x), value.y.atan2(value.x));
/// }
///
/// // These values are within the livezones.
/// let values = [Vec2::new(0.4, -0.5), Vec2::new(-0.3, 0.5)];
/// for value in values {
///     assert!(value.length() > exclusion.radius());
///     assert!(value.length() < bounds.radius());
///
///     // So the value should be normalized to fit the range.
///     let result = deadzone.process(value);
///     assert!(result.length() > 0.0);
///     assert!(result.length() < bounds.radius());
///     assert_eq!(result.y.atan2(result.x), value.y.atan2(value.x));
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct CircleDeadzone {
    /// The exclusion used for normalization.
    pub(crate) exclusion: CircleExclusion,

    /// Pre-calculated reciprocal of the livezone radius,
    /// preventing division during normalization.
    pub(crate) livezone_recip: f32,
}

#[typetag::serde]
impl DualAxisProcessor for CircleDeadzone {
    /// Normalizes all input values within the livezone regions and returns the result.
    #[must_use]
    fn process(&self, input_value: Vec2) -> Vec2 {
        let input_length_squared = input_value.length_squared();
        if input_length_squared == 0.0 {
            return Vec2::ZERO;
        }
        let input_length = input_length_squared.sqrt();
        let (deadzone, bound) = self.livezone_min_max();
        let clamped_input_length = input_length.min(bound);
        let distance = (clamped_input_length - deadzone).max(0.0);
        let magnitude_scale = (distance * self.livezone_recip) / input_length;
        input_value * magnitude_scale
    }
}

impl Default for CircleDeadzone {
    /// Creates a new [`CircleDeadzone`] that normalizes input values
    /// by clamping their magnitude to a maximum of `1.0`
    /// and excluding values with a magnitude less than `0.1`.
    #[inline]
    fn default() -> Self {
        Self::new(CircleExclusion::default())
    }
}

impl CircleDeadzone {
    /// Creates a new [`CircleDeadzone`] that normalizes input values
    /// within the livezone regions defined by the given `deadzone` and `bounds`.
    ///
    /// # Requirements
    ///
    /// - `deadzone.radius` >= `0.0`.
    ///
    /// # Panics
    ///
    /// Panics if any of the requirements isn't met.
    ///
    /// # Warning
    ///
    /// - Using an `exclusion` exceeding all bounds will exclude all input values.
    #[inline]
    pub fn new(exclusion: CircleExclusion) -> Self {
        let bounds = CircleBounds::default();
        Self {
            exclusion,
            livezone_recip: (bounds.radius - exclusion.radius).recip(),
        }
    }

    /// Returns the [`CircleExclusion`] used by this normalizer.
    #[inline]
    pub fn exclusion(&self) -> CircleExclusion {
        self.exclusion
    }

    /// Returns the [`CircleBounds`] used by this normalizer.
    #[inline]
    pub fn bounds(&self) -> CircleBounds {
        CircleBounds::default()
    }

    /// Returns the minimum and maximum bounds of the livezone range used by this normalizer.
    ///
    /// In simple terms, this returns `(exclusion.radius, bounds.radius)`.
    #[must_use]
    #[inline]
    pub fn livezone_min_max(&self) -> (f32, f32) {
        (self.exclusion.radius, self.bounds().radius)
    }
}

impl Eq for CircleDeadzone {}

impl Hash for CircleDeadzone {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.exclusion.hash(state);
    }
}

define_dual_axis_processor!(
    name: SquareDeadzone,
    perform: "livezone value normalization",
    stored_processor_type: AxisDeadzone,
    info: "Each axis is processed individually, resulting in a per-axis \"snapping\" or locked effect, \
        which enhances control precision for pure axial motion. \
        It is commonly known as the `CrossDeadzone` due to its shape, \
        formed by two intersecting [`AxisDeadzone`]s. \
        It is worth considering that this normalizer increases the magnitude of diagonal values. \
        If that is not your goal, you might want to explore alternative normalizers."
);

impl Default for SquareDeadzone {
    /// Creates a default [`SquareDeadzone`] that normalizes input values
    /// by clamping them to `[-1.0, 1.0]` and excluding those within `[-0.1, 0.1]` on each axis.
    fn default() -> Self {
        AxisDeadzone::default().extend_dual()
    }
}

// endregion deadzone

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::prelude::*;

    #[test]
    fn test_dual_axis_processing_pipeline() {
        // Add processors to a new pipeline.
        let mut pipeline = DualAxisProcessingPipeline::default()
            .with(AxisSensitivity(4.0).extend_dual())
            .with(AxisInverted.extend_dual())
            .with(AxisInverted.extend_dual())
            .with(AxisSensitivity(4.0).extend_dual());

        // Replace the 3rd processor.
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
    fn test_dual_axis_inverted() {
        // These should be the same.
        assert_eq!(DualAxisInverted::AllAxes, AxisInverted.extend_dual());
        assert_eq!(DualAxisInverted::OnlyX, AxisInverted.extend_dual_only_x());
        assert_eq!(DualAxisInverted::OnlyY, AxisInverted.extend_dual_only_y());

        // Check if applied.
        assert!(DualAxisInverted::AllAxes.applied_x());
        assert!(DualAxisInverted::AllAxes.applied_y());
        assert!(DualAxisInverted::OnlyX.applied_x());
        assert!(!DualAxisInverted::OnlyX.applied_y());
        assert!(!DualAxisInverted::OnlyY.applied_x());
        assert!(DualAxisInverted::OnlyY.applied_y());

        // And they can invert the direction.
        assert_eq!(DualAxisInverted::AllAxes.process(Vec2::ONE), Vec2::NEG_ONE);
        assert_eq!(
            DualAxisInverted::OnlyX.process(Vec2::ONE),
            Vec2::new(-1.0, 1.0)
        );
        assert_eq!(
            DualAxisInverted::OnlyY.process(Vec2::ONE),
            Vec2::new(1.0, -1.0)
        );
    }

    #[test]
    fn test_dual_axis_sensitivity() {
        let axis_x = AxisSensitivity(4.0);
        let axis_y = AxisSensitivity(5.0);

        // These should be the same.
        let all_axes = DualAxisSensitivity::AllAxes(axis_x);
        assert_eq!(all_axes, DualAxisSensitivity::new(4.0));
        assert_eq!(all_axes, axis_x.extend_dual());

        let xy = DualAxisSensitivity::XY(axis_x, axis_y);
        assert_eq!(xy, DualAxisSensitivity::new_on_xy(4.0, 5.0));
        assert_eq!(xy, axis_x.extend_dual_with_y(axis_y));
        assert_eq!(xy, axis_y.extend_dual_with_x(axis_x));

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
        assert_eq!(all_axes.process(value).x, axis_x.process(value.x));
        assert_eq!(all_axes.process(value).y, axis_x.process(value.y));
        assert_eq!(xy.process(value).x, axis_x.process(value.x));
        assert_eq!(xy.process(value).y, axis_y.process(value.y));
        assert_eq!(only_x.process(value).x, axis_x.process(value.x));
        assert_eq!(only_y.process(value).y, axis_y.process(value.y));
    }

    #[test]
    fn test_circle_value_bounds_constructors() {
        // 0 to 1
        let bounds = CircleBounds::default();
        assert_eq!(bounds.radius(), 1.0);

        // 0 to 3
        let bounds = CircleBounds::magnitude(3.0);
        assert_eq!(bounds.radius(), 3.0);

        // 0 to unlimited
        let bounds = CircleBounds::full_range();
        assert_eq!(bounds.radius(), f32::MAX);
    }

    #[test]
    fn test_circle_value_bounds_behavior() {
        // Set the bounds to 5 for magnitude.
        let bounds = CircleBounds::magnitude(5.0);

        // Getters.
        let radius = bounds.radius();
        assert_eq!(radius, 5.0);

        assert_eq!(bounds.radius_squared(), 25.0);

        // value.magnitude > radius_max
        let values = [Vec2::ONE * 5.0, Vec2::X * 10.0];
        for value in values {
            assert!(value.length() > radius);

            // So the value is out of the bounds.
            assert!(!bounds.contains(value));

            // So the value should be clamped to the maximum bound.
            let result = bounds.process(value);
            assert_eq!(result.length(), radius);
            assert_eq!(result.y.atan2(result.x), value.y.atan2(value.x));
        }

        // value.magnitude <= radius
        let values = [Vec2::ONE * 3.0, Vec2::X * 4.0];
        for value in values {
            assert!(value.length() <= radius);

            // So the value is within the bounds.
            assert!(bounds.contains(value));

            // So the value should be left unchanged.
            assert_eq!(bounds.process(value), value);
        }
    }

    #[test]
    fn test_square_value_bounds_default() {
        // -1 to 1 on each axis
        let bounds = SquareBounds::default();

        assert_eq!(bounds, AxisBounds::default().extend_dual());

        assert!(matches!(bounds.x(), Some(bounds_x) if bounds_x.min_max() == (-1.0, 1.0)));
        assert!(matches!(bounds.y(), Some(bounds_y) if bounds_y.min_max() == (-1.0, 1.0)));
    }

    #[test]
    fn test_square_value_bounds_behavior() {
        // Set the bounds to [-2, 2] on the X-axis and [-3, 3] on the Y-axis.
        let bounds_x = AxisBounds::magnitude(2.0);
        let bounds_y = AxisBounds::magnitude(3.0);
        let bounds = bounds_x.extend_dual_with_y(bounds_y);

        // These values are within the bounds.
        let values = [Vec2::ONE, Vec2::X];
        for value in values {
            assert!(bounds.contains(value).all());
            assert!(bounds.contains(value).x);
            assert!(bounds.contains(value).y);

            // So the value should be left unchanged.
            assert_eq!(bounds.process(value), value);
        }

        // These values are only within the X-axis bounds (outside Y).
        let values = [Vec2::new(2.0, -5.0), Vec2::Y * 5.0];
        for value in values {
            assert!(!bounds.contains(value).all());
            assert!(bounds.contains(value).any());
            assert!(bounds.contains(value).x);
            assert!(!bounds.contains(value).y);

            // So the X value should be left unchanged.
            assert_eq!(bounds.process(value).x, value.x);

            // And the Y value should be clamped to the closer bound.
            let clamped_y = bounds.process(value).y;
            assert!(clamped_y == bounds_y.min() || clamped_y == bounds_y.max());
        }

        // These values are only within the Y-axis bounds (outside X).
        let values = [Vec2::new(20.0, -2.0), Vec2::X * 5.0];
        for value in values {
            assert!(!bounds.contains(value).all());
            assert!(bounds.contains(value).any());
            assert!(!bounds.contains(value).x);
            assert!(bounds.contains(value).y);

            // So the Y value should be left unchanged.
            assert_eq!(bounds.process(value).y, value.y);

            // And the X value should be clamped to the closer bound.
            let clamped_x = bounds.process(value).x;
            assert!(clamped_x == bounds_x.min() || clamped_x == bounds_x.max());
        }

        // These values are out of all bounds.
        let values = [Vec2::new(5.0, -5.0), Vec2::ONE * 8.0];
        for value in values {
            assert!(!bounds.contains(value).all());
            assert!(!bounds.contains(value).any());
            assert!(!bounds.contains(value).x);
            assert!(!bounds.contains(value).y);

            // So the value should be clamped to the closer bound.
            let result = bounds.process(value);
            assert!(result.x == bounds_x.min() || result.x == bounds_x.max());
            assert!(result.y == bounds_y.min() || result.y == bounds_y.max());
        }
    }

    #[test]
    fn test_circle_exclusion_constructors() {
        // 0 to 0.1
        let exclusion = CircleExclusion::default();
        assert_eq!(exclusion.radius(), 0.1);
        assert_eq!(exclusion.radius_squared(), 0.010000001);

        // 0 to 0.5
        let exclusion = CircleExclusion::new(0.5);
        assert_eq!(exclusion.radius(), 0.5);
        assert_eq!(exclusion.radius_squared(), 0.25);
    }

    #[test]
    fn test_circle_exclusion_behavior() {
        // Set an exclusion with a radius of 9 for magnitudes.
        let exclusion = CircleExclusion::new(9.0);
        assert_eq!(exclusion.radius(), 9.0);
        assert_eq!(exclusion.radius_squared(), 81.0);

        // value.magnitude <= radius
        let values = [Vec2::ONE, Vec2::X, Vec2::new(0.5, 3.0)];
        for value in values {
            assert!(value.length() <= exclusion.radius());

            // So the value should be excluded.
            assert!(exclusion.contains(value));

            // So the value should be treated as zeros.
            assert_eq!(exclusion.process(value), Vec2::ZERO);
        }

        // value.magnitude >= radius
        let values = [Vec2::new(15.0, 10.0), Vec2::new(20.0, 1.5)];
        for value in values {
            assert!(value.length() >= exclusion.radius());

            // So the value is out of the range.
            assert!(!exclusion.contains(value));

            // So the value should be left unchanged.
            assert_eq!(exclusion.process(value), value);
        }
    }

    #[test]
    fn test_square_exclusion_default() {
        // -0.1 to 0.1 on each axis.
        let exclusion = SquareExclusion::default();

        assert_eq!(exclusion, AxisExclusion::default().extend_dual());

        assert!(matches!(exclusion.x(), Some(exclusion_x) if exclusion_x.min_max() == (-0.1, 0.1)));
        assert!(matches!(exclusion.y(), Some(exclusion_y) if exclusion_y.min_max() == (-0.1, 0.1)));
    }

    #[test]
    fn test_square_exclusion_behavior() {
        let exclusion = AxisExclusion::new(-0.3, 0.4).extend_dual();

        // These values within all exclusion ranges.
        let values = [Vec2::splat(0.3), Vec2::new(-0.2, 0.3)];
        for value in values {
            assert!(exclusion.contains(value).all());
            assert!(exclusion.contains(value).x);
            assert!(exclusion.contains(value).y);

            // So the value should be treated as zeros.
            assert_eq!(exclusion.process(value), Vec2::ZERO);
        }

        // These values within the X-axis exclusion (outside Y).
        let values = [Vec2::new(0.3, 5.0), Vec2::new(0.1, 18.0)];
        for value in values {
            assert!(!exclusion.contains(value).all());
            assert!(exclusion.contains(value).any());
            assert!(exclusion.contains(value).x);
            assert!(!exclusion.contains(value).y);

            // So the X value should be treated as zero.
            let result = exclusion.process(value);
            assert_eq!(result.x, 0.0);

            // And the Y value should be left unchanged.
            assert_eq!(result.y, value.y);
        }

        // These values within the Y-axis exclusion (outside X).
        let values = [Vec2::new(4.3, 0.1), Vec2::new(-30.0, 0.2)];
        for value in values {
            assert!(!exclusion.contains(value).all());
            assert!(exclusion.contains(value).any());
            assert!(!exclusion.contains(value).x);
            assert!(exclusion.contains(value).y);

            // So the Y value should be treated as zero.
            let result = exclusion.process(value);
            assert_eq!(result.y, 0.0);

            // And the X value should be left unchanged.
            assert_eq!(result.x, value.x);
        }

        // These values are out of all exclusion ranges.
        let values = [Vec2::splat(10.0), Vec2::new(80.0, 73.0)];
        for value in values {
            assert!(!exclusion.contains(value).all());
            assert!(!exclusion.contains(value).any());
            assert!(!exclusion.contains(value).x);
            assert!(!exclusion.contains(value).y);

            // So the value should be left unchanged.
            assert_eq!(exclusion.process(value), value);
        }
    }

    #[test]
    fn test_circle_deadzone() {
        let exclusion = CircleExclusion::new(0.3);
        let deadzone = CircleDeadzone::new(exclusion);
        assert_eq!(exclusion.normalized(), deadzone);

        // The bounds after normalization.
        let bounds = CircleBounds::default();

        // Inner factor.
        let expected_livezone_recip = (bounds.radius() - exclusion.radius()).recip();
        assert_eq!(deadzone.livezone_recip, expected_livezone_recip);

        // value.magnitude <= exclusion.radius
        let values = [Vec2::new(0.0, 0.2), Vec2::new(0.1, 0.15)];
        for value in values {
            assert!(value.length() <= exclusion.radius());

            // So the value should be excluded.
            assert!(exclusion.contains(value));
            assert!(bounds.contains(value));

            // So the value should be treated as zeros.
            let result = deadzone.process(value);
            assert_eq!(result, Vec2::ZERO);
        }

        // value.magnitude > bounds.radius
        let values = [Vec2::new(2.0, 10.0), Vec2::splat(5.0)];
        for value in values {
            assert!(value.length() > bounds.radius());

            // So the value is not within the bounds.
            assert!(!bounds.contains(value));

            // And they shouldn't be excluded.
            assert!(!exclusion.contains(value));

            // So the value should be clamped to the maximum bound.
            let result = deadzone.process(value);
            assert_eq!(result.length(), bounds.radius());
            assert_eq!(result.y.atan2(result.x), value.y.atan2(value.x));
        }

        // exclusion.radius <= value.magnitude <= bounds.radius
        let values = [Vec2::new(0.6, -0.5), Vec2::new(-0.3, -0.7)];
        for value in values {
            let magnitude = value.length();
            assert!(magnitude >= exclusion.radius());
            assert!(magnitude <= bounds.radius());

            // So the value shouldn't be excluded.
            assert!(!exclusion.contains(value));

            // And the value is within the bounds.
            assert!(bounds.contains(value));

            // So the value should be normalized to fit the range.
            let result = deadzone.process(value);
            assert!(result.length() > 0.0);
            assert!(result.length() < bounds.radius());
            assert_eq!(result.y.atan2(result.x), value.y.atan2(value.x));

            // The result is scaled by the ratio of the value in the livezone range.
            let value_in_livezone = magnitude - exclusion.radius();
            let livezone_width = bounds.radius() - exclusion.radius();
            let expected = (value / magnitude) * (value_in_livezone / livezone_width);
            let delta = result - expected;
            assert!(delta.x.abs() <= f32::EPSILON);
            assert!(delta.y.abs() <= f32::EPSILON);
        }
    }

    #[test]
    fn test_square_deadzone() {
        let exclusion_x = AxisExclusion::new(-0.2, 0.3);
        let exclusion_y = AxisExclusion::new(-0.3, 0.4);

        let axis_x = AxisDeadzone::new(exclusion_x);
        let axis_y = AxisDeadzone::new(exclusion_y);

        let deadzone = SquareDeadzone::XY(axis_x, axis_y);
        assert_eq!(deadzone, axis_x.extend_dual_with_y(axis_y));

        // The bounds after normalization.
        let bounds = AxisBounds::default();

        // These values should be excluded.
        let values = [Vec2::splat(0.1), Vec2::new(0.2, 0.05)];
        for value in values {
            assert!(exclusion_x.contains(value.x));
            assert!(exclusion_y.contains(value.y));

            // So the value should be treated as zeros.
            let result = deadzone.process(value);
            assert_eq!(result, Vec2::ZERO);
        }

        // These values should be excluded on the X-axis and normalized on the Y-axis.
        let values = [Vec2::new(0.2, 20.0), Vec2::new(-0.1, -60.0)];
        for value in values {
            assert!(exclusion_x.contains(value.x));
            assert!(!exclusion_y.contains(value.y));

            // So the X value should be treated as zero.
            let result = deadzone.process(value);
            assert_eq!(result.x, 0.0);
            assert_eq!(result.x, axis_x.process(value.x));

            // The result of X value is derived from the exclusion on the X-axis.
            assert_eq!(result.x, exclusion_x.process(value.x));

            // And the Y value is normalized to fit within the bounds on the Y-axis.
            assert_eq!(result.y, axis_y.process(value.y));
            assert_ne!(result.y, 0.0);
            assert!(bounds.contains(result.y));
        }

        // These values should be excluded on the Y-axis and normalized on the X-axis.
        let values = [Vec2::new(-30.2, 0.2), Vec2::new(-50.1, -0.1)];
        for value in values {
            assert!(!exclusion_x.contains(value.x));
            assert!(exclusion_y.contains(value.y));

            // So the Y value should be treated as zero.
            let result = deadzone.process(value);
            assert_eq!(result.y, 0.0);
            assert_eq!(result.y, axis_y.process(value.y));

            // The result of Y value is derived from the exclusion on the Y-axis.
            assert_eq!(result.y, exclusion_y.process(value.y));

            // And the X value is normalized to fit within the bounds on the X-axis.
            assert_eq!(result.x, axis_x.process(value.x));
            assert_ne!(result.x, 0.0);
            assert!(bounds.contains(result.x));
        }

        // These values are out of all exclusion ranges.
        let values = [Vec2::new(29.0, 20.0), Vec2::new(-35.0, -60.0)];
        for value in values {
            assert!(!exclusion_x.contains(value.x));
            assert!(!exclusion_y.contains(value.y));

            // So the value should be normalized into the range.
            let result = deadzone.process(value);
            assert_ne!(result.x, 0.0);
            assert_ne!(result.y, 0.0);
            assert!(bounds.contains(result.x));
            assert!(bounds.contains(result.y));

            // The results are derived from the deadzone on each axis.
            assert_eq!(result.x, axis_x.process(value.x));
            assert_eq!(result.y, axis_y.process(value.y));
        }
    }
}
