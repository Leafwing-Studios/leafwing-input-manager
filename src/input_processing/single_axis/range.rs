//! Range processors for single-axis inputs

use std::hash::{Hash, Hasher};

use bevy::{math::FloatOrd, prelude::Reflect};
use serde::{Deserialize, Serialize};

use super::AxisProcessor;

/// Specifies an acceptable min-max range for valid single-axis inputs,
/// restricting all value stays within intended limits
/// to avoid unexpected behavior caused by extreme inputs.
///
/// ```rust
/// use leafwing_input_manager::prelude::*;
///
/// // Restrict values to [-2.0, 1.5].
/// let bounds = AxisBounds::new(-2.0, 1.5);
///
/// // The ways to create an AxisProcessor.
/// let processor = AxisProcessor::from(bounds);
/// assert_eq!(processor, AxisProcessor::ValueBounds(bounds));
///
/// for value in -300..300 {
///     let value = value as f32 * 0.01;
///     assert_eq!(bounds.clamp(value), value.clamp(-2.0, 1.5));
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct AxisBounds {
    /// The minimum value of valid inputs.
    pub(crate) min: f32,

    /// The maximum value of valid inputs.
    pub(crate) max: f32,
}

impl AxisBounds {
    /// Unlimited [`AxisBounds`].
    pub const FULL_RANGE: Self = Self {
        min: f32::MIN,
        max: f32::MAX,
    };

    /// Creates an [`AxisBounds`] that restricts values to the given range `[min, max]`.
    ///
    /// # Requirements
    ///
    /// - `min` <= `max`.
    ///
    /// # Panics
    ///
    /// Panics if the requirements aren't met.
    #[inline]
    pub fn new(min: f32, max: f32) -> Self {
        // PartialOrd for f32 ensures that NaN values are checked during comparisons.
        assert!(min <= max);
        Self { min, max }
    }

    /// Creates an [`AxisBounds`] that restricts values within the range `[-threshold, threshold]`.
    ///
    /// # Requirements
    ///
    /// - `threshold` >= `0.0`.
    ///
    /// # Panics
    ///
    /// Panics if the requirements aren't met.
    #[doc(alias = "magnitude")]
    #[inline]
    pub fn symmetric(threshold: f32) -> Self {
        Self::new(-threshold, threshold)
    }

    /// Creates an [`AxisBounds`] that restricts values to a minimum value.
    #[inline]
    pub const fn at_least(min: f32) -> Self {
        Self {
            min,
            ..Self::FULL_RANGE
        }
    }

    /// Creates an [`AxisBounds`] that restricts values to a maximum value.
    #[inline]
    pub const fn at_most(max: f32) -> Self {
        Self {
            max,
            ..Self::FULL_RANGE
        }
    }

    /// Returns the minimum and maximum bounds.
    #[must_use]
    #[inline]
    pub fn min_max(&self) -> (f32, f32) {
        (self.min(), self.max())
    }

    /// Returns the minimum bound.
    #[must_use]
    #[inline]
    pub fn min(&self) -> f32 {
        self.min
    }

    /// Returns the maximum bound.
    #[must_use]
    #[inline]
    pub fn max(&self) -> f32 {
        self.max
    }

    /// Is the given `input_value` within the bounds?
    #[must_use]
    #[inline]
    pub fn contains(&self, input_value: f32) -> bool {
        self.min <= input_value && input_value <= self.max
    }

    /// Clamps `input_value` within the bounds.
    #[must_use]
    #[inline]
    pub fn clamp(&self, input_value: f32) -> f32 {
        // clamp() includes checks if either bound is set to NaN,
        // but the constructors guarantee that all bounds will not be NaN.
        input_value.min(self.max).max(self.min)
    }
}

impl Default for AxisBounds {
    /// Creates an [`AxisBounds`] that restricts values to the range `[-1.0, 1.0]`.
    #[inline]
    fn default() -> Self {
        Self {
            min: -1.0,
            max: 1.0,
        }
    }
}

impl From<AxisBounds> for AxisProcessor {
    fn from(value: AxisBounds) -> Self {
        Self::ValueBounds(value)
    }
}

impl Eq for AxisBounds {}

impl Hash for AxisBounds {
    fn hash<H: Hasher>(&self, state: &mut H) {
        FloatOrd(self.min).hash(state);
        FloatOrd(self.max).hash(state);
    }
}

