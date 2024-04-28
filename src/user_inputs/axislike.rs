use crate::input_processing::{AxisProcessor, DualAxisProcessor};
use crate::input_streams::InputStreams;
use crate::orientation::Rotation;
use bevy::math::Vec2;
use bevy::prelude::{Direction2d, Reflect};
use bevy::utils::petgraph::matrix_graph::Zero;
use serde::{Deserialize, Serialize};

/// Different ways that user input is represented on an axis.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize, Reflect)]
#[must_use]
pub enum AxisInputMode {
    /// Continuous input values, typically range from a negative maximum (e.g., `-1.0`)
    /// to a positive maximum (e.g., `1.0`), allowing for smooth and precise control.
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

/// The axes for dual-axis inputs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub enum DualAxis {
    /// The X-axis (typically horizontal movement).
    X,

    /// The Y-axis (typically vertical movement).
    Y,
}

impl DualAxis {
    /// Gets the component on the specified axis.
    #[must_use]
    #[inline]
    pub fn value(&self, value: Vec2) -> f32 {
        match self {
            Self::X => value.x,
            Self::Y => value.y,
        }
    }
}

/// The directions for dual-axis inputs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
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
    /// Checks if the given `value` is active in the specified direction.
    #[must_use]
    #[inline]
    pub fn is_active(&self, value: Vec2) -> bool {
        match self {
            Self::Up => value.y > 0.0,
            Self::Down => value.y < 0.0,
            Self::Left => value.x < 0.0,
            Self::Right => value.x > 0.0,
        }
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
        Self::analog(self.0 + other.0)
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

    /// The [`Direction2d`] that this axis is pointing towards, if any
    ///
    /// If the axis is neutral (x, y) = (0, 0), this will be `None`.
    #[must_use]
    #[inline]
    pub fn direction(&self) -> Option<Direction2d> {
        Direction2d::new(self.0).ok()
    }

    /// The [`Rotation`] (measured clockwise from midnight) that this axis is pointing towards, if any
    ///
    /// If the axis is neutral (x, y) = (0, 0), this will be `None`.
    #[must_use]
    #[inline]
    pub fn rotation(&self) -> Option<Rotation> {
        Rotation::from_xy(self.0).ok()
    }

    /// How far from the origin is this axis's position?
    ///
    /// Typically bounded by 0 and 1.
    ///
    /// If you only need to compare relative magnitudes, use [`Self::length_squared`] instead for faster computation.
    #[must_use]
    #[inline]
    pub fn length(&self) -> f32 {
        self.0.length()
    }

    /// The square of the axis' magnitude
    ///
    /// Typically bounded by 0 and 1.
    ///
    /// This is faster than [`Self::length`], as it avoids a square root, but will generally have less natural behavior.
    #[must_use]
    #[inline]
    pub fn length_squared(&self) -> f32 {
        self.0.length_squared()
    }

    /// Clamps the magnitude of the axis
    pub fn clamp_length(&mut self, max: f32) {
        self.0 = self.0.clamp_length_max(max);
    }
}

impl From<DualAxisData> for Vec2 {
    fn from(data: DualAxisData) -> Vec2 {
        data.0
    }
}
