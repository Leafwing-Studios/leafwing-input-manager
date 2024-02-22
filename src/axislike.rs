//! Tools for working with directional axis-like user inputs (gamesticks, D-Pads and emulated equivalents)

use crate::buttonlike::{MouseMotionDirection, MouseWheelDirection};
use crate::orientation::{Direction, Rotation};
use crate::user_input::InputKind;
use bevy::input::{
    gamepad::{GamepadAxisType, GamepadButtonType},
    keyboard::KeyCode,
};
use bevy::math::Vec2;
use bevy::reflect::Reflect;
use bevy::utils::FloatOrd;
use serde::{Deserialize, Serialize};

/// A single directional axis with a configurable trigger zone.
///
/// These can be stored in a [`InputKind`] to create a virtual button.
///
/// # Warning
///
/// `positive_low` must be greater than or equal to `negative_low` for this type to be validly constructed.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Reflect)]
pub struct SingleAxis {
    /// The axis that is being checked.
    pub axis_type: AxisType,
    /// Any axis value higher than this will trigger the input.
    pub positive_low: f32,
    /// Any axis value lower than this will trigger the input.
    pub negative_low: f32,
    /// Whether to invert output values from this axis.
    pub inverted: bool,
    /// How sensitive the axis is to input values.
    ///
    /// Since sensitivity is a multiplier, any value `>1.0` will increase sensitivity while any value `<1.0` will decrease sensitivity.
    /// This value should always be strictly positive: a value of 0 will cause the axis to stop functioning,
    /// while negative values will invert the direction.
    pub sensitivity: f32,
    /// The target value for this input, used for input mocking.
    ///
    /// WARNING: this field is ignored for the sake of [`Eq`] and [`Hash`](std::hash::Hash)
    pub value: Option<f32>,
}

impl SingleAxis {
    /// Creates a [`SingleAxis`] with the given `positive_low` and `negative_low` thresholds.
    #[must_use]
    pub const fn with_threshold(
        axis_type: AxisType,
        negative_low: f32,
        positive_low: f32,
    ) -> SingleAxis {
        SingleAxis {
            axis_type,
            positive_low,
            negative_low,
            inverted: false,
            sensitivity: 1.0,
            value: None,
        }
    }

    /// Creates a [`SingleAxis`] with both `positive_low` and `negative_low` set to `threshold`.
    #[must_use]
    pub fn symmetric(axis_type: impl Into<AxisType>, threshold: f32) -> SingleAxis {
        Self::with_threshold(axis_type.into(), -threshold, threshold)
    }

    /// Creates a [`SingleAxis`] with the specified `axis_type` and `value`.
    ///
    /// All thresholds are set to 0.0.
    /// Primarily useful for [input mocking](crate::input_mocking).
    #[must_use]
    pub fn from_value(axis_type: impl Into<AxisType>, value: f32) -> SingleAxis {
        SingleAxis {
            axis_type: axis_type.into(),
            positive_low: 0.0,
            negative_low: 0.0,
            inverted: false,
            sensitivity: 1.0,
            value: Some(value),
        }
    }

    /// Creates a [`SingleAxis`] corresponding to horizontal [`MouseWheel`](bevy::input::mouse::MouseWheel) movement
    #[must_use]
    pub const fn mouse_wheel_x() -> SingleAxis {
        let axis_type = AxisType::MouseWheel(MouseWheelAxisType::X);
        SingleAxis::with_threshold(axis_type, 0.0, 0.0)
    }

    /// Creates a [`SingleAxis`] corresponding to vertical [`MouseWheel`](bevy::input::mouse::MouseWheel) movement
    #[must_use]
    pub const fn mouse_wheel_y() -> SingleAxis {
        let axis_type = AxisType::MouseWheel(MouseWheelAxisType::Y);
        SingleAxis::with_threshold(axis_type, 0.0, 0.0)
    }

    /// Creates a [`SingleAxis`] corresponding to horizontal [`MouseMotion`](bevy::input::mouse::MouseMotion) movement
    #[must_use]
    pub const fn mouse_motion_x() -> SingleAxis {
        let axis_type = AxisType::MouseMotion(MouseMotionAxisType::X);
        SingleAxis::with_threshold(axis_type, 0.0, 0.0)
    }

