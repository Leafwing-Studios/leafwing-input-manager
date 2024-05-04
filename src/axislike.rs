//! Tools for working with directional axis-like user inputs (game sticks, D-Pads and emulated equivalents)

use bevy::prelude::{Direction2d, GamepadAxisType, GamepadButtonType, KeyCode, Reflect, Vec2};
use serde::{Deserialize, Serialize};

use crate::buttonlike::{MouseMotionDirection, MouseWheelDirection};
use crate::input_processing::*;
use crate::orientation::Rotation;
use crate::user_input::InputKind;

/// A single directional axis with a configurable trigger zone.
///
/// These can be stored in a [`InputKind`] to create a virtual button.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct SingleAxis {
    /// The axis that is being checked.
    pub axis_type: AxisType,

    /// The processor used to handle input values.
    pub processor: AxisProcessor,
}

impl SingleAxis {
    /// Creates a [`SingleAxis`] with the specified axis type.
    #[must_use]
    pub fn new(axis_type: impl Into<AxisType>) -> Self {
        Self {
            axis_type: axis_type.into(),
            processor: AxisProcessor::None,
        }
    }

    /// Creates a [`SingleAxis`] corresponding to horizontal [`MouseWheel`](bevy::input::mouse::MouseWheel) movement
    #[must_use]
    pub const fn mouse_wheel_x() -> Self {
        Self {
            axis_type: AxisType::MouseWheel(MouseWheelAxisType::X),
            processor: AxisProcessor::None,
        }
    }

    /// Creates a [`SingleAxis`] corresponding to vertical [`MouseWheel`](bevy::input::mouse::MouseWheel) movement
    #[must_use]
    pub const fn mouse_wheel_y() -> Self {
        Self {
            axis_type: AxisType::MouseWheel(MouseWheelAxisType::Y),
            processor: AxisProcessor::None,
        }
    }

    /// Creates a [`SingleAxis`] corresponding to horizontal [`MouseMotion`](bevy::input::mouse::MouseMotion) movement
    #[must_use]
    pub const fn mouse_motion_x() -> Self {
        Self {
            axis_type: AxisType::MouseMotion(MouseMotionAxisType::X),
            processor: AxisProcessor::None,
        }
    }

    /// Creates a [`SingleAxis`] corresponding to vertical [`MouseMotion`](bevy::input::mouse::MouseMotion) movement
    #[must_use]
    pub const fn mouse_motion_y() -> Self {
        Self {
            axis_type: AxisType::MouseMotion(MouseMotionAxisType::Y),
            processor: AxisProcessor::None,
        }
    }

    /// Get the "value" of this axis.
    /// If a processor is set, it will compute and return the processed value.
    /// Otherwise, pass the `input_value` through unchanged.
    #[must_use]
    #[inline]
    pub fn input_value(&self, input_value: f32) -> f32 {
        self.processor.process(input_value)
    }
}

impl WithAxisProcessingPipelineExt for SingleAxis {
    #[inline]
    fn reset_processing_pipeline(mut self) -> Self {
        self.processor = AxisProcessor::None;
        self
    }

    #[inline]
    fn replace_processing_pipeline(mut self, processor: impl Into<AxisProcessor>) -> Self {
        self.processor = processor.into();
        self
    }

    #[inline]
    fn with_processor(mut self, processor: impl Into<AxisProcessor>) -> Self {
        self.processor = self.processor.with_processor(processor);
        self
    }
}

impl PartialEq for SingleAxis {
    fn eq(&self, other: &Self) -> bool {
        self.axis_type == other.axis_type && self.processor == other.processor
    }
}

impl Eq for SingleAxis {}

impl std::hash::Hash for SingleAxis {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.axis_type.hash(state);
        self.processor.hash(state);
    }
}

/// Two directional axes combined as one input.
///
/// These can be stored in a [`VirtualDPad`], which is itself stored in an [`InputKind`] for consumption.
///
/// This input will generate a [`DualAxis`] which can be read with
/// [`ActionState::axis_pair`][crate::action_state::ActionState::axis_pair].
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct DualAxis {
    /// The horizontal axis that is being checked.
    pub x_axis_type: AxisType,

    /// The vertical axis that is being checked.
    pub y_axis_type: AxisType,

    /// The processor used to handle input values.
    pub processor: DualAxisProcessor,
}

impl DualAxis {
    /// Creates a [`DualAxis`] with the specified axis types.
    #[must_use]
    pub fn new(x_axis_type: impl Into<AxisType>, y_axis_type: impl Into<AxisType>) -> Self {
        Self {
            x_axis_type: x_axis_type.into(),
            y_axis_type: y_axis_type.into(),
            processor: DualAxisProcessor::None,
        }
    }

