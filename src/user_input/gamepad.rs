//! Gamepad inputs

use bevy::prelude::{
    Gamepad, GamepadAxis, GamepadAxisType, GamepadButton, GamepadButtonType, Reflect, Vec2,
};
use leafwing_input_manager_macros::serde_typetag;
use serde::{Deserialize, Serialize};

use crate as leafwing_input_manager;
use crate::axislike::AxisDirection;
use crate::clashing_inputs::BasicInputs;
use crate::input_processing::{
    AxisProcessor, DualAxisProcessor, WithAxisProcessingPipelineExt,
    WithDualAxisProcessingPipelineExt,
};
use crate::input_streams::InputStreams;
use crate::raw_inputs::RawInputs;
use crate::user_input::UserInput;
use crate::InputControlKind;

use super::{Axislike, Buttonlike, DualAxislike};

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

/// Provides button-like behavior for a specific direction on a [`GamepadAxisType`].
///
/// # Behaviors
///
/// - Gamepad Selection: By default, reads from **any connected gamepad**.
///     Use the [`InputMap::set_gamepad`] for specific ones.
/// - Activation: Only if the axis is currently held in the chosen direction.
/// - Single-Axis Value:
///   - `1.0`: The input is currently active.
///   - `0.0`: The input is inactive.
///
/// [`InputMap::set_gamepad`]: crate::input_map::InputMap::set_gamepad
///
/// ```rust,ignore
/// use bevy::prelude::*;
/// use bevy::input::InputPlugin;
/// use bevy::input::gamepad::GamepadEvent;
/// use leafwing_input_manager::prelude::*;
///
/// let mut app = App::new();
/// app.add_plugins(InputPlugin);
///
/// // Positive Y-axis movement on left stick
/// let input = GamepadControlDirection::LEFT_UP;
///
/// // Movement in the opposite direction doesn't activate the input
/// app.send_axis_values(GamepadControlAxis::LEFT_Y, [-1.0]);
/// app.update();
/// assert!(!app.pressed(input));
///
/// // Movement in the chosen direction activates the input
/// app.send_axis_values(GamepadControlAxis::LEFT_Y, [1.0]);
/// app.update();
/// assert!(app.pressed(input));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct GamepadControlDirection {
    /// The axis that this input tracks.
    pub(crate) axis: GamepadAxisType,

    /// The direction of the axis to monitor (positive or negative).
    pub(crate) side: AxisDirection,
}

impl GamepadControlDirection {
    /// Creates a [`GamepadControlDirection`] triggered by a negative value on the specified `axis`.
    #[inline]
    pub const fn negative(axis: GamepadAxisType) -> Self {
        let side = AxisDirection::Negative;
        Self { axis, side }
    }

    /// Creates a [`GamepadControlDirection`] triggered by a positive value on the specified `axis`.
    #[inline]
    pub const fn positive(axis: GamepadAxisType) -> Self {
        let side = AxisDirection::Positive;
        Self { axis, side }
    }

    /// "Up" on the left analog stick (positive Y-axis movement).
    pub const LEFT_UP: Self = Self::positive(GamepadAxisType::LeftStickY);

    /// "Down" on the left analog stick (negative Y-axis movement).
    pub const LEFT_DOWN: Self = Self::negative(GamepadAxisType::LeftStickY);

    /// "Left" on the left analog stick (negative X-axis movement).
    pub const LEFT_LEFT: Self = Self::negative(GamepadAxisType::LeftStickX);

    /// "Right" on the left analog stick (positive X-axis movement).
    pub const LEFT_RIGHT: Self = Self::positive(GamepadAxisType::LeftStickX);

    /// "Up" on the right analog stick (positive Y-axis movement).
    pub const RIGHT_UP: Self = Self::positive(GamepadAxisType::RightStickY);

    /// "Down" on the right analog stick (positive Y-axis movement).
    pub const RIGHT_DOWN: Self = Self::negative(GamepadAxisType::RightStickY);

    /// "Left" on the right analog stick (positive X-axis movement).
    pub const RIGHT_LEFT: Self = Self::negative(GamepadAxisType::RightStickX);

    /// "Right" on the right analog stick (positive X-axis movement).
    pub const RIGHT_RIGHT: Self = Self::positive(GamepadAxisType::RightStickX);
}

