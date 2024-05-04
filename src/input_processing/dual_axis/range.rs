//! Range processors for dual-axis inputs

use std::fmt::Debug;
use std::hash::Hash;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::DualAxisProcessor;
use crate::input_processing::single_axis::*;

/// Specifies a square-shaped region defining acceptable ranges for valid dual-axis inputs,
/// with independent min-max ranges for each axis, restricting all values stay within intended limits
/// to avoid unexpected behavior caused by extreme inputs.
///
/// In simple terms, this processor is just the dual-axis version of [`AxisBounds`].
/// Helpers like [`AxisBounds::extend_dual()`] and its peers can be used to create a [`DualAxisBounds`].
///
/// ```rust
/// use bevy::prelude::*;
/// use leafwing_input_manager::prelude::*;
///
/// // Restrict X to [-2.0, 2.5] and Y to [-1.0, 1.5].
/// let bounds = DualAxisBounds::new((-2.0, 2.5), (-1.0, 1.5));
/// assert_eq!(bounds.bounds_x().min_max(), (-2.0, 2.5));
/// assert_eq!(bounds.bounds_y().min_max(), (-1.0, 1.5));
///
/// // Another way to create a DualAxisBounds.
/// let bounds_x = AxisBounds::new(-2.0, 2.5);
/// let bounds_y = AxisBounds::new(-1.0, 1.5);
/// assert_eq!(bounds_x.extend_dual_with_y(bounds_y), bounds);
///
/// for x in -300..300 {
///     let x = x as f32 * 0.01;
///     for y in -300..300 {
///         let y = y as f32 * 0.01;
///         let value = Vec2::new(x, y);
///
///         assert_eq!(bounds.clamp(value).x, bounds_x.clamp(x));
///         assert_eq!(bounds.clamp(value).y, bounds_y.clamp(y));
///     }
/// }
/// ```
#[doc(alias("SquareBounds", "AxialBounds"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct DualAxisBounds {
    /// The [`AxisBounds`] for the X-axis inputs.
    pub(crate) bounds_x: AxisBounds,

    /// The [`AxisBounds`] for the Y-axis inputs.
    pub(crate) bounds_y: AxisBounds,
}

impl DualAxisBounds {
    /// Unlimited [`DualAxisBounds`].
    pub const FULL_RANGE: Self = AxisBounds::FULL_RANGE.extend_dual();

    /// Creates a [`DualAxisBounds`] that restricts values within the range `[min, max]` on each axis.
    ///
    /// # Requirements
    ///
    /// - `min` <= `max` on each axis.
    ///
    /// # Panics
    ///
    /// Panics if the requirements aren't met.
    #[inline]
    pub fn new((x_min, x_max): (f32, f32), (y_min, y_max): (f32, f32)) -> Self {
        Self {
            bounds_x: AxisBounds::new(x_min, x_max),
            bounds_y: AxisBounds::new(y_min, y_max),
        }
    }

    /// Creates a [`DualAxisBounds`] that restricts values within the same range `[min, max]` on both axes.
    ///
    /// # Requirements
    ///
    /// - `min` <= `max`.
    ///
    /// # Panics
    ///
    /// Panics if the requirements aren't met.
    #[inline]
    pub fn all(min: f32, max: f32) -> Self {
        let range = (min, max);
        Self::new(range, range)
    }

    /// Creates a [`DualAxisBounds`] that only restricts X values within the range `[min, max]`.
    ///
    /// # Requirements
    ///
    /// - `min` <= `max`.
    ///
    /// # Panics
    ///
    /// Panics if the requirements aren't met.
    #[inline]
    pub fn only_x(min: f32, max: f32) -> Self {
        Self {
            bounds_x: AxisBounds::new(min, max),
            ..Self::FULL_RANGE
        }
    }

    /// Creates a [`DualAxisBounds`] that only restricts Y values within the range `[min, max]`.
    ///
    /// # Requirements
    ///
    /// - `min` <= `max`.
    ///
    /// # Panics
    ///
    /// Panics if the requirements aren't met.
    #[inline]
    pub fn only_y(min: f32, max: f32) -> Self {
        Self {
            bounds_y: AxisBounds::new(min, max),
            ..Self::FULL_RANGE
        }
    }

    /// Creates a [`DualAxisBounds`] that restricts values within the range `[-threshold, threshold]` on each axis.
    ///
    /// # Requirements
    ///
    /// - `threshold` >= `0.0` on each axis.
    ///
    /// # Panics
    ///
    /// Panics if the requirements aren't met.
    #[doc(alias = "magnitude")]
    #[inline]
    pub fn symmetric(threshold_x: f32, threshold_y: f32) -> Self {
        Self {
            bounds_x: AxisBounds::symmetric(threshold_x),
            bounds_y: AxisBounds::symmetric(threshold_y),
        }
    }

    /// Creates a [`DualAxisBounds`] that restricts values within the range `[-threshold, threshold]` on both axes.
    ///
    /// # Requirements
    ///
    /// - `threshold` >= `0.0`.
    ///
    /// # Panics
    ///
    /// Panics if the requirements aren't met.
    #[doc(alias = "magnitude_all")]
    #[inline]
    pub fn symmetric_all(threshold: f32) -> Self {
        Self::symmetric(threshold, threshold)
    }

    /// Creates a [`DualAxisBounds`] that only restricts X values within the range `[-threshold, threshold]`.
    ///
    /// # Requirements
    ///
    /// - `threshold` >= `0.0`.
    ///
    /// # Panics
    ///
    /// Panics if the requirements aren't met.
    #[doc(alias = "magnitude_only_x")]
    #[inline]
    pub fn symmetric_only_x(threshold: f32) -> Self {
        Self {
            bounds_x: AxisBounds::symmetric(threshold),
            ..Self::FULL_RANGE
        }
    }

    /// Creates a [`DualAxisBounds`] that only restricts Y values within the range `[-threshold, threshold]`.
    ///
    /// # Requirements
    ///
    /// - `threshold` >= `0.0`.
    ///
    /// # Panics
    ///
    /// Panics if the requirements aren't met.
    #[doc(alias = "magnitude_only_y")]
    #[inline]
    pub fn symmetric_only_y(threshold: f32) -> Self {
        Self {
            bounds_y: AxisBounds::symmetric(threshold),
            ..Self::FULL_RANGE
        }
    }

