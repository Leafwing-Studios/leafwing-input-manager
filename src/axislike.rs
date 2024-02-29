//! Tools for working with directional axis-like user inputs (game sticks, D-Pads and emulated equivalents)

use crate::buttonlike::{MouseMotionDirection, MouseWheelDirection};
use crate::input_settings::{
    Deadzone2, DualAxisSettings, SingleAxisSettings, ValueLimit, DEFAULT_DEADZONE_UPPER,
};
use crate::orientation::Rotation;
use crate::user_input::InputKind;
use bevy::input::{
    gamepad::{GamepadAxisType, GamepadButtonType},
    keyboard::KeyCode,
};
use bevy::math::primitives::Direction2d;
use bevy::math::Vec2;
use bevy::reflect::Reflect;
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

    /// The input settings.
    pub settings: SingleAxisSettings,

    /// The target value for this input, used for input mocking.
    ///
    /// WARNING: this field is ignored for the sake of [`Eq`] and [`Hash`](std::hash::Hash)
    pub value: Option<f32>,
}

impl SingleAxis {
    /// Creates a [`SingleAxis`] with both `positive_low` and `negative_low` set to `threshold`.
    #[must_use]
    pub fn symmetric(axis_type: impl Into<AxisType>, threshold: f32) -> SingleAxis {
        Self {
            axis_type: axis_type.into(),
            settings: SingleAxisSettings::NO_DEADZONE.with_symmetric_deadzone(threshold),
            value: None,
        }
    }

    /// Creates a [`SingleAxis`] with the specified `axis_type` and `value`.
    ///
    /// All thresholds are set to 0.0.
    /// Primarily useful for [input mocking](crate::input_mocking).
    #[must_use]
    pub fn from_value(axis_type: impl Into<AxisType>, value: f32) -> SingleAxis {
        SingleAxis {
            axis_type: axis_type.into(),
            settings: SingleAxisSettings::NO_DEADZONE,
            value: Some(value),
        }
    }

    /// Creates a [`SingleAxis`] corresponding to horizontal [`MouseWheel`](bevy::input::mouse::MouseWheel) movement
    #[must_use]
    pub const fn mouse_wheel_x() -> SingleAxis {
        SingleAxis {
            axis_type: AxisType::MouseWheel(MouseWheelAxisType::X),
            settings: SingleAxisSettings::NO_DEADZONE,
            value: None,
        }
    }

    /// Creates a [`SingleAxis`] corresponding to vertical [`MouseWheel`](bevy::input::mouse::MouseWheel) movement
    #[must_use]
    pub const fn mouse_wheel_y() -> SingleAxis {
        SingleAxis {
            axis_type: AxisType::MouseWheel(MouseWheelAxisType::Y),
            settings: SingleAxisSettings::NO_DEADZONE,
            value: None,
        }
    }

    /// Creates a [`SingleAxis`] corresponding to horizontal [`MouseMotion`](bevy::input::mouse::MouseMotion) movement
    #[must_use]
    pub const fn mouse_motion_x() -> SingleAxis {
        SingleAxis {
            axis_type: AxisType::MouseMotion(MouseMotionAxisType::X),
            settings: SingleAxisSettings::NO_DEADZONE,
            value: None,
        }
    }

    /// Creates a [`SingleAxis`] corresponding to vertical [`MouseMotion`](bevy::input::mouse::MouseMotion) movement
    #[must_use]
    pub const fn mouse_motion_y() -> SingleAxis {
        SingleAxis {
            axis_type: AxisType::MouseMotion(MouseMotionAxisType::Y),
            settings: SingleAxisSettings::NO_DEADZONE,
            value: None,
        }
    }

    /// Creates a [`SingleAxis`] with the `axis_type` and `negative_low` set to `threshold`.
    ///
    /// Positive values will not trigger the input.
    pub fn negative_only(axis_type: impl Into<AxisType>, negative_low: f32) -> SingleAxis {
        let limit = ValueLimit::AtMost(negative_low);
        Self {
            axis_type: axis_type.into(),
            settings: SingleAxisSettings::NO_DEADZONE.with_input_limit(limit),
            value: None,
        }
    }

    /// Creates a [`SingleAxis`] with the `axis_type` and `positive_low` set to `threshold`.
    ///
    /// Negative values will not trigger the input.
    pub fn positive_only(axis_type: impl Into<AxisType>, positive_low: f32) -> SingleAxis {
        let limit = ValueLimit::AtLeast(positive_low);
        Self {
            axis_type: axis_type.into(),
            settings: SingleAxisSettings::NO_DEADZONE.with_input_limit(limit),
            value: None,
        }
    }

    /// Replaces the internal settings of the [`SingleAxis`] using the given `settings_replacer`.
    ///
    /// # Arguments
    ///
    /// - `settings_replacer`: A closure that takes a mutable reference to [`SingleAxisSettings`] and replaces it.
    #[inline]
    pub fn with_settings<F>(mut self, mut settings_replacer: F) -> Self
    where
        F: FnMut(&mut SingleAxisSettings) -> SingleAxisSettings,
    {
        self.settings = settings_replacer(&mut self.settings);
        self
    }
}

impl PartialEq for SingleAxis {
    fn eq(&self, other: &Self) -> bool {
        self.axis_type == other.axis_type && self.settings == other.settings
    }
}

