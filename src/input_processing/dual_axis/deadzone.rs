//! Deadzones for dual-axis inputs

use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use bevy::prelude::*;
use bevy::utils::FloatOrd;
use serde::{Deserialize, Serialize};

use super::bounds::*;
use super::DualAxisProcessor;
use crate::input_processing::single_axis::*;

/// Specifies a cross-shaped region for excluding dual-axis inputs,
/// with min-max independent min-max ranges for each axis, resulting in per-axis "snapping" effect,
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
///         assert_eq!(exclusion.process(value).x, exclusion_x.process(x));
///         assert_eq!(exclusion.process(value).y, exclusion_y.process(y));
///     }
/// }
/// ```
#[doc(alias = "CrossExclusion")]
#[doc(alias = "AxialExclusion")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct DualAxisExclusion {
    /// The [`AxisExclusion`] for the X-axis inputs.
    pub(crate) exclusion_x: AxisExclusion,

    /// The [`AxisExclusion`] for the Y-axis inputs.
    pub(crate) exclusion_y: AxisExclusion,
}

#[typetag::serde]
impl DualAxisProcessor for DualAxisExclusion {
    /// Excludes values within the specified region.
    #[must_use]
    #[inline]
    fn process(&self, input_value: Vec2) -> Vec2 {
        Vec2::new(
            self.exclusion_x.process(input_value.x),
            self.exclusion_y.process(input_value.y),
        )
    }
}

impl Default for DualAxisExclusion {
    /// Creates a [`DualAxisExclusion`] that excludes input values within `[-1.0, 1.0]` on both axes.
    #[inline]
    fn default() -> Self {
        Self {
            exclusion_x: AxisExclusion::default(),
            exclusion_y: AxisExclusion::default(),
        }
    }
}

impl DualAxisExclusion {
    /// Zero-size [`DualAxisExclusion`], leaving values as is.
    pub const ZERO: Self = Self {
        exclusion_x: AxisExclusion::ZERO,
        exclusion_y: AxisExclusion::ZERO,
    };

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

