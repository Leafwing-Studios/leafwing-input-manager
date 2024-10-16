//! Gamepad inputs

use std::hash::{Hash, Hasher};

use bevy::ecs::system::lifetimeless::SRes;
use bevy::ecs::system::{StaticSystemParam, SystemParam};
use bevy::input::gamepad::{GamepadAxisChangedEvent, GamepadButtonChangedEvent, GamepadEvent};
use bevy::input::{Axis, ButtonInput};
use bevy::math::FloatOrd;
use bevy::prelude::{
    Events, Gamepad, GamepadAxis, GamepadAxisType, GamepadButton, GamepadButtonType, Gamepads,
    Reflect, Res, ResMut, Vec2, World,
};
use leafwing_input_manager_macros::serde_typetag;
use serde::{Deserialize, Serialize};

use crate as leafwing_input_manager;
use crate::axislike::AxisDirection;
use crate::buttonlike::ButtonValue;
use crate::clashing_inputs::BasicInputs;
use crate::input_processing::{
    AxisProcessor, DualAxisProcessor, WithAxisProcessingPipelineExt,
    WithDualAxisProcessingPipelineExt,
};
use crate::user_input::UserInput;
use crate::InputControlKind;

use super::updating::{CentralInputStore, UpdatableInput};
use super::{Axislike, Buttonlike, DualAxislike};

/// Retrieves the first connected gamepad.
///
/// If no gamepad is connected, a synthetic gamepad with an ID of 0 is returned.
#[must_use]
pub fn find_gamepad(gamepads: &Gamepads) -> Gamepad {
    gamepads.iter().next().unwrap_or(Gamepad { id: 0 })
}

/// Retrieves the current value of the specified `axis`.
#[must_use]
#[inline]
fn read_axis_value(
    input_store: &CentralInputStore,
    gamepad: Gamepad,
    axis: GamepadAxisType,
) -> f32 {
    let axis = GamepadAxis::new(gamepad, axis);
    input_store.value(&axis)
}

/// Provides button-like behavior for a specific direction on a [`GamepadAxisType`].
///
/// By default, it reads from **any connected gamepad**.
/// Use the [`InputMap::set_gamepad`](crate::input_map::InputMap::set_gamepad) for specific ones.
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
/// GamepadControlAxis::LEFT_Y.set_value(app.world_mut(), -1.0);
/// app.update();
/// assert!(!app.read_pressed(input));
///
/// // Movement in the chosen direction activates the input
/// GamepadControlAxis::LEFT_Y.set_value(app.world_mut(), 1.0);
/// app.update();
/// assert!(app.read_pressed(input));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct GamepadControlDirection {
    /// The axis that this input tracks.
    pub axis: GamepadAxisType,

    /// The direction of the axis to monitor (positive or negative).
    pub direction: AxisDirection,

    /// The threshold value for the direction to be considered pressed.
    /// Must be non-negative.
    pub threshold: f32,
}

impl GamepadControlDirection {
    /// Creates a [`GamepadControlDirection`] triggered by a negative value on the specified `axis`.
    #[inline]
    pub const fn negative(axis: GamepadAxisType) -> Self {
        Self {
            axis,
            direction: AxisDirection::Negative,
            threshold: 0.0,
        }
    }

    /// Creates a [`GamepadControlDirection`] triggered by a positive value on the specified `axis`.
    #[inline]
    pub const fn positive(axis: GamepadAxisType) -> Self {
        Self {
            axis,
            direction: AxisDirection::Positive,
            threshold: 0.0,
        }
    }