#[serde_typetag]
impl UserInput for GamepadControlDirection {
    /// [`GamepadControlDirection`] acts as a virtual button.
    #[inline]
    fn kind(&self) -> InputControlKind {
        InputControlKind::Button
    }

    /// [`GamepadControlDirection`] represents a simple virtual button.
    #[inline]
    fn decompose(&self) -> BasicInputs {
        BasicInputs::Simple(Box::new(*self))
    }

    /// Creates a [`RawInputs`] from the direction directly.
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        RawInputs::from_gamepad_control_directions([*self])
    }
}

impl Buttonlike for GamepadControlDirection {
    /// Checks if there is any recent stick movement along the specified direction.
    #[must_use]
    #[inline]
    fn pressed(&self, input_streams: &InputStreams) -> bool {
        let value = read_axis_value(input_streams, self.axis);
        self.side.is_active(value)
    }
}

/// A wrapper around a specific [`GamepadAxisType`] (e.g., left stick X-axis, right stick Y-axis).
///
/// # Behaviors
///
/// - Gamepad Selection: By default, reads from **any connected gamepad**.
///     Use the [`InputMap::set_gamepad`] for specific ones.
/// - Raw Value: Captures the raw value on the axis, ranging from `-1.0` to `1.0`.
/// - Value Processing: Configure a pipeline to modify the raw value before use,
///     see [`WithAxisProcessingPipelineExt`] for details.
/// - Activation: Only if the processed value is non-zero.
///
/// [`InputMap::set_gamepad`]: crate::input_map::InputMap::set_gamepad
///
/// ```rust,ignore
/// use bevy::prelude::*;
/// use bevy::input::InputPlugin;
/// use leafwing_input_manager::prelude::*;
///
/// let mut app = App::new();
/// app.add_plugins(InputPlugin);
///
/// // Y-axis movement on left stick
/// let input = GamepadControlAxis::LEFT_Y;
///
/// // Movement on the chosen axis activates the input
/// app.send_axis_values(GamepadControlAxis::LEFT_Y, [1.0]);
/// app.update();
/// assert_eq!(app.read_axis_value(input), 1.0);
///
/// // You can configure a processing pipeline (e.g., doubling the value)
/// let doubled = GamepadControlAxis::LEFT_Y.sensitivity(2.0);
/// assert_eq!(app.read_axis_value(doubled), 2.0);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct GamepadControlAxis {
    /// The wrapped axis.
    pub(crate) axis: GamepadAxisType,

    /// A processing pipeline that handles input values.
    pub(crate) processors: Vec<AxisProcessor>,
}

impl GamepadControlAxis {
    /// Creates a [`GamepadControlAxis`] for continuous input from the given axis.
    /// No processing is applied to raw data from the gamepad.
    #[inline]
    pub const fn new(axis: GamepadAxisType) -> Self {
        Self {
            axis,
            processors: Vec::new(),
        }
    }

    /// The horizontal axis (X-axis) of the left stick.
    /// No processing is applied to raw data from the gamepad.
    pub const LEFT_X: Self = Self::new(GamepadAxisType::LeftStickX);

    /// The vertical axis (Y-axis) of the left stick.
    /// No processing is applied to raw data from the gamepad.
    pub const LEFT_Y: Self = Self::new(GamepadAxisType::LeftStickY);

    /// The left `Z` button. No processing is applied to raw data from the gamepad.
    pub const LEFT_Z: Self = Self::new(GamepadAxisType::LeftZ);

    /// The horizontal axis (X-axis) of the right stick.
    /// No processing is applied to raw data from the gamepad.
    pub const RIGHT_X: Self = Self::new(GamepadAxisType::RightStickX);

    /// The vertical axis (Y-axis) of the right stick.
    /// No processing is applied to raw data from the gamepad.
    pub const RIGHT_Y: Self = Self::new(GamepadAxisType::RightStickY);

    /// The right `Z` button. No processing is applied to raw data from the gamepad.
    pub const RIGHT_Z: Self = Self::new(GamepadAxisType::RightZ);
}

#[serde_typetag]
impl UserInput for GamepadControlAxis {
    /// [`GamepadControlAxis`] acts as an axis input.
    #[inline]
    fn kind(&self) -> InputControlKind {
        InputControlKind::Axis
    }