    /// Creates a [`SingleAxis`] corresponding to vertical [`MouseMotion`](bevy::input::mouse::MouseMotion) movement
    #[must_use]
    pub const fn mouse_motion_y() -> SingleAxis {
        let axis_type = AxisType::MouseMotion(MouseMotionAxisType::Y);
        SingleAxis::with_threshold(axis_type, 0.0, 0.0)
    }

    /// Creates a [`SingleAxis`] with the `axis_type` and `negative_low` set to `threshold`.
    ///
    /// Positive values will not trigger the input.
    pub fn negative_only(axis_type: impl Into<AxisType>, threshold: f32) -> SingleAxis {
        SingleAxis::with_threshold(axis_type.into(), threshold, f32::MAX)
    }

    /// Creates a [`SingleAxis`] with the `axis_type` and `positive_low` set to `threshold`.
    ///
    /// Negative values will not trigger the input.
    pub fn positive_only(axis_type: impl Into<AxisType>, threshold: f32) -> SingleAxis {
        SingleAxis::with_threshold(axis_type.into(), f32::MIN, threshold)
    }

    /// Returns this [`SingleAxis`] with the deadzone set to the specified value
    #[must_use]
    pub fn with_deadzone(mut self, deadzone: f32) -> SingleAxis {
        self.negative_low = -deadzone;
        self.positive_low = deadzone;
        self
    }

    /// Returns this [`SingleAxis`] with the sensitivity set to the specified value
    #[must_use]
    pub fn with_sensitivity(mut self, sensitivity: f32) -> SingleAxis {
        self.sensitivity = sensitivity;
        self
    }

    /// Returns this [`SingleAxis`] inverted.
    #[must_use]
    pub fn inverted(mut self) -> Self {
        self.inverted = !self.inverted;
        self
    }
}

impl PartialEq for SingleAxis {
    fn eq(&self, other: &Self) -> bool {
        self.axis_type == other.axis_type
            && FloatOrd(self.positive_low) == FloatOrd(other.positive_low)
            && FloatOrd(self.negative_low) == FloatOrd(other.negative_low)
            && FloatOrd(self.sensitivity) == FloatOrd(other.sensitivity)
    }
}

impl Eq for SingleAxis {}

impl std::hash::Hash for SingleAxis {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.axis_type.hash(state);
        FloatOrd(self.positive_low).hash(state);
        FloatOrd(self.negative_low).hash(state);
        FloatOrd(self.sensitivity).hash(state);
    }
}

/// Two directional axes combined as one input.
///
/// These can be stored in a [`VirtualDPad`], which is itself stored in an [`InputKind`] for consumption.
///
/// This input will generate a [`DualAxis`] which can be read with
/// [`ActionState::axis_pair`][crate::action_state::ActionState::axis_pair].
///
/// # Warning
///
/// `positive_low` must be greater than or equal to `negative_low` for both `x` and `y` for this type to be validly constructed.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Reflect)]
pub struct DualAxis {
    /// The axis representing horizontal movement.
    pub x: SingleAxis,
    /// The axis representing vertical movement.
    pub y: SingleAxis,
    /// The shape of the deadzone
    pub deadzone: DeadZoneShape,
}

impl DualAxis {
    /// The default size of the deadzone used by constructor methods.
    ///
    /// This cannot be changed, but the struct can be easily manually constructed.
    pub const DEFAULT_DEADZONE: f32 = 0.1;

    /// The default shape of the deadzone used by constructor methods.
    ///
    /// This cannot be changed, but the struct can be easily manually constructed.
    pub const DEFAULT_DEADZONE_SHAPE: DeadZoneShape = DeadZoneShape::Ellipse {
        radius_x: Self::DEFAULT_DEADZONE,
        radius_y: Self::DEFAULT_DEADZONE,
    };

    /// A deadzone with a size of 0.0 used by constructor methods.
    ///
    /// This cannot be changed, but the struct can be easily manually constructed.
    pub const ZERO_DEADZONE_SHAPE: DeadZoneShape = DeadZoneShape::Ellipse {
        radius_x: 0.0,
        radius_y: 0.0,
    };

