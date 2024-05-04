//! Gamepad inputs

use bevy::prelude::{
    Gamepad, GamepadAxis, GamepadAxisType, GamepadButton, GamepadButtonType, Reflect, Vec2,
};
use leafwing_input_manager_macros::serde_typetag;
use serde::{Deserialize, Serialize};

use crate as leafwing_input_manager;
use crate::axislike::{AxisDirection, AxisInputMode, DualAxisData};
use crate::input_processing::{
    AxisProcessor, DualAxisProcessor, WithAxisProcessorExt, WithDualAxisProcessorExt,
};
use crate::input_streams::InputStreams;
use crate::user_input::raw_inputs::RawInputs;
use crate::user_input::{InputKind, UserInput};

/// Retrieves the current value of the specified `axis`.
#[must_use]
#[inline]
fn read_axis_value(input_streams: &InputStreams, axis: GamepadAxisType) -> f32 {
    let gamepad_value_self = |gamepad: Gamepad| -> Option<f32> {
        let axis = GamepadAxis::new(gamepad, axis);
        input_streams.gamepad_axes.get(axis)
    };

    if let Some(gamepad) = input_streams.associated_gamepad {
        gamepad_value_self(gamepad).unwrap_or_default()
    } else {
        input_streams
            .gamepads
            .iter()
            .filter_map(gamepad_value_self)
            .find(|value| *value != 0.0)
            .unwrap_or_default()
    }
}

/// Captures values from a specified [`GamepadAxisType`] on a specific direction,
/// treated as a button press.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct GamepadControlDirection {
    /// The [`GamepadAxisType`] that this input tracks.
    pub(crate) axis: GamepadAxisType,

    /// The direction of the axis.
    pub(crate) direction: AxisDirection,
}

impl GamepadControlDirection {
    /// Creates a negative [`GamepadControlDirection`] of the given `axis`.
    #[inline]
    pub const fn negative(axis: GamepadAxisType) -> Self {
        Self {
            axis,
            direction: AxisDirection::Negative,
        }
    }

    /// Creates a negative [`GamepadControlDirection`] of the given `axis`.
    #[inline]
    pub const fn positive(axis: GamepadAxisType) -> Self {
        Self {
            axis,
            direction: AxisDirection::Positive,
        }
    }

    /// The upward [`GamepadControlDirection`] of the left stick.
    pub const LEFT_UP: Self = Self::positive(GamepadAxisType::LeftStickY);

    /// The downward [`GamepadControlDirection`] of the left stick.
    pub const LEFT_DOWN: Self = Self::negative(GamepadAxisType::LeftStickY);

    /// The leftward [`GamepadControlDirection`] of the left stick.
    pub const LEFT_LEFT: Self = Self::negative(GamepadAxisType::LeftStickX);

    /// The rightward [`GamepadControlDirection`] of the left stick.
    pub const LEFT_RIGHT: Self = Self::positive(GamepadAxisType::LeftStickX);

    /// The upward [`GamepadControlDirection`] of the right stick.
    pub const RIGHT_UP: Self = Self::positive(GamepadAxisType::RightStickY);

    /// The downward [`GamepadControlDirection`] of the right stick.
    pub const RIGHT_DOWN: Self = Self::negative(GamepadAxisType::RightStickY);

    /// The leftward [`GamepadControlDirection`] of the right stick.
    pub const RIGHT_LEFT: Self = Self::negative(GamepadAxisType::RightStickX);

    /// The rightward [`GamepadControlDirection`] of the right stick.
    pub const RIGHT_RIGHT: Self = Self::positive(GamepadAxisType::RightStickX);
}

#[serde_typetag]
impl UserInput for GamepadControlDirection {
    /// [`GamepadControlDirection`] always acts as a virtual button.
    #[inline]
    fn kind(&self) -> InputKind {
        InputKind::Button
    }

    /// Creates a [`RawInputs`] from the direction directly.
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        RawInputs::from_gamepad_control_directions([*self])
    }

    /// Returns a list that only contains the [`GamepadControlDirection`] itself,
    /// as it represents a simple virtual button.
    #[must_use]
    #[inline]
    fn destructure(&self) -> Vec<Box<dyn UserInput>> {
        vec![Box::new(*self)]
    }

    /// Checks if there is any recent stick movement along the specified direction.
    ///
    /// When a [`Gamepad`] is specified, only checks the movement on the specified gamepad.
    /// Otherwise, checks the movement on any connected gamepads.
    #[must_use]
    #[inline]
    fn pressed(&self, input_streams: &InputStreams) -> bool {
        let value = read_axis_value(input_streams, self.axis);
        self.direction.is_active(value)
    }

    /// Retrieves the amount of the stick movement along the specified direction,
    /// returning `0.0` for no movement and `1.0` for full movement.
    ///
    /// When a [`Gamepad`] is specified, only retrieves the value on the specified gamepad.
    /// Otherwise, retrieves the value on any connected gamepads.
    #[must_use]
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        f32::from(self.pressed(input_streams))
    }

    /// Always returns [`None`] as [`GamepadControlDirection`] doesn't represent dual-axis input.
    #[must_use]
    #[inline]
    fn axis_pair(&self, _input_streams: &InputStreams) -> Option<DualAxisData> {
        None
    }
}

