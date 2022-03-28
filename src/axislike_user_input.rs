//! Tools for working with directional axis-like user inputs (gamesticks, D-Pads and emulated equvalents)

use crate::orientation::{Direction, Rotation};
use bevy::math::Vec2;

/// A high-level abstract user input that varies from -1 to 1, inclusive, along two axes
///
/// The neutral origin is always at 0, 0.
/// When constructed; the magnitude is capped at 1, but direction is preserved.
///
/// This struct should store the processed form of your raw inputs in a device-agnostic fashion.
/// Any deadzone correction, rescaling or drift-correction should be done at an earlier level.
#[derive(Debug, Clone, PartialEq)]
pub struct AxisPair {
    xy: Vec2,
}

// Constructors
impl AxisPair {
    /// Creates a new [`AxisPair`] from the provided (x,y) coordinates
    ///
    /// The direction is preserved, by the magnitude will be clamped to at most 1.
    pub fn new(xy: Vec2) -> AxisPair {
        let magnitude = xy.length();
        if magnitude <= 1. {
            AxisPair { xy }
        } else {
            AxisPair { xy: xy / magnitude }
        }
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
    /// If the axis is neutral (x,y) = (0,0), a (0, 0) `Direction` will be returned
    #[must_use]
    #[inline]
    pub fn direction(&self) -> Direction {
        Direction::new(self.xy)
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
    pub fn magnitude(&self) -> f32 {
        self.xy.length()
    }

    /// The square of the axis' magnitude
    ///
    /// Always bounded between 0 and 1.
    ///
    /// This is faster than `magnitude`, as it avoids a square root, but will generally have less natural behavior.
    #[must_use]
    #[inline]
    pub fn magnitude_squared(&self) -> f32 {
        self.xy.length_squared()
    }
}
