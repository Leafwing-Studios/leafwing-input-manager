//! Value bounds for dual-axis inputs

use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use bevy::prelude::*;
use bevy::utils::FloatOrd;
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
/// // Restrict X to [-2, 3] and Y to [-1, 4].
/// let bounds = DualAxisBounds::new((-2.0, 3.0), (-1.0, 4.0));
/// assert_eq!(bounds.bounds_x().min_max(), (-2.0, 3.0));
/// assert_eq!(bounds.bounds_y().min_max(), (-1.0, 4.0));
///
/// // Another way to create a DualAxisBounds.
/// let bounds_x = AxisBounds::new(-2.0, 3.0);
/// let bounds_y = AxisBounds::new(-1.0, 4.0);
/// assert_eq!(bounds_x.extend_dual_with_y(bounds_y), bounds);
///
/// for x in -300..300 {
///     let x = x as f32 * 0.01;
///     for y in -300..300 {
///         let y = y as f32 * 0.01;
///         let value = Vec2::new(x, y);
///
///         assert_eq!(bounds.process(value).x, bounds_x.process(x));
///         assert_eq!(bounds.process(value).y, bounds_y.process(y));
///     }
/// }
/// ```
#[doc(alias = "SquareBounds")]
#[doc(alias = "AxialBounds")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct DualAxisBounds {
    /// The [`AxisBounds`] for the X-axis inputs.
    pub(crate) bounds_x: AxisBounds,

    /// The [`AxisBounds`] for the Y-axis inputs.
    pub(crate) bounds_y: AxisBounds,
}

#[typetag::serde]
impl DualAxisProcessor for DualAxisBounds {
    /// Clamps `input_value` within the bounds.
    #[must_use]
    #[inline]
    fn process(&self, input_value: Vec2) -> Vec2 {
        Vec2::new(
            self.bounds_x.process(input_value.x),
            self.bounds_y.process(input_value.y),
        )
    }
}

impl Default for DualAxisBounds {
    /// Creates a [`DualAxisBounds`] that restricts values within the range `[-1.0, 1.0]` on both axes.
    #[inline]
    fn default() -> Self {
        Self {
            bounds_x: AxisBounds::default(),
            bounds_y: AxisBounds::default(),
        }
    }
}