    /// Creates a [`DualAxisBounds`] that restricts values to a minimum value on each axis.
    #[inline]
    pub const fn at_least(x_min: f32, y_min: f32) -> Self {
        Self {
            bounds_x: AxisBounds::at_least(x_min),
            bounds_y: AxisBounds::at_least(y_min),
        }
    }

    /// Creates a [`DualAxisBounds`] that restricts values to a minimum value on both axes.
    #[inline]
    pub const fn at_least_all(min: f32) -> Self {
        AxisBounds::at_least(min).extend_dual()
    }

    /// Creates a [`DualAxisBounds`] that only restricts X values to a minimum value.
    #[inline]
    pub const fn at_least_only_x(min: f32) -> Self {
        Self {
            bounds_x: AxisBounds::at_least(min),
            ..Self::FULL_RANGE
        }
    }

    /// Creates a [`DualAxisBounds`] that only restricts Y values to a minimum value.
    #[inline]
    pub const fn at_least_only_y(min: f32) -> Self {
        Self {
            bounds_y: AxisBounds::at_least(min),
            ..Self::FULL_RANGE
        }
    }

    /// Creates a [`DualAxisBounds`] that restricts values to a maximum value on each axis.
    #[inline]
    pub const fn at_most(x_max: f32, y_max: f32) -> Self {
        Self {
            bounds_x: AxisBounds::at_most(x_max),
            bounds_y: AxisBounds::at_most(y_max),
        }
    }

    /// Creates a [`DualAxisBounds`] that restricts values to a maximum value on both axes.
    #[inline]
    pub const fn at_most_all(max: f32) -> Self {
        AxisBounds::at_most(max).extend_dual()
    }

    /// Creates a [`DualAxisBounds`] that only restricts X values to a maximum value.
    #[inline]
    pub const fn at_most_only_x(max: f32) -> Self {
        Self {
            bounds_x: AxisBounds::at_most(max),
            ..Self::FULL_RANGE
        }
    }

    /// Creates a [`DualAxisBounds`] that only restricts Y values to a maximum value.
    #[inline]
    pub const fn at_most_only_y(max: f32) -> Self {
        Self {
            bounds_y: AxisBounds::at_most(max),
            ..Self::FULL_RANGE
        }
    }

    /// Returns the bounds for inputs along each axis.
    #[inline]
    pub fn bounds(&self) -> (AxisBounds, AxisBounds) {
        (self.bounds_x, self.bounds_y)
    }

    /// Returns the bounds for the X-axis inputs.
    #[inline]
    pub fn bounds_x(&self) -> AxisBounds {
        self.bounds().0
    }

    /// Returns the bounds for the Y-axis inputs.
    #[inline]
    pub fn bounds_y(&self) -> AxisBounds {
        self.bounds().1
    }

    /// Is `input_value` is within the bounds?
    #[must_use]
    #[inline]
    pub fn contains(&self, input_value: Vec2) -> BVec2 {
        BVec2::new(
            self.bounds_x.contains(input_value.x),
            self.bounds_y.contains(input_value.y),
        )
    }

    /// Clamps `input_value` within the bounds.
    #[must_use]
    #[inline]
    pub fn clamp(&self, input_value: Vec2) -> Vec2 {
        Vec2::new(
            self.bounds_x.clamp(input_value.x),
            self.bounds_y.clamp(input_value.y),
        )
    }
}

impl Default for DualAxisBounds {
    /// Creates a [`DualAxisBounds`] that restricts values within the range `[-1.0, 1.0]` on both axes.
    #[inline]
    fn default() -> Self {
        AxisBounds::default().extend_dual()
    }
}

impl From<DualAxisBounds> for DualAxisProcessor {
    fn from(value: DualAxisBounds) -> Self {
        Self::ValueBounds(value)
    }
}

impl AxisBounds {
    /// Creates a [`DualAxisBounds`] using `self` for both axes.
    #[inline]
    pub const fn extend_dual(self) -> DualAxisBounds {
        DualAxisBounds {
            bounds_x: self,
            bounds_y: self,
        }
    }

    /// Creates a [`DualAxisBounds`] only using `self` for the X-axis.
    #[inline]
    pub const fn extend_dual_only_x(self) -> DualAxisBounds {
        DualAxisBounds {
            bounds_x: self,
            ..DualAxisBounds::FULL_RANGE
        }
    }

    /// Creates a [`DualAxisBounds`] only using `self` to the Y-axis.
    #[inline]
    pub const fn extend_dual_only_y(self) -> DualAxisBounds {
        DualAxisBounds {
            bounds_y: self,
            ..DualAxisBounds::FULL_RANGE
        }
    }

    /// Creates a [`DualAxisBounds`] using `self` to the Y-axis with the given `bounds_x` to the X-axis.
    #[inline]
    pub const fn extend_dual_with_x(self, bounds_x: Self) -> DualAxisBounds {
        DualAxisBounds {
            bounds_x,
            bounds_y: self,
        }
    }

    /// Creates a [`DualAxisBounds`] using `self` to the X-axis with the given `bounds_y` to the Y-axis.
    #[inline]
    pub const fn extend_dual_with_y(self, bounds_y: Self) -> DualAxisBounds {
        DualAxisBounds {
            bounds_x: self,
            bounds_y,
        }
    }
}

impl From<AxisBounds> for DualAxisProcessor {
    fn from(bounds: AxisBounds) -> Self {
        Self::ValueBounds(bounds.extend_dual())
    }
}