    /// Creates a [`DualAxis`] with both `positive_low` and `negative_low` in both axes set to `threshold` with a `deadzone_shape`.
    #[must_use]
    pub fn symmetric(
        x_axis_type: impl Into<AxisType>,
        y_axis_type: impl Into<AxisType>,
        deadzone_shape: DeadZoneShape,
    ) -> DualAxis {
        DualAxis {
            x: SingleAxis::symmetric(x_axis_type, 0.0),
            y: SingleAxis::symmetric(y_axis_type, 0.0),
            deadzone: deadzone_shape,
        }
    }

    /// Creates a [`SingleAxis`] with the specified `axis_type` and `value`.
    ///
    /// All thresholds are set to 0.0.
    /// Primarily useful for [input mocking](crate::input_mocking).
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
            deadzone: Self::DEFAULT_DEADZONE_SHAPE,
        }
    }

    /// Creates a [`DualAxis`] for the left analogue stick of the gamepad.
    #[must_use]
    pub const fn left_stick() -> DualAxis {
        let axis_x = AxisType::Gamepad(GamepadAxisType::LeftStickX);
        let axis_y = AxisType::Gamepad(GamepadAxisType::LeftStickY);
        DualAxis {
            x: SingleAxis::with_threshold(axis_x, 0.0, 0.0),
            y: SingleAxis::with_threshold(axis_y, 0.0, 0.0),
            deadzone: Self::DEFAULT_DEADZONE_SHAPE,
        }
    }

    /// Creates a [`DualAxis`] for the right analogue stick of the gamepad.
    #[must_use]
    pub const fn right_stick() -> DualAxis {
        let axis_x = AxisType::Gamepad(GamepadAxisType::RightStickX);
        let axis_y = AxisType::Gamepad(GamepadAxisType::RightStickY);
        DualAxis {
            x: SingleAxis::with_threshold(axis_x, 0.0, 0.0),
            y: SingleAxis::with_threshold(axis_y, 0.0, 0.0),
            deadzone: Self::DEFAULT_DEADZONE_SHAPE,
        }
    }

    /// Creates a [`DualAxis`] corresponding to horizontal and vertical [`MouseWheel`](bevy::input::mouse::MouseWheel) movement
    pub const fn mouse_wheel() -> DualAxis {
        DualAxis {
            x: SingleAxis::mouse_wheel_x(),
            y: SingleAxis::mouse_wheel_y(),
            deadzone: Self::ZERO_DEADZONE_SHAPE,
        }
    }

    /// Creates a [`DualAxis`] corresponding to horizontal and vertical [`MouseMotion`](bevy::input::mouse::MouseMotion) movement
    pub const fn mouse_motion() -> DualAxis {
        DualAxis {
            x: SingleAxis::mouse_motion_x(),
            y: SingleAxis::mouse_motion_y(),
            deadzone: Self::ZERO_DEADZONE_SHAPE,
        }
    }

    /// Returns this [`DualAxis`] with the deadzone set to the specified values and shape
    #[must_use]
    pub fn with_deadzone(mut self, deadzone: DeadZoneShape) -> DualAxis {
        self.deadzone = deadzone;
        self
    }

    /// Returns this [`DualAxis`] with the sensitivity set to the specified values
    #[must_use]
    pub fn with_sensitivity(mut self, x_sensitivity: f32, y_sensitivity: f32) -> DualAxis {
        self.x.sensitivity = x_sensitivity;
        self.y.sensitivity = y_sensitivity;
        self
    }

    /// Returns this [`DualAxis`] with an inverted X-axis.
    #[must_use]
    pub fn inverted_x(mut self) -> DualAxis {
        self.x = self.x.inverted();
        self
    }

    /// Returns this [`DualAxis`] with an inverted Y-axis.
    #[must_use]
    pub fn inverted_y(mut self) -> DualAxis {
        self.y = self.y.inverted();
        self
    }

    /// Returns this [`DualAxis`] with both axes inverted.
    #[must_use]
    pub fn inverted(mut self) -> DualAxis {
        self.x = self.x.inverted();
        self.y = self.y.inverted();
        self
    }
}