    /// Sets the `threshold` value.
    ///
    /// # Requirements
    ///
    /// - `threshold` >= `0.0`.
    ///
    /// # Panics
    ///
    /// Panics if the requirement isn't met.
    #[inline]
    pub fn threshold(mut self, threshold: f32) -> Self {
        assert!(threshold >= 0.0);
        self.threshold = threshold;
        self
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

    /// "Down" on the right analog stick (negative Y-axis movement).
    pub const RIGHT_DOWN: Self = Self::negative(GamepadAxisType::RightStickY);

    /// "Left" on the right analog stick (negative X-axis movement).
    pub const RIGHT_LEFT: Self = Self::negative(GamepadAxisType::RightStickX);

    /// "Right" on the right analog stick (positive X-axis movement).
    pub const RIGHT_RIGHT: Self = Self::positive(GamepadAxisType::RightStickX);
}

impl UserInput for GamepadControlDirection {
    /// [`GamepadControlDirection`] acts as a virtual button.
    #[inline]
    fn kind(&self) -> InputControlKind {
        InputControlKind::Button
    }

    /// [`GamepadControlDirection`] represents a simple virtual button.
    #[inline]
    fn decompose(&self) -> BasicInputs {
        BasicInputs::Simple(Box::new((*self).threshold(0.0)))
    }
}

#[serde_typetag]
impl Buttonlike for GamepadControlDirection {
    /// Checks if there is any recent stick movement along the specified direction.
    #[must_use]
    #[inline]
    fn pressed(&self, input_store: &CentralInputStore, gamepad: Gamepad) -> bool {
        let value = read_axis_value(input_store, gamepad, self.axis);
        self.direction.is_active(value, self.threshold)
    }

    /// Sends a [`GamepadEvent::Axis`] event with a magnitude of 1.0 for the specified direction on the provided [`Gamepad`].
    fn press_as_gamepad(&self, world: &mut World, gamepad: Option<Gamepad>) {
        let gamepad = gamepad.unwrap_or(find_gamepad(world.resource::<Gamepads>()));

        let event = GamepadEvent::Axis(GamepadAxisChangedEvent {
            gamepad,
            axis_type: self.axis,
            value: self.direction.full_active_value(),
        });
        world.resource_mut::<Events<GamepadEvent>>().send(event);
    }

    /// Sends a [`GamepadEvent::Axis`] event with a magnitude of 0.0 for the specified direction.
    fn release_as_gamepad(&self, world: &mut World, gamepad: Option<Gamepad>) {
        let gamepad = gamepad.unwrap_or(find_gamepad(world.resource::<Gamepads>()));

        let event = GamepadEvent::Axis(GamepadAxisChangedEvent {
            gamepad,
            axis_type: self.axis,
            value: 0.0,
        });
        world.resource_mut::<Events<GamepadEvent>>().send(event);
    }

    fn set_value_as_gamepad(&self, world: &mut World, value: f32, gamepad: Option<Gamepad>) {
        if value > 0.0 {
            self.press_as_gamepad(world, gamepad);
        } else {
            self.release_as_gamepad(world, gamepad);
        }
    }
}

impl Eq for GamepadControlDirection {}

impl Hash for GamepadControlDirection {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.axis.hash(state);
        self.direction.hash(state);
        FloatOrd(self.threshold).hash(state);
    }
}

impl UpdatableInput for GamepadAxis {
    type SourceData = SRes<Axis<GamepadAxis>>;

    fn compute(
        mut central_input_store: ResMut<CentralInputStore>,
        source_data: StaticSystemParam<Self::SourceData>,
    ) {
        for axis in source_data.devices() {
            let value = source_data.get(*axis).unwrap_or_default();

            central_input_store.update_axislike(*axis, value);
        }
    }
}

/// Unlike [`GamepadButtonType`], this struct represents a specific axis on a specific gamepad.
///
/// In the majority of cases, [`GamepadControlAxis`] or [`GamepadStick`] should be used instead.
impl UserInput for GamepadAxis {
    fn kind(&self) -> InputControlKind {
        InputControlKind::Axis
    }

    fn decompose(&self) -> BasicInputs {
        BasicInputs::Composite(vec![
            Box::new(GamepadControlDirection::negative(self.axis_type)),
            Box::new(GamepadControlDirection::positive(self.axis_type)),
        ])
    }
}