    /// [`GamepadControlAxis`] represents a composition of two [`GamepadControlDirection`]s.
    #[inline]
    fn decompose(&self) -> BasicInputs {
        BasicInputs::Composite(vec![
            Box::new(GamepadControlDirection::negative(self.axis)),
            Box::new(GamepadControlDirection::positive(self.axis)),
        ])
    }

    /// Creates a [`RawInputs`] from the [`GamepadAxisType`] used by the axis.
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        RawInputs::from_gamepad_axes([self.axis])
    }
}

impl Axislike for GamepadControlAxis {
    /// Retrieves the current value of this axis after processing by the associated processors.
    #[must_use]
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        let value = read_axis_value(input_streams, self.axis);
        self.processors
            .iter()
            .fold(value, |value, processor| processor.process(value))
    }
}

impl WithAxisProcessingPipelineExt for GamepadControlAxis {
    #[inline]
    fn reset_processing_pipeline(mut self) -> Self {
        self.processors.clear();
        self
    }

    #[inline]
    fn replace_processing_pipeline(
        mut self,
        processors: impl IntoIterator<Item = AxisProcessor>,
    ) -> Self {
        self.processors = processors.into_iter().collect();
        self
    }

    #[inline]
    fn with_processor(mut self, processor: impl Into<AxisProcessor>) -> Self {
        self.processors.push(processor.into());
        self
    }
}

/// A gamepad stick (e.g., left stick and right stick).
///
/// # Behaviors
///
/// - Gamepad Selection: By default, reads from **any connected gamepad**.
///     Use the [`InputMap::set_gamepad`] for specific ones.
/// - Raw Value: Captures the raw value on both axes, ranging from `-1.0` to `1.0`.
/// - Value Processing: Configure a pipeline to modify the raw value before use,
///     see [`WithDualAxisProcessingPipelineExt`] for details.
/// - Activation: Only if its processed value is non-zero on either axis.
/// - Single-Axis Value: Reports the magnitude of the processed value.
///
/// [`InputMap::set_gamepad`]: crate::input_map::InputMap::set_gamepad
///
/// ```rust,ignore
/// use bevy::prelude::*;
/// use bevy::input::InputPlugin;
/// use leafwing_input_manager::prelude::*;
///
/// let mut app = App::new();
/// app.add_plugins(InputPlugin);
///
/// // Left stick
/// let input = GamepadStick::LEFT;
///
/// // Movement on either axis activates the input
/// app.send_axis_values(GamepadControlAxis::LEFT_Y, [1.0]);
/// app.update();
/// assert_eq!(app.read_axis_values(input), [0.0, 1.0]);
///
/// // You can configure a processing pipeline (e.g., doubling the Y value)
/// let doubled = GamepadStick::LEFT.sensitivity_y(2.0);
/// assert_eq!(app.read_axis_values(doubled), [2.0]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct GamepadStick {
    /// Horizontal movement of the stick.
    pub(crate) x: GamepadAxisType,

    /// Vertical movement of the stick.
    pub(crate) y: GamepadAxisType,

    /// A processing pipeline that handles input values.
    pub(crate) processors: Vec<DualAxisProcessor>,
}

impl GamepadStick {
    /// The left gamepad stick. No processing is applied to raw data from the gamepad.
    pub const LEFT: Self = Self {
        x: GamepadAxisType::LeftStickX,
        y: GamepadAxisType::LeftStickY,
        processors: Vec::new(),
    };

    /// The right gamepad stick. No processing is applied to raw data from the gamepad.
    pub const RIGHT: Self = Self {
        x: GamepadAxisType::RightStickX,
        y: GamepadAxisType::RightStickY,
        processors: Vec::new(),
    };

    /// Retrieves the current X and Y values of this stick after processing by the associated processors.
    #[must_use]
    #[inline]
    fn processed_value(&self, input_streams: &InputStreams) -> Vec2 {
        let x = read_axis_value(input_streams, self.x);
        let y = read_axis_value(input_streams, self.y);
        self.processors
            .iter()
            .fold(Vec2::new(x, y), |value, processor| processor.process(value))
    }
}

#[serde_typetag]
impl UserInput for GamepadStick {
    /// [`GamepadStick`] acts as a dual-axis input.
    #[inline]
    fn kind(&self) -> InputControlKind {
        InputControlKind::DualAxis
    }