#[allow(clippy::doc_markdown)] // False alarm because it thinks DPad is an unquoted item
/// A virtual DPad that you can get an [`DualAxis`] from.
///
/// Typically, you don't want to store a [`DualAxis`] in this type,
/// even though it can be stored as an [`InputKind`].
///
/// Instead, use it directly as [`InputKind::DualAxis`]!
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
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
            up: InputKind::PhysicalKey(KeyCode::ArrowUp),
            down: InputKind::PhysicalKey(KeyCode::ArrowDown),
            left: InputKind::PhysicalKey(KeyCode::ArrowLeft),
            right: InputKind::PhysicalKey(KeyCode::ArrowRight),
        }
    }

    /// Generates a [`VirtualDPad`] corresponding to the `WASD` keys on the standard US QWERTY layout.
    ///
    /// Note that on other keyboard layouts, different keys need to be pressed.
    /// The _location_ of the keys is the same on all keyboard layouts.
    /// This ensures that the classic triangular shape is retained on all layouts,
    /// which enables comfortable movement controls.
    pub const fn wasd() -> VirtualDPad {
        VirtualDPad {
            up: InputKind::PhysicalKey(KeyCode::KeyW),
            down: InputKind::PhysicalKey(KeyCode::KeyS),
            left: InputKind::PhysicalKey(KeyCode::KeyA),
            right: InputKind::PhysicalKey(KeyCode::KeyD),
        }
    }

    #[allow(clippy::doc_markdown)] // False alarm because it thinks DPad is an unquoted item
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
    /// North corresponds to up, west corresponds to left,
    /// east corresponds to right, and south corresponds to down
    pub const fn gamepad_face_buttons() -> VirtualDPad {
        VirtualDPad {
            up: InputKind::GamepadButton(GamepadButtonType::North),
            down: InputKind::GamepadButton(GamepadButtonType::South),
            left: InputKind::GamepadButton(GamepadButtonType::West),
            right: InputKind::GamepadButton(GamepadButtonType::East),
        }
    }

    /// Generates a [`VirtualDPad`] corresponding to discretized mousewheel movements
    pub const fn mouse_wheel() -> VirtualDPad {
        VirtualDPad {
            up: InputKind::MouseWheel(MouseWheelDirection::Up),
            down: InputKind::MouseWheel(MouseWheelDirection::Down),
            left: InputKind::MouseWheel(MouseWheelDirection::Left),
            right: InputKind::MouseWheel(MouseWheelDirection::Right),
        }
    }

    /// Generates a [`VirtualDPad`] corresponding to discretized mouse motions
    pub const fn mouse_motion() -> VirtualDPad {
        VirtualDPad {
            up: InputKind::MouseMotion(MouseMotionDirection::Up),
            down: InputKind::MouseMotion(MouseMotionDirection::Down),
            left: InputKind::MouseMotion(MouseMotionDirection::Left),
            right: InputKind::MouseMotion(MouseMotionDirection::Right),
        }
    }

    /// Returns this [`VirtualDPad`] but with `up` and `down` swapped.
    pub fn inverted_y(mut self) -> Self {
        std::mem::swap(&mut self.up, &mut self.down);
        self
    }

    /// Returns this [`VirtualDPad`] but with `left` and `right` swapped.
    pub fn inverted_x(mut self) -> Self {
        std::mem::swap(&mut self.left, &mut self.right);
        self
    }

    /// Returns this [`VirtualDPad`] but with inverted inputs.
    pub fn inverted(mut self) -> Self {
        std::mem::swap(&mut self.up, &mut self.down);
        std::mem::swap(&mut self.left, &mut self.right);
        self
    }
}

/// A virtual Axis that you can get a value between -1 and 1 from.
///
/// Typically, you don't want to store a [`SingleAxis`] in this type,
/// even though it can be stored as an [`InputKind`].
///
/// Instead, use it directly as [`InputKind::SingleAxis`]!
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub struct VirtualAxis {
    /// The input that represents the negative direction of this virtual axis
    pub negative: InputKind,
    /// The input that represents the positive direction of this virtual axis
    pub positive: InputKind,
}

impl VirtualAxis {
    /// Helper function for generating a [`VirtualAxis`] from arbitrary keycodes, shorthand for
    /// wrapping each key in [`InputKind::PhysicalKey`]
    pub const fn from_keys(negative: KeyCode, positive: KeyCode) -> VirtualAxis {
        VirtualAxis {
            negative: InputKind::PhysicalKey(negative),
            positive: InputKind::PhysicalKey(positive),
        }
    }

