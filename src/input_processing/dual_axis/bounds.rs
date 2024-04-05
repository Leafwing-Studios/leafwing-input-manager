use std::hash::{Hash, Hasher};

use bevy::prelude::*;
use bevy::utils::FloatOrd;
use serde::{Deserialize, Serialize};

use super::super::single_axis::*;
use super::DualAxisProcessor;
use crate::define_dual_axis_processor;

define_dual_axis_processor!(
    name: DualAxisBounds,
    perform: "axial bounds",
    stored_processor_type: AxisBounds
);

impl Default for DualAxisBounds {
    /// Creates a new [`DualAxisBounds`] with bounds set to `[-1.0, 1.0]` on each axis.
    #[inline]
    fn default() -> Self {
        AxisBounds::default().extend_dual()
    }
}

impl DualAxisBounds {
    /// Checks whether the `input_value` is within the bounds along each axis.
    #[must_use]
    #[inline]
    pub fn contains(&self, input_value: Vec2) -> BVec2 {
        let Vec2 { x, y } = input_value;
        match self {
            Self::OnlyX(bounds) => BVec2::new(bounds.contains(x), true),
            Self::OnlyY(bounds) => BVec2::new(true, bounds.contains(y)),
            Self::All(bounds) => BVec2::new(bounds.contains(x), bounds.contains(y)),
            Self::Separate(bounds_x, bounds_y) => {
                BVec2::new(bounds_x.contains(x), bounds_y.contains(y))
            }
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dual_axis_value_bounds_default() {
        // -1 to 1 on each axis
        let bounds = DualAxisBounds::default();

        assert_eq!(bounds, AxisBounds::default().extend_dual());

        assert!(matches!(bounds.x(), Some(bounds_x) if bounds_x.min_max() == (-1.0, 1.0)));
        assert!(matches!(bounds.y(), Some(bounds_y) if bounds_y.min_max() == (-1.0, 1.0)));
    }

    #[test]
    fn test_dual_axis_value_bounds_behavior() {
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
}