/// Specifies a cross-shaped region for excluding dual-axis inputs,
/// with min-max independent min-max ranges for each axis, resulting in a per-axis "snapping" effect,
/// helping filter out minor fluctuations to enhance control precision for pure axial motion.
///
/// In simple terms, this processor is just the dual-axis version of [`AxisExclusion`].
/// Helpers like [`AxisExclusion::extend_dual()`] and its peers can be used to create a [`DualAxisExclusion`].
///
/// ```rust
/// use bevy::prelude::*;
/// use leafwing_input_manager::prelude::*;
///
/// // Exclude X within [-0.2, 0.3] and Y within [-0.1, 0.4].
/// let exclusion = DualAxisExclusion::new((-0.2, 0.3), (-0.1, 0.4));
/// assert_eq!(exclusion.exclusion_x().min_max(), (-0.2, 0.3));
/// assert_eq!(exclusion.exclusion_y().min_max(), (-0.1, 0.4));
///
/// // Another way to create a DualAxisExclusion.
/// let exclusion_x = AxisExclusion::new(-0.2, 0.3);
/// let exclusion_y = AxisExclusion::new(-0.1, 0.4);
/// assert_eq!(exclusion_x.extend_dual_with_y(exclusion_y), exclusion);
///
/// for x in -300..300 {
///     let x = x as f32 * 0.01;
///     for y in -300..300 {
///         let y = y as f32 * 0.01;
///         let value = Vec2::new(x, y);
///
///         assert_eq!(exclusion.exclude(value).x, exclusion_x.exclude(x));
///         assert_eq!(exclusion.exclude(value).y, exclusion_y.exclude(y));
///     }
/// }
/// ```
#[doc(alias("CrossExclusion", "AxialExclusion"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct DualAxisExclusion {
    /// The [`AxisExclusion`] for the X-axis inputs.
    pub(crate) exclusion_x: AxisExclusion,

    /// The [`AxisExclusion`] for the Y-axis inputs.
    pub(crate) exclusion_y: AxisExclusion,
}

impl DualAxisExclusion {
    /// Zero-size [`DualAxisExclusion`], leaving values as is.
    pub const ZERO: Self = AxisExclusion::ZERO.extend_dual();

    /// Creates a [`DualAxisExclusion`] that ignores values within the range `[negative_max, positive_min]` on each axis.
    ///
    /// # Requirements
    ///
    /// - `negative_max` <= `0.0` <= `positive_min` on each axis.
    ///
    /// # Panics
    ///
    /// Panics if the requirements aren't met.
    #[inline]
    pub fn new(
        (x_negative_max, x_positive_min): (f32, f32),
        (y_negative_max, y_positive_min): (f32, f32),
    ) -> Self {
        Self {
            exclusion_x: AxisExclusion::new(x_negative_max, x_positive_min),
            exclusion_y: AxisExclusion::new(y_negative_max, y_positive_min),
        }
    }

    /// Creates a [`DualAxisExclusion`] that ignores values within the range `[negative_max, positive_min]` on both axes.
    ///
    /// # Requirements
    ///
    /// - `negative_max` <= `0.0` <= `positive_min`.
    ///
    /// # Panics
    ///
    /// Panics if the requirements aren't met.
    #[inline]
    pub fn all(negative_max: f32, positive_min: f32) -> Self {
        let range = (negative_max, positive_min);
        Self::new(range, range)
    }

    /// Creates a [`DualAxisExclusion`] that only ignores X values within the range `[negative_max, positive_min]`.
    ///
    /// # Requirements
    ///
    /// - `negative_max` <= `0.0` <= `positive_min`.
    ///
    /// # Panics
    ///
    /// Panics if the requirements aren't met.
    #[inline]
    pub fn only_x(negative_max: f32, positive_min: f32) -> Self {
        Self {
            exclusion_x: AxisExclusion::new(negative_max, positive_min),
            ..Self::ZERO
        }
    }

    /// Creates a [`DualAxisExclusion`] that only ignores Y values within the range `[negative_max, positive_min]`.
    ///
    /// # Requirements
    ///
    /// - `negative_max` <= `0.0` <= `positive_min`.
    ///
    /// # Panics
    ///
    /// Panics if the requirements aren't met.
    #[inline]
    pub fn only_y(negative_max: f32, positive_min: f32) -> Self {
        Self {
            exclusion_y: AxisExclusion::new(negative_max, positive_min),
            ..Self::ZERO
        }
    }

    /// Creates a [`DualAxisExclusion`] that ignores values within the range `[-threshold, threshold]` on each axis.
    ///
    /// # Requirements
    ///
    /// - `threshold` >= `0.0` on each axis.
    ///
    /// # Panics
    ///
    /// Panics if the requirements aren't met.
    #[doc(alias = "magnitude")]
    #[inline]
    pub fn symmetric(threshold_x: f32, threshold_y: f32) -> Self {
        Self {
            exclusion_x: AxisExclusion::symmetric(threshold_x),
            exclusion_y: AxisExclusion::symmetric(threshold_y),
        }
    }

    /// Creates a [`DualAxisExclusion`] that ignores values within the range `[-threshold, threshold]` on both axes.
    ///
    /// # Requirements
    ///
    /// - `threshold` >= `0.0`.
    ///
    /// # Panics
    ///
    /// Panics if the requirements aren't met.
    #[doc(alias = "magnitude_all")]
    #[inline]
    pub fn symmetric_all(threshold: f32) -> Self {
        Self::symmetric(threshold, threshold)
    }

    /// Creates a [`DualAxisExclusion`] that only ignores X values within the range `[-threshold, threshold]`.
    ///
    /// # Requirements
    ///
    /// - `threshold` >= `0.0`.
    ///
    /// # Panics
    ///
    /// Panics if the requirements aren't met.
    #[doc(alias = "magnitude_only_x")]
    #[inline]
    pub fn symmetric_only_x(threshold: f32) -> Self {
        Self {
            exclusion_x: AxisExclusion::symmetric(threshold),
            ..Self::ZERO
        }
    }

    /// Creates a [`DualAxisExclusion`] that only ignores Y values within the range `[-threshold, threshold]`.
    ///
    /// # Requirements
    ///
    /// - `threshold` >= `0.0`.
    ///
    /// # Panics
    ///
    /// Panics if the requirements aren't met.
    #[doc(alias = "magnitude_only_y")]
    #[inline]
    pub fn symmetric_only_y(threshold: f32) -> Self {
        Self {
            exclusion_y: AxisExclusion::symmetric(threshold),
            ..Self::ZERO
        }
    }