    /// Generates a [`VirtualAxis`] corresponding to the horizontal arrow keyboard keycodes
    pub const fn horizontal_arrow_keys() -> VirtualAxis {
        VirtualAxis::from_keys(KeyCode::ArrowLeft, KeyCode::ArrowRight)
    }

    /// Generates a [`VirtualAxis`] corresponding to the horizontal arrow keyboard keycodes
    pub const fn vertical_arrow_keys() -> VirtualAxis {
        VirtualAxis::from_keys(KeyCode::ArrowDown, KeyCode::ArrowUp)
    }

    /// Generates a [`VirtualAxis`] corresponding to the `AD` keyboard keycodes.
    pub const fn ad() -> VirtualAxis {
        VirtualAxis::from_keys(KeyCode::KeyA, KeyCode::KeyD)
    }

    /// Generates a [`VirtualAxis`] corresponding to the `WS` keyboard keycodes.
    pub const fn ws() -> VirtualAxis {
        VirtualAxis::from_keys(KeyCode::KeyS, KeyCode::KeyW)
    }

    #[allow(clippy::doc_markdown)]
    /// Generates a [`VirtualAxis`] corresponding to the horizontal DPad buttons on a gamepad.
    pub const fn horizontal_dpad() -> VirtualAxis {
        VirtualAxis {
            negative: InputKind::GamepadButton(GamepadButtonType::DPadLeft),
            positive: InputKind::GamepadButton(GamepadButtonType::DPadRight),
        }
    }

    #[allow(clippy::doc_markdown)]
    /// Generates a [`VirtualAxis`] corresponding to the vertical DPad buttons on a gamepad.
    pub const fn vertical_dpad() -> VirtualAxis {
        VirtualAxis {
            negative: InputKind::GamepadButton(GamepadButtonType::DPadDown),
            positive: InputKind::GamepadButton(GamepadButtonType::DPadUp),
        }
    }

    /// Generates a [`VirtualAxis`] corresponding to the horizontal gamepad face buttons.
    pub const fn horizontal_gamepad_face_buttons() -> VirtualAxis {
        VirtualAxis {
            negative: InputKind::GamepadButton(GamepadButtonType::West),
            positive: InputKind::GamepadButton(GamepadButtonType::East),
        }
    }

    /// Generates a [`VirtualAxis`] corresponding to the vertical gamepad face buttons.
    pub const fn vertical_gamepad_face_buttons() -> VirtualAxis {
        VirtualAxis {
            negative: InputKind::GamepadButton(GamepadButtonType::South),
            positive: InputKind::GamepadButton(GamepadButtonType::North),
        }
    }

    /// Returns this [`VirtualAxis`] but with flipped positive/negative inputs.
    #[must_use]
    pub fn inverted(mut self) -> Self {
        std::mem::swap(&mut self.positive, &mut self.negative);
        self
    }
}

/// The type of axis used by a [`UserInput`](crate::user_input::UserInput).
///
/// This is stored in either a [`SingleAxis`] or [`DualAxis`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum AxisType {
    /// Input associated with a gamepad, such as the triggers or one axis of an analog stick.
    Gamepad(GamepadAxisType),
    /// Input associated with a mouse wheel.
    MouseWheel(MouseWheelAxisType),
    /// Input associated with movement of the mouse
    MouseMotion(MouseMotionAxisType),
}

/// The motion direction of the mouse wheel.
///
/// Stored in the [`AxisType`] enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
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