/// Captures values from a specified [`GamepadAxisType`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct GamepadControlAxis {
    /// The [`GamepadAxisType`] that this input tracks.
    pub(crate) axis: GamepadAxisType,

    /// The method to interpret values on the axis,
    /// either [`AxisInputMode::Analog`] or [`AxisInputMode::Digital`].
    pub(crate) input_mode: AxisInputMode,

    /// The [`AxisProcessor`] used to handle input values.
    pub(crate) processor: AxisProcessor,
}

impl GamepadControlAxis {
    /// Creates a [`GamepadControlAxis`] for continuous input from the given axis.
    /// No processing is applied to raw data from the gamepad.
    #[inline]
    pub const fn new(axis: GamepadAxisType) -> Self {
        Self {
            axis,
            input_mode: AxisInputMode::Analog,
            processor: AxisProcessor::None,
        }
    }

    /// Creates a [`GamepadControlAxis`] for discrete input from the given axis.
    /// No processing is applied to raw data from the gamepad.
    #[inline]
    pub const fn digital(axis: GamepadAxisType) -> Self {
        Self {
            axis,
            input_mode: AxisInputMode::Digital,
            processor: AxisProcessor::None,
        }
    }

    /// The horizontal [`GamepadControlAxis`] for continuous input from the left stick.
    /// No processing is applied to raw data from the gamepad.
    pub const LEFT_X: Self = Self::new(GamepadAxisType::LeftStickX);

    /// The vertical [`GamepadControlAxis`] for continuous input from the left stick.
    /// No processing is applied to raw data from the gamepad.
    pub const LEFT_Y: Self = Self::new(GamepadAxisType::LeftStickY);

    /// The horizontal [`GamepadControlAxis`] for continuous input from the right stick.
    /// No processing is applied to raw data from the gamepad.
    pub const RIGHT_X: Self = Self::new(GamepadAxisType::RightStickX);

    /// The vertical [`GamepadControlAxis`] for continuous input from the right stick.
    /// No processing is applied to raw data from the gamepad.
    pub const RIGHT_Y: Self = Self::new(GamepadAxisType::RightStickY);

    /// The horizontal [`GamepadControlAxis`] for discrete input from the left stick.
    /// No processing is applied to raw data from the gamepad.
    pub const LEFT_X_DIGITAL: Self = Self::digital(GamepadAxisType::LeftStickX);

    /// The vertical [`GamepadControlAxis`] for discrete input from the left stick.
    /// No processing is applied to raw data from the gamepad.
    pub const LEFT_Y_DIGITAL: Self = Self::digital(GamepadAxisType::LeftStickY);

    /// The horizontal [`GamepadControlAxis`] for discrete input from the right stick.
    /// No processing is applied to raw data from the gamepad.
    pub const RIGHT_X_DIGITAL: Self = Self::digital(GamepadAxisType::RightStickX);

    /// The vertical [`GamepadControlAxis`] for discrete input from the right stick.
    /// No processing is applied to raw data from the gamepad.
    pub const RIGHT_Y_DIGITAL: Self = Self::digital(GamepadAxisType::RightStickY);
}

#[serde_typetag]
impl UserInput for GamepadControlAxis {
    /// [`GamepadControlAxis`] always acts as an axis input.
    #[inline]
    fn kind(&self) -> InputKind {
        InputKind::Axis
    }

    /// Creates a [`RawInputs`] from the [`GamepadAxisType`] used by the axis.
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        RawInputs::from_gamepad_axes([self.axis])
    }

    /// Returns both positive and negative [`GamepadControlDirection`]s to represent the movement.
    #[must_use]
    #[inline]
    fn destructure(&self) -> Vec<Box<dyn UserInput>> {
        vec![
            Box::new(GamepadControlDirection::negative(self.axis)),
            Box::new(GamepadControlDirection::positive(self.axis)),
        ]
    }

    /// Checks if this axis has a non-zero value.
    ///
    /// When a [`Gamepad`] is specified, only checks if the axis is active on the specified gamepad.
    /// Otherwise, checks if the axis is active on any connected gamepads.
    #[must_use]
    #[inline]
    fn pressed(&self, input_streams: &InputStreams) -> bool {
        self.value(input_streams) != 0.0
    }

    /// Retrieves the current value of this axis after processing by the associated processor.
    ///
    /// When a [`Gamepad`] is specified, only retrieves the value on the specified gamepad.
    /// Otherwise, retrieves the value on any connected gamepads.
    #[must_use]
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        let value = read_axis_value(input_streams, self.axis);
        let value = self.input_mode.axis_value(value);
        self.processor.process(value)
    }

    /// Always returns [`None`] as [`GamepadControlAxis`] doesn't represent dual-axis input.
    #[must_use]
    #[inline]
    fn axis_pair(&self, _input_streams: &InputStreams) -> Option<DualAxisData> {
        None
    }
}

impl WithAxisProcessorExt for GamepadControlAxis {
    #[inline]
    fn no_processor(mut self) -> Self {
        self.processor = AxisProcessor::None;
        self
    }

