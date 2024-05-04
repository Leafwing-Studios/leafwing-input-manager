//! Tools for working with directional axis-like user inputs (game sticks, D-Pads and emulated equivalents)

use bevy::prelude::{Direction2d, Reflect, Vec2};
use serde::{Deserialize, Serialize};

use crate::orientation::Rotation;

/// Different ways that user input is represented on an axis.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize, Reflect)]
#[must_use]
pub enum AxisInputMode {
    /// Continuous input values, typically range from a negative maximum (e.g., `-1.0`)
    /// to a positive maximum (e.g., `1.0`), allowing for smooth and precise control.
    #[default]
    Analog,

    /// Discrete input values, using three distinct values to represent the states:
    /// `-1.0` for active in negative direction, `0.0` for inactive, and `1.0` for active in positive direction.
    Digital,
}

impl AxisInputMode {
    /// Converts the given `f32` value based on the current [`AxisInputMode`].
    ///
    /// # Returns
    ///
    /// - [`AxisInputMode::Analog`]: Leaves values as is.
    /// - [`AxisInputMode::Digital`]: Maps negative values to `-1.0` and positive values to `1.0`, leaving others as is.
    #[must_use]
    #[inline]
    pub fn axis_value(&self, value: f32) -> f32 {
        match self {
            Self::Analog => value,
            Self::Digital => {
                if value < 0.0 {
                    -1.0
                } else if value > 0.0 {
                    1.0
                } else {
                    value
                }
            }
        }
    }

    /// Converts the given [`Vec2`] value based on the current [`AxisInputMode`].
    ///
    /// # Returns
    ///
    /// - [`AxisInputMode::Analog`]: Leaves values as is.
    /// - [`AxisInputMode::Digital`]: Maps negative values to `-1.0` and positive values to `1.0` along each axis, leaving others as is.
    #[must_use]
    #[inline]
    pub fn dual_axis_value(&self, value: Vec2) -> Vec2 {
        match self {
            Self::Analog => value,
            Self::Digital => Vec2::new(self.axis_value(value.x), self.axis_value(value.y)),
        }
    }

    /// Computes the magnitude of given [`Vec2`] value based on the current [`AxisInputMode`].
    ///
    /// # Returns
    ///
    /// - [`AxisInputMode::Analog`]: Leaves values as is.
    /// - [`AxisInputMode::Digital`]: `1.0` for non-zero values, `0.0` for others.
    #[must_use]
    #[inline]
    pub fn dual_axis_magnitude(&self, value: Vec2) -> f32 {
        match self {
            Self::Analog => value.length(),
            Self::Digital => f32::from(value != Vec2::ZERO),
        }
    }
}

/// The directions for single-axis inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub enum AxisDirection {
    /// Negative direction.
    Negative,

    /// Positive direction.
    Positive,
}

impl AxisDirection {
    /// Returns the full active value along an axis.
    #[must_use]
    #[inline]
    pub fn full_active_value(&self) -> f32 {
        match self {
            Self::Negative => -1.0,
            Self::Positive => 1.0,
        }
    }

    /// Checks if the given `value` represents an active input in this direction.
    #[must_use]
    #[inline]
    pub fn is_active(&self, value: f32) -> bool {
        match self {
            Self::Negative => value < 0.0,
            Self::Positive => value > 0.0,
        }
    }
}

/// An axis for dual-axis inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub enum DualAxisType {
    /// The X-axis (typically horizontal movement).
    X,

    /// The Y-axis (typically vertical movement).
    Y,
}

impl DualAxisType {
    /// Returns both X and Y axes.
    #[inline]
    pub const fn axes() -> [Self; 2] {
        [Self::X, Self::Y]
    }

    /// Returns the positive and negative [`DualAxisDirection`]s for the current axis.
    #[inline]
    pub const fn directions(&self) -> [DualAxisDirection; 2] {
        [self.negative(), self.positive()]
    }

    /// Returns the negative [`DualAxisDirection`] for the current axis.
    #[inline]
    pub const fn negative(&self) -> DualAxisDirection {
        match self {
            Self::X => DualAxisDirection::Left,
            Self::Y => DualAxisDirection::Down,
        }
    }

    /// Returns the positive [`DualAxisDirection`] for the current axis.
    #[inline]
    pub const fn positive(&self) -> DualAxisDirection {
        match self {
            Self::X => DualAxisDirection::Right,
            Self::Y => DualAxisDirection::Up,
        }
    }

    /// Returns the value along the current axis.
    #[must_use]
    #[inline]
    pub const fn get_value(&self, value: Vec2) -> f32 {
        match self {
            Self::X => value.x,
            Self::Y => value.y,
        }
    }

    /// Creates a [`Vec2`] with the specified `value` on this axis and `0.0` on the other.
    #[must_use]
    #[inline]
    pub const fn dual_axis_value(&self, value: f32) -> Vec2 {
        match self {
            Self::X => Vec2::new(value, 0.0),
            Self::Y => Vec2::new(0.0, value),
        }
    }
}

/// The directions for dual-axis inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub enum DualAxisDirection {
    /// Upward direction.
    Up,

    /// Downward direction.
    Down,

    /// Leftward direction.
    Left,

    /// Rightward direction.
    Right,
}