    /// Returns the exclusion ranges for inputs along each axis.
    #[inline]
    pub fn exclusions(&self) -> (AxisExclusion, AxisExclusion) {
        (self.exclusion_x, self.exclusion_y)
    }

    /// Returns the exclusion range for the X-axis inputs.
    #[inline]
    pub fn exclusion_x(&self) -> AxisExclusion {
        self.exclusions().0
    }

    /// Returns the exclusion range for the Y-axis inputs.
    #[inline]
    pub fn exclusion_y(&self) -> AxisExclusion {
        self.exclusions().1
    }

    /// Is the `input_value` within the exclusion range?
    #[must_use]
    #[inline]
    pub fn contains(&self, input_value: Vec2) -> BVec2 {
        BVec2::new(
            self.exclusion_x.contains(input_value.x),
            self.exclusion_y.contains(input_value.y),
        )
    }

    /// Excludes values within the specified region.
    #[must_use]
    #[inline]
    pub fn exclude(&self, input_value: Vec2) -> Vec2 {
        Vec2::new(
            self.exclusion_x.exclude(input_value.x),
            self.exclusion_y.exclude(input_value.y),
        )
    }

    /// Creates a [`DualAxisDeadZone`] using `self` as the exclusion range.
    pub fn scaled(self) -> DualAxisDeadZone {
        DualAxisDeadZone::new(self.exclusion_x.min_max(), self.exclusion_y.min_max())
    }
}

impl Default for DualAxisExclusion {
    /// Creates a [`DualAxisExclusion`] that excludes input values within `[-1.0, 1.0]` on both axes.
    #[inline]
    fn default() -> Self {
        AxisExclusion::default().extend_dual()
    }
}

impl From<DualAxisExclusion> for DualAxisProcessor {
    fn from(value: DualAxisExclusion) -> Self {
        Self::Exclusion(value)
    }
}

impl AxisExclusion {
    /// Creates a [`DualAxisExclusion`] using `self` for both axes.
    #[inline]
    pub const fn extend_dual(self) -> DualAxisExclusion {
        DualAxisExclusion {
            exclusion_x: self,
            exclusion_y: self,
        }
    }

    /// Creates a [`DualAxisExclusion`] only using `self` for the X-axis.
    #[inline]
    pub const fn extend_dual_only_x(self) -> DualAxisExclusion {
        DualAxisExclusion {
            exclusion_x: self,
            ..DualAxisExclusion::ZERO
        }
    }

    /// Creates a [`DualAxisExclusion`] only using `self` to the Y-axis.
    #[inline]
    pub const fn extend_dual_only_y(self) -> DualAxisExclusion {
        DualAxisExclusion {
            exclusion_y: self,
            ..DualAxisExclusion::ZERO
        }
    }

    /// Creates a [`DualAxisExclusion`] using `self` to the Y-axis with the given `bounds_x` to the X-axis.
    #[inline]
    pub const fn extend_dual_with_x(self, exclusion_x: Self) -> DualAxisExclusion {
        DualAxisExclusion {
            exclusion_x,
            exclusion_y: self,
        }
    }

    /// Creates a [`DualAxisExclusion`] using `self` to the X-axis with the given `bounds_y` to the Y-axis.
    #[inline]
    pub const fn extend_dual_with_y(self, exclusion_y: Self) -> DualAxisExclusion {
        DualAxisExclusion {
            exclusion_x: self,
            exclusion_y,
        }
    }
}

impl From<AxisExclusion> for DualAxisProcessor {
    fn from(exclusion: AxisExclusion) -> Self {
        Self::Exclusion(exclusion.extend_dual())
    }
}

/// A scaled version of [`DualAxisExclusion`] with the bounds
/// set to [`DualAxisBounds::symmetric_all(1.0)`](DualAxisBounds::default)
/// that normalizes non-excluded input values into the "live zone",
/// the remaining range within the bounds after dead zone exclusion.
///
/// Each axis is processed individually, resulting in a per-axis "snapping" effect,
/// which enhances control precision for pure axial motion.
///
/// It is worth considering that this normalizer increases the magnitude of diagonal values.
/// If that is not your goal, you might want to explore alternative normalizers.
///
/// In simple terms, this processor is just the dual-axis version of [`AxisDeadZone`].
/// Helpers like [`AxisDeadZone::extend_dual()`] and its peers can be used to create a [`DualAxisDeadZone`].
///
/// ```rust
/// use bevy::prelude::*;
/// use leafwing_input_manager::prelude::*;
///
/// // Exclude X within [-0.2, 0.3] and Y within [-0.1, 0.4].
/// let deadzone = DualAxisDeadZone::new((-0.2, 0.3), (-0.1, 0.4));
/// assert_eq!(deadzone.deadzone_x().exclusion().min_max(), (-0.2, 0.3));
/// assert_eq!(deadzone.deadzone_y().exclusion().min_max(), (-0.1, 0.4));
///
/// // Another way to create a DualAxisDeadZone.
/// let deadzone_x = AxisDeadZone::new(-0.2, 0.3);
/// let deadzone_y = AxisDeadZone::new(-0.1, 0.4);
/// assert_eq!(deadzone_x.extend_dual_with_y(deadzone_y), deadzone);
///
/// for x in -300..300 {
///     let x = x as f32 * 0.01;
///     for y in -300..300 {
///         let y = y as f32 * 0.01;
///         let value = Vec2::new(x, y);
///
///         assert_eq!(deadzone.normalize(value).x, deadzone_x.normalize(x));
///         assert_eq!(deadzone.normalize(value).y, deadzone_y.normalize(y));
///     }
/// }
/// ```
#[doc(alias("CrossDeadZone", "AxialDeadZone"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct DualAxisDeadZone {
    /// The [`AxisDeadZone`] for the X-axis inputs.
    pub(crate) deadzone_x: AxisDeadZone,

    /// The [`AxisDeadZone`] for the Y-axis inputs.
    pub(crate) deadzone_y: AxisDeadZone,
}

impl DualAxisDeadZone {
    /// Zero-size [`DualAxisDeadZone`], only restricting values to the range `[-1.0, 1.0]` on both axes.
    pub const ZERO: Self = AxisDeadZone::ZERO.extend_dual();