    #[inline]
    fn replace_processor(mut self, processor: impl Into<AxisProcessor>) -> Self {
        self.processor = processor.into();
        self
    }

    #[inline]
    fn with_processor(mut self, processor: impl Into<AxisProcessor>) -> Self {
        self.processor = self.processor.with_processor(processor);
        self
    }
}

/// Captures values from two specified [`GamepadAxisType`]s as the X and Y axes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct GamepadStick {
    /// The [`GamepadAxisType`] used for the X-axis.
    pub(crate) x: GamepadAxisType,

    /// The [`GamepadAxisType`] used for the Y-axis.
    pub(crate) y: GamepadAxisType,

    /// The method to interpret values on both axes,
    /// either [`AxisInputMode::Analog`] or [`AxisInputMode::Digital`].
    pub(crate) input_mode: AxisInputMode,

    /// The [`DualAxisProcessor`] used to handle input values.
    pub(crate) processor: DualAxisProcessor,
}

impl GamepadStick {
    /// Creates a [`GamepadStick`] for continuous input from two given axes as the X and Y axes.
    /// No processing is applied to raw data from the gamepad.
    #[inline]
    pub const fn new(x: GamepadAxisType, y: GamepadAxisType) -> Self {
        Self {
            x,
            y,
            input_mode: AxisInputMode::Analog,
            processor: DualAxisProcessor::None,
        }
    }

    /// Creates a [`GamepadStick`] for discrete input from two given axes as the X and Y axes.
    /// No processing is applied to raw data from the gamepad.
    #[inline]
    pub const fn digital(x: GamepadAxisType, y: GamepadAxisType) -> Self {
        Self {
            x,
            y,
            input_mode: AxisInputMode::Analog,
            processor: DualAxisProcessor::None,
        }
    }

    /// The left [`GamepadStick`] for continuous input on the X and Y axes.
    /// No processing is applied to raw data from the gamepad.
    pub const LEFT: Self = Self::new(GamepadAxisType::LeftStickX, GamepadAxisType::LeftStickY);

    /// The right [`GamepadStick`] for continuous input on the X and Y axes.
    /// No processing is applied to raw data from the gamepad.
    pub const RIGHT: Self = Self::new(GamepadAxisType::RightStickX, GamepadAxisType::RightStickY);

    /// The left [`GamepadStick`] for discrete input from the left stick on the X and Y axes.
    /// No processing is applied to raw data from the gamepad.
    pub const LEFT_DIGITAL: Self =
        Self::digital(GamepadAxisType::LeftStickX, GamepadAxisType::LeftStickY);

    /// The right [`GamepadStick`] for discrete input from the right stick on the X and Y axes.
    /// No processing is applied to raw data from the gamepad.
    pub const RIGHT_DIGITAL: Self =
        Self::digital(GamepadAxisType::RightStickX, GamepadAxisType::RightStickY);

    /// Retrieves the current X and Y values of this stick after processing by the associated processor.
    ///
    /// When a [`Gamepad`] is specified, only retrieves the value on the specified gamepad.
    /// Otherwise, retrieves the value on any connected gamepads.
    #[must_use]
    #[inline]
    fn processed_value(&self, input_streams: &InputStreams) -> Vec2 {
        let x = read_axis_value(input_streams, self.x);
        let y = read_axis_value(input_streams, self.y);
        let value = Vec2::new(x, y);
        let value = self.input_mode.dual_axis_value(value);
        self.processor.process(value)
    }
}

#[serde_typetag]
impl UserInput for GamepadStick {
    /// [`GamepadStick`] always acts as a dual-axis input.
    #[inline]
    fn kind(&self) -> InputKind {
        InputKind::DualAxis
    }

    /// Creates a [`RawInputs`] from two [`GamepadAxisType`]s used by the stick.
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        RawInputs::from_gamepad_axes([self.x, self.y])
    }

    /// Returns four [`GamepadControlDirection`]s to represent the movement.
    #[must_use]
    #[inline]
    fn destructure(&self) -> Vec<Box<dyn UserInput>> {
        vec![
            Box::new(GamepadControlDirection::negative(self.x)),
            Box::new(GamepadControlDirection::positive(self.x)),
            Box::new(GamepadControlDirection::negative(self.y)),
            Box::new(GamepadControlDirection::positive(self.y)),
        ]
    }

    /// Checks if this stick has a non-zero magnitude.
    ///
    /// When a [`Gamepad`] is specified, only checks if the stick is active on the specified gamepad.
    /// Otherwise, checks if the stick is active on any connected gamepads.
    #[must_use]
    #[inline]
    fn pressed(&self, input_streams: &InputStreams) -> bool {
        self.processed_value(input_streams) != Vec2::ZERO
    }

    /// Retrieves the magnitude of the value from this stick after processing by the associated processor.
    ///
    /// When a [`Gamepad`] is specified, only retrieves the value on the specified gamepad.
    /// Otherwise, retrieves the value on any connected gamepads.
    #[must_use]
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        let value = self.processed_value(input_streams);
        self.input_mode.dual_axis_magnitude(value)
    }

    /// Retrieves the current X and Y values of this stick after processing by the associated processor.
    ///
    /// When a [`Gamepad`] is specified, only retrieves the value on the specified gamepad.
    /// Otherwise, retrieves the value on any connected gamepads.
    #[must_use]
    #[inline]
    fn axis_pair(&self, input_streams: &InputStreams) -> Option<DualAxisData> {
        let value = self.processed_value(input_streams);
        Some(DualAxisData::from_xy(value))
    }
}