    /// Creates a [`DualAxis`] for the left analogue stick of the gamepad.
    #[must_use]
    pub fn left_stick() -> Self {
        Self {
            x_axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickX),
            y_axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickY),
            processor: CircleDeadZone::default().into(),
        }
    }

    /// Creates a [`DualAxis`] for the right analogue stick of the gamepad.
    #[must_use]
    pub fn right_stick() -> Self {
        Self {
            x_axis_type: AxisType::Gamepad(GamepadAxisType::RightStickX),
            y_axis_type: AxisType::Gamepad(GamepadAxisType::RightStickY),
            processor: CircleDeadZone::default().into(),
        }
    }

    /// Creates a [`DualAxis`] corresponding to horizontal and vertical [`MouseWheel`](bevy::input::mouse::MouseWheel) movement
    pub const fn mouse_wheel() -> Self {
        Self {
            x_axis_type: AxisType::MouseWheel(MouseWheelAxisType::X),
            y_axis_type: AxisType::MouseWheel(MouseWheelAxisType::Y),
            processor: DualAxisProcessor::None,
        }
    }

    /// Creates a [`DualAxis`] corresponding to horizontal and vertical [`MouseMotion`](bevy::input::mouse::MouseMotion) movement
    pub const fn mouse_motion() -> Self {
        Self {
            x_axis_type: AxisType::MouseMotion(MouseMotionAxisType::X),
            y_axis_type: AxisType::MouseMotion(MouseMotionAxisType::Y),
            processor: DualAxisProcessor::None,
        }
    }

    /// Get the "value" of these axes.
    /// If a processor is set, it will compute and return the processed value.
    /// Otherwise, pass the `input_value` through unchanged.
    #[must_use]
    #[inline]
    pub fn input_value(&self, input_value: Vec2) -> Vec2 {
        self.processor.process(input_value)
    }
}

impl WithDualAxisProcessingPipelineExt for DualAxis {
    #[inline]
    fn reset_processing_pipeline(mut self) -> Self {
        self.processor = DualAxisProcessor::None;
        self
    }

    #[inline]
    fn replace_processing_pipeline(mut self, processor: impl Into<DualAxisProcessor>) -> Self {
        self.processor = processor.into();
        self
    }

    #[inline]
    fn with_processor(mut self, processor: impl Into<DualAxisProcessor>) -> Self {
        self.processor = self.processor.with_processor(processor);
        self
    }
}

impl PartialEq for DualAxis {
    fn eq(&self, other: &Self) -> bool {
        self.x_axis_type == other.x_axis_type
            && self.y_axis_type == other.y_axis_type
            && self.processor == other.processor
    }
}

impl Eq for DualAxis {}

impl std::hash::Hash for DualAxis {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x_axis_type.hash(state);
        self.y_axis_type.hash(state);
        self.processor.hash(state);
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
    /// The processor used to handle input values.
    pub processor: DualAxisProcessor,
}

impl VirtualDPad {
    /// Generates a [`VirtualDPad`] corresponding to the arrow keyboard keycodes
    pub fn arrow_keys() -> VirtualDPad {
        VirtualDPad {
            up: InputKind::PhysicalKey(KeyCode::ArrowUp),
            down: InputKind::PhysicalKey(KeyCode::ArrowDown),
            left: InputKind::PhysicalKey(KeyCode::ArrowLeft),
            right: InputKind::PhysicalKey(KeyCode::ArrowRight),
            processor: CircleDeadZone::default().into(),
        }
    }

    /// Generates a [`VirtualDPad`] corresponding to the `WASD` keys on the standard US QWERTY layout.
    ///
    /// Note that on other keyboard layouts, different keys need to be pressed.
    /// The _location_ of the keys is the same on all keyboard layouts.
    /// This ensures that the classic triangular shape is retained on all layouts,
    /// which enables comfortable movement controls.
    pub fn wasd() -> VirtualDPad {
        VirtualDPad {
            up: InputKind::PhysicalKey(KeyCode::KeyW),
            down: InputKind::PhysicalKey(KeyCode::KeyS),
            left: InputKind::PhysicalKey(KeyCode::KeyA),
            right: InputKind::PhysicalKey(KeyCode::KeyD),
            processor: CircleDeadZone::default().into(),
        }
    }

    #[allow(clippy::doc_markdown)] // False alarm because it thinks DPad is an unquoted item
    /// Generates a [`VirtualDPad`] corresponding to the DPad on a gamepad
    pub fn dpad() -> VirtualDPad {
        VirtualDPad {
            up: InputKind::GamepadButton(GamepadButtonType::DPadUp),
            down: InputKind::GamepadButton(GamepadButtonType::DPadDown),
            left: InputKind::GamepadButton(GamepadButtonType::DPadLeft),
            right: InputKind::GamepadButton(GamepadButtonType::DPadRight),
            processor: CircleDeadZone::default().into(),
        }
    }

    /// Generates a [`VirtualDPad`] corresponding to the face buttons on a gamepad
    ///
    /// North corresponds to up, west corresponds to left,
    /// east corresponds to right, and south corresponds to down
    pub fn gamepad_face_buttons() -> VirtualDPad {
        VirtualDPad {
            up: InputKind::GamepadButton(GamepadButtonType::North),
            down: InputKind::GamepadButton(GamepadButtonType::South),
            left: InputKind::GamepadButton(GamepadButtonType::West),
            right: InputKind::GamepadButton(GamepadButtonType::East),
            processor: CircleDeadZone::default().into(),
        }
    }