#[serde_typetag]
impl Axislike for GamepadAxis {
    fn value(&self, input_store: &CentralInputStore, gamepad: Gamepad) -> f32 {
        read_axis_value(input_store, gamepad, self.axis_type)
    }
}

/// A wrapper around a specific [`GamepadAxisType`] (e.g., left stick X-axis, right stick Y-axis).
///
/// By default, it reads from **any connected gamepad**.
/// Use the [`InputMap::set_gamepad`](crate::input_map::InputMap::set_gamepad) for specific ones.
///
/// # Value Processing
///
/// You can customize how the values are processed using a pipeline of processors.
/// See [`WithAxisProcessingPipelineExt`] for details.
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
/// GamepadControlAxis::LEFT_Y.set_value(app.world_mut(), 1.0);
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
}

#[serde_typetag]
impl Axislike for GamepadControlAxis {
    /// Retrieves the current value of this axis after processing by the associated processors.
    #[must_use]
    #[inline]
    fn value(&self, input_store: &CentralInputStore, gamepad: Gamepad) -> f32 {
        let value = read_axis_value(input_store, gamepad, self.axis);
        self.processors
            .iter()
            .fold(value, |value, processor| processor.process(value))
    }

    /// Sends a [`GamepadEvent::Axis`] event with the specified value on the provided [`Gamepad`].
    fn set_value_as_gamepad(&self, world: &mut World, value: f32, gamepad: Option<Gamepad>) {
        let gamepad = gamepad.unwrap_or(find_gamepad(world.resource::<Gamepads>()));

        let event = GamepadEvent::Axis(GamepadAxisChangedEvent {
            gamepad,
            axis_type: self.axis,
            value,
        });
        world.resource_mut::<Events<GamepadEvent>>().send(event);
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
/// By default, it reads from **any connected gamepad**.
/// Use the [`InputMap::set_gamepad`](crate::input_map::InputMap::set_gamepad) for specific ones.
///
/// # Value Processing
///
/// You can customize how the values are processed using a pipeline of processors.
/// See [`WithDualAxisProcessingPipelineExt`] for details.
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
/// GamepadControlAxis::LEFT_Y.set_value(app.world_mut(), 1.0);
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
}

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
}

#[serde_typetag]
impl DualAxislike for GamepadStick {
    /// Retrieves the current X and Y values of this stick after processing by the associated processors.
    #[must_use]
    #[inline]
    fn axis_pair(&self, input_store: &CentralInputStore, gamepad: Gamepad) -> Vec2 {
        let x = read_axis_value(input_store, gamepad, self.x);
        let y = read_axis_value(input_store, gamepad, self.y);
        self.processors
            .iter()
            .fold(Vec2::new(x, y), |value, processor| processor.process(value))
    }

    /// Sends a [`GamepadEvent::Axis`] event with the specified values on the provided [`Gamepad`].
    fn set_axis_pair_as_gamepad(&self, world: &mut World, value: Vec2, gamepad: Option<Gamepad>) {
        let gamepad = gamepad.unwrap_or(find_gamepad(world.resource::<Gamepads>()));

        let event = GamepadEvent::Axis(GamepadAxisChangedEvent {
            gamepad,
            axis_type: self.x,
            value: value.x,
        });
        world.resource_mut::<Events<GamepadEvent>>().send(event);

        let event = GamepadEvent::Axis(GamepadAxisChangedEvent {
            gamepad,
            axis_type: self.y,
            value: value.y,
        });
        world.resource_mut::<Events<GamepadEvent>>().send(event);
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
    input_store: &CentralInputStore,
    gamepad: Gamepad,
    button: GamepadButtonType,
) -> bool {
    let button = GamepadButton::new(gamepad, button);
    input_store.pressed(&button)
}

/// Retrieves the current value of the given [`GamepadButtonType`].
///
/// This will be 0.0 if the button is released, and 1.0 if it is pressed.
/// Physically triggerlike buttons will return a value between 0.0 and 1.0,
/// depending on how far the button is pressed.
#[must_use]
#[inline]
fn button_value(
    input_store: &CentralInputStore,
    gamepad: Gamepad,
    button: GamepadButtonType,
) -> f32 {
    let button = GamepadButton::new(gamepad, button);
    input_store.button_value(&button)
}

/// The [`SystemParam`] that combines the [`ButtonInput`] and [`Axis`] resources for [`GamepadButton`]s.
#[derive(SystemParam)]
pub struct GamepadButtonInput<'w> {
    /// The [`ButtonInput`] for [`GamepadButton`]s.
    pub buttons: Res<'w, ButtonInput<GamepadButton>>,

