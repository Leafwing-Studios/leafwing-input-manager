//! Tools for working with directional axis-like user inputs (gamesticks, D-Pads and emulated equvalents)

use crate::orientation::{Direction, Rotation};
use crate::user_input::InputKind;
use bevy_input::{
    gamepad::{GamepadAxisType, GamepadButtonType},
    keyboard::KeyCode,
};
use bevy_math::Vec2;
use bevy_utils::FloatOrd;
use serde::{Deserialize, Serialize};

/// A single directional axis with a configurable trigger zone.
///
/// These can be stored in a [`InputKind`] to create a virtual button.
///
/// # Warning
///
/// `positive_low` must be greater than or equal to `negative_low` for this type to be validly constructed.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SingleAxis {
    /// The axis that is being checked.
    pub axis_type: AxisType,
    /// Any axis value higher than this will trigger the input.
    pub positive_low: f32,
    /// Any axis value lower than this will trigger the input.
    pub negative_low: f32,
    /// The target value for this input, used for input mocking.
    ///
    /// WARNING: this field is ignored for the sake of [`Eq`] and [`Hash`](std::hash::Hash)
    pub value: Option<f32>,
}

impl SingleAxis {
    /// Creates a [`SingleAxis`] with both `positive_low` and `negative_low` set to `threshold`.
    #[must_use]
    pub fn symmetric(axis_type: impl Into<AxisType>, threshold: f32) -> SingleAxis {
        SingleAxis {
            axis_type: axis_type.into(),
            positive_low: threshold,
            negative_low: threshold,
            value: None,
        }
    }
}

impl PartialEq for SingleAxis {
    fn eq(&self, other: &Self) -> bool {
        self.axis_type == other.axis_type
            && FloatOrd(self.positive_low) == FloatOrd(other.positive_low)
            && FloatOrd(self.negative_low) == FloatOrd(other.negative_low)
    }
}
impl Eq for SingleAxis {}
impl std::hash::Hash for SingleAxis {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.axis_type.hash(state);
        FloatOrd(self.positive_low).hash(state);
        FloatOrd(self.negative_low).hash(state);
    }
}

/// Two directional axes combined as one input.
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
pub struct DualAxis {
    /// The axis representing horizontal movement.
    pub x_axis_type: AxisType,
    /// The axis representing vertical movement.
    pub y_axis_type: AxisType,
    /// If the stick is moved right more than this amount the input will be triggered.
    pub x_positive_low: f32,
    /// If the stick is moved left more than this amount the input will be triggered.
    pub x_negative_low: f32,
    /// If the stick is moved up more than this amount the input will be triggered.
    pub y_positive_low: f32,
    /// If the stick is moved down more than this amount the input will be triggered.
    pub y_negative_low: f32,
    /// The target value for this input, used for input mocking.
    ///
    /// WARNING: this field is ignored for the sake of [`Eq`] and [`Hash`](std::hash::Hash)
    pub value: Option<Vec2>,
}

impl DualAxis {
    /// The default size of the deadzone used by constructor methods.
    ///
    /// This cannot be changed, but the struct can be easily manually constructed.
    pub const DEFAULT_DEADZONE: f32 = 0.1;

    /// Creates a [`DualAxis`] with both `positive_low` and `negative_low` in both axes set to `threshold`.
    #[must_use]
    pub fn symmetric(
        x_axis_type: impl Into<AxisType>,
        y_axis_type: impl Into<AxisType>,
        threshold: f32,
    ) -> DualAxis {
        DualAxis {
            x_axis_type: x_axis_type.into(),
            y_axis_type: y_axis_type.into(),
            x_positive_low: threshold,
            x_negative_low: threshold,
            y_positive_low: threshold,
            y_negative_low: threshold,
            value: None,
        }
    }

    /// Creates a [`DualAxis`] for the left analogue stick of the gamepad.
    #[must_use]
    pub fn left_stick() -> DualAxis {
        DualAxis::symmetric(
            GamepadAxisType::LeftStickX,
            GamepadAxisType::LeftStickY,
            Self::DEFAULT_DEADZONE,
        )
    }

    /// Creates a [`DualAxis`] for the right analogue stick of the gamepad.
    #[must_use]
    pub fn right_stick() -> DualAxis {
        DualAxis::symmetric(
            GamepadAxisType::RightStickX,
            GamepadAxisType::LeftStickY,
            Self::DEFAULT_DEADZONE,
        )
    }
}

impl PartialEq for DualAxis {
    fn eq(&self, other: &Self) -> bool {
        self.x_axis_type == other.x_axis_type
            && self.y_axis_type == other.y_axis_type
            && FloatOrd(self.x_positive_low) == FloatOrd(other.x_positive_low)
            && FloatOrd(self.x_negative_low) == FloatOrd(other.x_negative_low)
            && FloatOrd(self.y_positive_low) == FloatOrd(other.y_positive_low)
            && FloatOrd(self.y_negative_low) == FloatOrd(other.y_negative_low)
    }
}
impl Eq for DualAxis {}
impl std::hash::Hash for DualAxis {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x_axis_type.hash(state);
        self.y_axis_type.hash(state);
        FloatOrd(self.x_positive_low).hash(state);
        FloatOrd(self.x_negative_low).hash(state);
        FloatOrd(self.y_positive_low).hash(state);
        FloatOrd(self.y_negative_low).hash(state);
    }
}

