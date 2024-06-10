//! Circular range processors for dual-axis inputs

use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use bevy::{
    math::FloatOrd,
    prelude::{Reflect, Vec2},
};
use serde::{Deserialize, Serialize};

use super::DualAxisProcessor;

/// Specifies a circular region defining acceptable ranges for valid dual-axis inputs,
/// with a radius defining the maximum threshold magnitude,
/// restricting all values stay within intended limits
/// to avoid unexpected behavior caused by extreme inputs.
///
/// ```rust
/// use bevy::prelude::*;
/// use leafwing_input_manager::prelude::*;
///
/// // Restrict magnitudes to no greater than 2
/// let bounds = CircleBounds::new(2.0);
///
/// for x in -300..300 {
///     let x = x as f32 * 0.01;
///     for y in -300..300 {
///         let y = y as f32 * 0.01;
///         let value = Vec2::new(x, y);
///         assert_eq!(bounds.clamp(value), value.clamp_length_max(2.0));
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

    /// Clamps the magnitude of `input_value` within the bounds.
    #[must_use]
    #[inline]
    pub fn clamp(&self, input_value: Vec2) -> Vec2 {
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

impl From<CircleBounds> for DualAxisProcessor {
    fn from(value: CircleBounds) -> Self {
        Self::CircleBounds(value)
    }
}

impl Eq for CircleBounds {}

impl Hash for CircleBounds {
    fn hash<H: Hasher>(&self, state: &mut H) {
        FloatOrd(self.radius).hash(state);
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
///             assert_eq!(exclusion.exclude(value), Vec2::ZERO);
///         } else {
///             assert!(!exclusion.contains(value));
///             assert_eq!(exclusion.exclude(value), value);
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
        CircleDeadZone::new(self.radius())
    }

    /// Excludes input values with a magnitude less than the `radius`.
    #[must_use]
    #[inline]
    pub fn exclude(&self, input_value: Vec2) -> Vec2 {
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

impl From<CircleExclusion> for DualAxisProcessor {
    fn from(value: CircleExclusion) -> Self {
        Self::CircleExclusion(value)
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
///             assert_eq!(deadzone.normalize(value), Vec2::ZERO);
///         }
///
///         // Values within the live zone are scaled linearly.
///         else if value.length() <= 1.0 {
///             assert!(deadzone.within_livezone(value));
///
///             let expected_scale = f32::inverse_lerp(0.2, 1.0, value.length());
///             let expected = value.normalize() * expected_scale;
///             let delta = (deadzone.normalize(value) - expected).abs();
///             assert!(delta.x <= 0.00001);
///             assert!(delta.y <= 0.00001);
///         }
///
///         // Values outside the bounds are restricted to the region.
///         else {
///             assert!(!deadzone.within_bounds(value));
///
///             let expected = value.clamp_length_max(1.0);
///             let delta = (deadzone.normalize(value) - expected).abs();
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

    /// Normalizes input values into the live zone.
    #[must_use]
    pub fn normalize(&self, input_value: Vec2) -> Vec2 {
        let input_length = input_value.length();
        if input_length == 0.0 {
            return Vec2::ZERO;
        }

        // Clamp out-of-bounds values to a maximum magnitude of 1.0,
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
        CircleDeadZone::new(0.1)
    }
}

impl From<CircleDeadZone> for DualAxisProcessor {
    fn from(value: CircleDeadZone) -> Self {
        Self::CircleDeadZone(value)
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
    use bevy::prelude::FloatExt;

    #[test]
    fn test_circle_value_bounds() {
        fn test_bounds(bounds: CircleBounds, radius: f32) {
            assert_eq!(bounds.radius(), radius);

            let processor = DualAxisProcessor::CircleBounds(bounds);
            assert_eq!(DualAxisProcessor::from(bounds), processor);

            for x in -300..300 {
                let x = x as f32 * 0.01;
                for y in -300..300 {
                    let y = y as f32 * 0.01;
                    let value = Vec2::new(x, y);

                    assert_eq!(processor.process(value), bounds.clamp(value));

                    if value.length() <= radius {
                        assert!(bounds.contains(value));
                    } else {
                        assert!(!bounds.contains(value));
                    }

                    let expected = value.clamp_length_max(radius);
                    let delta = (bounds.clamp(value) - expected).abs();
                    assert!(delta.x <= f32::EPSILON);
                    assert!(delta.y <= f32::EPSILON);
                }
            }
        }

        let bounds = CircleBounds::FULL_RANGE;
        test_bounds(bounds, f32::MAX);

        let bounds = CircleBounds::default();
        test_bounds(bounds, 1.0);

        let bounds = CircleBounds::new(2.0);
        test_bounds(bounds, 2.0);
    }

    #[test]
    fn test_circle_exclusion() {
        fn test_exclusion(exclusion: CircleExclusion, radius: f32) {
            assert_eq!(exclusion.radius(), radius);

            let processor = DualAxisProcessor::CircleExclusion(exclusion);
            assert_eq!(DualAxisProcessor::from(exclusion), processor);

            for x in -300..300 {
                let x = x as f32 * 0.01;
                for y in -300..300 {
                    let y = y as f32 * 0.01;
                    let value = Vec2::new(x, y);

                    assert_eq!(processor.process(value), exclusion.exclude(value));

                    if value.length() <= radius {
                        assert!(exclusion.contains(value));
                        assert_eq!(exclusion.exclude(value), Vec2::ZERO);
                    } else {
                        assert!(!exclusion.contains(value));
                        assert_eq!(exclusion.exclude(value), value);
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

            let processor = DualAxisProcessor::CircleDeadZone(deadzone);
            assert_eq!(DualAxisProcessor::from(deadzone), processor);

            for x in -300..300 {
                let x = x as f32 * 0.01;
                for y in -300..300 {
                    let y = y as f32 * 0.01;
                    let value = Vec2::new(x, y);

                    assert_eq!(processor.process(value), deadzone.normalize(value));

                    // Values within the dead zone are treated as zeros.
                    if value.length() <= radius {
                        assert!(deadzone.within_exclusion(value));
                        assert_eq!(deadzone.normalize(value), Vec2::ZERO);
                    }
                    // Values within the live zone are scaled linearly.
                    else if value.length() <= 1.0 {
                        assert!(deadzone.within_livezone(value));

                        let expected_scale = f32::inverse_lerp(radius, 1.0, value.length());
                        let expected = value.normalize() * expected_scale;
                        let delta = (deadzone.normalize(value) - expected).abs();
                        assert!(delta.x <= 0.00001);
                        assert!(delta.y <= 0.00001);
                    }
                    // Values outside the bounds are restricted to the region.
                    else {
                        assert!(!deadzone.within_bounds(value));

                        let expected = value.clamp_length_max(1.0);
                        let delta = (deadzone.normalize(value) - expected).abs();
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