    /// [`GamepadStick`] represents a composition of four [`GamepadControlDirection`]s.
    #[inline]
    fn decompose(&self) -> BasicInputs {
        BasicInputs::Composite(vec![
            Box::new(GamepadControlDirection::negative(self.x)),
            Box::new(GamepadControlDirection::positive(self.x)),
            Box::new(GamepadControlDirection::negative(self.y)),
            Box::new(GamepadControlDirection::positive(self.y)),
        ])
    }

    /// Creates a [`RawInputs`] from two [`GamepadAxisType`]s used by the stick.
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        RawInputs::from_gamepad_axes([self.x, self.y])
    }
}

impl DualAxislike for GamepadStick {
    /// Retrieves the current X and Y values of this stick after processing by the associated processors.
    #[must_use]
    #[inline]
    fn axis_pair(&self, input_streams: &InputStreams) -> Vec2 {
        self.processed_value(input_streams)
    }
}

impl WithDualAxisProcessingPipelineExt for GamepadStick {
    #[inline]
    fn reset_processing_pipeline(mut self) -> Self {
        self.processors.clear();
        self
    }

    #[inline]
    fn replace_processing_pipeline(
        mut self,
        processor: impl IntoIterator<Item = DualAxisProcessor>,
    ) -> Self {
        self.processors = processor.into_iter().collect();
        self
    }

    #[inline]
    fn with_processor(mut self, processor: impl Into<DualAxisProcessor>) -> Self {
        self.processors.push(processor.into());
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

/// Checks if the given [`GamepadButtonType`] is currently pressed.
#[must_use]
#[inline]
fn button_pressed_any(input_streams: &InputStreams, button: GamepadButtonType) -> bool {
    input_streams
        .gamepads
        .iter()
        .any(|gamepad| button_pressed(input_streams, gamepad, button))
}

/// Retrieves the current value of the given [`GamepadButtonType`].
#[must_use]
#[inline]
fn button_value(
    input_streams: &InputStreams,
    gamepad: Gamepad,
    button: GamepadButtonType,
) -> Option<f32> {
    // This implementation differs from `button_pressed()`
    // because the upstream bevy::input still waffles about whether triggers are buttons or axes.
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
    f32::from(button_pressed_any(input_streams, button))
}

// Built-in support for Bevy's GamepadButtonType.
#[serde_typetag]
impl UserInput for GamepadButtonType {
    /// [`GamepadButtonType`] acts as a button.
    #[inline]
    fn kind(&self) -> InputControlKind {
        InputControlKind::Button
    }

    /// Creates a [`BasicInputs`] that only contains the [`GamepadButtonType`] itself,
    /// as it represents a simple physical button.
    #[inline]
    fn decompose(&self) -> BasicInputs {
        BasicInputs::Simple(Box::new(*self))
    }

    /// Creates a [`RawInputs`] from the button directly.
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        RawInputs::from_gamepad_buttons([*self])
    }
}

impl Buttonlike for GamepadButtonType {
    /// Checks if the specified button is currently pressed down.
    #[must_use]
    #[inline]
    fn pressed(&self, input_streams: &InputStreams) -> bool {
        if let Some(gamepad) = input_streams.associated_gamepad {
            button_pressed(input_streams, gamepad, *self)
        } else {
            button_pressed_any(input_streams, *self)
        }
    }
}

/// A virtual single-axis control constructed by combining two [`GamepadButtonType`]s.
/// One button represents the negative direction (left for the X-axis, down for the Y-axis),
/// while the other represents the positive direction (right for the X-axis, up for the Y-axis).
///
/// # Behaviors
///
/// - Gamepad Selection: By default, reads from **any connected gamepad**.
///     Use the [`InputMap::set_gamepad`] for specific ones.
/// - Raw Value:
///   - `-1.0`: Only the negative button is currently pressed.
///   - `1.0`: Only the positive button is currently pressed.
///   - `0.0`: Neither button is pressed, or both are pressed simultaneously.
/// - Value Processing: Configure a pipeline to modify the raw value before use,
///     see [`WithAxisProcessingPipelineExt`] for details.
/// - Activation: Only if the processed value is non-zero.
///
/// [`InputMap::set_gamepad`]: crate::input_map::InputMap::set_gamepad
///
/// ```rust,ignore
/// use bevy::prelude::*;
/// use bevy::input::InputPlugin;
/// use leafwing_input_manager::prelude::*;
///
/// let mut app = App::new();
/// app.add_plugins(InputPlugin);
///
/// // Define a virtual Y-axis using D-pad "up" and "down" buttons
/// let axis = GamepadVirtualAxis::DPAD_Y;
///
/// // Pressing either button activates the input
/// app.press_input(GamepadButtonType::DPadUp);
/// app.update();
/// assert_eq!(app.read_axis_values(axis), [1.0]);
///
/// // You can configure a processing pipeline (e.g., doubling the value)
/// let doubled = GamepadVirtualAxis::DPAD_Y.sensitivity(2.0);
/// assert_eq!(app.read_axis_values(doubled), [2.0]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct GamepadVirtualAxis {
    /// The button that represents the negative direction.
    pub(crate) negative: GamepadButtonType,