impl WithDualAxisProcessorExt for GamepadStick {
    #[inline]
    fn no_processor(mut self) -> Self {
        self.processor = DualAxisProcessor::None;
        self
    }

    #[inline]
    fn replace_processor(mut self, processor: impl Into<DualAxisProcessor>) -> Self {
        self.processor = processor.into();
        self
    }

    #[inline]
    fn with_processor(mut self, processor: impl Into<DualAxisProcessor>) -> Self {
        self.processor = self.processor.with_processor(processor);
        self
    }
}

/// Checks if the given [`GamepadButtonType`] is currently pressed.
#[must_use]
#[inline]
fn button_pressed(
    input_streams: &InputStreams,
    gamepad: Gamepad,
    button: GamepadButtonType,
) -> bool {
    let button = GamepadButton::new(gamepad, button);
    input_streams.gamepad_buttons.pressed(button)
}

/// Retrieves the current value of the given [`GamepadButtonType`].
#[must_use]
#[inline]
fn button_value(
    input_streams: &InputStreams,
    gamepad: Gamepad,
    button: GamepadButtonType,
) -> Option<f32> {
    // This implementation differs from `button_pressed()` because the upstream bevy::input
    // still waffles about whether triggers are buttons or axes.
    // So, we consider the axes for consistency with other gamepad axes (e.g., thumb sticks).
    let button = GamepadButton::new(gamepad, button);
    input_streams.gamepad_button_axes.get(button)
}

/// Retrieves the current value of the given [`GamepadButtonType`] on any connected gamepad.
#[must_use]
#[inline]
fn button_value_any(input_streams: &InputStreams, button: GamepadButtonType) -> f32 {
    for gamepad in input_streams.gamepads.iter() {
        if let Some(value) = button_value(input_streams, gamepad, button) {
            return value;
        }
    }
    0.0
}

// Built-in support for Bevy's GamepadButtonType.
#[serde_typetag]
impl UserInput for GamepadButtonType {
    /// [`GamepadButtonType`] always acts as a button.
    #[inline]
    fn kind(&self) -> InputKind {
        InputKind::Button
    }

    /// Creates a [`RawInputs`] from the button directly.
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        RawInputs::from_gamepad_buttons([*self])
    }

    /// Returns a list that only contains the [`GamepadButtonType`] itself,
    /// as it represents a simple physical button.
    #[must_use]
    #[inline]
    fn destructure(&self) -> Vec<Box<dyn UserInput>> {
        vec![Box::new(*self)]
    }

    /// Checks if the specified button is currently pressed down.
    ///
    /// When a [`Gamepad`] is specified, only checks if the button is pressed on the specified gamepad.
    /// Otherwise, checks if the button is pressed on any connected gamepads.
    #[must_use]
    #[inline]
    fn pressed(&self, input_streams: &InputStreams) -> bool {
        if let Some(gamepad) = input_streams.associated_gamepad {
            button_pressed(input_streams, gamepad, *self)
        } else {
            input_streams
                .gamepads
                .iter()
                .any(|gamepad| button_pressed(input_streams, gamepad, *self))
        }
    }

    /// Retrieves the strength of the button press for the specified button.
    ///
    /// When a [`Gamepad`] is specified, only retrieves the value on the specified gamepad.
    /// Otherwise, retrieves the value on any connected gamepads.
    #[must_use]
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        if let Some(gamepad) = input_streams.associated_gamepad {
            button_value(input_streams, gamepad, *self).unwrap_or_default()
        } else {
            button_value_any(input_streams, *self)
        }
    }

    /// Always returns [`None`] as [`GamepadButtonType`] doesn't represent dual-axis input.
    #[must_use]
    #[inline]
    fn axis_pair(&self, _input_streams: &InputStreams) -> Option<DualAxisData> {
        None
    }
}

/// A virtual single-axis control constructed from two [`GamepadButtonType`]s.
/// One button represents the negative direction (typically left or down),
/// while the other represents the positive direction (typically right or up).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct GamepadVirtualAxis {
    /// The [`GamepadButtonType`] used for the negative direction (typically left or down).
    pub(crate) negative: GamepadButtonType,

    /// The [`GamepadButtonType`] used for the positive direction (typically right or up).
    pub(crate) positive: GamepadButtonType,

    /// The [`AxisProcessor`] used to handle input values.
    pub(crate) processor: AxisProcessor,
}

impl GamepadVirtualAxis {
    /// Creates a new [`GamepadVirtualAxis`] with two given [`GamepadButtonType`]s.
    /// One button represents the negative direction (typically left or down),
    /// while the other represents the positive direction (typically right or up).
    /// No processing is applied to raw data from the gamepad.
    #[inline]
    pub const fn new(negative: GamepadButtonType, positive: GamepadButtonType) -> Self {
        Self {
            negative,
            positive,
            processor: AxisProcessor::None,
        }
    }

