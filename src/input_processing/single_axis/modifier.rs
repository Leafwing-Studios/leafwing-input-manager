use std::hash::{Hash, Hasher};

use bevy::prelude::Reflect;
use bevy::utils::FloatOrd;
use leafwing_input_manager_macros::serde_typetag;
use serde::{Deserialize, Serialize};

use crate::input_processing::AxisProcessor;

/// Flips the sign of single-axis input values, resulting in a directional reversal of control.
///
/// ```rust
/// use leafwing_input_manager::prelude::*;
///
/// assert_eq!(AxisInverted.process(2.5), -2.5);
/// assert_eq!(AxisInverted.process(-2.5), 2.5);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct AxisInverted;

#[serde_typetag]
impl AxisProcessor for AxisInverted {
    /// Returns the opposite value of the `input_value`.
    #[must_use]
    #[inline]
    fn process(&self, input_value: f32) -> f32 {
        -input_value
    }
}

/// Scales input values on an axis using a specified multiplier,
/// allowing fine-tuning the responsiveness of controls.
///
/// ```rust
/// use leafwing_input_manager::prelude::*;
///
/// // Doubled!
/// assert_eq!(AxisSensitivity(2.0).process(2.0), 4.0);
///
/// // Halved!
/// assert_eq!(AxisSensitivity(0.5).process(2.0), 1.0);
///
/// // Negated and halved!
/// assert_eq!(AxisSensitivity(-0.5).process(2.0), -1.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct AxisSensitivity(pub f32);

#[serde_typetag]
impl AxisProcessor for AxisSensitivity {
    /// Multiples the `input_value` by the specified sensitivity factor.
    #[must_use]
    #[inline]
    fn process(&self, input_value: f32) -> f32 {
        self.0 * input_value
    }
}

impl Eq for AxisSensitivity {}

impl Hash for AxisSensitivity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        FloatOrd(self.0).hash(state);
    }
}

#[cfg(test)]
mod tests {
    use crate::input_processing::single_axis::*;

    #[test]
    fn test_axis_inverted() {
        for value in -300..300 {
            let value = value as f32 * 0.01;

            assert_eq!(AxisInverted.process(value), -value);
            assert_eq!(AxisInverted.process(-value), value);
        }
    }

    #[test]
    fn test_axis_sensitivity() {
        for value in -300..300 {
            let value = value as f32 * 0.01;

            for sensitivity in -300..300 {
                let sensitivity = sensitivity as f32 * 0.01;

                let scale = AxisSensitivity(sensitivity);
                assert_eq!(scale.process(value), sensitivity * value);
            }
        }
    }
}