    /// The button that represents the positive direction.
    pub(crate) positive: GamepadButtonType,

    /// A processing pipeline that handles input values.
    pub(crate) processors: Vec<AxisProcessor>,
}

impl GamepadVirtualAxis {
    /// Creates a new [`GamepadVirtualAxis`] with two given [`GamepadButtonType`]s.
    /// No processing is applied to raw data from the gamepad.
    #[inline]
    pub const fn new(negative: GamepadButtonType, positive: GamepadButtonType) -> Self {
        Self {
            negative,
            positive,
            processors: Vec::new(),
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
    /// [`GamepadVirtualAxis`] acts as an axis input.
    #[inline]
    fn kind(&self) -> InputControlKind {
        InputControlKind::Axis
    }

    /// Returns the two [`GamepadButtonType`]s used by this axis.
    #[inline]
    fn decompose(&self) -> BasicInputs {
        BasicInputs::Composite(vec![Box::new(self.negative), Box::new(self.positive)])
    }

    /// Creates a [`RawInputs`] from two [`GamepadButtonType`]s used by this axis.
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        RawInputs::from_gamepad_buttons([self.negative, self.positive])
    }
}

impl Axislike for GamepadVirtualAxis {
    /// Retrieves the current value of this axis after processing by the associated processors.
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
        self.processors
            .iter()
            .fold(value, |value, processor| processor.process(value))
    }
}

impl WithAxisProcessingPipelineExt for GamepadVirtualAxis {
    #[inline]
    fn reset_processing_pipeline(mut self) -> Self {
        self.processors.clear();
        self
    }

    #[inline]
    fn replace_processing_pipeline(
        mut self,
        processors: impl IntoIterator<Item = AxisProcessor>,
    ) -> Self {
        self.processors = processors.into_iter().collect();
        self
    }

    #[inline]
    fn with_processor(mut self, processor: impl Into<AxisProcessor>) -> Self {
        self.processors.push(processor.into());
        self
    }
}

/// A virtual dual-axis control constructed from four [`GamepadButtonType`]s.
/// Each button represents a specific direction (up, down, left, right),
/// functioning similarly to a directional pad (D-pad) on both X and Y axes,
/// and offering intermediate diagonals by means of two-button combinations.
///
/// # Behaviors
///
/// - Gamepad Selection: By default, reads from **any connected gamepad**.
///     Use the [`InputMap::set_gamepad`] for specific ones.
/// - Raw Value: Each axis behaves as follows:
///   - `-1.0`: Only the negative button is currently pressed (Down/Left).
///   - `1.0`: Only the positive button is currently pressed (Up/Right).
///   - `0.0`: Neither button is pressed, or both buttons on the same axis are pressed simultaneously.
/// - Value Processing: Configure a pipeline to modify the raw value before use,
///     see [`WithDualAxisProcessingPipelineExt`] for details.
/// - Activation: Only if the processed value is non-zero on either axis.
///
/// [`InputMap::set_gamepad`]: crate::input_map::InputMap::set_gamepad
///
/// ```rust,ignore
/// use bevy::prelude::*;
/// use bevy::input::InputPlugin;
/// use leafwing_input_manager::prelude::*;
///
/// let mut app = App::new();
/// app.add_plugins(InputPlugin);
///
/// // Define a virtual D-pad using the physical D-pad buttons
/// let input = GamepadVirtualDPad::DPAD;
///
/// // Pressing a D-pad button activates the corresponding axis
/// app.press_input(GamepadButtonType::DPadUp);
/// app.update();
/// assert_eq!(app.read_axis_values(input), [0.0, 1.0]);
///
/// // You can configure a processing pipeline (e.g., doubling the Y value)
/// let doubled = GamepadVirtualDPad::DPAD.sensitivity_y(2.0);
/// assert_eq!(app.read_axis_values(doubled), [0.0, 2.0]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct GamepadVirtualDPad {
    /// The button for the upward direction.
    pub(crate) up: GamepadButtonType,

    /// The button for the downward direction.
    pub(crate) down: GamepadButtonType,

    /// The button for the leftward direction.
    pub(crate) left: GamepadButtonType,

    /// The button for the rightward direction.
    pub(crate) right: GamepadButtonType,

    /// A processing pipeline that handles input values.
    pub(crate) processors: Vec<DualAxisProcessor>,
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
            processors: Vec::new(),
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

    /// Retrieves the current X and Y values of this D-pad after processing by the associated processors.
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
        self.processors
            .iter()
            .fold(value, |value, processor| processor.process(value))
    }
}