/// The motion direction of the mouse.
///
/// Stored in the [`AxisType`] enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
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

    /// The [`Direction`] that this axis is pointing towards, if any
    ///
    /// If the axis is neutral (x,y) = (0,0), a (0, 0) `None` will be returned
    #[must_use]
    #[inline]
    pub fn direction(&self) -> Option<Direction> {
        // TODO: replace this quick-n-dirty hack once Direction::new no longer panics
        (self.xy.length() > 0.00001).then(|| Direction::new(self.xy))
    }

    /// The [`Rotation`] (measured clockwise from midnight) that this axis is pointing towards, if any
    ///
    /// If the axis is neutral (x,y) = (0,0), this will be `None`
    #[must_use]
    #[inline]
    pub fn rotation(&self) -> Option<Rotation> {
        Rotation::from_xy(self.xy).ok()
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

/// The shape of the deadzone for a [`DualAxis`] input.
///
/// Input values that are on the boundary of the shape are counted as inside.
/// If the size of a shape is 0.0, then all input values are read, except for 0.0.
///
/// All inputs are scaled to be continuous.
/// So with an ellipse deadzone with a radius of 0.1, the input range `0.1..=1.0` will be scaled to `0.0..=1.0`.
///
/// Deadzone values should be in the range `0.0..=1.0`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Reflect)]
pub enum DeadZoneShape {
    /// Deadzone with the shape of a cross.
    ///
    /// The cross is represented by horizontal and vertical rectangles.
    /// Each axis is handled separately which creates a per-axis "snapping" effect.
    Cross {
        /// The width of the horizontal axis.
        ///
        /// Affects the snapping of the y-axis.
        horizontal_width: f32,
        /// The width of the vertical axis.
        ///
        /// Affects the snapping of the x-axis.
        vertical_width: f32,
    },
    /// Deadzone with the shape of an ellipse.
    Ellipse {
        /// The horizontal radius of the ellipse.
        radius_x: f32,
        /// The vertical radius of the ellipse.
        radius_y: f32,
    },
}

impl Eq for DeadZoneShape {}

impl std::hash::Hash for DeadZoneShape {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            DeadZoneShape::Cross {
                horizontal_width,
                vertical_width,
            } => {
                FloatOrd(*horizontal_width).hash(state);
                FloatOrd(*vertical_width).hash(state);
            }
            DeadZoneShape::Ellipse { radius_x, radius_y } => {
                FloatOrd(*radius_x).hash(state);
                FloatOrd(*radius_y).hash(state);
            }
        }
    }
}

impl DeadZoneShape {
    /// Computes the input value based on the deadzone.
    pub fn deadzone_input_value(&self, x: f32, y: f32) -> Option<DualAxisData> {
        match self {
            DeadZoneShape::Cross {
                horizontal_width,
                vertical_width,
            } => self.cross_deadzone_value(x, y, *horizontal_width, *vertical_width),
            DeadZoneShape::Ellipse { radius_x, radius_y } => {
                self.ellipse_deadzone_value(x, y, *radius_x, *radius_y)
            }
        }
    }

    /// Computes the input value based on the cross deadzone.
    fn cross_deadzone_value(
        &self,
        x: f32,
        y: f32,
        horizontal_width: f32,
        vertical_width: f32,
    ) -> Option<DualAxisData> {
        let new_x = deadzone_axis_value(x, vertical_width);
        let new_y = deadzone_axis_value(y, horizontal_width);
        let is_outside_deadzone = new_x != 0.0 || new_y != 0.0;
        is_outside_deadzone.then(|| DualAxisData::new(new_x, new_y))
    }

    /// Computes the input value based on the ellipse deadzone.
    fn ellipse_deadzone_value(
        &self,
        x: f32,
        y: f32,
        radius_x: f32,
        radius_y: f32,
    ) -> Option<DualAxisData> {
        let x_ratio = x / radius_x.max(f32::EPSILON);
        let y_ratio = y / radius_y.max(f32::EPSILON);
        let is_outside_deadzone = x_ratio.powi(2) + y_ratio.powi(2) >= 1.0;
        is_outside_deadzone.then(|| {
            let new_x = deadzone_axis_value(x, radius_x);
            let new_y = deadzone_axis_value(y, radius_y);
            DualAxisData::new(new_x, new_y)
        })
    }
}

/// Applies the given deadzone to the axis value.
///
/// Returns 0.0 if the axis value is within the deadzone.
/// Otherwise, returns the normalized axis value between -1.0 and 1.0.
pub(crate) fn deadzone_axis_value(axis_value: f32, deadzone: f32) -> f32 {
    let abs_axis_value = axis_value.abs();
    if abs_axis_value <= deadzone {
        0.0
    } else {
        axis_value.signum() * (abs_axis_value - deadzone) / (1.0 - deadzone)
    }
}