    /// The [`Axis`] for [`GamepadButton`]s.
    pub axes: Res<'w, Axis<GamepadButton>>,
}

impl UpdatableInput for GamepadButton {
    type SourceData = GamepadButtonInput<'static>;

    fn compute(
        mut central_input_store: ResMut<CentralInputStore>,
        source_data: StaticSystemParam<Self::SourceData>,
    ) {
        for button in source_data.buttons.get_pressed() {
            let value = source_data.axes.get(*button).unwrap_or(1.0);
            central_input_store.update_buttonlike(*button, ButtonValue::new(true, value));
        }

        for button in source_data.buttons.get_just_released() {
            let value = source_data.axes.get(*button).unwrap_or(0.0);
            central_input_store.update_buttonlike(*button, ButtonValue::new(false, value));
        }
    }
}

/// Unlike [`GamepadButtonType`], this struct represents a specific button on a specific gamepad.
///
/// In the majority of cases, [`GamepadButtonType`] should be used instead.
impl UserInput for GamepadButton {
    fn kind(&self) -> InputControlKind {
        InputControlKind::Button
    }

    fn decompose(&self) -> BasicInputs {
        BasicInputs::Simple(Box::new(*self))
    }
}

#[serde_typetag]
impl Buttonlike for GamepadButton {
    /// WARNING: The supplied gamepad is ignored, as the button is already specific to a gamepad.
    fn pressed(&self, input_store: &CentralInputStore, _gamepad: Gamepad) -> bool {
        button_pressed(input_store, self.gamepad, self.button_type)
    }

    /// WARNING: The supplied gamepad is ignored, as the button is already specific to a gamepad.
    fn value(&self, input_store: &CentralInputStore, _gamepad: Gamepad) -> f32 {
        button_value(input_store, self.gamepad, self.button_type)
    }

    fn press(&self, world: &mut World) {
        let event = GamepadEvent::Button(GamepadButtonChangedEvent {
            gamepad: self.gamepad,
            button_type: self.button_type,
            value: 1.0,
        });
        world.resource_mut::<Events<GamepadEvent>>().send(event);
    }

    fn release(&self, world: &mut World) {
        let event = GamepadEvent::Button(GamepadButtonChangedEvent {
            gamepad: self.gamepad,
            button_type: self.button_type,
            value: 0.0,
        });
        world.resource_mut::<Events<GamepadEvent>>().send(event);
    }

    fn set_value(&self, world: &mut World, value: f32) {
        let event = GamepadEvent::Button(GamepadButtonChangedEvent {
            gamepad: self.gamepad,
            button_type: self.button_type,
            value,
        });
        world.resource_mut::<Events<GamepadEvent>>().send(event);
    }
}

// Built-in support for Bevy's GamepadButtonType.
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
}

#[serde_typetag]
impl Buttonlike for GamepadButtonType {
    /// Checks if the specified button is currently pressed down.
    #[must_use]
    #[inline]
    fn pressed(&self, input_store: &CentralInputStore, gamepad: Gamepad) -> bool {
        button_pressed(input_store, gamepad, *self)
    }

