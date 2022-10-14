//! Tools for working with directional axis-like user inputs (gamesticks, D-Pads and emulated equvalents)

use crate::buttonlike::{MouseMotionDirection, MouseWheelDirection};
use crate::orientation::{Direction, Rotation};
use crate::user_input::InputKind;
use bevy::input::{
    gamepad::{GamepadAxisType, GamepadButtonType},
    keyboard::KeyCode,
};
use bevy::math::Vec2;
use bevy::utils::FloatOrd;
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
            negative_low: -threshold,
            value: None,
        }
    }

    /// Creates a [`SingleAxis`] with the specified `axis_type` and `value`.
    ///
    /// All thresholds are set to 0.0.
    /// Primarily useful for [input mocking](crate::MockInput).
    #[must_use]
    pub fn from_value(axis_type: impl Into<AxisType>, value: f32) -> SingleAxis {
        SingleAxis {
            axis_type: axis_type.into(),
            positive_low: 0.0,
            negative_low: 0.0,
            value: Some(value),
        }
    }

    /// Creates a [`SingleAxis`] corresponding to horizontal [`MouseWheel`](bevy::input::mouse::MouseWheel) movement
    #[must_use]
    pub const fn mouse_wheel_x() -> SingleAxis {
        SingleAxis {
            axis_type: AxisType::MouseWheel(MouseWheelAxisType::X),
            positive_low: 0.,
            negative_low: 0.,
            value: None,
        }
    }

    /// Creates a [`SingleAxis`] corresponding to vertical [`MouseWheel`](bevy::input::mouse::MouseWheel) movement
    #[must_use]
    pub const fn mouse_wheel_y() -> SingleAxis {
        SingleAxis {
            axis_type: AxisType::MouseWheel(MouseWheelAxisType::Y),
            positive_low: 0.,
            negative_low: 0.,
            value: None,
        }
    }

    /// Creates a [`SingleAxis`] corresponding to horizontal [`MouseMotion`](bevy::input::mouse::MouseMotion) movement
    #[must_use]
    pub const fn mouse_motion_x() -> SingleAxis {
        SingleAxis {
            axis_type: AxisType::MouseMotion(MouseMotionAxisType::X),
            positive_low: 0.,
            negative_low: 0.,
            value: None,
        }
    }

    /// Creates a [`SingleAxis`] corresponding to vertical [`MouseMotion`](bevy::input::mouse::MouseMotion) movement
    #[must_use]
    pub const fn mouse_motion_y() -> SingleAxis {
        SingleAxis {
            axis_type: AxisType::MouseMotion(MouseMotionAxisType::Y),
            positive_low: 0.,
            negative_low: 0.,
            value: None,
        }
    }

    /// Creates a [`SingleAxis`] with the `axis_type` and `negative_low` set to `threshold`.
    ///
    /// Positive values will not trigger the input.
    pub fn negative_only(axis_type: impl Into<AxisType>, threshold: f32) -> SingleAxis {
        SingleAxis {
            axis_type: axis_type.into(),
            negative_low: threshold,
            positive_low: f32::MAX,
            value: None,
        }
    }

    /// Creates a [`SingleAxis`] with the `axis_type` and `positive_low` set to `threshold`.
    ///
    /// Negative values will not trigger the input.
    pub fn positive_only(axis_type: impl Into<AxisType>, threshold: f32) -> SingleAxis {
        SingleAxis {
            axis_type: axis_type.into(),
            negative_low: f32::MIN,
            positive_low: threshold,
            value: None,
        }
    }

    /// Returns this [`SingleAxis`] with the deadzone set to the specified value
    #[must_use]
    pub fn with_deadzone(mut self, deadzone: f32) -> SingleAxis {
        self.negative_low = deadzone;
        self.positive_low = deadzone;
        self
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
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DualAxis {
    /// The axis representing horizontal movement.
    pub x: SingleAxis,
    /// The axis representing vertical movement.
    pub y: SingleAxis,
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
            x: SingleAxis::symmetric(x_axis_type, threshold),
            y: SingleAxis::symmetric(y_axis_type, threshold),
        }
    }

    /// Creates a [`SingleAxis`] with the specified `axis_type` and `value`.
    ///
    /// All thresholds are set to 0.0.
    /// Primarily useful for [input mocking](crate::MockInput).
    #[must_use]
    pub fn from_value(
        x_axis_type: impl Into<AxisType>,
        y_axis_type: impl Into<AxisType>,
        x_value: f32,
        y_value: f32,
    ) -> DualAxis {
        DualAxis {
            x: SingleAxis::from_value(x_axis_type, x_value),
            y: SingleAxis::from_value(y_axis_type, y_value),
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
            GamepadAxisType::RightStickY,
            Self::DEFAULT_DEADZONE,
        )
    }

    /// Creates a [`DualAxis`] corresponding to horizontal and vertical [`MouseWheel`](bevy::input::mouse::MouseWheel) movement
    pub const fn mouse_wheel() -> DualAxis {
        DualAxis {
            x: SingleAxis::mouse_wheel_x(),
            y: SingleAxis::mouse_wheel_y(),
        }
    }

    /// Creates a [`DualAxis`] corresponding to horizontal and vertical [`MouseMotion`](bevy::input::mouse::MouseMotion) movement
    pub const fn mouse_motion() -> DualAxis {
        DualAxis {
            x: SingleAxis::mouse_motion_x(),
            y: SingleAxis::mouse_motion_y(),
        }
    }

    /// Returns this [`DualAxis`] with the deadzone set to the specified value
    #[must_use]
    pub fn with_deadzone(mut self, deadzone: f32) -> DualAxis {
        self.x = self.x.with_deadzone(deadzone);
        self.y = self.y.with_deadzone(deadzone);
        self
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

    /// Generates a [`VirtualDPad`] corresponding to discretized mousewheel movements
    pub fn mouse_wheel() -> VirtualDPad {
        VirtualDPad {
            up: InputKind::MouseWheel(MouseWheelDirection::Up),
            down: InputKind::MouseWheel(MouseWheelDirection::Down),
            left: InputKind::MouseWheel(MouseWheelDirection::Left),
            right: InputKind::MouseWheel(MouseWheelDirection::Right),
        }
    }

    /// Generates a [`VirtualDPad`] corresponding to discretized mouse motions
    pub fn mouse_motion() -> VirtualDPad {
        VirtualDPad {
            up: InputKind::MouseMotion(MouseMotionDirection::Up),
            down: InputKind::MouseMotion(MouseMotionDirection::Down),
            left: InputKind::MouseMotion(MouseMotionDirection::Left),
            right: InputKind::MouseMotion(MouseMotionDirection::Right),
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
    /// Input associated with movement of the mouse
    MouseMotion(MouseMotionAxisType),
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

/// The direction of motion of the mouse.
///
/// Stored in the [`AxisType`] enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseMotionAxisType {
    /// Horizontal movement.
    X,
    /// Vertical movement.
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

impl From<MouseMotionAxisType> for AxisType {
    fn from(axis_type: MouseMotionAxisType) -> Self {
        AxisType::MouseMotion(axis_type)
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

impl TryFrom<AxisType> for MouseMotionAxisType {
    type Error = AxisConversionError;

    fn try_from(axis_type: AxisType) -> Result<Self, AxisConversionError> {
        match axis_type {
            AxisType::MouseMotion(inner) => Ok(inner),
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
    pub fn new(x: f32, y: f32) -> DualAxisData {
        DualAxisData {
            xy: Vec2::new(x, y),
        }
    }

    /// Creates a new [`AxisPair`] directly from a [`Vec2`]
    pub fn from_xy(xy: Vec2) -> DualAxisData {
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