impl DualAxisDirection {
    /// Returns the [`DualAxisType`] associated with this direction.
    #[inline]
    pub fn axis(&self) -> DualAxisType {
        match self {
            Self::Up => DualAxisType::Y,
            Self::Down => DualAxisType::Y,
            Self::Left => DualAxisType::X,
            Self::Right => DualAxisType::X,
        }
    }

    /// Returns the [`AxisDirection`] (positive or negative) on the axis.
    #[inline]
    pub fn axis_direction(&self) -> AxisDirection {
        match self {
            Self::Up => AxisDirection::Positive,
            Self::Down => AxisDirection::Negative,
            Self::Left => AxisDirection::Negative,
            Self::Right => AxisDirection::Positive,
        }
    }

    /// Returns the full active value along both axes.
    #[must_use]
    #[inline]
    pub fn full_active_value(&self) -> Vec2 {
        match self {
            Self::Up => Vec2::Y,
            Self::Down => Vec2::NEG_Y,
            Self::Left => Vec2::NEG_X,
            Self::Right => Vec2::X,
        }
    }

    /// Checks if the given `value` represents an active input in this direction.
    #[must_use]
    #[inline]
    pub fn is_active(&self, value: Vec2) -> bool {
        let component_along_axis = self.axis().get_value(value);
        self.axis_direction().is_active(component_along_axis)
    }
}

/// A combined input data from two axes (X and Y).
///
/// This struct stores the X and Y values as a [`Vec2`] in a device-agnostic way,
/// meaning it works consistently regardless of the specific input device (gamepad, joystick, etc.).
/// It assumes any calibration (deadzone correction, rescaling, drift correction, etc.)
/// has already been applied at an earlier stage of processing.
///
/// The neutral origin of this input data is always at `(0, 0)`.
/// When working with gamepad axes, both X and Y values are typically bounded by `[-1.0, 1.0]`.
/// However, this range may not apply to other input types, such as mousewheel data which can have a wider range.
#[derive(Default, Debug, Copy, Clone, PartialEq, Deserialize, Serialize, Reflect)]
#[must_use]
pub struct DualAxisData(Vec2);

impl DualAxisData {
    /// All zeros.
    pub const ZERO: Self = Self(Vec2::ZERO);

    /// Creates a [`DualAxisData`] with the given values.
    pub const fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }

    /// Creates a [`DualAxisData`] directly from the given [`Vec2`].
    pub const fn from_xy(xy: Vec2) -> Self {
        Self(xy)
    }

    /// Combines the directional input from this [`DualAxisData`] with another.
    ///
    /// This is useful if you have multiple sticks bound to the same game action,
    /// and you want to get their combined position.
    ///
    /// # Warning
    ///
    /// This method performs vector addition on the X and Y components.
    /// While the direction is preserved, the combined magnitude might exceed the expected
    /// range for certain input devices (e.g., gamepads typically have a maximum magnitude of `1.0`).
    ///
    /// To ensure the combined input stays within the expected range,
    /// consider using [`Self::clamp_length`] on the returned value.
    pub fn merged_with(&self, other: Self) -> Self {
        Self(self.0 + other.0)
    }

    /// The value along the X-axis, typically ranging from `-1.0` to `1.0`.
    #[must_use]
    #[inline]
    pub const fn x(&self) -> f32 {
        self.0.x
    }

    /// The value along the Y-axis, typically ranging from `-1.0` to `1.0`.
    #[must_use]
    #[inline]
    pub const fn y(&self) -> f32 {
        self.0.y
    }

    /// The values along each axis, each typically ranging from `-1.0` to `1.0`.
    #[must_use]
    #[inline]
    pub const fn xy(&self) -> Vec2 {
        self.0
    }

    /// The [`Direction2d`] that this axis is pointing towards, if not neutral.
    #[must_use]
    #[inline]
    pub fn direction(&self) -> Option<Direction2d> {
        Direction2d::new(self.0).ok()
    }

    /// The [`Rotation`] (measured clockwise from midnight) that this axis is pointing towards, if not neutral.
    #[must_use]
    #[inline]
    pub fn rotation(&self) -> Option<Rotation> {
        Rotation::from_xy(self.0).ok()
    }

    /// Computes the magnitude of the value (distance from the origin), typically bounded by `[0, 1]`.
    ///
    /// If you only need to compare relative magnitudes, use [`Self::length_squared`] instead for faster computation.
    #[must_use]
    #[inline]
    pub fn length(&self) -> f32 {
        self.0.length()
    }

    /// Computes the squared magnitude, typically bounded by `[0, 1]`.
    ///
    /// This is faster than [`Self::length`], as it avoids a square root, but will generally have less natural behavior.
    #[must_use]
    #[inline]
    pub fn length_squared(&self) -> f32 {
        self.0.length_squared()
    }

    /// Clamps the value to a maximum magnitude.
    #[inline]
    pub fn clamp_length(&mut self, max: f32) {
        self.0 = self.0.clamp_length_max(max);
    }
}

impl From<DualAxisData> for Vec2 {
    fn from(data: DualAxisData) -> Vec2 {
        data.0
    }
}
