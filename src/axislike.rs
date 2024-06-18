//! Tools for working with directional axis-like user inputs (game sticks, D-Pads and emulated equivalents)

use bevy::{
    math::Rot2,
    prelude::{Dir2, Reflect, Vec2},
};
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

/// A wrapped [`Vec2`] that represents the combination of two input axes.
///
/// The neutral origin is always at 0, 0.
/// When working with gamepad axes, both `x` and `y` values are bounded by [-1.0, 1.0].
/// For other input axes (such as mousewheel data), this may not be true!
///
/// This struct should store the processed form of your raw inputs in a device-agnostic fashion.
/// Any deadzone correction, rescaling or drift-correction should be done at an earlier level.
#[derive(Debug, Copy, Clone, PartialEq, Default, Deserialize, Serialize, Reflect)]
pub struct DualAxisData {
    xy: Vec2,
}

// Constructors
impl DualAxisData {
    /// Creates a new [`DualAxisData`] from the provided (x,y) coordinates
    pub fn new(x: f32, y: f32) -> DualAxisData {
        DualAxisData {
            xy: Vec2::new(x, y),
        }
    }

    /// Creates a new [`DualAxisData`] directly from a [`Vec2`]
    pub fn from_xy(xy: Vec2) -> DualAxisData {
        DualAxisData { xy }
    }

    /// Merge the state of this [`DualAxisData`] with another.
    ///
    /// This is useful if you have multiple sticks bound to the same game action,
    /// and you want to get their combined position.
    ///
    /// # Warning
    ///
    /// This method can result in values with a greater maximum magnitude than expected!
    /// Use [`DualAxisData::clamp_length`] to limit the resulting direction.
    pub fn merged_with(&self, other: DualAxisData) -> DualAxisData {
        DualAxisData::from_xy(self.xy() + other.xy())
    }
}

// Methods
impl DualAxisData {
    /// The value along the x-axis, typically ranging from -1 to 1
    #[must_use]
    #[inline]
    pub fn x(&self) -> f32 {
        self.xy.x
    }

    /// The value along the y-axis, typically ranging from -1 to 1
    #[must_use]
    #[inline]
    pub fn y(&self) -> f32 {
        self.xy.y
    }

    /// The (x, y) values, each typically ranging from -1 to 1
    #[must_use]
    #[inline]
    pub fn xy(&self) -> Vec2 {
        self.xy
    }

    /// The [`Dir2`] that this axis is pointing towards, if any
    ///
    /// If the axis is neutral (x,y) = (0,0), a (0, 0) `None` will be returned
    #[must_use]
    #[inline]
    pub fn direction(&self) -> Option<Dir2> {
        Dir2::new(self.xy).ok()
    }

    /// The [`Rotation`] (measured clockwise from midnight) that this axis is pointing towards, if any
    ///
    /// If the axis is neutral (x,y) = (0,0), this will be `None`
    #[must_use]
    #[inline]
    pub fn rotation(&self) -> Option<Rot2> {
        self.direction().map(|dir| dir.rotation_from_x())
    }

    /// How far from the origin is this axis's position?
    ///
    /// Typically bounded by 0 and 1.
    ///
    /// If you only need to compare relative magnitudes, use `magnitude_squared` instead for faster computation.
    #[must_use]
    #[inline]
    pub fn length(&self) -> f32 {
        self.xy.length()
    }

    /// The square of the axis' magnitude
    ///
    /// Typically bounded by 0 and 1.
    ///
    /// This is faster than `magnitude`, as it avoids a square root, but will generally have less natural behavior.
    #[must_use]
    #[inline]
    pub fn length_squared(&self) -> f32 {
        self.xy.length_squared()
    }

    /// Clamps the magnitude of the axis
    pub fn clamp_length(&mut self, max: f32) {
        self.xy = self.xy.clamp_length_max(max);
    }
}

impl From<DualAxisData> for Vec2 {
    fn from(data: DualAxisData) -> Vec2 {
        data.xy
    }
}