#[serde_typetag]
impl UserInput for GamepadVirtualDPad {
    /// [`GamepadVirtualDPad`] acts as a dual-axis input.
    #[inline]
    fn kind(&self) -> InputControlKind {
        InputControlKind::DualAxis
    }

    /// Returns the four [`GamepadButtonType`]s used by this D-pad.
    #[inline]
    fn decompose(&self) -> BasicInputs {
        BasicInputs::Composite(vec![
            Box::new(self.up),
            Box::new(self.down),
            Box::new(self.left),
            Box::new(self.right),
        ])
    }

    /// Creates a [`RawInputs`] from four [`GamepadButtonType`]s used by this D-pad.
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        RawInputs::from_gamepad_buttons([self.up, self.down, self.left, self.right])
    }
}

impl DualAxislike for GamepadVirtualDPad {
    /// Retrieves the current X and Y values of this D-pad after processing by the associated processors.
    #[must_use]
    #[inline]
    fn axis_pair(&self, input_streams: &InputStreams) -> Vec2 {
        self.processed_value(input_streams)
    }
}

impl WithDualAxisProcessingPipelineExt for GamepadVirtualDPad {
    #[inline]
    fn reset_processing_pipeline(mut self) -> Self {
        self.processors.clear();
        self
    }

    #[inline]
    fn replace_processing_pipeline(
        mut self,
        processor: impl IntoIterator<Item = DualAxisProcessor>,
    ) -> Self {
        self.processors = processor.into_iter().collect();
        self
    }