/// Specifies an exclusion range for excluding single-axis inputs,
/// helping filter out minor fluctuations and unintended movements.
///
/// In simple terms, this processor behaves like an [`AxisDeadZone`] without normalization.
///
/// # Examples
///
/// ```rust
/// use leafwing_input_manager::prelude::*;
///
/// // Exclude values between -0.2 and 0.3
/// let exclusion = AxisExclusion::new(-0.2, 0.3);
///
/// // The ways to create an AxisProcessor.
/// let processor = AxisProcessor::from(exclusion);
/// assert_eq!(processor, AxisProcessor::Exclusion(exclusion));
///
/// for value in -300..300 {
///     let value = value as f32 * 0.01;
///
///     if -0.2 <= value && value <= 0.3 {
///         assert!(exclusion.contains(value));
///         assert_eq!(exclusion.exclude(value), 0.0);
///     } else {
///         assert!(!exclusion.contains(value));
///         assert_eq!(exclusion.exclude(value), value);
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct AxisExclusion {
    /// The maximum negative value treated as zero.
    pub(crate) negative_max: f32,

    /// The minimum positive value treated as zero.
    pub(crate) positive_min: f32,
}

impl AxisExclusion {
    /// Zero-size [`AxisExclusion`], leaving values as is.
    pub const ZERO: Self = Self {
        negative_max: 0.0,
        positive_min: 0.0,
    };

    /// Creates an [`AxisExclusion`] that ignores values within the range `[negative_max, positive_min]`.
    ///
    /// # Requirements
    ///
    /// - `negative_max` <= `0.0` <= `positive_min`.
    ///
    /// # Panics
    ///
    /// Panics if the requirements aren't met.
    #[inline]
    pub fn new(negative_max: f32, positive_min: f32) -> Self {
        assert!(negative_max <= 0.0);
        assert!(positive_min >= 0.0);
        Self {
            negative_max,
            positive_min,
        }
    }

    /// Creates an [`AxisExclusion`] that ignores values within the range `[-threshold, threshold]`.
    ///
    /// # Requirements
    ///
    /// - `threshold` >= `0.0`.
    ///
    /// # Panics
    ///
    /// Panics if the requirements aren't met.
    #[doc(alias = "magnitude")]
    #[inline]
    pub fn symmetric(threshold: f32) -> Self {
        Self::new(-threshold, threshold)
    }

    /// Returns the minimum and maximum bounds.
    #[must_use]
    #[inline]
    pub fn min_max(&self) -> (f32, f32) {
        (self.negative_max, self.positive_min)
    }

    /// Returns the minimum bound.
    #[must_use]
    #[inline]
    pub fn min(&self) -> f32 {
        self.negative_max
    }

    /// Returns the maximum bounds.
    #[must_use]
    #[inline]
    pub fn max(&self) -> f32 {
        self.positive_min
    }

    /// Is `input_value` within the deadzone?
    #[must_use]
    #[inline]
    pub fn contains(&self, input_value: f32) -> bool {
        self.negative_max <= input_value && input_value <= self.positive_min
    }

    /// Excludes values within the specified range.
    #[must_use]
    #[inline]
    pub fn exclude(&self, input_value: f32) -> f32 {
        if self.contains(input_value) {
            0.0
        } else {
            input_value
        }
    }

    /// Creates an [`AxisDeadZone`] using `self` as the exclusion range.
    #[inline]
    pub fn scaled(self) -> AxisDeadZone {
        AxisDeadZone::new(self.negative_max, self.positive_min)
    }
}

impl Default for AxisExclusion {
    /// Creates an [`AxisExclusion`] that ignores values within the range `[-0.1, 0.1]`.
    #[inline]
    fn default() -> Self {
        Self {
            negative_max: -0.1,
            positive_min: 0.1,
        }
    }
}

impl From<AxisExclusion> for AxisProcessor {
    fn from(value: AxisExclusion) -> Self {
        Self::Exclusion(value)
    }
}

impl Eq for AxisExclusion {}

impl Hash for AxisExclusion {
    fn hash<H: Hasher>(&self, state: &mut H) {
        FloatOrd(self.negative_max).hash(state);
        FloatOrd(self.positive_min).hash(state);
    }
}