    /// Creates a [`DualAxisDeadZone`] that excludes values within the range `[negative_max, positive_min]` on each axis.
    ///
    /// # Requirements
    ///
    /// - `negative_max` <= `0.0` <= `positive_min` on each axis.
    ///
    /// # Panics
    ///
    /// Panics if the requirements aren't met.
    #[inline]
    pub fn new(
        (x_negative_max, x_positive_min): (f32, f32),
        (y_negative_max, y_positive_min): (f32, f32),
    ) -> Self {
        Self {
            deadzone_x: AxisDeadZone::new(x_negative_max, x_positive_min),
            deadzone_y: AxisDeadZone::new(y_negative_max, y_positive_min),
        }
    }

    /// Creates a [`DualAxisDeadZone`] that excludes values within the range `[negative_max, positive_min]` on both axes.
    ///
    /// # Requirements
    ///
    /// - `negative_max` <= `0.0` <= `positive_min`.
    ///
    /// # Panics
    ///
    /// Panics if the requirements aren't met.
    #[inline]
    pub fn all(negative_max: f32, positive_min: f32) -> Self {
        let range = (negative_max, positive_min);
        Self::new(range, range)
    }

    /// Creates a [`DualAxisDeadZone`] that only excludes X values within the range `[negative_max, positive_min]`.
    ///
    /// # Requirements
    ///
    /// - `negative_max` <= `0.0` <= `positive_min`.
    ///
    /// # Panics
    ///
    /// Panics if the requirements aren't met.
    #[inline]
    pub fn only_x(negative_max: f32, positive_min: f32) -> Self {
        Self {
            deadzone_x: AxisDeadZone::new(negative_max, positive_min),
            ..Self::ZERO
        }
    }

    /// Creates a [`DualAxisDeadZone`] that only excludes Y values within the range `[negative_max, positive_min]`.
    ///
    /// # Requirements
    ///
    /// - `negative_max` <= `0.0` <= `positive_min`.
    ///
    /// # Panics
    ///
    /// Panics if the requirements aren't met.
    #[inline]
    pub fn only_y(negative_max: f32, positive_min: f32) -> Self {
        Self {
            deadzone_y: AxisDeadZone::new(negative_max, positive_min),
            ..Self::ZERO
        }
    }

    /// Creates a [`DualAxisDeadZone`] that excludes values within the range `[-threshold, threshold]` on each axis.
    ///
    /// # Requirements
    ///
    /// - `threshold` >= `0.0` on each axis.
    ///
    /// # Panics
    ///
    /// Panics if the requirements aren't met.
    #[doc(alias = "magnitude")]
    #[inline]
    pub fn symmetric(threshold_x: f32, threshold_y: f32) -> Self {
        Self {
            deadzone_x: AxisDeadZone::symmetric(threshold_x),
            deadzone_y: AxisDeadZone::symmetric(threshold_y),
        }
    }

    /// Creates a [`DualAxisDeadZone`] that excludes values within the range `[-threshold, threshold]` on both axes.
    ///
    /// # Requirements
    ///
    /// - `threshold` >= `0.0`.
    ///
    /// # Panics
    ///
    /// Panics if the requirements aren't met.
    #[doc(alias = "magnitude_all")]
    #[inline]
    pub fn symmetric_all(threshold: f32) -> Self {
        Self::symmetric(threshold, threshold)
    }

    /// Creates a [`DualAxisDeadZone`] that only excludes X values within the range `[-threshold, threshold]`.
    ///
    /// # Requirements
    ///
    /// - `threshold` >= `0.0`.
    ///
    /// # Panics
    ///
    /// Panics if the requirements aren't met.
    #[doc(alias = "magnitude_only_x")]
    #[inline]
    pub fn symmetric_only_x(threshold: f32) -> Self {
        Self {
            deadzone_x: AxisDeadZone::symmetric(threshold),
            ..Self::ZERO
        }
    }

    /// Creates a [`DualAxisDeadZone`] that only excludes Y values within the range `[-threshold, threshold]`.
    ///
    /// # Requirements
    ///
    /// - `threshold` >= `0.0`.
    ///
    /// # Panics
    ///
    /// Panics if the requirements aren't met.
    #[doc(alias = "magnitude_only_y")]
    #[inline]
    pub fn symmetric_only_y(threshold: f32) -> Self {
        Self {
            deadzone_y: AxisDeadZone::symmetric(threshold),
            ..Self::ZERO
        }
    }

    /// Returns the dead zones for inputs along each axis.
    #[inline]
    pub fn deadzones(&self) -> (AxisDeadZone, AxisDeadZone) {
        (self.deadzone_x, self.deadzone_y)
    }

    /// Returns the dead zone for the X-axis inputs.
    #[inline]
    pub fn deadzone_x(&self) -> AxisDeadZone {
        self.deadzones().0
    }

    /// Returns the dead zone for the Y-axis inputs.
    #[inline]
    pub fn deadzone_y(&self) -> AxisDeadZone {
        self.deadzones().1
    }

    /// Returns the [`DualAxisExclusion`] used by this deadzone.
    #[inline]
    pub fn exclusion(&self) -> DualAxisExclusion {
        DualAxisExclusion {
            exclusion_x: self.deadzone_x.exclusion(),
            exclusion_y: self.deadzone_y.exclusion(),
        }
    }

    /// Returns the [`DualAxisBounds`] used by this deadzone.
    #[inline]
    pub fn bounds(&self) -> DualAxisBounds {
        DualAxisBounds::default()
    }

    /// Is the given `input_value` within the exclusion ranges?
    #[must_use]
    #[inline]
    pub fn within_exclusion(&self, input_value: Vec2) -> BVec2 {
        BVec2::new(
            self.deadzone_x.within_exclusion(input_value.x),
            self.deadzone_y.within_exclusion(input_value.y),
        )
    }

    /// Is the given `input_value` within the bounds?
    #[must_use]
    #[inline]
    pub fn within_bounds(&self, input_value: Vec2) -> BVec2 {
        BVec2::new(
            self.deadzone_x.within_bounds(input_value.x),
            self.deadzone_y.within_bounds(input_value.y),
        )
    }