    #[inline]
    fn with_processor(mut self, processor: impl Into<DualAxisProcessor>) -> Self {
        self.processors.push(processor.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input_mocking::MockInput;
    use crate::plugin::AccumulatorPlugin;
    use bevy::input::gamepad::{
        GamepadConnection, GamepadConnectionEvent, GamepadEvent, GamepadInfo,
    };
    use bevy::input::InputPlugin;
    use bevy::prelude::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(InputPlugin)
            .add_plugins(AccumulatorPlugin);

        // WARNING: you MUST register your gamepad during tests,
        // or all gamepad input mocking actions will fail
        let mut gamepad_events = app.world_mut().resource_mut::<Events<GamepadEvent>>();
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

    #[test]
    fn test_gamepad_axes() {
        let left_up = GamepadControlDirection::LEFT_UP;
        assert_eq!(left_up.kind(), InputControlKind::Button);
        let raw_inputs = RawInputs::from_gamepad_control_directions([left_up]);
        assert_eq!(left_up.raw_inputs(), raw_inputs);

        // The opposite of left up
        let left_down = GamepadControlDirection::LEFT_DOWN;
        assert_eq!(left_down.kind(), InputControlKind::Button);
        let raw_inputs = RawInputs::from_gamepad_control_directions([left_up]);
        assert_eq!(left_up.raw_inputs(), raw_inputs);

        let left_x = GamepadControlAxis::LEFT_X;
        assert_eq!(left_x.kind(), InputControlKind::Axis);
        let raw_inputs = RawInputs::from_gamepad_axes([left_x.axis]);
        assert_eq!(left_x.raw_inputs(), raw_inputs);

        let left_y = GamepadControlAxis::LEFT_Y;
        assert_eq!(left_y.kind(), InputControlKind::Axis);
        let raw_inputs = RawInputs::from_gamepad_axes([left_y.axis]);
        assert_eq!(left_y.raw_inputs(), raw_inputs);

        let left = GamepadStick::LEFT;
        assert_eq!(left.kind(), InputControlKind::DualAxis);
        let raw_inputs = RawInputs::from_gamepad_axes([left.x, left.y]);
        assert_eq!(left.raw_inputs(), raw_inputs);

        // Up; but for the other stick
        let right_up = GamepadControlDirection::RIGHT_DOWN;
        assert_eq!(right_up.kind(), InputControlKind::Button);
        let raw_inputs = RawInputs::from_gamepad_control_directions([right_up]);
        assert_eq!(right_up.raw_inputs(), raw_inputs);

        let right_y = GamepadControlAxis::RIGHT_Y;
        assert_eq!(right_y.kind(), InputControlKind::Axis);
        let raw_inputs = RawInputs::from_gamepad_axes([right_y.axis]);
        assert_eq!(right_y.raw_inputs(), raw_inputs);

        let right = GamepadStick::RIGHT;
        assert_eq!(right.kind(), InputControlKind::DualAxis);
        let raw_inputs = RawInputs::from_gamepad_axes([right_y.axis]);
        assert_eq!(right_y.raw_inputs(), raw_inputs);

        // No inputs
        let mut app = test_app();
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);

        assert!(!left_up.pressed(&inputs));
        assert!(!left_down.pressed(&inputs));
        assert!(!right_up.pressed(&inputs));
        assert_eq!(left_x.value(&inputs), 0.0);
        assert_eq!(left_y.value(&inputs), 0.0);
        assert_eq!(right_y.value(&inputs), 0.0);
        assert_eq!(left.axis_pair(&inputs), Vec2::ZERO);
        assert_eq!(right.axis_pair(&inputs), Vec2::ZERO);

        // Left stick moves upward
        let data = Vec2::new(0.0, 1.0);
        let mut app = test_app();
        app.press_input(GamepadControlDirection::LEFT_UP);
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);

        assert!(left_up.pressed(&inputs));
        assert!(!left_down.pressed(&inputs));
        assert!(!right_up.pressed(&inputs));
        assert_eq!(left_x.value(&inputs), 0.0);
        assert_eq!(left_y.value(&inputs), 1.0);
        assert_eq!(right_y.value(&inputs), 0.0);
        assert_eq!(left.axis_pair(&inputs), data);
        assert_eq!(right.axis_pair(&inputs), Vec2::ZERO);

        // Set Y-axis of left stick to 0.6
        let data = Vec2::new(0.0, 0.6);
        let mut app = test_app();
        app.send_axis_values(GamepadControlAxis::LEFT_Y, [data.y]);
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);

        assert!(left_up.pressed(&inputs));
        assert!(!left_down.pressed(&inputs));
        assert!(!right_up.pressed(&inputs));
        assert_eq!(left_x.value(&inputs), 0.0);
        assert_eq!(left_y.value(&inputs), 0.6);
        assert_eq!(right_y.value(&inputs), 0.0);
        assert_eq!(left.axis_pair(&inputs), data);
        assert_eq!(right.axis_pair(&inputs), Vec2::ZERO);