    /// The [`GamepadVirtualAxis`] using the horizontal D-Pad button mappings.
    /// No processing is applied to raw data from the gamepad.
    ///
    /// - [`GamepadButtonType::DPadLeft`] for negative direction.
    /// - [`GamepadButtonType::DPadRight`] for positive direction.
    pub const DPAD_X: Self = Self::new(GamepadButtonType::DPadLeft, GamepadButtonType::DPadRight);

    /// The [`GamepadVirtualAxis`] using the vertical D-Pad button mappings.
    /// No processing is applied to raw data from the gamepad.
    ///
    /// - [`GamepadButtonType::DPadDown`] for negative direction.
    /// - [`GamepadButtonType::DPadUp`] for positive direction.
    pub const DPAD_Y: Self = Self::new(GamepadButtonType::DPadDown, GamepadButtonType::DPadUp);

    /// The [`GamepadVirtualAxis`] using the horizontal action pad button mappings.
    /// No processing is applied to raw data from the gamepad.
    ///
    /// - [`GamepadButtonType::West`] for negative direction.
    /// - [`GamepadButtonType::East`] for positive direction.
    pub const ACTION_PAD_X: Self = Self::new(GamepadButtonType::West, GamepadButtonType::East);

    /// The [`GamepadVirtualAxis`] using the vertical action pad button mappings.
    /// No processing is applied to raw data from the gamepad.
    ///
    /// - [`GamepadButtonType::South`] for negative direction.
    /// - [`GamepadButtonType::North`] for positive direction.
    pub const ACTION_PAD_Y: Self = Self::new(GamepadButtonType::South, GamepadButtonType::North);
}

#[serde_typetag]
impl UserInput for GamepadVirtualAxis {
    /// [`GamepadVirtualAxis`] always acts as an axis input.
    #[inline]
    fn kind(&self) -> InputKind {
        InputKind::Axis
    }

    /// Creates a [`RawInputs`] from two [`GamepadButtonType`]s used by this axis.
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        RawInputs::from_gamepad_buttons([self.negative, self.positive])
    }

    /// Returns the two [`GamepadButtonType`]s used by this axis.
    #[must_use]
    #[inline]
    fn destructure(&self) -> Vec<Box<dyn UserInput>> {
        vec![Box::new(self.negative), Box::new(self.positive)]
    }

    /// Checks if this axis has a non-zero value after processing by the associated processor.
    ///
    /// When a [`Gamepad`] is specified, only checks if the buttons are pressed on the specified gamepad.
    /// Otherwise, checks if the buttons are pressed on any connected gamepads.
    #[must_use]
    #[inline]
    fn pressed(&self, input_streams: &InputStreams) -> bool {
        self.value(input_streams) != 0.0
    }

    /// Retrieves the current value of this axis after processing by the associated processor.
    ///
    /// When a [`Gamepad`] is specified, only retrieves the value on the specified gamepad.
    /// Otherwise, retrieves the value on any connected gamepads.
    #[must_use]
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        let value = if let Some(gamepad) = input_streams.associated_gamepad {
            let negative = button_value(input_streams, gamepad, self.negative).unwrap_or_default();
            let positive = button_value(input_streams, gamepad, self.positive).unwrap_or_default();
            positive - negative
        } else {
            let negative = button_value_any(input_streams, self.negative);
            let positive = button_value_any(input_streams, self.positive);
            positive - negative
        };
        self.processor.process(value)
    }

    /// Always returns [`None`] as [`GamepadVirtualAxis`] doesn't represent dual-axis input.
    #[must_use]
    #[inline]
    fn axis_pair(&self, _input_streams: &InputStreams) -> Option<DualAxisData> {
        None
    }
}

impl WithAxisProcessorExt for GamepadVirtualAxis {
    #[inline]
    fn no_processor(mut self) -> Self {
        self.processor = AxisProcessor::None;
        self
    }

    #[inline]
    fn replace_processor(mut self, processor: impl Into<AxisProcessor>) -> Self {
        self.processor = processor.into();
        self
    }

    #[inline]
    fn with_processor(mut self, processor: impl Into<AxisProcessor>) -> Self {
        self.processor = self.processor.with_processor(processor);
        self
    }
}

/// A virtual dual-axis control constructed from four [`GamepadButtonType`]s.
/// Each button represents a specific direction (up, down, left, right),
/// functioning similarly to a directional pad (D-pad) on both X and Y axes,
/// and offering intermediate diagonals by means of two-button combinations.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct GamepadVirtualDPad {
    /// The [`GamepadButtonType`] used for the upward direction.
    pub(crate) up: GamepadButtonType,

    /// The [`GamepadButtonType`] used for the downward direction.
    pub(crate) down: GamepadButtonType,

    /// The [`GamepadButtonType`] used for the leftward direction.
    pub(crate) left: GamepadButtonType,

    /// The [`GamepadButtonType`] used for the rightward direction.
    pub(crate) right: GamepadButtonType,

    /// The [`DualAxisProcessor`] used to handle input values.
    pub(crate) processor: DualAxisProcessor,
}