impl Eq for SingleAxis {}

impl std::hash::Hash for SingleAxis {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.axis_type.hash(state);
        self.settings.hash(state);
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
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Reflect)]
pub struct DualAxis {
    /// The horizontal axis that is being checked.
    pub x_axis_type: AxisType,

    /// The vertical axis that is being checked.
    pub y_axis_type: AxisType,

    /// The input settings.
    pub settings: DualAxisSettings,

    /// The target value for this input, used for input mocking.
    ///
    /// WARNING: this field is ignored for the sake of [`Eq`] and [`Hash`](std::hash::Hash)
    pub value: Option<Vec2>,
}

impl DualAxis {
    /// The default size of the deadzone used by constructor methods.
    ///
    /// This cannot be changed, but the struct can be easily manually constructed.
    pub const DEFAULT_DEADZONE: f32 = DEFAULT_DEADZONE_UPPER;

    /// The default shape of the deadzone used by constructor methods.
    ///
    /// This cannot be changed, but the struct can be easily manually constructed.
    pub const DEFAULT_DEADZONE_SHAPE: Deadzone2 = Deadzone2::CIRCLE_DEFAULT;

    /// A deadzone with a size of 0.0 used by constructor methods.
    ///
    /// This cannot be changed, but the struct can be easily manually constructed.
    pub const ZERO_DEADZONE_SHAPE: Deadzone2 = Deadzone2::None;

    /// Creates a [`DualAxis`] with both `positive_low` and `negative_low` in both axes set to `threshold` with a `deadzone_shape`.
    #[must_use]
    pub fn symmetric(
        x_axis_type: impl Into<AxisType>,
        y_axis_type: impl Into<AxisType>,
        deadzone: Deadzone2,
    ) -> DualAxis {
        DualAxis {
            x_axis_type: x_axis_type.into(),
            y_axis_type: y_axis_type.into(),
            settings: DualAxisSettings::NO_DEADZONE.with_deadzone(deadzone),
            value: None,
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
            x_axis_type: x_axis_type.into(),
            y_axis_type: y_axis_type.into(),
            settings: DualAxisSettings::CIRCLE_DEFAULT,
            value: Some(Vec2::new(x_value, y_value)),
        }
    }

    /// Creates a [`DualAxis`] for the left analogue stick of the gamepad.
    #[must_use]
    pub const fn left_stick() -> DualAxis {
        DualAxis {
            x_axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickX),
            y_axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickY),
            settings: DualAxisSettings::CIRCLE_DEFAULT,
            value: None,
        }
    }

    /// Creates a [`DualAxis`] for the right analogue stick of the gamepad.
    #[must_use]
    pub const fn right_stick() -> DualAxis {
        DualAxis {
            x_axis_type: AxisType::Gamepad(GamepadAxisType::RightStickX),
            y_axis_type: AxisType::Gamepad(GamepadAxisType::RightStickY),
            settings: DualAxisSettings::CIRCLE_DEFAULT,
            value: None,
        }
    }

    /// Creates a [`DualAxis`] corresponding to horizontal and vertical [`MouseWheel`](bevy::input::mouse::MouseWheel) movement
    pub const fn mouse_wheel() -> DualAxis {
        DualAxis {
            x_axis_type: AxisType::MouseWheel(MouseWheelAxisType::X),
            y_axis_type: AxisType::MouseWheel(MouseWheelAxisType::Y),
            settings: DualAxisSettings::NO_DEADZONE,
            value: None,
        }
    }

    /// Creates a [`DualAxis`] corresponding to horizontal and vertical [`MouseMotion`](bevy::input::mouse::MouseMotion) movement
    pub const fn mouse_motion() -> DualAxis {
        DualAxis {
            x_axis_type: AxisType::MouseMotion(MouseMotionAxisType::X),
            y_axis_type: AxisType::MouseMotion(MouseMotionAxisType::Y),
            settings: DualAxisSettings::NO_DEADZONE,
            value: None,
        }
    }

    /// Replaces the internal settings of the [`DualAxis`] using the given `settings_replacer`.
    ///
    /// # Arguments
    ///
    /// - `settings_replacer`: A closure that takes a mutable reference to [`DualAxisSettings`] and replaces it.
    #[inline]
    pub fn with_settings<F>(mut self, mut settings_replacer: F) -> Self
    where
        F: FnMut(&mut DualAxisSettings) -> DualAxisSettings,
    {
        self.settings = settings_replacer(&mut self.settings);
        self
    }
}

impl PartialEq for DualAxis {
    fn eq(&self, other: &Self) -> bool {
        self.x_axis_type == other.x_axis_type
            && self.y_axis_type == other.y_axis_type
            && self.settings == other.settings
    }
}

impl Eq for DualAxis {}

impl std::hash::Hash for DualAxis {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x_axis_type.hash(state);
        self.y_axis_type.hash(state);
        self.settings.hash(state);
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

    /// The [`Direction2d`] that this axis is pointing towards, if any
    ///
    /// If the axis is neutral (x,y) = (0,0), a (0, 0) `None` will be returned
    #[must_use]
    #[inline]
    pub fn direction(&self) -> Option<Direction2d> {
        Direction2d::new(self.xy).ok()
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
