//! Tools for working with directional axis-like user inputs (game sticks, D-Pads and emulated equivalents)

use bevy::prelude::{Reflect, Vec2};
use serde::{Deserialize, Serialize};

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