    /// Is the given `input_value` within the lower live zone?
    #[must_use]
    #[inline]
    pub fn within_livezone_lower(&self, input_value: Vec2) -> BVec2 {
        BVec2::new(
            self.deadzone_x.within_livezone_lower(input_value.x),
            self.deadzone_y.within_livezone_lower(input_value.y),
        )
    }

    /// Is the given `input_value` within the upper live zone?
    #[must_use]
    #[inline]
    pub fn within_livezone_upper(&self, input_value: Vec2) -> BVec2 {
        BVec2::new(
            self.deadzone_x.within_livezone_upper(input_value.x),
            self.deadzone_y.within_livezone_upper(input_value.y),
        )
    }

    /// Normalizes input values into the live zone.
    #[must_use]
    #[inline]
    pub fn normalize(&self, input_value: Vec2) -> Vec2 {
        Vec2::new(
            self.deadzone_x.normalize(input_value.x),
            self.deadzone_y.normalize(input_value.y),
        )
    }
}

impl Default for DualAxisDeadZone {
    /// Creates a [`DualAxisDeadZone`] that excludes input values within the deadzone `[-0.1, 0.1]` on both axes.
    fn default() -> Self {
        AxisDeadZone::default().extend_dual()
    }
}

impl From<DualAxisDeadZone> for DualAxisProcessor {
    fn from(value: DualAxisDeadZone) -> Self {
        Self::DeadZone(value)
    }
}

impl AxisDeadZone {
    /// Creates a [`DualAxisDeadZone`] using `self` for both axes.
    #[inline]
    pub const fn extend_dual(self) -> DualAxisDeadZone {
        DualAxisDeadZone {
            deadzone_x: self,
            deadzone_y: self,
        }
    }

    /// Creates a [`DualAxisDeadZone`] only using `self` for the X-axis.
    #[inline]
    pub const fn extend_dual_only_x(self) -> DualAxisDeadZone {
        DualAxisDeadZone {
            deadzone_x: self,
            ..DualAxisDeadZone::ZERO
        }
    }

    /// Creates a [`DualAxisDeadZone`] only using `self` to the Y-axis.
    #[inline]
    pub const fn extend_dual_only_y(self) -> DualAxisDeadZone {
        DualAxisDeadZone {
            deadzone_y: self,
            ..DualAxisDeadZone::ZERO
        }
    }

    /// Creates a [`DualAxisDeadZone`] using `self` to the Y-axis with the given `bounds_x` to the X-axis.
    #[inline]
    pub const fn extend_dual_with_x(self, deadzone_x: Self) -> DualAxisDeadZone {
        DualAxisDeadZone {
            deadzone_x,
            deadzone_y: self,
        }
    }

    /// Creates a [`DualAxisDeadZone`] using `self` to the X-axis with the given `bounds_y` to the Y-axis.
    #[inline]
    pub const fn extend_dual_with_y(self, deadzone_y: Self) -> DualAxisDeadZone {
        DualAxisDeadZone {
            deadzone_x: self,
            deadzone_y,
        }
    }
}

impl From<AxisDeadZone> for DualAxisProcessor {
    fn from(deadzone: AxisDeadZone) -> Self {
        Self::DeadZone(deadzone.extend_dual())
    }
}