/// A scaled version of [`AxisExclusion`] with the bounds
/// set to [`AxisBounds::magnitude(1.0)`](AxisBounds::default)
/// that normalizes non-excluded input values into the "live zone",
/// the remaining range within the bounds after dead zone exclusion.
///
/// # Examples
///
/// ```rust
/// use bevy::prelude::*;
/// use leafwing_input_manager::prelude::*;
///
/// // Exclude values between -0.2 and 0.3
/// let deadzone = AxisDeadZone::new(-0.2, 0.3);
///
/// // Another way to create an AxisDeadzone.
/// let exclusion = AxisExclusion::new(-0.2, 0.3);
/// assert_eq!(exclusion.scaled(), deadzone);
///
/// // The ways to create an AxisProcessor.
/// let processor = AxisProcessor::from(deadzone);
/// assert_eq!(processor, AxisProcessor::DeadZone(deadzone));
///
/// // The bounds after normalization.
/// let bounds = deadzone.bounds();
/// assert_eq!(bounds.min(), -1.0);
/// assert_eq!(bounds.max(), 1.0);
///
/// for value in -300..300 {
///     let value = value as f32 * 0.01;
///
///     // Values within the dead zone are treated as zero.
///     if -0.2 <= value && value <= 0.3 {
///         assert!(deadzone.within_exclusion(value));
///         assert_eq!(deadzone.normalize(value), 0.0);
///     }
///
///     // Values within the live zone are scaled linearly.
///     else if -1.0 <= value && value < -0.2 {
///         assert!(deadzone.within_livezone_lower(value));
///
///         let expected = f32::inverse_lerp(-1.0, -0.2, value) - 1.0;
///         assert!((deadzone.normalize(value) - expected).abs() <= f32::EPSILON);
///     } else if 0.3 < value && value <= 1.0 {
///         assert!(deadzone.within_livezone_upper(value));
///
///         let expected = f32::inverse_lerp(0.3, 1.0, value);
///         assert!((deadzone.normalize(value) - expected).abs() <= f32::EPSILON);
///     }
///
///     // Values outside the bounds are restricted to the range.
///     else {
///         assert!(!deadzone.within_bounds(value));
///         assert_eq!(deadzone.normalize(value), value.clamp(-1.0, 1.0));
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct AxisDeadZone {
    /// The [`AxisExclusion`] used for normalization.
    pub(crate) exclusion: AxisExclusion,

    /// Pre-calculated reciprocal of the lower live zone size,
    /// preventing division during normalization.
    pub(crate) livezone_lower_recip: f32,

    /// Pre-calculated reciprocal of the upper live zone size,
    /// preventing division during normalization.
    pub(crate) livezone_upper_recip: f32,
}

impl AxisDeadZone {
    /// Zero-size [`AxisDeadZone`], only restricting values to the range `[-1.0, 1.0]`.
    pub const ZERO: Self = Self {
        exclusion: AxisExclusion::ZERO,
        livezone_lower_recip: 1.0,
        livezone_upper_recip: 1.0,
    };

    /// Creates an [`AxisDeadZone`] that excludes input values
    /// within the given deadzone `[negative_max, positive_min]`.
    ///
    /// # Requirements
    ///
    /// - `negative_max` <= `0.0` <= `positive_min`.
    ///
    /// # Panics
    ///
    /// Panics if the requirements aren't met.
    #[inline]
    pub fn new(negative_max: f32, positive_min: f32) -> Self {
        let (bound_min, bound_max) = AxisBounds::default().min_max();
        Self {
            exclusion: AxisExclusion::new(negative_max, positive_min),
            livezone_lower_recip: (negative_max - bound_min).recip(),
            livezone_upper_recip: (bound_max - positive_min).recip(),
        }
    }

    /// Creates an [`AxisDeadZone`] that excludes input values below a `threshold` magnitude
    /// and then normalizes non-excluded input values into the valid range `[-1.0, 1.0]`.
    ///
    /// # Requirements
    ///
    /// - `threshold` >= `0.0`.
    ///
    /// # Panics
    ///
    /// Panics if the requirements aren't met.
    #[doc(alias = "magnitude")]
    #[inline]
    pub fn symmetric(threshold: f32) -> Self {
        AxisDeadZone::new(-threshold, threshold)
    }

    /// Returns the [`AxisExclusion`] used by this deadzone.
    #[inline]
    pub fn exclusion(&self) -> AxisExclusion {
        self.exclusion
    }

