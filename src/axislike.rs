//! Tools for working with directional axis-like user inputs (gamesticks, D-Pads and emulated equvalents)

use crate::orientation::{Direction, Rotation};
use crate::user_input::InputKind;
use bevy_core::FloatOrd;
use bevy_input::{
    gamepad::{GamepadAxisType, GamepadButtonType},
    keyboard::KeyCode,
};
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

/// A single gamepad axis with a configurable trigger zone.
///
/// These can be stored in a [`InputKind`] to create a virtual button.
///
/// # Warning
///
/// `positive_low` must be greater than or equal to `negative_low` for this type to be validly constructed.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SingleGamepadAxis {
    /// The axis that is being checked.
    pub axis: GamepadAxisType,
    /// Any axis value higher than this will trigger the input.
    pub positive_low: f32,
    /// Any axis value lower than this will trigger the input.
    pub negative_low: f32,
}

impl SingleGamepadAxis {
    /// Creates a [`SingleGamepadAxis`] with both `positive_low` and `negative_low` set to `threshold`.
    #[must_use]
    pub const fn symmetric(axis: GamepadAxisType, threshold: f32) -> SingleGamepadAxis {
        SingleGamepadAxis {
            axis,
            positive_low: threshold,
            negative_low: threshold,
        }
    }
}

impl PartialEq for SingleGamepadAxis {
    fn eq(&self, other: &Self) -> bool {
        self.axis == other.axis
            && FloatOrd(self.positive_low) == FloatOrd(other.positive_low)
            && FloatOrd(self.negative_low) == FloatOrd(other.negative_low)
    }
}
impl Eq for SingleGamepadAxis {}
impl std::hash::Hash for SingleGamepadAxis {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.axis.hash(state);
        FloatOrd(self.positive_low).hash(state);
        FloatOrd(self.negative_low).hash(state);
    }
}

/// Two gamepad axes combined as one input.
///
/// These can be stored in a [`VirtualDPad`], which is itself stored in an [`InputKind`] for consumption.
///
/// This input will generate [`AxisPair`] can be read with
/// [`ActionState::action_axis_pair()`][crate::ActionState::action_axis_pair()].
///
/// # Warning
///
/// `positive_low` must be greater than or equal to `negative_low` for both `x` and `y` for this type to be validly constructed.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DualGamepadAxis {
    /// The gamepad axis to use as the x axis.
    pub x_axis: GamepadAxisType,
    /// The gamepad axis to use as the y axis.
    pub y_axis: GamepadAxisType,
    /// If the stick is moved right more than this amount the input will be triggered.
    pub x_positive_low: f32,
    /// If the stick is moved left more than this amount the input will be triggered.
    pub x_negative_low: f32,
    /// If the stick is moved up more than this amount the input will be triggered.
    pub y_positive_low: f32,
    /// If the stick is moved down more than this amount the input will be triggered.
    pub y_negative_low: f32,
}

impl DualGamepadAxis {
    /// The default size of the deadzone used by constructor methods.
    ///
    /// This cannot be changed, but the struct can be easily manually constructed.
    pub const DEFAULT_DEADZONE: f32 = 0.1;

    /// Creates a [`DualGamepadAxis`] with both `positive_low` and `negative_low` in both axes set to `threshold`.
    #[must_use]
    pub const fn symmetric(
        x_axis: GamepadAxisType,
        y_axis: GamepadAxisType,
        threshold: f32,
    ) -> DualGamepadAxis {
        DualGamepadAxis {
            x_axis,
            y_axis,
            x_positive_low: threshold,
            x_negative_low: threshold,
            y_positive_low: threshold,
            y_negative_low: threshold,
        }
    }

    /// Creates a [`DualGamepadAxis`] for the left analogue stick of the gamepad.
    #[must_use]
    pub const fn left_stick() -> DualGamepadAxis {
        DualGamepadAxis::symmetric(
            GamepadAxisType::LeftStickX,
            GamepadAxisType::LeftStickY,
            Self::DEFAULT_DEADZONE,
        )
    }

    /// Creates a [`DualGamepadAxis`] for the right analogue stick of the gamepad.
    #[must_use]
    pub const fn right_stick() -> DualGamepadAxis {
        DualGamepadAxis::symmetric(
            GamepadAxisType::RightStickX,
            GamepadAxisType::LeftStickY,
            Self::DEFAULT_DEADZONE,
        )
    }
}

impl PartialEq for DualGamepadAxis {
    fn eq(&self, other: &Self) -> bool {
        self.x_axis == other.x_axis
            && self.y_axis == other.y_axis
            && FloatOrd(self.x_positive_low) == FloatOrd(other.x_positive_low)
            && FloatOrd(self.x_negative_low) == FloatOrd(other.x_negative_low)
            && FloatOrd(self.y_positive_low) == FloatOrd(other.y_positive_low)
            && FloatOrd(self.y_negative_low) == FloatOrd(other.y_negative_low)
    }
}
impl Eq for DualGamepadAxis {}
impl std::hash::Hash for DualGamepadAxis {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x_axis.hash(state);
        self.y_axis.hash(state);
        FloatOrd(self.x_positive_low).hash(state);
        FloatOrd(self.x_negative_low).hash(state);
        FloatOrd(self.y_positive_low).hash(state);
        FloatOrd(self.y_negative_low).hash(state);
    }
}

#[allow(clippy::doc_markdown)] // False alarm because it thinks DPad is an un-quoted item
/// A virtual DPad that you can get an [`AxisPair`] from
///
/// Typically, you don't want to store a [`DualGamepadAxis`] in this type,
/// even though it can be stored as an [`InputKind`].
///
/// Instead, use it directly as [`InputKind::DualGamepadAxis`]!
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

impl VirtualDPad {
    /// Generates a [`VirtualDPad`] corresponding to the arrow keyboard keycodes
    pub const fn arrow_keys() -> VirtualDPad {
        VirtualDPad {
            up: InputKind::Keyboard(KeyCode::Up),
            down: InputKind::Keyboard(KeyCode::Down),
            left: InputKind::Keyboard(KeyCode::Left),
            right: InputKind::Keyboard(KeyCode::Right),
        }
    }

    /// Generates a [`VirtualDPad`] corresponding to the `WASD` keyboard keycodes
    pub const fn wasd() -> VirtualDPad {
        VirtualDPad {
            up: InputKind::Keyboard(KeyCode::W),
            down: InputKind::Keyboard(KeyCode::S),
            left: InputKind::Keyboard(KeyCode::A),
            right: InputKind::Keyboard(KeyCode::D),
        }
    }

    /// Generates a [`VirtualDPad`] corresponding to the DPad on a gamepad
    pub const fn dpad() -> VirtualDPad {
        VirtualDPad {
            up: InputKind::GamepadButton(GamepadButtonType::DPadUp),
            down: InputKind::GamepadButton(GamepadButtonType::DPadDown),
            left: InputKind::GamepadButton(GamepadButtonType::DPadLeft),
            right: InputKind::GamepadButton(GamepadButtonType::DPadRight),
        }
    }

    /// Generates a [`VirtualDPad`] corresponding to the face buttons on a gamepad
    ///
    /// North corresponds to up, west corresponds to left, east corresponds to right, south corresponds to down
    pub const fn gamepad_face_buttons() -> VirtualDPad {
        VirtualDPad {
            up: InputKind::GamepadButton(GamepadButtonType::North),
            down: InputKind::GamepadButton(GamepadButtonType::South),
            left: InputKind::GamepadButton(GamepadButtonType::West),
            right: InputKind::GamepadButton(GamepadButtonType::East),
        }
    }
}