impl From<DualAxisExclusion> for DualAxisDeadZone {
    fn from(exclusion: DualAxisExclusion) -> Self {
        Self::new(
            exclusion.exclusion_x.min_max(),
            exclusion.exclusion_y.min_max(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dual_axis_value_bounds() {
        fn test_bounds(
            bounds: DualAxisBounds,
            (x_min, x_max): (f32, f32),
            (y_min, y_max): (f32, f32),
        ) {
            assert_eq!(bounds.bounds_x().min_max(), (x_min, x_max));
            assert_eq!(bounds.bounds_y().min_max(), (y_min, y_max));

            let bounds_x = AxisBounds::new(x_min, x_max);
            let bounds_y = AxisBounds::new(y_min, y_max);
            assert_eq!(bounds_x.extend_dual_with_y(bounds_y), bounds);
            assert_eq!(bounds_y.extend_dual_with_x(bounds_x), bounds);

            let (bx, by) = bounds.bounds();
            assert_eq!(bx, bounds_x);
            assert_eq!(by, bounds_y);

            assert_eq!(
                DualAxisProcessor::from(bounds_x),
                DualAxisProcessor::ValueBounds(DualAxisBounds::all(x_min, x_max))
            );

            let processor = DualAxisProcessor::ValueBounds(bounds);
            assert_eq!(DualAxisProcessor::from(bounds), processor);

            for x in -300..300 {
                let x = x as f32 * 0.01;
                for y in -300..300 {
                    let y = y as f32 * 0.01;
                    let value = Vec2::new(x, y);

                    assert_eq!(processor.process(value), bounds.clamp(value));

                    let expected = BVec2::new(bounds_x.contains(x), bounds_y.contains(y));
                    assert_eq!(bounds.contains(value), expected);

                    let expected = Vec2::new(bounds_x.clamp(x), bounds_y.clamp(y));
                    assert_eq!(bounds.clamp(value), expected);
                }
            }
        }

        let full_range = (f32::MIN, f32::MAX);

        let bounds = DualAxisBounds::FULL_RANGE;
        test_bounds(bounds, full_range, full_range);

        let bounds = DualAxisBounds::default();
        test_bounds(bounds, (-1.0, 1.0), (-1.0, 1.0));

        let bounds = DualAxisBounds::new((-2.0, 2.5), (-1.0, 1.5));
        test_bounds(bounds, (-2.0, 2.5), (-1.0, 1.5));

        let bounds = DualAxisBounds::all(-2.0, 2.5);
        test_bounds(bounds, (-2.0, 2.5), (-2.0, 2.5));

        let bounds = DualAxisBounds::only_x(-2.0, 2.5);
        test_bounds(bounds, (-2.0, 2.5), full_range);

        let bounds = DualAxisBounds::only_y(-1.0, 1.5);
        test_bounds(bounds, full_range, (-1.0, 1.5));

        let bounds = DualAxisBounds::symmetric(2.0, 2.5);
        test_bounds(bounds, (-2.0, 2.0), (-2.5, 2.5));

        let bounds = DualAxisBounds::symmetric_all(2.5);
        test_bounds(bounds, (-2.5, 2.5), (-2.5, 2.5));

        let bounds = DualAxisBounds::symmetric_only_x(2.5);
        test_bounds(bounds, (-2.5, 2.5), full_range);

        let bounds = DualAxisBounds::symmetric_only_y(2.5);
        test_bounds(bounds, full_range, (-2.5, 2.5));

        let bounds = DualAxisBounds::at_least(2.0, 2.5);
        test_bounds(bounds, (2.0, f32::MAX), (2.5, f32::MAX));

        let bounds = DualAxisBounds::at_least_all(2.5);
        test_bounds(bounds, (2.5, f32::MAX), (2.5, f32::MAX));

        let bounds = DualAxisBounds::at_least_only_x(2.5);
        test_bounds(bounds, (2.5, f32::MAX), full_range);

        let bounds = DualAxisBounds::at_least_only_y(2.5);
        test_bounds(bounds, full_range, (2.5, f32::MAX));

        let bounds = DualAxisBounds::at_most(2.0, 2.5);
        test_bounds(bounds, (f32::MIN, 2.0), (f32::MIN, 2.5));

        let bounds = DualAxisBounds::at_most_all(2.5);
        test_bounds(bounds, (f32::MIN, 2.5), (f32::MIN, 2.5));

        let bounds = DualAxisBounds::at_most_only_x(2.5);
        test_bounds(bounds, (f32::MIN, 2.5), full_range);

        let bounds = DualAxisBounds::at_most_only_y(2.5);
        test_bounds(bounds, full_range, (f32::MIN, 2.5));

        let bounds_x = AxisBounds::new(-2.0, 2.5);
        let bounds_y = AxisBounds::new(-1.0, 1.5);

        test_bounds(bounds_x.extend_dual(), (-2.0, 2.5), (-2.0, 2.5));

        test_bounds(bounds_y.extend_dual(), (-1.0, 1.5), (-1.0, 1.5));

        test_bounds(bounds_x.extend_dual_only_x(), (-2.0, 2.5), full_range);

        test_bounds(bounds_y.extend_dual_only_y(), full_range, (-1.0, 1.5));

        test_bounds(
            bounds_x.extend_dual_with_y(bounds_y),
            (-2.0, 2.5),
            (-1.0, 1.5),
        );

        test_bounds(
            bounds_y.extend_dual_with_x(bounds_x),
            (-2.0, 2.5),
            (-1.0, 1.5),
        );
    }

    #[test]
    fn test_dual_axis_exclusion() {
        fn test_exclusion(
            exclusion: DualAxisExclusion,
            (x_negative_max, x_positive_min): (f32, f32),
            (y_negative_max, y_positive_min): (f32, f32),
        ) {
            assert_eq!(
                exclusion.exclusion_x.min_max(),
                (x_negative_max, x_positive_min)
            );
            assert_eq!(
                exclusion.exclusion_y.min_max(),
                (y_negative_max, y_positive_min)
            );

            let exclusion_x = AxisExclusion::new(x_negative_max, x_positive_min);
            let exclusion_y = AxisExclusion::new(y_negative_max, y_positive_min);
            assert_eq!(exclusion_x.extend_dual_with_y(exclusion_y), exclusion);

            let (ex, ey) = exclusion.exclusions();
            assert_eq!(ex, exclusion_x);
            assert_eq!(ey, exclusion_y);

            assert_eq!(
                DualAxisProcessor::from(exclusion_x),
                DualAxisProcessor::Exclusion(DualAxisExclusion::all(
                    x_negative_max,
                    x_positive_min
                ))
            );

            let processor = DualAxisProcessor::Exclusion(exclusion);
            assert_eq!(DualAxisProcessor::from(exclusion), processor);

            for x in -300..300 {
                let x = x as f32 * 0.01;
                for y in -300..300 {
                    let y = y as f32 * 0.01;
                    let value = Vec2::new(x, y);

                    assert_eq!(processor.process(value), exclusion.exclude(value));

                    assert_eq!(
                        exclusion.contains(value),
                        BVec2::new(exclusion_x.contains(x), exclusion_y.contains(y))
                    );

                    assert_eq!(
                        exclusion.exclude(value),
                        Vec2::new(exclusion_x.exclude(x), exclusion_y.exclude(y))
                    );
                }
            }
        }

        let zero_size = (0.0, 0.0);

        let exclusion = DualAxisExclusion::ZERO;
        test_exclusion(exclusion, zero_size, zero_size);

        let exclusion = DualAxisExclusion::default();
        test_exclusion(exclusion, (-0.1, 0.1), (-0.1, 0.1));

        let exclusion = DualAxisExclusion::new((-0.2, 0.3), (-0.1, 0.4));
        test_exclusion(exclusion, (-0.2, 0.3), (-0.1, 0.4));

        let exclusion = DualAxisExclusion::all(-0.2, 0.3);
        test_exclusion(exclusion, (-0.2, 0.3), (-0.2, 0.3));

        let exclusion = DualAxisExclusion::only_x(-0.2, 0.3);
        test_exclusion(exclusion, (-0.2, 0.3), zero_size);

        let exclusion = DualAxisExclusion::only_y(-0.1, 0.4);
        test_exclusion(exclusion, zero_size, (-0.1, 0.4));

        let exclusion = DualAxisExclusion::symmetric(0.2, 0.3);
        test_exclusion(exclusion, (-0.2, 0.2), (-0.3, 0.3));

        let exclusion = DualAxisExclusion::symmetric_all(0.3);
        test_exclusion(exclusion, (-0.3, 0.3), (-0.3, 0.3));

        let exclusion = DualAxisExclusion::symmetric_only_x(0.3);
        test_exclusion(exclusion, (-0.3, 0.3), zero_size);

        let exclusion = DualAxisExclusion::symmetric_only_y(0.3);
        test_exclusion(exclusion, zero_size, (-0.3, 0.3));

        let exclusion_x = AxisExclusion::new(-0.2, 0.3);
        let exclusion_y = AxisExclusion::new(-0.1, 0.4);

        test_exclusion(exclusion_x.extend_dual(), (-0.2, 0.3), (-0.2, 0.3));

        test_exclusion(exclusion_y.extend_dual(), (-0.1, 0.4), (-0.1, 0.4));

        test_exclusion(exclusion_x.extend_dual_only_x(), (-0.2, 0.3), zero_size);

        test_exclusion(exclusion_y.extend_dual_only_y(), zero_size, (-0.1, 0.4));

        test_exclusion(
            exclusion_x.extend_dual_with_y(exclusion_y),
            (-0.2, 0.3),
            (-0.1, 0.4),
        );

        test_exclusion(
            exclusion_y.extend_dual_with_x(exclusion_x),
            (-0.2, 0.3),
            (-0.1, 0.4),
        );
    }

    #[test]
    fn test_dual_axis_deadzone() {
        fn test_deadzone(
            deadzone: DualAxisDeadZone,
            (x_negative_max, x_positive_min): (f32, f32),
            (y_negative_max, y_positive_min): (f32, f32),
        ) {
            assert_eq!(
                deadzone.deadzone_x.exclusion().min_max(),
                (x_negative_max, x_positive_min)
            );
            assert_eq!(
                deadzone.deadzone_y.exclusion().min_max(),
                (y_negative_max, y_positive_min)
            );

            let deadzone_x = AxisDeadZone::new(x_negative_max, x_positive_min);
            let deadzone_y = AxisDeadZone::new(y_negative_max, y_positive_min);
            assert_eq!(deadzone_x.extend_dual_with_y(deadzone_y), deadzone);

            let exclusion = DualAxisExclusion::new(
                (x_negative_max, x_positive_min),
                (y_negative_max, y_positive_min),
            );
            assert_eq!(exclusion.scaled(), deadzone);

            let (dx, dy) = deadzone.deadzones();
            assert_eq!(dx, deadzone_x);
            assert_eq!(dy, deadzone_y);

            assert_eq!(
                DualAxisProcessor::from(deadzone_x),
                DualAxisProcessor::DeadZone(DualAxisDeadZone::all(x_negative_max, x_positive_min))
            );

            let processor = DualAxisProcessor::DeadZone(deadzone);
            assert_eq!(DualAxisProcessor::from(deadzone), processor);

            for x in -300..300 {
                let x = x as f32 * 0.01;
                for y in -300..300 {
                    let y = y as f32 * 0.01;
                    let value = Vec2::new(x, y);

                    assert_eq!(processor.process(value), deadzone.normalize(value));

                    assert_eq!(
                        deadzone.within_exclusion(value),
                        BVec2::new(
                            deadzone_x.within_exclusion(x),
                            deadzone_y.within_exclusion(y)
                        )
                    );

                    assert_eq!(
                        deadzone.within_bounds(value),
                        BVec2::new(deadzone_x.within_bounds(x), deadzone_y.within_bounds(y))
                    );

                    assert_eq!(
                        deadzone.within_livezone_lower(value),
                        BVec2::new(
                            deadzone_x.within_livezone_lower(x),
                            deadzone_y.within_livezone_lower(y)
                        )
                    );

                    assert_eq!(
                        deadzone.within_livezone_upper(value),
                        BVec2::new(
                            deadzone_x.within_livezone_upper(x),
                            deadzone_y.within_livezone_upper(y)
                        )
                    );

                    assert_eq!(
                        deadzone.normalize(value),
                        Vec2::new(deadzone_x.normalize(x), deadzone_y.normalize(y))
                    );
                }
            }
        }

        let zero_size = (0.0, 0.0);

        let deadzone = DualAxisDeadZone::ZERO;
        test_deadzone(deadzone, zero_size, zero_size);

        let deadzone = DualAxisDeadZone::default();
        test_deadzone(deadzone, (-0.1, 0.1), (-0.1, 0.1));

        let deadzone = DualAxisDeadZone::new((-0.2, 0.3), (-0.1, 0.4));
        test_deadzone(deadzone, (-0.2, 0.3), (-0.1, 0.4));

        let deadzone = DualAxisDeadZone::all(-0.2, 0.3);
        test_deadzone(deadzone, (-0.2, 0.3), (-0.2, 0.3));

        let deadzone = DualAxisDeadZone::only_x(-0.2, 0.3);
        test_deadzone(deadzone, (-0.2, 0.3), zero_size);

        let deadzone = DualAxisDeadZone::only_y(-0.1, 0.4);
        test_deadzone(deadzone, zero_size, (-0.1, 0.4));

        let deadzone = DualAxisDeadZone::symmetric(0.2, 0.3);
        test_deadzone(deadzone, (-0.2, 0.2), (-0.3, 0.3));

        let deadzone = DualAxisDeadZone::symmetric_all(0.3);
        test_deadzone(deadzone, (-0.3, 0.3), (-0.3, 0.3));

        let deadzone = DualAxisDeadZone::symmetric_only_x(0.3);
        test_deadzone(deadzone, (-0.3, 0.3), zero_size);

        let deadzone = DualAxisDeadZone::symmetric_only_y(0.3);
        test_deadzone(deadzone, zero_size, (-0.3, 0.3));

        let deadzone_x = AxisDeadZone::new(-0.2, 0.3);
        let deadzone_y = AxisDeadZone::new(-0.1, 0.4);

        test_deadzone(deadzone_x.extend_dual(), (-0.2, 0.3), (-0.2, 0.3));

        test_deadzone(deadzone_y.extend_dual(), (-0.1, 0.4), (-0.1, 0.4));

        test_deadzone(deadzone_x.extend_dual_only_x(), (-0.2, 0.3), zero_size);

        test_deadzone(deadzone_y.extend_dual_only_y(), zero_size, (-0.1, 0.4));

        test_deadzone(
            deadzone_x.extend_dual_with_y(deadzone_y),
            (-0.2, 0.3),
            (-0.1, 0.4),
        );

        test_deadzone(
            deadzone_y.extend_dual_with_x(deadzone_x),
            (-0.2, 0.3),
            (-0.1, 0.4),
        );
    }
}