    /// Returns the [`AxisBounds`] used by this deadzone.
    #[inline]
    pub fn bounds(&self) -> AxisBounds {
        AxisBounds::default()
    }

    /// Returns the minimum and maximum bounds of the lower live zone used for normalization.
    ///
    /// In simple terms, this returns `(bounds.min, exclusion.min)`.
    #[must_use]
    #[inline]
    pub fn livezone_lower_min_max(&self) -> (f32, f32) {
        (self.bounds().min(), self.exclusion.min())
    }

    /// Returns the minimum and maximum bounds of the upper live zone used for normalization.
    ///
    /// In simple terms, this returns `(exclusion.max, bounds.max)`.
    #[must_use]
    #[inline]
    pub fn livezone_upper_min_max(&self) -> (f32, f32) {
        (self.exclusion.max(), self.bounds().max())
    }

    /// Is the given `input_value` within the exclusion range?
    #[must_use]
    #[inline]
    pub fn within_exclusion(&self, input_value: f32) -> bool {
        self.exclusion.contains(input_value)
    }

    /// Is the given `input_value` within the bounds?
    #[must_use]
    #[inline]
    pub fn within_bounds(&self, input_value: f32) -> bool {
        self.bounds().contains(input_value)
    }

    /// Is the given `input_value` within the lower live zone?
    #[must_use]
    #[inline]
    pub fn within_livezone_lower(&self, input_value: f32) -> bool {
        let (min, max) = self.livezone_lower_min_max();
        min <= input_value && input_value <= max
    }

    /// Is the given `input_value` within the upper live zone?
    #[must_use]
    #[inline]
    pub fn within_livezone_upper(&self, input_value: f32) -> bool {
        let (min, max) = self.livezone_upper_min_max();
        min <= input_value && input_value <= max
    }

    /// Normalizes input values into the live zone.
    #[must_use]
    pub fn normalize(&self, input_value: f32) -> f32 {
        // Clamp out-of-bounds values to [-1, 1],
        // and then exclude values within the dead zone,
        // and finally linearly scale the result to the live zone.
        if input_value <= 0.0 {
            let (bound, deadzone) = self.livezone_lower_min_max();
            let clamped_input = input_value.max(bound);
            let distance_to_deadzone = (clamped_input - deadzone).min(0.0);
            distance_to_deadzone * self.livezone_lower_recip
        } else {
            let (deadzone, bound) = self.livezone_upper_min_max();
            let clamped_input = input_value.min(bound);
            let distance_to_deadzone = (clamped_input - deadzone).max(0.0);
            distance_to_deadzone * self.livezone_upper_recip
        }
    }
}

impl Default for AxisDeadZone {
    /// Creates an [`AxisDeadZone`] that excludes input values within the deadzone `[-0.1, 0.1]`.
    #[inline]
    fn default() -> Self {
        AxisDeadZone::new(-0.1, 0.1)
    }
}

impl From<AxisDeadZone> for AxisProcessor {
    fn from(value: AxisDeadZone) -> Self {
        Self::DeadZone(value)
    }
}

impl Eq for AxisDeadZone {}

impl Hash for AxisDeadZone {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.exclusion.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::FloatExt;

    #[test]
    fn test_axis_value_bounds() {
        fn test_bounds(bounds: AxisBounds, min: f32, max: f32) {
            assert_eq!(bounds.min(), min);
            assert_eq!(bounds.max(), max);
            assert_eq!(bounds.min_max(), (min, max));

            let processor = AxisProcessor::ValueBounds(bounds);
            assert_eq!(AxisProcessor::from(bounds), processor);

            for value in -300..300 {
                let value = value as f32 * 0.01;

                assert_eq!(bounds.clamp(value), processor.process(value));

                if min <= value && value <= max {
                    assert!(bounds.contains(value));
                } else {
                    assert!(!bounds.contains(value));
                }

                assert_eq!(bounds.clamp(value), value.clamp(min, max));
            }
        }

        let bounds = AxisBounds::FULL_RANGE;
        test_bounds(bounds, f32::MIN, f32::MAX);

        let bounds = AxisBounds::default();
        test_bounds(bounds, -1.0, 1.0);

        let bounds = AxisBounds::new(-2.0, 2.5);
        test_bounds(bounds, -2.0, 2.5);

        let bounds = AxisBounds::symmetric(2.0);
        test_bounds(bounds, -2.0, 2.0);

        let bounds = AxisBounds::at_least(-1.0);
        test_bounds(bounds, -1.0, f32::MAX);

        let bounds = AxisBounds::at_most(1.5);
        test_bounds(bounds, f32::MIN, 1.5);
    }