    /// Retrieves the current value of the specified button.
    ///
    /// This will be 0.0 if the button is released, and 1.0 if it is pressed.
    /// Physically triggerlike buttons will return a value between 0.0 and 1.0,
    /// depending on how far the button is pressed.
    #[must_use]
    #[inline]
    fn value(&self, input_store: &CentralInputStore, gamepad: Gamepad) -> f32 {
        button_value(input_store, gamepad, *self)
    }

    /// Sends a [`GamepadEvent::Button`] event with a magnitude of 1.0 in the direction defined by `self` on the provided [`Gamepad`].
    fn press_as_gamepad(&self, world: &mut World, gamepad: Option<Gamepad>) {
        let gamepad = gamepad.unwrap_or(find_gamepad(world.resource::<Gamepads>()));

        let event = GamepadEvent::Button(GamepadButtonChangedEvent {
            gamepad,
            button_type: *self,
            value: 1.0,
        });
        world.resource_mut::<Events<GamepadEvent>>().send(event);
    }

    /// Sends a [`GamepadEvent::Button`] event with a magnitude of 0.0 in the direction defined by `self` on the provided [`Gamepad`].
    fn release_as_gamepad(&self, world: &mut World, gamepad: Option<Gamepad>) {
        let gamepad = gamepad.unwrap_or(find_gamepad(world.resource::<Gamepads>()));

        let event = GamepadEvent::Button(GamepadButtonChangedEvent {
            gamepad,
            button_type: *self,
            value: 0.0,
        });
        world.resource_mut::<Events<GamepadEvent>>().send(event);
    }