    /// Creates a [`DualAxisExclusion`] that ignores values within the range `[negative_max, positive_min]` on both axis.
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
    #[doc(alias = "symmetric")]
    #[inline]
    pub fn magnitude(threshold_x: f32, threshold_y: f32) -> Self {
        Self {
            exclusion_x: AxisExclusion::magnitude(threshold_x),
            exclusion_y: AxisExclusion::magnitude(threshold_y),
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
    #[doc(alias = "symmetric_all")]
    #[inline]
    pub fn magnitude_all(threshold: f32) -> Self {
        Self::magnitude(threshold, threshold)
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
    #[doc(alias = "symmetric_only_x")]
    #[inline]
    pub fn magnitude_only_x(threshold: f32) -> Self {
        Self {
            exclusion_x: AxisExclusion::magnitude(threshold),
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
    #[doc(alias = "symmetric_only_y")]
    #[inline]
    pub fn magnitude_only_y(threshold: f32) -> Self {
        Self {
            exclusion_y: AxisExclusion::magnitude(threshold),
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

    /// Creates a [`DualAxisDeadZone`] using `self` as the exclusion range.
    pub fn scaled(self) -> DualAxisDeadZone {
        self.into()
    }
}

impl AxisExclusion {
    /// Creates a [`DualAxisExclusion`] using `self` for both axes.
    #[inline]
    pub fn extend_dual(self) -> DualAxisExclusion {
        DualAxisExclusion {
            exclusion_x: self,
            exclusion_y: self,
        }
    }

    /// Creates a [`DualAxisExclusion`] only using `self` for the X-axis.
    #[inline]
    pub fn extend_dual_only_x(self) -> DualAxisExclusion {
        DualAxisExclusion {
            exclusion_x: self,
            ..DualAxisExclusion::ZERO
        }
    }

    /// Creates a [`DualAxisExclusion`] only using `self` to the Y-axis.
    #[inline]
    pub fn extend_dual_only_y(self) -> DualAxisExclusion {
        DualAxisExclusion {
            exclusion_y: self,
            ..DualAxisExclusion::ZERO
        }
    }

    /// Creates a [`DualAxisExclusion`] using `self` to the Y-axis with the given `bounds_x` to the X-axis.
    #[inline]
    pub fn extend_dual_with_x(self, exclusion_x: Self) -> DualAxisExclusion {
        DualAxisExclusion {
            exclusion_x,
            exclusion_y: self,
        }
    }

    /// Creates a [`DualAxisExclusion`] using `self` to the X-axis with the given `bounds_y` to the Y-axis.
    #[inline]
    pub fn extend_dual_with_y(self, exclusion_y: Self) -> DualAxisExclusion {
        DualAxisExclusion {
            exclusion_x: self,
            exclusion_y,
        }
    }
}

impl From<AxisExclusion> for DualAxisExclusion {
    fn from(exclusion: AxisExclusion) -> Self {
        exclusion.extend_dual()
    }
}

/// A scaled version of [`DualAxisExclusion`] with the bounds
/// set to [`DualAxisBounds::magnitude_all(1.0)`](DualAxisBounds::default)
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
///         assert_eq!(deadzone.process(value).x, deadzone_x.process(x));
///         assert_eq!(deadzone.process(value).y, deadzone_y.process(y));
///     }
/// }
/// ```
#[doc(alias = "CrossDeadZone")]
#[doc(alias = "AxialDeadZone")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct DualAxisDeadZone {
    /// The [`AxisDeadZone`] for the X-axis inputs.
    pub(crate) deadzone_x: AxisDeadZone,

    /// The [`AxisDeadZone`] for the Y-axis inputs.
    pub(crate) deadzone_y: AxisDeadZone,
}

#[typetag::serde]
impl DualAxisProcessor for DualAxisDeadZone {
    /// Normalizes input values into the live zone.
    #[must_use]
    #[inline]
    fn process(&self, input_value: Vec2) -> Vec2 {
        Vec2::new(
            self.deadzone_x.process(input_value.x),
            self.deadzone_y.process(input_value.y),
        )
    }
}

impl Default for DualAxisDeadZone {
    /// Creates a [`DualAxisDeadZone`] that excludes input values within the deadzone `[-0.1, 0.1]` on both axes.
    fn default() -> Self {
        AxisDeadZone::default().extend_dual()
    }
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
    #[doc(alias = "symmetric")]
    #[inline]
    pub fn magnitude(threshold_x: f32, threshold_y: f32) -> Self {
        Self {
            deadzone_x: AxisDeadZone::magnitude(threshold_x),
            deadzone_y: AxisDeadZone::magnitude(threshold_y),
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
    #[doc(alias = "symmetric_all")]
    #[inline]
    pub fn magnitude_all(threshold: f32) -> Self {
        Self::magnitude(threshold, threshold)
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
    #[doc(alias = "symmetric_only_x")]
    #[inline]
    pub fn magnitude_only_x(threshold: f32) -> Self {
        Self {
            deadzone_x: AxisDeadZone::magnitude(threshold),
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
    #[doc(alias = "symmetric_only_y")]
    #[inline]
    pub fn magnitude_only_y(threshold: f32) -> Self {
        Self {
            deadzone_y: AxisDeadZone::magnitude(threshold),
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

impl From<AxisDeadZone> for DualAxisDeadZone {
    fn from(deadzone: AxisDeadZone) -> Self {
        deadzone.extend_dual()
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

/// Specifies a cross-shaped region for excluding dual-axis inputs,
/// with a radius defining the maximum excluded magnitude,
/// helping filter out minor fluctuations and unintended movements.
///
/// ```rust
/// use bevy::prelude::*;
/// use leafwing_input_manager::prelude::*;
///
/// // Exclude magnitudes less than or equal to 0.2
/// let exclusion = CircleExclusion::new(0.2);
///
/// for x in -300..300 {
///     let x = x as f32 * 0.01;
///     for y in -300..300 {
///         let y = y as f32 * 0.01;
///         let value = Vec2::new(x, y);
///
///         if value.length() <= 0.2 {
///             assert!(exclusion.contains(value));
///             assert_eq!(exclusion.process(value), Vec2::ZERO);
///         } else {
///             assert!(!exclusion.contains(value));
///             assert_eq!(exclusion.process(value), value);
///         }
///     }
/// }
/// ```
#[doc(alias = "RadialExclusion")]
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct CircleExclusion {
    /// Pre-calculated squared radius of the circle, preventing redundant calculations.
    pub(crate) radius_squared: f32,
}

#[typetag::serde]
impl DualAxisProcessor for CircleExclusion {
    /// Excludes input values with a magnitude less than the `radius`.
    #[must_use]
    #[inline]
    fn process(&self, input_value: Vec2) -> Vec2 {
        if self.contains(input_value) {
            Vec2::ZERO
        } else {
            input_value
        }
    }
}

impl Default for CircleExclusion {
    /// Creates a [`CircleExclusion`] that ignores input values below a minimum magnitude of `0.1`.
    fn default() -> Self {
        Self::new(0.1)
    }
}

impl CircleExclusion {
    /// Zero-size [`CircleExclusion`], leaving values as is.
    pub const ZERO: Self = Self {
        radius_squared: 0.0,
    };

    /// Creates a [`CircleExclusion`] that ignores input values below a minimum magnitude.
    ///
    /// # Requirements
    ///
    /// - `radius` >= `0.0`.
    ///
    /// # Panics
    ///
    /// Panics if the requirements aren't met.
    #[doc(alias = "magnitude")]
    #[doc(alias = "from_radius")]
    #[inline]
    pub fn new(threshold: f32) -> Self {
        assert!(threshold >= 0.0);
        Self {
            radius_squared: threshold.powi(2),
        }
    }

    /// Returns the radius of the circle.
    #[must_use]
    #[inline]
    pub fn radius(&self) -> f32 {
        self.radius_squared.sqrt()
    }

    /// Checks whether the `input_value` should be excluded.
    #[must_use]
    #[inline]
    pub fn contains(&self, input_value: Vec2) -> bool {
        input_value.length_squared() <= self.radius_squared
    }

    /// Creates a [`CircleDeadZone`] using `self` as the exclusion range.
    #[inline]
    pub fn scaled(self) -> CircleDeadZone {
        self.into()
    }
}

impl Eq for CircleExclusion {}

impl Hash for CircleExclusion {
    fn hash<H: Hasher>(&self, state: &mut H) {
        FloatOrd(self.radius_squared).hash(state);
    }
}

/// A scaled version of [`CircleExclusion`] with the bounds
/// set to [`CircleBounds::new(1.0)`](CircleBounds::default)
/// that normalizes non-excluded input values into the "live zone",
/// the remaining range within the bounds after dead zone exclusion.
///
/// Normalizes input values by clamping them within [`CircleBounds::default`],
/// excluding values via a specified [`CircleExclusion`], and scaling unchanged values linearly in between.
///
/// In simple terms, this processor functions as a scaled [`CircleExclusion`].
/// This processor is useful for filtering out minor fluctuations and unintended movements.
///
/// It is worth considering that this normalizer reduces input values on diagonals.
/// If that is not your goal, you might want to explore alternative normalizers.
///
/// ```rust
/// use bevy::prelude::*;
/// use leafwing_input_manager::prelude::*;
///
/// // Exclude magnitudes less than or equal to 0.2
/// let deadzone = CircleDeadZone::new(0.2);
///
/// for x in -300..300 {
///     let x = x as f32 * 0.01;
///     for y in -300..300 {
///         let y = y as f32 * 0.01;
///         let value = Vec2::new(x, y);
///
///         // Values within the dead zone are treated as zeros.
///         if value.length() <= 0.2 {
///             assert!(deadzone.within_exclusion(value));
///             assert_eq!(deadzone.process(value), Vec2::ZERO);
///         }
///
///         // Values within the live zone are scaled linearly.
///         else if value.length() <= 1.0 {
///             assert!(deadzone.within_livezone(value));
///
///             let expected_scale = f32::inverse_lerp(0.2, 1.0, value.length());
///             let expected = value.normalize() * expected_scale;
///             let delta = (deadzone.process(value) - expected).abs();
///             assert!(delta.x <= 0.00001);
///             assert!(delta.y <= 0.00001);
///         }
///
///         // Values outside the bounds are restricted to the region.
///         else {
///             assert!(!deadzone.within_bounds(value));
///
///             let expected = value.clamp_length_max(1.0);
///             let delta = (deadzone.process(value) - expected).abs();
///             assert!(delta.x <= 0.00001);
///             assert!(delta.y <= 0.00001);
///         }
///     }
/// }
/// ```
#[doc(alias = "RadialDeadZone")]
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct CircleDeadZone {
    /// The radius of the circle.
    pub(crate) radius: f32,

    /// Pre-calculated reciprocal of the live zone size, preventing division during normalization.
    pub(crate) livezone_recip: f32,
}

#[typetag::serde]
impl DualAxisProcessor for CircleDeadZone {
    /// Normalizes input values into the live zone.
    #[must_use]
    fn process(&self, input_value: Vec2) -> Vec2 {
        let input_length = input_value.length();
        if input_length == 0.0 {
            return Vec2::ZERO;
        }

        // Clamp out-of-bounds values to a maximum magnitude of 1,
        // and then exclude values within the dead zone,
        // and finally linearly scale the result to the live zone.
        let (deadzone, bound) = self.livezone_min_max();
        let clamped_input_length = input_length.min(bound);
        let offset_to_deadzone = (clamped_input_length - deadzone).max(0.0);
        let magnitude_scale = (offset_to_deadzone * self.livezone_recip) / input_length;
        input_value * magnitude_scale
    }
}

impl Default for CircleDeadZone {
    /// Creates a [`CircleDeadZone`] that excludes input values below a minimum magnitude of `0.1`.
    #[inline]
    fn default() -> Self {
        CircleExclusion::default().into()
    }
}

impl CircleDeadZone {
    /// Zero-size [`CircleDeadZone`], only restricting values to a maximum magnitude of `1.0`.
    pub const ZERO: Self = Self {
        radius: 0.0,
        livezone_recip: 1.0,
    };

    /// Creates a [`CircleDeadZone`] that excludes input values below a minimum magnitude.
    ///
    /// # Requirements
    ///
    /// - `threshold` >= `0.0`.
    ///
    /// # Panics
    ///
    /// Panics if the requirements aren't met.
    #[doc(alias = "magnitude")]
    #[doc(alias = "from_radius")]
    #[inline]
    pub fn new(threshold: f32) -> Self {
        let bounds = CircleBounds::default();
        Self {
            radius: threshold,
            livezone_recip: (bounds.radius - threshold).recip(),
        }
    }

    /// Returns the radius of the circle.
    #[must_use]
    #[inline]
    pub fn radius(&self) -> f32 {
        self.radius
    }

    /// Returns the [`CircleExclusion`] used by this deadzone.
    #[inline]
    pub fn exclusion(&self) -> CircleExclusion {
        CircleExclusion::new(self.radius)
    }

    /// Returns the [`CircleBounds`] used by this deadzone.
    #[inline]
    pub fn bounds(&self) -> CircleBounds {
        CircleBounds::default()
    }

    /// Returns the minimum and maximum radii of the live zone used by this deadzone.
    ///
    /// In simple terms, this returns `(self.radius, bounds.radius)`.
    #[must_use]
    #[inline]
    pub fn livezone_min_max(&self) -> (f32, f32) {
        (self.radius, self.bounds().radius)
    }

    /// Is the given `input_value` within the exclusion range?
    #[must_use]
    #[inline]
    pub fn within_exclusion(&self, input_value: Vec2) -> bool {
        self.exclusion().contains(input_value)
    }

    /// Is the given `input_value` within the bounds?
    #[must_use]
    #[inline]
    pub fn within_bounds(&self, input_value: Vec2) -> bool {
        self.bounds().contains(input_value)
    }

    /// Is the given `input_value` within the live zone?
    #[must_use]
    #[inline]
    pub fn within_livezone(&self, input_value: Vec2) -> bool {
        let input_length = input_value.length();
        let (min, max) = self.livezone_min_max();
        min <= input_length && input_length <= max
    }
}

impl From<CircleExclusion> for CircleDeadZone {
    fn from(exclusion: CircleExclusion) -> Self {
        Self::new(exclusion.radius())
    }
}

impl Eq for CircleDeadZone {}

impl Hash for CircleDeadZone {
    fn hash<H: Hasher>(&self, state: &mut H) {
        FloatOrd(self.radius).hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

            for x in -300..300 {
                let x = x as f32 * 0.01;
                for y in -300..300 {
                    let y = y as f32 * 0.01;
                    let value = Vec2::new(x, y);

                    assert_eq!(
                        exclusion.contains(value),
                        BVec2::new(exclusion_x.contains(x), exclusion_y.contains(y))
                    );

                    assert_eq!(
                        exclusion.process(value),
                        Vec2::new(exclusion_x.process(x), exclusion_y.process(y))
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

        let exclusion = DualAxisExclusion::magnitude(0.2, 0.3);
        test_exclusion(exclusion, (-0.2, 0.2), (-0.3, 0.3));

        let exclusion = DualAxisExclusion::magnitude_all(0.3);
        test_exclusion(exclusion, (-0.3, 0.3), (-0.3, 0.3));

        let exclusion = DualAxisExclusion::magnitude_only_x(0.3);
        test_exclusion(exclusion, (-0.3, 0.3), zero_size);

        let exclusion = DualAxisExclusion::magnitude_only_y(0.3);
        test_exclusion(exclusion, zero_size, (-0.3, 0.3));

        let exclusion_x = AxisExclusion::new(-0.2, 0.3);
        let exclusion_y = AxisExclusion::new(-0.1, 0.4);

        test_exclusion(exclusion_x.extend_dual(), (-0.2, 0.3), (-0.2, 0.3));
        test_exclusion(exclusion_x.into(), (-0.2, 0.3), (-0.2, 0.3));
        test_exclusion(exclusion_y.into(), (-0.1, 0.4), (-0.1, 0.4));

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

            for x in -300..300 {
                let x = x as f32 * 0.01;
                for y in -300..300 {
                    let y = y as f32 * 0.01;
                    let value = Vec2::new(x, y);

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
                        deadzone.process(value),
                        Vec2::new(deadzone_x.process(x), deadzone_y.process(y))
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

        let deadzone = DualAxisDeadZone::magnitude(0.2, 0.3);
        test_deadzone(deadzone, (-0.2, 0.2), (-0.3, 0.3));

        let deadzone = DualAxisDeadZone::magnitude_all(0.3);
        test_deadzone(deadzone, (-0.3, 0.3), (-0.3, 0.3));

        let deadzone = DualAxisDeadZone::magnitude_only_x(0.3);
        test_deadzone(deadzone, (-0.3, 0.3), zero_size);

        let deadzone = DualAxisDeadZone::magnitude_only_y(0.3);
        test_deadzone(deadzone, zero_size, (-0.3, 0.3));
    }

    #[test]
    fn test_circle_exclusion() {
        fn test_exclusion(exclusion: CircleExclusion, radius: f32) {
            assert_eq!(exclusion.radius(), radius);

            for x in -300..300 {
                let x = x as f32 * 0.01;
                for y in -300..300 {
                    let y = y as f32 * 0.01;
                    let value = Vec2::new(x, y);

                    if value.length() <= radius {
                        assert!(exclusion.contains(value));
                        assert_eq!(exclusion.process(value), Vec2::ZERO);
                    } else {
                        assert!(!exclusion.contains(value));
                        assert_eq!(exclusion.process(value), value);
                    }
                }
            }
        }

        let exclusion = CircleExclusion::ZERO;
        test_exclusion(exclusion, 0.0);

        let exclusion = CircleExclusion::default();
        test_exclusion(exclusion, 0.1);

        let exclusion = CircleExclusion::new(0.5);
        test_exclusion(exclusion, 0.5);
    }

    #[test]
    fn test_circle_deadzone() {
        fn test_deadzone(deadzone: CircleDeadZone, radius: f32) {
            assert_eq!(deadzone.radius(), radius);

            let exclusion = CircleExclusion::new(radius);
            assert_eq!(exclusion.scaled(), deadzone);

            for x in -300..300 {
                let x = x as f32 * 0.01;
                for y in -300..300 {
                    let y = y as f32 * 0.01;
                    let value = Vec2::new(x, y);

                    // Values within the dead zone are treated as zeros.
                    if value.length() <= radius {
                        assert!(deadzone.within_exclusion(value));
                        assert_eq!(deadzone.process(value), Vec2::ZERO);
                    }
                    // Values within the live zone are scaled linearly.
                    else if value.length() <= 1.0 {
                        assert!(deadzone.within_livezone(value));

                        let expected_scale = f32::inverse_lerp(radius, 1.0, value.length());
                        let expected = value.normalize() * expected_scale;
                        let delta = (deadzone.process(value) - expected).abs();
                        assert!(delta.x <= 0.00001);
                        assert!(delta.y <= 0.00001);
                    }
                    // Values outside the bounds are restricted to the region.
                    else {
                        assert!(!deadzone.within_bounds(value));

                        let expected = value.clamp_length_max(1.0);
                        let delta = (deadzone.process(value) - expected).abs();
                        assert!(delta.x <= 0.00001);
                        assert!(delta.y <= 0.00001);
                    }
                }
            }
        }

        let deadzone = CircleDeadZone::ZERO;
        test_deadzone(deadzone, 0.0);

        let deadzone = CircleDeadZone::default();
        test_deadzone(deadzone, 0.1);

        let deadzone = CircleDeadZone::new(0.5);
        test_deadzone(deadzone, 0.5);
    }
}
