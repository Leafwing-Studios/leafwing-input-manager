//! Modifiers for dual-axis inputs

use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};

use bevy::prelude::{BVec2, Reflect, Vec2};
use bevy::utils::FloatOrd;
use serde::{Deserialize, Serialize};

use crate::input_processing::DualAxisProcessor;

/// Flips the sign of dual-axis input values, resulting in a directional reversal of control.
///
/// ```rust
/// use bevy::prelude::*;
/// use leafwing_input_manager::prelude::*;
///
/// let value = Vec2::new(1.5, 2.0);
/// let Vec2 { x, y } = value;
///
/// assert_eq!(DualAxisInverted::ALL.process(value), -value);
/// assert_eq!(DualAxisInverted::ALL.process(-value), value);
///
/// assert_eq!(DualAxisInverted::ONLY_X.process(value), Vec2::new(-x, y));
/// assert_eq!(DualAxisInverted::ONLY_X.process(-value), Vec2::new(x, -y));
///
/// assert_eq!(DualAxisInverted::ONLY_Y.process(value), Vec2::new(x, -y));
/// assert_eq!(DualAxisInverted::ONLY_Y.process(-value), Vec2::new(-x, y));
#[derive(Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct DualAxisInverted(Vec2);

#[typetag::serde]
impl DualAxisProcessor for DualAxisInverted {
    /// Multiples the `input_value` by the specified inversion vector.
    #[must_use]
    #[inline]
    fn process(&self, input_value: Vec2) -> Vec2 {
        self.0 * input_value
    }
}

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
        self.0.cmpne(Vec2::NEG_ONE)
    }
}

impl Eq for DualAxisInverted {}

impl Hash for DualAxisInverted {
    fn hash<H: Hasher>(&self, state: &mut H) {
        FloatOrd(self.0.x).hash(state);
        FloatOrd(self.0.y).hash(state);
    }
}

impl Debug for DualAxisInverted {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "DualAxisInverted({:?})", self.inverted())
    }
}

/// Scales dual-axis input values using a specified multiplier,
/// allowing fine-tuning the responsiveness of controls.
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
/// assert_eq!(neg_x_half_y.process(value).x, -x);
/// assert_eq!(neg_x_half_y.process(value).y, 0.5 * y);
///
/// // Doubled X and doubled Y
/// let double = DualAxisSensitivity::all(2.0);
/// assert_eq!(double.process(value), 2.0 * value);
///
/// // Halved X
/// let half_x = DualAxisSensitivity::only_x(0.5);
/// assert_eq!(half_x.process(value).x, 0.5 * x);
/// assert_eq!(half_x.process(value).y, y);
///
/// // Negated and doubled Y
/// let neg_double_y = DualAxisSensitivity::only_y(-2.0);
/// assert_eq!(neg_double_y.process(value).x, x);
/// assert_eq!(neg_double_y.process(value).y, -2.0 * y);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct DualAxisSensitivity(pub(crate) Vec2);

#[typetag::serde]
impl DualAxisProcessor for DualAxisSensitivity {
    /// Multiples the `input_value` by the specified sensitivity vector.
    #[must_use]
    #[inline]
    fn process(&self, input_value: Vec2) -> Vec2 {
        self.0 * input_value
    }
}

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
    use crate::input_processing::dual_axis::*;

    #[test]
    fn test_dual_axis_inverted() {
        for x in -300..300 {
            let x = x as f32 * 0.01;

            for y in -300..300 {
                let y = y as f32 * 0.01;
                let value = Vec2::new(x, y);

                assert_eq!(DualAxisInverted::ALL.process(value), -value);
                assert_eq!(DualAxisInverted::ALL.process(-value), value);

                assert_eq!(DualAxisInverted::ONLY_X.process(value), Vec2::new(-x, y));
                assert_eq!(DualAxisInverted::ONLY_X.process(-value), Vec2::new(x, -y));

                assert_eq!(DualAxisInverted::ONLY_Y.process(value), Vec2::new(x, -y));
                assert_eq!(DualAxisInverted::ONLY_Y.process(-value), Vec2::new(-x, y));
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
                assert_eq!(all.sensitivities(), Vec2::splat(sensitivity));
                assert_eq!(all.process(value), sensitivity * value);

                let only_x = DualAxisSensitivity::only_x(sensitivity);
                assert_eq!(only_x.sensitivities(), Vec2::new(sensitivity, 1.0));
                assert_eq!(only_x.process(value).x, x * sensitivity);
                assert_eq!(only_x.process(value).y, y);

                let only_y = DualAxisSensitivity::only_y(sensitivity);
                assert_eq!(only_y.sensitivities(), Vec2::new(1.0, sensitivity));
                assert_eq!(only_y.process(value).x, x);
                assert_eq!(only_y.process(value).y, y * sensitivity);

                let sensitivity2 = y;
                let separate = DualAxisSensitivity::new(sensitivity, sensitivity2);
                assert_eq!(
                    separate.sensitivities(),
                    Vec2::new(sensitivity, sensitivity2)
                );
                assert_eq!(separate.process(value).x, x * sensitivity);
                assert_eq!(separate.process(value).y, y * sensitivity2);
            }
        }
    }
}
