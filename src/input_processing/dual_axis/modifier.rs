//! Modifiers for dual-axis inputs

use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use bevy::prelude::{BVec2, Reflect, Vec2};
use bevy::utils::FloatOrd;
use serde::{Deserialize, Serialize};

use super::DualAxisProcessor;

/// Flips the sign of dual-axis input values, resulting in a directional reversal of control.
///
/// ```rust
/// use bevy::prelude::*;
/// use leafwing_input_manager::prelude::*;
///
/// let value = Vec2::new(1.5, 2.0);
/// let Vec2 { x, y } = value;
///
/// assert_eq!(DualAxisInverted::ALL.invert(value), -value);
/// assert_eq!(DualAxisInverted::ALL.invert(-value), value);
///
/// assert_eq!(DualAxisInverted::ONLY_X.invert(value), Vec2::new(-x, y));
/// assert_eq!(DualAxisInverted::ONLY_X.invert(-value), Vec2::new(x, -y));
///
/// assert_eq!(DualAxisInverted::ONLY_Y.invert(value), Vec2::new(x, -y));
/// assert_eq!(DualAxisInverted::ONLY_Y.invert(-value), Vec2::new(-x, y));
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct DualAxisInverted(Vec2);

impl DualAxisInverted {
    /// The [`DualAxisInverted`] that inverts both axes.
    pub const ALL: Self = Self(Vec2::NEG_ONE);

    /// The [`DualAxisInverted`] that only inverts the X-axis inputs.
    pub const ONLY_X: Self = Self(Vec2::new(-1.0, 1.0));

    /// The [`DualAxisInverted`] that only inverts the Y-axis inputs.
    pub const ONLY_Y: Self = Self(Vec2::new(1.0, -1.0));

    /// Are inputs inverted on both axes?
    #[must_use]
    #[inline]
    pub fn inverted(&self) -> BVec2 {
        self.0.cmpeq(Vec2::NEG_ONE)
    }

    /// Multiples the `input_value` by the specified inversion vector.
    #[must_use]
    #[inline]
    pub fn invert(&self, input_value: Vec2) -> Vec2 {
        self.0 * input_value
    }
}

impl From<DualAxisInverted> for DualAxisProcessor {
    fn from(value: DualAxisInverted) -> Self {
        Self::Inverted(value)
    }
}

impl Eq for DualAxisInverted {}

impl Hash for DualAxisInverted {
    fn hash<H: Hasher>(&self, state: &mut H) {
        FloatOrd(self.0.x).hash(state);
        FloatOrd(self.0.y).hash(state);
    }
}

/// Scales dual-axis input values using a specified multiplier to fine-tune the responsiveness of control.
///
/// # Examples
///
/// ```rust
/// use bevy::prelude::*;
/// use leafwing_input_manager::prelude::*;
///
/// let value = Vec2::new(1.5, 2.5);
/// let Vec2 { x, y } = value;
///
/// // Negated X and halved Y
/// let neg_x_half_y = DualAxisSensitivity::new(-1.0, 0.5);
/// assert_eq!(neg_x_half_y.scale(value).x, -x);
/// assert_eq!(neg_x_half_y.scale(value).y, 0.5 * y);
///
/// // Doubled X and doubled Y
/// let double = DualAxisSensitivity::all(2.0);
/// assert_eq!(double.scale(value), 2.0 * value);
///
/// // Halved X
/// let half_x = DualAxisSensitivity::only_x(0.5);
/// assert_eq!(half_x.scale(value).x, 0.5 * x);
/// assert_eq!(half_x.scale(value).y, y);
///
/// // Negated and doubled Y
/// let neg_double_y = DualAxisSensitivity::only_y(-2.0);
/// assert_eq!(neg_double_y.scale(value).x, x);
/// assert_eq!(neg_double_y.scale(value).y, -2.0 * y);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct DualAxisSensitivity(pub(crate) Vec2);

impl DualAxisSensitivity {
    /// Creates a [`DualAxisSensitivity`] with the given values for each axis separately.
    #[inline]
    pub const fn new(sensitivity_x: f32, sensitivity_y: f32) -> Self {
        Self(Vec2::new(sensitivity_x, sensitivity_y))
    }

    /// Creates a [`DualAxisSensitivity`] with the same value for both axes.
    #[inline]
    pub const fn all(sensitivity: f32) -> Self {
        Self::new(sensitivity, sensitivity)
    }

    /// Creates a [`DualAxisSensitivity`] that only affects the X-axis using the given value.
    #[inline]
    pub const fn only_x(sensitivity: f32) -> Self {
        Self::new(sensitivity, 1.0)
    }