#[allow(clippy::doc_markdown)] // False alarm because it thinks DPad is an un-quoted item
/// A virtual DPad that you can get an [`AxisPair`] from
///
/// Typically, you don't want to store a [`DualAxis`] in this type,
/// even though it can be stored as an [`InputKind`].
///
/// Instead, use it directly as [`InputKind::DualAxis`]!
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
    pub fn arrow_keys() -> VirtualDPad {
        VirtualDPad {
            up: InputKind::Keyboard(KeyCode::Up),
            down: InputKind::Keyboard(KeyCode::Down),
            left: InputKind::Keyboard(KeyCode::Left),
            right: InputKind::Keyboard(KeyCode::Right),
        }
    }

    /// Generates a [`VirtualDPad`] corresponding to the `WASD` keyboard keycodes
    pub fn wasd() -> VirtualDPad {
        VirtualDPad {
            up: InputKind::Keyboard(KeyCode::W),
            down: InputKind::Keyboard(KeyCode::S),
            left: InputKind::Keyboard(KeyCode::A),
            right: InputKind::Keyboard(KeyCode::D),
        }
    }

    #[allow(clippy::doc_markdown)] // False alarm because it thinks DPad is an un-quoted item
    /// Generates a [`VirtualDPad`] corresponding to the DPad on a gamepad
    pub fn dpad() -> VirtualDPad {
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
    pub fn gamepad_face_buttons() -> VirtualDPad {
        VirtualDPad {
            up: InputKind::GamepadButton(GamepadButtonType::North),
            down: InputKind::GamepadButton(GamepadButtonType::South),
            left: InputKind::GamepadButton(GamepadButtonType::West),
            right: InputKind::GamepadButton(GamepadButtonType::East),
        }
    }
}

/// The type of axis used by a [`UserInput`](crate::user_input::UserInput).
///
/// This is stored in either a [`SingleAxis`] or [`DualAxis`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AxisType {
    /// Input associated with a gamepad, such as the triggers or one axis of an analog stick.
    Gamepad(GamepadAxisType),
    /// Input associated with a mouse wheel.
    MouseWheel(MouseWheelAxisType),
}

/// The direction of motion of the mouse wheel.
///
/// Stored in the [`AxisType`] enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseWheelAxisType {
    /// Horizontal movement.
    ///
    /// This is much less common than the `Y` variant, and is only supported on some devices.
    X,
    /// Vertical movement.
    ///
    /// This is the standard behavior for a mouse wheel, used to scroll up and down pages.
    Y,
}

impl From<GamepadAxisType> for AxisType {
    fn from(axis_type: GamepadAxisType) -> Self {
        AxisType::Gamepad(axis_type)
    }
}

impl From<MouseWheelAxisType> for AxisType {
    fn from(axis_type: MouseWheelAxisType) -> Self {
        AxisType::MouseWheel(axis_type)
    }
}

impl TryFrom<AxisType> for GamepadAxisType {
    type Error = AxisConversionError;

    fn try_from(axis_type: AxisType) -> Result<Self, AxisConversionError> {
        match axis_type {
            AxisType::Gamepad(inner) => Ok(inner),
            _ => Err(AxisConversionError),
        }
    }
}

impl TryFrom<AxisType> for MouseWheelAxisType {
    type Error = AxisConversionError;

    fn try_from(axis_type: AxisType) -> Result<Self, AxisConversionError> {
        match axis_type {
            AxisType::MouseWheel(inner) => Ok(inner),
            _ => Err(AxisConversionError),
        }
    }
}

/// An [`AxisType`] could not be converted into a more specialized variant
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AxisConversionError;

/// A wrapped [`Vec2`] that represents the combaination of two input axes.
///
/// The neutral origin is always at 0, 0.
/// When working with gamepad axes, both `x` and `y` values are bounded by [-1.0, 1.0].
/// For other input axes (such as mousewheel data), this may not be true!
///
/// This struct should store the processed form of your raw inputs in a device-agnostic fashion.
/// Any deadzone correction, rescaling or drift-correction should be done at an earlier level.
#[derive(Debug, Copy, Clone, PartialEq, Default, Deserialize, Serialize)]
pub struct DualAxisData {
    xy: Vec2,
}

// Constructors
impl DualAxisData {
    /// Creates a new [`AxisPair`] from the provided (x,y) coordinates
    pub fn new(xy: Vec2) -> DualAxisData {
        DualAxisData { xy }
    }

    /// Merge the state of this [`AxisPair`] with another.
    ///
    /// This is useful if you have multiple sticks bound to the same game action,
    /// and you want to get their combined position.
    ///
    /// # Warning
    ///
    /// This method can result in values with a greater maximum magnitude than expected!
    /// Use [`AxisPair::clamp_length`] to limit the resulting direction.
    pub fn merged_with(&self, other: DualAxisData) -> DualAxisData {
        DualAxisData::new(self.xy() + other.xy())
    }
}

// Methods
impl DualAxisData {
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

    /// Clamps the magnitude of the axis
    pub fn clamp_length(&mut self, max: f32) {
        self.xy = self.xy.clamp_length_max(max);
    }
}
