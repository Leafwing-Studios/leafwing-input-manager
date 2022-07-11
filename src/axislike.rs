//! Tools for working with directional axis-like user inputs (gamesticks, D-Pads and emulated equvalents)

use crate::orientation::{Direction, Rotation};
use crate::user_input::InputKind;
use bevy_math::Vec2;
use serde::{Deserialize, Serialize};

/// A high-level abstract user input that varies from -1 to 1, inclusive, along two axes
///
/// The neutral origin is always at 0, 0.
/// When constructed; the magnitude is capped at 1, but direction is preserved.
///
/// This struct should store the processed form of your raw inputs in a device-agnostic fashion.
/// Any deadzone correction, rescaling or drift-correction should be done at an earlier level.
#[derive(Debug, Copy, Clone, PartialEq, Default, Deserialize, Serialize)]
pub struct AxisPair {
    xy: Vec2,
}

// Constructors
impl AxisPair {
    /// Creates a new [`AxisPair`] from the provided (x,y) coordinates
    ///
    /// The direction is preserved, by the magnitude will be clamped to at most 1.
    pub fn new(xy: Vec2) -> AxisPair {
        AxisPair {
            xy: xy.clamp_length_max(1.0),
        }
    }

    /// Merge the state of this [`AxisPair`] with another.
    ///
    /// This is useful if you have multiple sticks bound to the same game action,
    /// and you want to get their combined position.
    pub fn merged_with(&self, other: AxisPair) -> AxisPair {
        AxisPair::new((self.xy() + other.xy()).clamp_length_max(1.0))
    }
}

// Methods
impl AxisPair {
    /// The value along the x-axis, ranging from -1 to 1
    #[must_use]
    #[inline]
    pub fn x(&self) -> f32 {
        self.xy.x
    }

    /// The value along the y-axis, ranging from -1 to 1
    #[must_use]
    #[inline]
    pub fn y(&self) -> f32 {
        self.xy.y
    }

    /// The (x, y) values, each ranging from -1 to 1
    #[must_use]
    #[inline]
    pub fn xy(&self) -> Vec2 {
        self.xy
    }

    /// The [`Direction`] that this axis is pointing towards, if any
    ///
    /// If the axis is neutral (x,y) = (0,0), a (0, 0) `None` will be returned
    #[must_use]
    #[inline]
    pub fn direction(&self) -> Option<Direction> {
        // TODO: replace this quick-n-dirty hack once Direction::new no longer panics
        if self.xy.length() > 0.00001 {
            return Some(Direction::new(self.xy));
        }
        None
    }

    /// The [`Rotation`] (measured clockwise from midnight) that this axis is pointing towards, if any
    ///
    /// If the axis is neutral (x,y) = (0,0), this will be `None`
    #[must_use]
    #[inline]
    pub fn rotation(&self) -> Option<Rotation> {
        match Rotation::from_xy(self.xy) {
            Ok(rotation) => Some(rotation),
            Err(_) => None,
        }
    }

    /// How far from the origin is this axis's position?
    ///
    /// Always bounded between 0 and 1.
    ///
    /// If you only need to compare relative magnitudes, use `magnitude_squared` instead for faster computation.
    #[must_use]
    #[inline]
    pub fn length(&self) -> f32 {
        self.xy.length()
    }

    /// The square of the axis' magnitude
    ///
    /// Always bounded between 0 and 1.
    ///
    /// This is faster than `magnitude`, as it avoids a square root, but will generally have less natural behavior.
    #[must_use]
    #[inline]
    pub fn length_squared(&self) -> f32 {
        self.xy.length_squared()
    }
}

#[allow(clippy::doc_markdown)] // False alarm because it thinks DPad is an un-quoted item
/// A virtual DPad that you can get an [`AxisPair`] from
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VirtualDPad {
    /// The input that represents the up direction in this virtual DPad
    pub up: InputKind,
    /// The input that represents the down direction in this virtual DPad
    pub down: InputKind,
    /// The input that represents the left direction in this virtual DPad
    pub left: InputKind,
    /// The input that represents the right direction in this virtual DPad
    pub right: InputKind,
}