    #[test]
    fn test_axis_exclusion() {
        fn test_exclusion(exclusion: AxisExclusion, min: f32, max: f32) {
            assert_eq!(exclusion.min(), min);
            assert_eq!(exclusion.max(), max);
            assert_eq!(exclusion.min_max(), (min, max));

            let processor = AxisProcessor::Exclusion(exclusion);
            assert_eq!(AxisProcessor::from(exclusion), processor);

            for value in -300..300 {
                let value = value as f32 * 0.01;

                assert_eq!(exclusion.exclude(value), processor.process(value));

                if min <= value && value <= max {
                    assert!(exclusion.contains(value));
                    assert_eq!(exclusion.exclude(value), 0.0);
                } else {
                    assert!(!exclusion.contains(value));
                    assert_eq!(exclusion.exclude(value), value);
                }
            }
        }

        let exclusion = AxisExclusion::ZERO;
        test_exclusion(exclusion, 0.0, 0.0);

        let exclusion = AxisExclusion::default();
        test_exclusion(exclusion, -0.1, 0.1);

        let exclusion = AxisExclusion::new(-2.0, 2.5);
        test_exclusion(exclusion, -2.0, 2.5);

        let exclusion = AxisExclusion::symmetric(1.5);
        test_exclusion(exclusion, -1.5, 1.5);
    }

    #[test]
    fn test_axis_deadzone() {
        fn test_deadzone(deadzone: AxisDeadZone, min: f32, max: f32) {
            let exclusion = deadzone.exclusion();
            assert_eq!(exclusion.min_max(), (min, max));

            assert_eq!(deadzone.livezone_lower_min_max(), (-1.0, min));
            let width_recip = (min + 1.0).recip();
            assert!((deadzone.livezone_lower_recip - width_recip).abs() <= f32::EPSILON);

            assert_eq!(deadzone.livezone_upper_min_max(), (max, 1.0));
            let width_recip = (1.0 - max).recip();
            assert!((deadzone.livezone_upper_recip - width_recip).abs() <= f32::EPSILON);

            assert_eq!(AxisExclusion::new(min, max).scaled(), deadzone);

            let processor = AxisProcessor::DeadZone(deadzone);
            assert_eq!(AxisProcessor::from(deadzone), processor);

            for value in -300..300 {
                let value = value as f32 * 0.01;

                assert_eq!(deadzone.normalize(value), processor.process(value));

                // Values within the dead zone are treated as zero.
                if min <= value && value <= max {
                    assert!(deadzone.within_exclusion(value));
                    assert_eq!(deadzone.normalize(value), 0.0);
                }
                // Values within the live zone are scaled linearly.
                else if -1.0 <= value && value < min {
                    assert!(deadzone.within_livezone_lower(value));

                    let expected = f32::inverse_lerp(-1.0, min, value) - 1.0;
                    let delta = (deadzone.normalize(value) - expected).abs();
                    assert!(delta <= f32::EPSILON);
                } else if max < value && value <= 1.0 {
                    assert!(deadzone.within_livezone_upper(value));

                    let expected = f32::inverse_lerp(max, 1.0, value);
                    let delta = (deadzone.normalize(value) - expected).abs();
                    assert!(delta <= f32::EPSILON);
                }
                // Values outside the bounds are restricted to the nearest valid value.
                else {
                    assert!(!deadzone.within_bounds(value));
                    assert_eq!(deadzone.normalize(value), value.clamp(-1.0, 1.0));
                }
            }
        }

        let deadzone = AxisDeadZone::ZERO;
        test_deadzone(deadzone, 0.0, 0.0);

        let deadzone = AxisDeadZone::default();
        test_deadzone(deadzone, -0.1, 0.1);

        let deadzone = AxisDeadZone::new(-0.2, 0.3);
        test_deadzone(deadzone, -0.2, 0.3);

        let deadzone = AxisDeadZone::symmetric(0.4);
        test_deadzone(deadzone, -0.4, 0.4);
    }
}