    /// Sends a [`GamepadEvent::Button`] event with the specified value in the direction defined by `self` on the provided [`Gamepad`].
    fn set_value_as_gamepad(&self, world: &mut World, value: f32, gamepad: Option<Gamepad>) {
        let gamepad = gamepad.unwrap_or(find_gamepad(world.resource::<Gamepads>()));

        let event = GamepadEvent::Button(GamepadButtonChangedEvent {
            gamepad,
            button_type: *self,
            value,
        });
        world.resource_mut::<Events<GamepadEvent>>().send(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::{AccumulatorPlugin, CentralInputStorePlugin};
    use bevy::input::gamepad::{
        GamepadConnection, GamepadConnectionEvent, GamepadEvent, GamepadInfo,
    };
    use bevy::input::InputPlugin;
    use bevy::prelude::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins((InputPlugin, AccumulatorPlugin, CentralInputStorePlugin));

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

        // The opposite of left up
        let left_down = GamepadControlDirection::LEFT_DOWN;
        assert_eq!(left_down.kind(), InputControlKind::Button);

        let left_x = GamepadControlAxis::LEFT_X;
        assert_eq!(left_x.kind(), InputControlKind::Axis);

        let left_y = GamepadControlAxis::LEFT_Y;
        assert_eq!(left_y.kind(), InputControlKind::Axis);

        let left = GamepadStick::LEFT;
        assert_eq!(left.kind(), InputControlKind::DualAxis);

        // Up; but for the other stick
        let right_up = GamepadControlDirection::RIGHT_DOWN;
        assert_eq!(right_up.kind(), InputControlKind::Button);

        let right_y = GamepadControlAxis::RIGHT_Y;
        assert_eq!(right_y.kind(), InputControlKind::Axis);

        let right = GamepadStick::RIGHT;
        assert_eq!(right.kind(), InputControlKind::DualAxis);

        // No inputs
        let mut app = test_app();
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();
        let gamepad = Gamepad::new(0);

        assert!(!left_up.pressed(inputs, gamepad));
        assert!(!left_down.pressed(inputs, gamepad));
        assert!(!right_up.pressed(inputs, gamepad));
        assert_eq!(left_x.value(inputs, gamepad), 0.0);
        assert_eq!(left_y.value(inputs, gamepad), 0.0);
        assert_eq!(right_y.value(inputs, gamepad), 0.0);
        assert_eq!(left.axis_pair(inputs, gamepad), Vec2::ZERO);
        assert_eq!(right.axis_pair(inputs, gamepad), Vec2::ZERO);

        // Left stick moves upward
        let data = Vec2::new(0.0, 1.0);
        let mut app = test_app();
        GamepadControlDirection::LEFT_UP.press_as_gamepad(app.world_mut(), Some(gamepad));
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(left_up.pressed(inputs, gamepad));
        assert!(!left_down.pressed(inputs, gamepad));
        assert!(!right_up.pressed(inputs, gamepad));
        assert_eq!(left_x.value(inputs, gamepad), 0.0);
        assert_eq!(left_y.value(inputs, gamepad), 1.0);
        assert_eq!(right_y.value(inputs, gamepad), 0.0);
        assert_eq!(left.axis_pair(inputs, gamepad), data);
        assert_eq!(right.axis_pair(inputs, gamepad), Vec2::ZERO);

        // Set Y-axis of left stick to 0.6
        let data = Vec2::new(0.0, 0.6);
        let mut app = test_app();
        GamepadControlAxis::LEFT_Y.set_value_as_gamepad(app.world_mut(), data.y, Some(gamepad));
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(left_up.pressed(inputs, gamepad));
        assert!(!left_down.pressed(inputs, gamepad));
        assert!(!right_up.pressed(inputs, gamepad));
        assert_eq!(left_x.value(inputs, gamepad), 0.0);
        assert_eq!(left_y.value(inputs, gamepad), 0.6);
        assert_eq!(right_y.value(inputs, gamepad), 0.0);
        assert_eq!(left.axis_pair(inputs, gamepad), data);
        assert_eq!(right.axis_pair(inputs, gamepad), Vec2::ZERO);

        // Set left stick to (0.6, 0.4)
        let data = Vec2::new(0.6, 0.4);
        let mut app = test_app();
        GamepadStick::LEFT.set_axis_pair_as_gamepad(app.world_mut(), data, Some(gamepad));
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(left_up.pressed(inputs, gamepad));
        assert!(!left_down.pressed(inputs, gamepad));
        assert!(!right_up.pressed(inputs, gamepad));
        assert_eq!(left_x.value(inputs, gamepad), data.x);
        assert_eq!(left_y.value(inputs, gamepad), data.y);
        assert_eq!(right_y.value(inputs, gamepad), 0.0);
        assert_eq!(left.axis_pair(inputs, gamepad), data);
        assert_eq!(right.axis_pair(inputs, gamepad), Vec2::ZERO);
    }

    #[test]
    #[ignore = "Input mocking is subtly broken: https://github.com/Leafwing-Studios/leafwing-input-manager/issues/516"]
    fn test_gamepad_buttons() {
        let up = GamepadButtonType::DPadUp;
        assert_eq!(up.kind(), InputControlKind::Button);

        let left = GamepadButtonType::DPadLeft;
        assert_eq!(left.kind(), InputControlKind::Button);

        let down = GamepadButtonType::DPadDown;
        assert_eq!(left.kind(), InputControlKind::Button);

        let right = GamepadButtonType::DPadRight;
        assert_eq!(left.kind(), InputControlKind::Button);

        // No inputs
        let mut app = test_app();
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        let gamepad = Gamepad::new(0);

        assert!(!up.pressed(inputs, gamepad));
        assert!(!left.pressed(inputs, gamepad));
        assert!(!down.pressed(inputs, gamepad));
        assert!(!right.pressed(inputs, gamepad));

        // Press DPadLeft
        let mut app = test_app();
        GamepadButtonType::DPadLeft.press(app.world_mut());
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(!up.pressed(inputs, gamepad));
        assert!(left.pressed(inputs, gamepad));
        assert!(!down.pressed(inputs, gamepad));
        assert!(!right.pressed(inputs, gamepad));
    }
}