impl GamepadVirtualDPad {
    /// Creates a new [`GamepadVirtualDPad`] with four given [`GamepadButtonType`]s.
    /// Each button represents a specific direction (up, down, left, right).
    #[inline]
    pub const fn new(
        up: GamepadButtonType,
        down: GamepadButtonType,
        left: GamepadButtonType,
        right: GamepadButtonType,
    ) -> Self {
        Self {
            up,
            down,
            left,
            right,
            processor: DualAxisProcessor::None,
        }
    }

    /// Creates a new [`GamepadVirtualDPad`] using the common D-Pad button mappings.
    ///
    /// - [`GamepadButtonType::DPadUp`] for upward direction.
    /// - [`GamepadButtonType::DPadDown`] for downward direction.
    /// - [`GamepadButtonType::DPadLeft`] for leftward direction.
    /// - [`GamepadButtonType::DPadRight`] for rightward direction.
    pub const DPAD: Self = Self::new(
        GamepadButtonType::DPadUp,
        GamepadButtonType::DPadDown,
        GamepadButtonType::DPadLeft,
        GamepadButtonType::DPadRight,
    );

    /// Creates a new [`GamepadVirtualDPad`] using the common action pad button mappings.
    ///
    /// - [`GamepadButtonType::North`] for upward direction.
    /// - [`GamepadButtonType::South`] for downward direction.
    /// - [`GamepadButtonType::West`] for leftward direction.
    /// - [`GamepadButtonType::East`] for rightward direction.
    pub const ACTION_PAD: Self = Self::new(
        GamepadButtonType::North,
        GamepadButtonType::South,
        GamepadButtonType::West,
        GamepadButtonType::East,
    );

    /// Retrieves the current X and Y values of this D-pad after processing by the associated processor.
    ///
    /// When a [`Gamepad`] is specified, only retrieves the value on the specified gamepad.
    /// Otherwise, retrieves the value on any connected gamepads.
    #[inline]
    fn processed_value(&self, input_streams: &InputStreams) -> Vec2 {
        let value = if let Some(gamepad) = input_streams.associated_gamepad {
            let up = button_value(input_streams, gamepad, self.up).unwrap_or_default();
            let down = button_value(input_streams, gamepad, self.down).unwrap_or_default();
            let left = button_value(input_streams, gamepad, self.left).unwrap_or_default();
            let right = button_value(input_streams, gamepad, self.right).unwrap_or_default();
            Vec2::new(right - left, up - down)
        } else {
            let up = button_value_any(input_streams, self.up);
            let down = button_value_any(input_streams, self.down);
            let left = button_value_any(input_streams, self.left);
            let right = button_value_any(input_streams, self.right);
            Vec2::new(right - left, up - down)
        };
        self.processor.process(value)
    }
}

#[serde_typetag]
impl UserInput for GamepadVirtualDPad {
    /// [`GamepadVirtualDPad`] always acts as a dual-axis input.
    #[inline]
    fn kind(&self) -> InputKind {
        InputKind::DualAxis
    }

    /// Creates a [`RawInputs`] from four [`GamepadButtonType`]s used by this D-pad.
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        RawInputs::from_gamepad_buttons([self.up, self.down, self.left, self.right])
    }

    /// Returns the four [`GamepadButtonType`]s used by this D-pad.
    #[must_use]
    #[inline]
    fn destructure(&self) -> Vec<Box<dyn UserInput>> {
        vec![
            Box::new(self.up),
            Box::new(self.down),
            Box::new(self.left),
            Box::new(self.right),
        ]
    }

    /// Checks if this D-pad has a non-zero magnitude after processing by the associated processor.
    ///
    /// When a [`Gamepad`] is specified, only checks if the button is pressed on the specified gamepad.
    /// Otherwise, checks if the button is pressed on any connected gamepads.
    #[must_use]
    #[inline]
    fn pressed(&self, input_streams: &InputStreams) -> bool {
        self.processed_value(input_streams) != Vec2::ZERO
    }

    /// Retrieves the magnitude of the value from this D-pad after processing by the associated processor.
    ///
    /// When a [`Gamepad`] is specified, only retrieves the value on the specified gamepad.
    /// Otherwise, retrieves the value on any connected gamepads.
    #[must_use]
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        self.processed_value(input_streams).length()
    }

    /// Retrieves the current X and Y values of this D-pad after processing by the associated processor.
    ///
    /// When a [`Gamepad`] is specified, only retrieves the value on the specified gamepad.
    /// Otherwise, retrieves the value on any connected gamepads.
    #[must_use]
    #[inline]
    fn axis_pair(&self, input_streams: &InputStreams) -> Option<DualAxisData> {
        let value = self.processed_value(input_streams);
        Some(DualAxisData::from_xy(value))
    }
}

impl WithDualAxisProcessorExt for GamepadVirtualDPad {
    #[inline]
    fn no_processor(mut self) -> Self {
        self.processor = DualAxisProcessor::None;
        self
    }