    /// Generates a [`VirtualDPad`] corresponding to discretized mousewheel movements
    pub fn mouse_wheel() -> VirtualDPad {
        VirtualDPad {
            up: InputKind::MouseWheel(MouseWheelDirection::Up),
            down: InputKind::MouseWheel(MouseWheelDirection::Down),
            left: InputKind::MouseWheel(MouseWheelDirection::Left),
            right: InputKind::MouseWheel(MouseWheelDirection::Right),
            processor: DualAxisProcessor::None,
        }
    }

    /// Generates a [`VirtualDPad`] corresponding to discretized mouse motions
    pub fn mouse_motion() -> VirtualDPad {
        VirtualDPad {
            up: InputKind::MouseMotion(MouseMotionDirection::Up),
            down: InputKind::MouseMotion(MouseMotionDirection::Down),
            left: InputKind::MouseMotion(MouseMotionDirection::Left),
            right: InputKind::MouseMotion(MouseMotionDirection::Right),
            processor: DualAxisProcessor::None,
        }
    }

    /// Get the "value" of these axes.
    /// If a processor is set, it will compute and return the processed value.
    /// Otherwise, pass the `input_value` through unchanged.
    #[must_use]
    #[inline]
    pub fn input_value(&self, input_value: Vec2) -> Vec2 {
        self.processor.process(input_value)
    }
}

impl WithDualAxisProcessingPipelineExt for VirtualDPad {
    #[inline]
    fn reset_processing_pipeline(mut self) -> Self {
        self.processor = DualAxisProcessor::None;
        self
    }

    #[inline]
    fn replace_processing_pipeline(mut self, processor: impl Into<DualAxisProcessor>) -> Self {
        self.processor = processor.into();
        self
    }

    #[inline]
    fn with_processor(mut self, processor: impl Into<DualAxisProcessor>) -> Self {
        self.processor = self.processor.with_processor(processor);
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
    /// The processor used to handle input values.
    pub processor: AxisProcessor,
}

impl VirtualAxis {
    /// Helper function for generating a [`VirtualAxis`] from arbitrary keycodes, shorthand for
    /// wrapping each key in [`InputKind::PhysicalKey`]
    pub const fn from_keys(negative: KeyCode, positive: KeyCode) -> VirtualAxis {
        VirtualAxis {
            negative: InputKind::PhysicalKey(negative),
            positive: InputKind::PhysicalKey(positive),
            processor: AxisProcessor::None,
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
            processor: AxisProcessor::None,
        }
    }

    #[allow(clippy::doc_markdown)]
    /// Generates a [`VirtualAxis`] corresponding to the vertical DPad buttons on a gamepad.
    pub const fn vertical_dpad() -> VirtualAxis {
        VirtualAxis {
            negative: InputKind::GamepadButton(GamepadButtonType::DPadDown),
            positive: InputKind::GamepadButton(GamepadButtonType::DPadUp),
            processor: AxisProcessor::None,
        }
    }

    /// Generates a [`VirtualAxis`] corresponding to the horizontal gamepad face buttons.
    pub const fn horizontal_gamepad_face_buttons() -> VirtualAxis {
        VirtualAxis {
            negative: InputKind::GamepadButton(GamepadButtonType::West),
            positive: InputKind::GamepadButton(GamepadButtonType::East),
            processor: AxisProcessor::None,
        }
    }

    /// Generates a [`VirtualAxis`] corresponding to the vertical gamepad face buttons.
    pub const fn vertical_gamepad_face_buttons() -> VirtualAxis {
        VirtualAxis {
            negative: InputKind::GamepadButton(GamepadButtonType::South),
            positive: InputKind::GamepadButton(GamepadButtonType::North),
            processor: AxisProcessor::None,
        }
    }

    /// Get the "value" of the axis.
    /// If a processor is set, it will compute and return the processed value.
    /// Otherwise, pass the `input_value` through unchanged.
    #[must_use]
    #[inline]
    pub fn input_value(&self, input_value: f32) -> f32 {
        self.processor.process(input_value)
    }
}

impl WithAxisProcessingPipelineExt for VirtualAxis {
    #[inline]
    fn reset_processing_pipeline(mut self) -> Self {
        self.processor = AxisProcessor::None;
        self
    }

    #[inline]
    fn replace_processing_pipeline(mut self, processor: impl Into<AxisProcessor>) -> Self {
        self.processor = processor.into();
        self
    }

    #[inline]
    fn with_processor(mut self, processor: impl Into<AxisProcessor>) -> Self {
        self.processor = self.processor.with_processor(processor);
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

impl MouseWheelAxisType {
    /// Returns the corresponding [`DualAxisType`].
    pub fn axis(&self) -> DualAxisType {
        match self {
            Self::X => DualAxisType::X,
            Self::Y => DualAxisType::Y,
        }
    }
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

impl MouseMotionAxisType {
    /// Returns the corresponding [`DualAxisType`].
    pub fn axis(&self) -> DualAxisType {
        match self {
            Self::X => DualAxisType::X,
            Self::Y => DualAxisType::Y,
        }
    }
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