    /// Creates a [`DualAxisSensitivity`] that only affects the Y-axis using the given value.
    #[inline]
    pub const fn only_y(sensitivity: f32) -> Self {
        Self::new(1.0, sensitivity)
    }

    /// Returns the sensitivity values.
    #[must_use]
    #[inline]
    pub fn sensitivities(&self) -> Vec2 {
        self.0
    }

    /// Multiples the `input_value` by the specified sensitivity vector.
    #[must_use]
    #[inline]
    pub fn scale(&self, input_value: Vec2) -> Vec2 {
        self.0 * input_value
    }
}

impl From<DualAxisSensitivity> for DualAxisProcessor {
    fn from(value: DualAxisSensitivity) -> Self {
        Self::Sensitivity(value)
    }
}

impl Eq for DualAxisSensitivity {}

impl Hash for DualAxisSensitivity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        FloatOrd(self.0.x).hash(state);
        FloatOrd(self.0.y).hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dual_axis_inverted() {
        let all = DualAxisInverted::ALL;
        assert_eq!(all.inverted(), BVec2::TRUE);

        let only_x = DualAxisInverted::ONLY_X;
        assert_eq!(only_x.inverted(), BVec2::new(true, false));

        let only_y = DualAxisInverted::ONLY_Y;
        assert_eq!(only_y.inverted(), BVec2::new(false, true));

        for x in -300..300 {
            let x = x as f32 * 0.01;

            for y in -300..300 {
                let y = y as f32 * 0.01;
                let value = Vec2::new(x, y);

                let processor = DualAxisProcessor::Inverted(all);
                assert_eq!(DualAxisProcessor::from(all), processor);
                assert_eq!(processor.process(value), all.invert(value));
                assert_eq!(all.invert(value), -value);
                assert_eq!(all.invert(-value), value);

                let processor = DualAxisProcessor::Inverted(only_x);
                assert_eq!(DualAxisProcessor::from(only_x), processor);
                assert_eq!(processor.process(value), only_x.invert(value));
                assert_eq!(only_x.invert(value), Vec2::new(-x, y));
                assert_eq!(only_x.invert(-value), Vec2::new(x, -y));

                let processor = DualAxisProcessor::Inverted(only_y);
                assert_eq!(DualAxisProcessor::from(only_y), processor);
                assert_eq!(processor.process(value), only_y.invert(value));
                assert_eq!(only_y.invert(value), Vec2::new(x, -y));
                assert_eq!(only_y.invert(-value), Vec2::new(-x, y));
            }
        }
    }

    #[test]
    fn test_dual_axis_sensitivity() {
        for x in -300..300 {
            let x = x as f32 * 0.01;

            for y in -300..300 {
                let y = y as f32 * 0.01;
                let value = Vec2::new(x, y);

                let sensitivity = x;

                let all = DualAxisSensitivity::all(sensitivity);
                let processor = DualAxisProcessor::Sensitivity(all);
                assert_eq!(DualAxisProcessor::from(all), processor);
                assert_eq!(processor.process(value), all.scale(value));
                assert_eq!(all.sensitivities(), Vec2::splat(sensitivity));
                assert_eq!(all.scale(value), sensitivity * value);

                let only_x = DualAxisSensitivity::only_x(sensitivity);
                let processor = DualAxisProcessor::Sensitivity(only_x);
                assert_eq!(DualAxisProcessor::from(only_x), processor);
                assert_eq!(processor.process(value), only_x.scale(value));
                assert_eq!(only_x.sensitivities(), Vec2::new(sensitivity, 1.0));
                assert_eq!(only_x.scale(value).x, x * sensitivity);
                assert_eq!(only_x.scale(value).y, y);

                let only_y = DualAxisSensitivity::only_y(sensitivity);
                let processor = DualAxisProcessor::Sensitivity(only_y);
                assert_eq!(DualAxisProcessor::from(only_y), processor);
                assert_eq!(processor.process(value), only_y.scale(value));
                assert_eq!(only_y.sensitivities(), Vec2::new(1.0, sensitivity));
                assert_eq!(only_y.scale(value).x, x);
                assert_eq!(only_y.scale(value).y, y * sensitivity);

                let sensitivity2 = y;
                let separate = DualAxisSensitivity::new(sensitivity, sensitivity2);
                let processor = DualAxisProcessor::Sensitivity(separate);
                assert_eq!(DualAxisProcessor::from(separate), processor);
                assert_eq!(processor.process(value), separate.scale(value));
                assert_eq!(
                    separate.sensitivities(),
                    Vec2::new(sensitivity, sensitivity2)
                );
                assert_eq!(separate.scale(value).x, x * sensitivity);
                assert_eq!(separate.scale(value).y, y * sensitivity2);
            }
        }
    }
}