    #[inline]
    fn replace_processor(mut self, processor: impl Into<DualAxisProcessor>) -> Self {
        self.processor = processor.into();
        self
    }

    #[inline]
    fn with_processor(mut self, processor: impl Into<DualAxisProcessor>) -> Self {
        self.processor = self.processor.with_processor(processor);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input_mocking::MockInput;
    use bevy::input::gamepad::{
        GamepadConnection, GamepadConnectionEvent, GamepadEvent, GamepadInfo,
    };
    use bevy::input::InputPlugin;
    use bevy::prelude::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(InputPlugin);

        // WARNING: you MUST register your gamepad during tests,
        // or all gamepad input mocking actions will fail
        let mut gamepad_events = app.world.resource_mut::<Events<GamepadEvent>>();
        gamepad_events.send(GamepadEvent::Connection(GamepadConnectionEvent {
            // This MUST be consistent with any other mocked events
            gamepad: Gamepad { id: 1 },
            connection: GamepadConnection::Connected(GamepadInfo {
                name: "TestController".into(),
            }),
        }));

        // Ensure that the gamepad is picked up by the appropriate system
        app.update();
        // Ensure that the connection event is flushed through
        app.update();

        app
    }

    fn check(
        input: &impl UserInput,
        input_streams: &InputStreams,
        expected_pressed: bool,
        expected_value: f32,
        expected_axis_pair: Option<DualAxisData>,
    ) {
        assert_eq!(input.pressed(input_streams), expected_pressed);
        assert_eq!(input.value(input_streams), expected_value);
        assert_eq!(input.axis_pair(input_streams), expected_axis_pair);
    }

    fn pressed(input: &impl UserInput, input_streams: &InputStreams) {
        check(input, input_streams, true, 1.0, None);
    }

    fn released(input: &impl UserInput, input_streams: &InputStreams) {
        check(input, input_streams, false, 0.0, None);
    }

    #[test]
    fn test_gamepad_axes() {
        let left_up = GamepadControlDirection::LEFT_UP;
        assert_eq!(left_up.kind(), InputKind::Button);
        let raw_inputs = RawInputs::from_gamepad_control_directions([left_up]);
        assert_eq!(left_up.raw_inputs(), raw_inputs);

        // The opposite of left up
        let left_down = GamepadControlDirection::LEFT_DOWN;
        assert_eq!(left_down.kind(), InputKind::Button);
        let raw_inputs = RawInputs::from_gamepad_control_directions([left_up]);
        assert_eq!(left_up.raw_inputs(), raw_inputs);

        let left_x = GamepadControlAxis::LEFT_X;
        assert_eq!(left_x.kind(), InputKind::Axis);
        let raw_inputs = RawInputs::from_gamepad_axes([left_x.axis]);
        assert_eq!(left_x.raw_inputs(), raw_inputs);

        let left_y = GamepadControlAxis::LEFT_Y;
        assert_eq!(left_y.kind(), InputKind::Axis);
        let raw_inputs = RawInputs::from_gamepad_axes([left_y.axis]);
        assert_eq!(left_y.raw_inputs(), raw_inputs);

        let left = GamepadStick::LEFT;
        assert_eq!(left.kind(), InputKind::DualAxis);
        let raw_inputs = RawInputs::from_gamepad_axes([left.x, left.y]);
        assert_eq!(left.raw_inputs(), raw_inputs);

        // Up; but for the other stick
        let right_up = GamepadControlDirection::RIGHT_DOWN;
        assert_eq!(right_up.kind(), InputKind::Button);
        let raw_inputs = RawInputs::from_gamepad_control_directions([right_up]);
        assert_eq!(right_up.raw_inputs(), raw_inputs);

        let right_y = GamepadControlAxis::RIGHT_Y;
        assert_eq!(right_y.kind(), InputKind::Axis);
        let raw_inputs = RawInputs::from_gamepad_axes([right_y.axis]);
        assert_eq!(right_y.raw_inputs(), raw_inputs);

        let right = GamepadStick::RIGHT;
        assert_eq!(right.kind(), InputKind::DualAxis);
        let raw_inputs = RawInputs::from_gamepad_axes([right_y.axis]);
        assert_eq!(right_y.raw_inputs(), raw_inputs);

        // No inputs
        let zeros = Some(DualAxisData::ZERO);
        let mut app = test_app();
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        released(&left_up, &inputs);
        released(&left_down, &inputs);
        released(&right_up, &inputs);
        released(&left_x, &inputs);
        released(&left_y, &inputs);
        released(&right_y, &inputs);
        check(&left, &inputs, false, 0.0, zeros);
        check(&right, &inputs, false, 0.0, zeros);

        // Left stick moves upward
        let data = DualAxisData::new(0.0, 1.0);
        let mut app = test_app();
        app.press_input(GamepadControlDirection::LEFT_UP);
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        pressed(&left_up, &inputs);
        released(&left_down, &inputs);
        released(&right_up, &inputs);
        released(&left_x, &inputs);
        check(&left_y, &inputs, true, data.y(), None);
        released(&right_y, &inputs);
        check(&left, &inputs, true, data.length(), Some(data));
        check(&right, &inputs, false, 0.0, zeros);

        // Set Y-axis of left stick to 0.6
        let data = DualAxisData::new(0.0, 0.6);
        let mut app = test_app();
        app.send_axis_values(GamepadControlAxis::LEFT_Y, [data.y()]);
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        pressed(&left_up, &inputs);
        released(&left_down, &inputs);
        released(&right_up, &inputs);
        released(&left_x, &inputs);
        check(&left_y, &inputs, true, data.y(), None);
        released(&right_y, &inputs);
        check(&left, &inputs, true, data.length(), Some(data));
        check(&right, &inputs, false, 0.0, zeros);

        // Set left stick to (0.6, 0.4)
        let data = DualAxisData::new(0.6, 0.4);
        let mut app = test_app();
        app.send_axis_values(GamepadStick::LEFT, [data.x(), data.y()]);
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        pressed(&left_up, &inputs);
        released(&left_down, &inputs);
        released(&right_up, &inputs);
        check(&left_x, &inputs, true, data.x(), None);
        check(&left_y, &inputs, true, data.y(), None);
        released(&right_y, &inputs);
        check(&left, &inputs, true, data.length(), Some(data));
        check(&right, &inputs, false, 0.0, zeros);
    }