impl DualAxisBounds {
    /// Unlimited [`DualAxisBounds`].
    pub const FULL_RANGE: Self = Self {
        bounds_x: AxisBounds::FULL_RANGE,
        bounds_y: AxisBounds::FULL_RANGE,
    };

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
    #[doc(alias = "symmetric")]
    #[inline]
    pub fn magnitude(threshold_x: f32, threshold_y: f32) -> Self {
        Self {
            bounds_x: AxisBounds::magnitude(threshold_x),
            bounds_y: AxisBounds::magnitude(threshold_y),
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
    #[doc(alias = "symmetric_all")]
    #[inline]
    pub fn magnitude_all(threshold: f32) -> Self {
        Self::magnitude(threshold, threshold)
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
    #[doc(alias = "symmetric_only_x")]
    #[inline]
    pub fn magnitude_only_x(threshold: f32) -> Self {
        Self {
            bounds_x: AxisBounds::magnitude(threshold),
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
    #[doc(alias = "symmetric_only_y")]
    #[inline]
    pub fn magnitude_only_y(threshold: f32) -> Self {
        Self {
            bounds_y: AxisBounds::magnitude(threshold),
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
        Self {
            bounds_x: AxisBounds::at_least(min),
            bounds_y: AxisBounds::at_least(min),
        }
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
        Self {
            bounds_x: AxisBounds::at_most(max),
            bounds_y: AxisBounds::at_most(max),
        }
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

impl From<AxisBounds> for DualAxisBounds {
    fn from(bounds: AxisBounds) -> Self {
        bounds.extend_dual()
    }
}

/// Specifies a circular region defining acceptable ranges for valid dual-axis inputs,
/// with a radius defining the maximum threshold magnitude,
/// restricting all values stay within intended limits
/// to avoid unexpected behavior caused by extreme inputs.
///
/// ```rust
/// use bevy::prelude::*;
/// use leafwing_input_manager::prelude::*;
///
/// // Restrict magnitudes to no greater than 5
/// let bounds = CircleBounds::new(5.0);
///
/// for x in -300..300 {
///     let x = x as f32 * 0.01;
///     for y in -300..300 {
///         let y = y as f32 * 0.01;
///         let value = Vec2::new(x, y);
///         assert_eq!(bounds.process(value), value.clamp_length_max(5.0));
///     }
/// }
/// ```
#[doc(alias = "RadialBounds")]
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct CircleBounds {
    /// The maximum radius of the circle.
    pub(crate) radius: f32,
}

#[typetag::serde]
impl DualAxisProcessor for CircleBounds {
    /// Clamps the magnitude of `input_value` within the bounds.
    #[must_use]
    #[inline]
    fn process(&self, input_value: Vec2) -> Vec2 {
        input_value.clamp_length_max(self.radius)
    }
}

impl Default for CircleBounds {
    /// Creates a [`CircleBounds`] that restricts the values to a maximum magnitude of `1.0`.
    #[inline]
    fn default() -> Self {
        Self::new(1.0)
    }
}

impl CircleBounds {
    /// Unlimited [`CircleBounds`].
    pub const FULL_RANGE: Self = Self { radius: f32::MAX };

    /// Creates a [`CircleBounds`] that restricts input values to a maximum magnitude.
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
        assert!(threshold >= 0.0);
        Self { radius: threshold }
    }

    /// Returns the radius of the bounds.
    #[must_use]
    #[inline]
    pub fn radius(&self) -> f32 {
        self.radius
    }

    /// Is the `input_value` is within the bounds?
    #[must_use]
    #[inline]
    pub fn contains(&self, input_value: Vec2) -> bool {
        input_value.length() <= self.radius
    }
}

impl Eq for CircleBounds {}

impl Hash for CircleBounds {
    fn hash<H: Hasher>(&self, state: &mut H) {
        FloatOrd(self.radius).hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dual_axis_value_bounds_constructors() {
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

            let (bx, by) = bounds.bounds();
            assert_eq!(bx, bounds_x);
            assert_eq!(by, bounds_y);

            for x in -300..300 {
                let x = x as f32 * 0.01;
                for y in -300..300 {
                    let y = y as f32 * 0.01;
                    let value = Vec2::new(x, y);

                    let expected = BVec2::new(bounds_x.contains(x), bounds_y.contains(y));
                    assert_eq!(bounds.contains(value), expected);

                    let expected = Vec2::new(bounds_x.process(x), bounds_y.process(y));
                    assert_eq!(bounds.process(value), expected);
                }
            }
        }

        let full_range = (f32::MIN, f32::MAX);

        let bounds = DualAxisBounds::FULL_RANGE;
        test_bounds(bounds, full_range, full_range);

        let bounds = DualAxisBounds::default();
        test_bounds(bounds, (-1.0, 1.0), (-1.0, 1.0));

        let bounds = DualAxisBounds::new((-2.0, 3.0), (-1.0, 4.0));
        test_bounds(bounds, (-2.0, 3.0), (-1.0, 4.0));

        let bounds = DualAxisBounds::all(-2.0, 3.0);
        test_bounds(bounds, (-2.0, 3.0), (-2.0, 3.0));

        let bounds = DualAxisBounds::only_x(-2.0, 3.0);
        test_bounds(bounds, (-2.0, 3.0), full_range);

        let bounds = DualAxisBounds::only_y(-1.0, 4.0);
        test_bounds(bounds, full_range, (-1.0, 4.0));

        let bounds = DualAxisBounds::magnitude(2.0, 3.0);
        test_bounds(bounds, (-2.0, 2.0), (-3.0, 3.0));

        let bounds = DualAxisBounds::magnitude_all(3.0);
        test_bounds(bounds, (-3.0, 3.0), (-3.0, 3.0));

        let bounds = DualAxisBounds::magnitude_only_x(3.0);
        test_bounds(bounds, (-3.0, 3.0), full_range);

        let bounds = DualAxisBounds::magnitude_only_y(3.0);
        test_bounds(bounds, full_range, (-3.0, 3.0));

        let bounds = DualAxisBounds::at_least(2.0, 3.0);
        test_bounds(bounds, (2.0, f32::MAX), (3.0, f32::MAX));

        let bounds = DualAxisBounds::at_least_all(3.0);
        test_bounds(bounds, (3.0, f32::MAX), (3.0, f32::MAX));

        let bounds = DualAxisBounds::at_least_only_x(3.0);
        test_bounds(bounds, (3.0, f32::MAX), full_range);

        let bounds = DualAxisBounds::at_least_only_y(3.0);
        test_bounds(bounds, full_range, (3.0, f32::MAX));

        let bounds = DualAxisBounds::at_most(2.0, 3.0);
        test_bounds(bounds, (f32::MIN, 2.0), (f32::MIN, 3.0));

        let bounds = DualAxisBounds::at_most_all(3.0);
        test_bounds(bounds, (f32::MIN, 3.0), (f32::MIN, 3.0));

        let bounds = DualAxisBounds::at_most_only_x(3.0);
        test_bounds(bounds, (f32::MIN, 3.0), full_range);

        let bounds = DualAxisBounds::at_most_only_y(3.0);
        test_bounds(bounds, full_range, (f32::MIN, 3.0));

        let bounds_x = AxisBounds::new(-2.0, 3.0);
        let bounds_y = AxisBounds::new(-1.0, 4.0);

        test_bounds(bounds_x.extend_dual(), (-2.0, 3.0), (-2.0, 3.0));
        test_bounds(bounds_x.into(), (-2.0, 3.0), (-2.0, 3.0));

        test_bounds(bounds_y.extend_dual(), (-1.0, 4.0), (-1.0, 4.0));
        test_bounds(bounds_y.into(), (-1.0, 4.0), (-1.0, 4.0));

        test_bounds(bounds_x.extend_dual_only_x(), (-2.0, 3.0), full_range);

        test_bounds(bounds_y.extend_dual_only_y(), full_range, (-1.0, 4.0));

        test_bounds(
            bounds_x.extend_dual_with_y(bounds_y),
            (-2.0, 3.0),
            (-1.0, 4.0),
        );

        test_bounds(
            bounds_y.extend_dual_with_x(bounds_x),
            (-2.0, 3.0),
            (-1.0, 4.0),
        );
    }

    #[test]
    fn test_circle_value_bounds_constructors() {
        fn test_bounds(bounds: CircleBounds, radius: f32) {
            assert_eq!(bounds.radius(), radius);

            for x in -300..300 {
                let x = x as f32 * 0.01;
                for y in -300..300 {
                    let y = y as f32 * 0.01;
                    let value = Vec2::new(x, y);

                    if value.length() <= radius {
                        assert!(bounds.contains(value));
                    } else {
                        assert!(!bounds.contains(value));
                    }

                    let expected = value.clamp_length_max(radius);
                    let delta = (bounds.process(value) - expected).abs();
                    assert!(delta.x <= f32::EPSILON);
                    assert!(delta.y <= f32::EPSILON);
                }
            }
        }

        let bounds = CircleBounds::FULL_RANGE;
        test_bounds(bounds, f32::MAX);

        let bounds = CircleBounds::default();
        test_bounds(bounds, 1.0);

        let bounds = CircleBounds::new(3.0);
        test_bounds(bounds, 3.0);
    }
}
