//! Deadzones for dual-axis inputs

use std::hash::{Hash, Hasher};

use bevy::prelude::*;
use bevy::utils::FloatOrd;
use serde::{Deserialize, Serialize};

use super::super::single_axis::*;
use super::bounds::*;
use super::DualAxisProcessor;
use crate::define_dual_axis_processor;

// region exclusion

define_dual_axis_processor!(
    name: DualAxisExclusion,
    perform: "axial exclusion ranges",
    stored_processor_type: AxisExclusion
);

impl Default for DualAxisExclusion {
    /// Creates a default [`DualAxisExclusion`] that excludes input values within `[-1.0, 1.0]` on each axis.
    #[inline]
    fn default() -> Self {
        AxisExclusion::default().extend_dual()
    }
}

impl DualAxisExclusion {
    /// Checks whether the `input_value` should be excluded.
    #[must_use]
    #[inline]
    pub fn contains(&self, input_value: Vec2) -> BVec2 {
        let Vec2 { x, y } = input_value;
        match self {
            Self::All(exclusion) => BVec2::new(exclusion.contains(x), exclusion.contains(y)),
            Self::Separate(exclusion_x, exclusion_y) => {
                BVec2::new(exclusion_x.contains(x), exclusion_y.contains(y))
            }
            Self::OnlyX(exclusion) => BVec2::new(exclusion.contains(x), false),
            Self::OnlyY(exclusion) => BVec2::new(false, exclusion.contains(y)),
        }
    }
}

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

// endregion exclusion

// region deadzone

define_dual_axis_processor!(
    name: DualAxisDeadzone,
    perform: "livezone value normalization",
    stored_processor_type: AxisDeadzone,
    info: "Each axis is processed individually, resulting in a per-axis \"snapping\" or locked effect, \
        which enhances control precision for pure axial motion. \
        It is commonly known as the `CrossDeadzone` due to its shape, \
        formed by two intersecting [`AxisDeadzone`]s. \
        It is worth considering that this normalizer increases the magnitude of diagonal values. \
        If that is not your goal, you might want to explore alternative normalizers."
);

impl Default for DualAxisDeadzone {
    /// Creates a default [`DualAxisDeadzone`] that normalizes input values
    /// by clamping them to `[-1.0, 1.0]` and excluding those within `[-0.1, 0.1]` on each axis.
    fn default() -> Self {
        AxisDeadzone::default().extend_dual()
    }
}

/// Defines a deadzone that normalizes input values by clamping them within [`CircleBounds::default`],
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
    /// Processes the `input_value` by clamping them within [`CircleBounds::default`],
    /// excluding those within the deadzone, and scaling unchanged values linearly in between.
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

// endregion deadzone

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dual_axis_exclusion_default() {
        // -0.1 to 0.1 on each axis.
        let exclusion = DualAxisExclusion::default();

        assert_eq!(exclusion, AxisExclusion::default().extend_dual());

        assert!(matches!(exclusion.x(), Some(exclusion_x) if exclusion_x.min_max() == (-0.1, 0.1)));
        assert!(matches!(exclusion.y(), Some(exclusion_y) if exclusion_y.min_max() == (-0.1, 0.1)));
    }

    #[test]
    fn test_dual_axis_exclusion_behavior() {
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
    fn test_dual_axis_deadzone() {
        let exclusion_x = AxisExclusion::new(-0.2, 0.3);
        let exclusion_y = AxisExclusion::new(-0.3, 0.4);

        let axis_x = AxisDeadzone::new(exclusion_x);
        let axis_y = AxisDeadzone::new(exclusion_y);

        let deadzone = DualAxisDeadzone::Separate(axis_x, axis_y);
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

            // And the value shouldn't be excluded.
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
}