    #[test]
    #[ignore = "Input mocking is subtly broken: https://github.com/Leafwing-Studios/leafwing-input-manager/issues/516"]
    fn test_gamepad_buttons() {
        let up = GamepadButtonType::DPadUp;
        assert_eq!(up.kind(), InputKind::Button);
        let raw_inputs = RawInputs::from_gamepad_buttons([up]);
        assert_eq!(up.raw_inputs(), raw_inputs);

        let left = GamepadButtonType::DPadLeft;
        assert_eq!(left.kind(), InputKind::Button);
        let raw_inputs = RawInputs::from_gamepad_buttons([left]);
        assert_eq!(left.raw_inputs(), raw_inputs);

        let down = GamepadButtonType::DPadDown;
        assert_eq!(left.kind(), InputKind::Button);
        let raw_inputs = RawInputs::from_gamepad_buttons([down]);
        assert_eq!(down.raw_inputs(), raw_inputs);

        let right = GamepadButtonType::DPadRight;
        assert_eq!(left.kind(), InputKind::Button);
        let raw_inputs = RawInputs::from_gamepad_buttons([right]);
        assert_eq!(right.raw_inputs(), raw_inputs);

        let x_axis = GamepadVirtualAxis::DPAD_X;
        assert_eq!(x_axis.kind(), InputKind::Axis);
        let raw_inputs = RawInputs::from_gamepad_buttons([left, right]);
        assert_eq!(x_axis.raw_inputs(), raw_inputs);

        let y_axis = GamepadVirtualAxis::DPAD_Y;
        assert_eq!(y_axis.kind(), InputKind::Axis);
        let raw_inputs = RawInputs::from_gamepad_buttons([down, up]);
        assert_eq!(y_axis.raw_inputs(), raw_inputs);

        let dpad = GamepadVirtualDPad::DPAD;
        assert_eq!(dpad.kind(), InputKind::DualAxis);
        let raw_inputs = RawInputs::from_gamepad_buttons([up, down, left, right]);
        assert_eq!(dpad.raw_inputs(), raw_inputs);

        // No inputs
        let zeros = Some(DualAxisData::ZERO);
        let mut app = test_app();
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        released(&up, &inputs);
        released(&down, &inputs);
        released(&left, &inputs);
        released(&right, &inputs);
        released(&x_axis, &inputs);
        released(&y_axis, &inputs);
        check(&dpad, &inputs, false, 0.0, zeros);

        // Press DPadLeft
        let data = DualAxisData::new(1.0, 0.0);
        let mut app = test_app();
        app.press_input(GamepadButtonType::DPadLeft);
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        released(&up, &inputs);
        released(&down, &inputs);
        released(&left, &inputs);
        pressed(&right, &inputs);
        check(&x_axis, &inputs, true, data.x(), None);
        released(&y_axis, &inputs);
        check(&dpad, &inputs, true, data.length(), Some(data));

        // Set the X-axis to 0.6
        let data = DualAxisData::new(0.6, 0.0);
        let mut app = test_app();
        app.send_axis_values(GamepadVirtualAxis::DPAD_X, [data.x()]);
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        released(&up, &inputs);
        released(&down, &inputs);
        released(&left, &inputs);
        pressed(&right, &inputs);
        check(&x_axis, &inputs, true, data.x(), None);
        released(&y_axis, &inputs);
        check(&dpad, &inputs, true, data.length(), Some(data));

        // Set the axes to (0.6, 0.4)
        let data = DualAxisData::new(0.6, 0.4);
        let mut app = test_app();
        app.send_axis_values(GamepadVirtualDPad::DPAD, [data.x(), data.y()]);
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        pressed(&up, &inputs);
        released(&down, &inputs);
        released(&left, &inputs);
        pressed(&right, &inputs);
        check(&x_axis, &inputs, true, data.x(), None);
        check(&y_axis, &inputs, true, data.y(), None);
        check(&dpad, &inputs, true, data.length(), Some(data));
    }
}