        // Set left stick to (0.6, 0.4)
        let data = Vec2::new(0.6, 0.4);
        let mut app = test_app();
        app.send_axis_values(GamepadStick::LEFT, [data.x, data.y]);
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);

        assert!(left_up.pressed(&inputs));
        assert!(!left_down.pressed(&inputs));
        assert!(!right_up.pressed(&inputs));
        assert_eq!(left_x.value(&inputs), data.x);
        assert_eq!(left_y.value(&inputs), data.y);
        assert_eq!(right_y.value(&inputs), 0.0);
        assert_eq!(left.axis_pair(&inputs), data);
        assert_eq!(right.axis_pair(&inputs), Vec2::ZERO);
    }

    #[test]
    #[ignore = "Input mocking is subtly broken: https://github.com/Leafwing-Studios/leafwing-input-manager/issues/516"]
    fn test_gamepad_buttons() {
        let up = GamepadButtonType::DPadUp;
        assert_eq!(up.kind(), InputControlKind::Button);
        let raw_inputs = RawInputs::from_gamepad_buttons([up]);
        assert_eq!(up.raw_inputs(), raw_inputs);

        let left = GamepadButtonType::DPadLeft;
        assert_eq!(left.kind(), InputControlKind::Button);
        let raw_inputs = RawInputs::from_gamepad_buttons([left]);
        assert_eq!(left.raw_inputs(), raw_inputs);

        let down = GamepadButtonType::DPadDown;
        assert_eq!(left.kind(), InputControlKind::Button);
        let raw_inputs = RawInputs::from_gamepad_buttons([down]);
        assert_eq!(down.raw_inputs(), raw_inputs);

        let right = GamepadButtonType::DPadRight;
        assert_eq!(left.kind(), InputControlKind::Button);
        let raw_inputs = RawInputs::from_gamepad_buttons([right]);
        assert_eq!(right.raw_inputs(), raw_inputs);

        let x_axis = GamepadVirtualAxis::DPAD_X;
        assert_eq!(x_axis.kind(), InputControlKind::Axis);
        let raw_inputs = RawInputs::from_gamepad_buttons([left, right]);
        assert_eq!(x_axis.raw_inputs(), raw_inputs);

        let y_axis = GamepadVirtualAxis::DPAD_Y;
        assert_eq!(y_axis.kind(), InputControlKind::Axis);
        let raw_inputs = RawInputs::from_gamepad_buttons([down, up]);
        assert_eq!(y_axis.raw_inputs(), raw_inputs);

        let dpad = GamepadVirtualDPad::DPAD;
        assert_eq!(dpad.kind(), InputControlKind::DualAxis);
        let raw_inputs = RawInputs::from_gamepad_buttons([up, down, left, right]);
        assert_eq!(dpad.raw_inputs(), raw_inputs);

        // No inputs
        let zeros = Vec2::new(0.0, 0.0);
        let mut app = test_app();
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);

        assert!(!up.pressed(&inputs));
        assert!(!left.pressed(&inputs));
        assert!(!down.pressed(&inputs));
        assert!(!right.pressed(&inputs));
        assert_eq!(x_axis.value(&inputs), 0.0);
        assert_eq!(y_axis.value(&inputs), 0.0);
        assert_eq!(dpad.axis_pair(&inputs), zeros);

        // Press DPadLeft
        let data = Vec2::new(1.0, 0.0);
        let mut app = test_app();
        app.press_input(GamepadButtonType::DPadLeft);
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);

        assert!(!up.pressed(&inputs));
        assert!(left.pressed(&inputs));
        assert!(!down.pressed(&inputs));
        assert!(!right.pressed(&inputs));
        assert_eq!(x_axis.value(&inputs), 1.0);
        assert_eq!(y_axis.value(&inputs), 0.0);
        assert_eq!(dpad.axis_pair(&inputs), data);

        // Set the X-axis to 0.6
        let data = Vec2::new(0.6, 0.0);
        let mut app = test_app();
        app.send_axis_values(GamepadVirtualAxis::DPAD_X, [data.x]);
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);

        assert!(!up.pressed(&inputs));
        assert!(left.pressed(&inputs));
        assert!(!down.pressed(&inputs));
        assert!(!right.pressed(&inputs));
        assert_eq!(x_axis.value(&inputs), 0.6);
        assert_eq!(y_axis.value(&inputs), 0.0);
        assert_eq!(dpad.axis_pair(&inputs), data);

        // Set the axes to (0.6, 0.4)
        let data = Vec2::new(0.6, 0.4);
        let mut app = test_app();
        app.send_axis_values(GamepadVirtualDPad::DPAD, [data.x, data.y]);
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);

        assert!(!up.pressed(&inputs));
        assert!(left.pressed(&inputs));
        assert!(!down.pressed(&inputs));
        assert!(!right.pressed(&inputs));
        assert_eq!(x_axis.value(&inputs), data.x);
        assert_eq!(y_axis.value(&inputs), data.y);
        assert_eq!(dpad.axis_pair(&inputs), data);
    }
}
