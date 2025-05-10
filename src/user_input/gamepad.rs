//! Gamepad inputs

use std::hash::{Hash, Hasher};

use bevy::ecs::system::lifetimeless::{Read, SQuery};
use bevy::ecs::system::{StaticSystemParam, SystemParam, SystemState};
use bevy::input::gamepad::{
    GamepadInput, RawGamepadAxisChangedEvent, RawGamepadButtonChangedEvent, RawGamepadEvent,
};
use bevy::input::{Axis, ButtonInput};
use bevy::math::FloatOrd;
use bevy::prelude::{
    Entity, Events, Gamepad, GamepadAxis, GamepadButton, Query, Reflect, Res, ResMut, Vec2, With,
    World,
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
/// If no gamepad is connected, `Entity::PLACEHOLDER` is returned.
#[must_use]
pub fn find_gamepad(gamepads: Option<Query<Entity, With<Gamepad>>>) -> Entity {
    match gamepads {
        None => Entity::PLACEHOLDER,
        Some(gamepads) => gamepads.iter().next().unwrap_or(Entity::PLACEHOLDER),
    }
}

/// Retrieves the current value of the specified `axis`.
#[must_use]
#[inline]
fn read_axis_value(input_store: &CentralInputStore, gamepad: Entity, axis: GamepadAxis) -> f32 {
    let axis = SpecificGamepadAxis::new(gamepad, axis);
    input_store.value(&axis)
}

/// A [`GamepadAxis`] for a specific gamepad (as opposed to all gamepads).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub struct SpecificGamepadAxis {
    /// The gamepad that this axis is attached to.
    pub gamepad: Entity,
    /// The axis.
    pub axis: GamepadAxis,
}

impl SpecificGamepadAxis {
    /// Creates a new [`SpecificGamepadAxis`] with the given gamepad and axis.
    pub fn new(gamepad: Entity, axis: GamepadAxis) -> Self {
        Self { gamepad, axis }
    }
}

/// A [`GamepadButton`] for a specific gamepad (as opposed to all gamepads).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub struct SpecificGamepadButton {
    /// The gamepad that this button is attached to.
    pub gamepad: Entity,
    /// The button.
    pub button: GamepadButton,
}

impl SpecificGamepadButton {
    /// Creates a new [`SpecificGamepadButton`] with the given gamepad and
    /// button.
    pub fn new(gamepad: Entity, button: GamepadButton) -> Self {
        Self { gamepad, button }
    }
}

/// Provides button-like behavior for a specific direction on a [`GamepadAxis`].
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
    pub axis: GamepadAxis,

    /// The direction of the axis to monitor (positive or negative).
    pub direction: AxisDirection,

    /// The threshold value for the direction to be considered pressed.
    /// Must be non-negative.
    pub threshold: f32,
}

impl GamepadControlDirection {
    /// Creates a [`GamepadControlDirection`] triggered by a negative value on the specified `axis`.
    #[inline]
    pub const fn negative(axis: GamepadAxis) -> Self {
        Self {
            axis,
            direction: AxisDirection::Negative,
            threshold: 0.0,
        }
    }

    /// Creates a [`GamepadControlDirection`] triggered by a positive value on the specified `axis`.
    #[inline]
    pub const fn positive(axis: GamepadAxis) -> Self {
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
    pub const LEFT_UP: Self = Self::positive(GamepadAxis::LeftStickY);

    /// "Down" on the left analog stick (negative Y-axis movement).
    pub const LEFT_DOWN: Self = Self::negative(GamepadAxis::LeftStickY);

    /// "Left" on the left analog stick (negative X-axis movement).
    pub const LEFT_LEFT: Self = Self::negative(GamepadAxis::LeftStickX);

    /// "Right" on the left analog stick (positive X-axis movement).
    pub const LEFT_RIGHT: Self = Self::positive(GamepadAxis::LeftStickX);

    /// "Up" on the right analog stick (positive Y-axis movement).
    pub const RIGHT_UP: Self = Self::positive(GamepadAxis::RightStickY);

    /// "Down" on the right analog stick (negative Y-axis movement).
    pub const RIGHT_DOWN: Self = Self::negative(GamepadAxis::RightStickY);

    /// "Left" on the right analog stick (negative X-axis movement).
    pub const RIGHT_LEFT: Self = Self::negative(GamepadAxis::RightStickX);

    /// "Right" on the right analog stick (positive X-axis movement).
    pub const RIGHT_RIGHT: Self = Self::positive(GamepadAxis::RightStickX);
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
    #[inline]
    fn pressed(&self, input_store: &CentralInputStore, gamepad: Entity) -> bool {
        let value = read_axis_value(input_store, gamepad, self.axis);
        self.direction.is_active(value, self.threshold)
    }

    /// Sends a [`RawGamepadEvent::Axis`] event with a magnitude of 1.0 for the specified direction on the provided gamepad [`Entity`].
    fn press_as_gamepad(&self, world: &mut World, gamepad: Option<Entity>) {
        let mut query_state = SystemState::<Query<Entity, With<Gamepad>>>::new(world);
        let query = query_state.get(world);
        let gamepad = gamepad.unwrap_or(find_gamepad(Some(query)));

        let event = RawGamepadEvent::Axis(RawGamepadAxisChangedEvent {
            gamepad,
            axis: self.axis,
            value: self.direction.full_active_value(),
        });
        world.resource_mut::<Events<RawGamepadEvent>>().send(event);
    }

    /// Sends a [`RawGamepadEvent::Axis`] event with a magnitude of 0.0 for the specified direction.
    fn release_as_gamepad(&self, world: &mut World, gamepad: Option<Entity>) {
        let mut query_state = SystemState::<Query<Entity, With<Gamepad>>>::new(world);
        let query = query_state.get(world);
        let gamepad = gamepad.unwrap_or(find_gamepad(Some(query)));

        let event = RawGamepadEvent::Axis(RawGamepadAxisChangedEvent {
            gamepad,
            axis: self.axis,
            value: 0.0,
        });
        world.resource_mut::<Events<RawGamepadEvent>>().send(event);
    }

    fn set_value_as_gamepad(&self, world: &mut World, value: f32, gamepad: Option<Entity>) {
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
    type SourceData = SQuery<(Entity, Read<Gamepad>)>;

    fn compute(
        mut central_input_store: ResMut<CentralInputStore>,
        source_data: StaticSystemParam<Self::SourceData>,
    ) {
        for (gamepad_entity, gamepad) in source_data.iter() {
            for input in gamepad.get_analog_axes() {
                let GamepadInput::Axis(axis) = input else {
                    continue;
                };
                let value = gamepad.get(*axis).unwrap_or_default();
                central_input_store.update_axislike(
                    SpecificGamepadAxis {
                        gamepad: gamepad_entity,
                        axis: *axis,
                    },
                    value,
                );
                central_input_store.update_axislike(*axis, value);
            }
        }
    }
}

impl UserInput for GamepadAxis {
    fn kind(&self) -> InputControlKind {
        InputControlKind::Axis
    }

    fn decompose(&self) -> BasicInputs {
        BasicInputs::Composite(vec![
            Box::new(GamepadControlDirection::negative(*self)),
            Box::new(GamepadControlDirection::positive(*self)),
        ])
    }
}

#[serde_typetag]
impl Axislike for GamepadAxis {
    fn value(&self, input_store: &CentralInputStore, gamepad: Entity) -> f32 {
        read_axis_value(input_store, gamepad, *self)
    }
}

/// Unlike [`GamepadButton`], this struct represents a specific axis on a specific gamepad.
///
/// In the majority of cases, [`GamepadControlAxis`] or [`GamepadStick`] should be used instead.
impl UserInput for SpecificGamepadAxis {
    fn kind(&self) -> InputControlKind {
        InputControlKind::Axis
    }

    fn decompose(&self) -> BasicInputs {
        BasicInputs::Composite(vec![
            Box::new(GamepadControlDirection::negative(self.axis)),
            Box::new(GamepadControlDirection::positive(self.axis)),
        ])
    }
}

#[serde_typetag]
impl Axislike for SpecificGamepadAxis {
    fn value(&self, input_store: &CentralInputStore, gamepad: Entity) -> f32 {
        read_axis_value(input_store, gamepad, self.axis)
    }
}

/// A wrapper around a specific [`GamepadAxis`] (e.g., left stick X-axis, right stick Y-axis).
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
    pub axis: GamepadAxis,

    /// A processing pipeline that handles input values.
    pub processors: Vec<AxisProcessor>,
}

impl GamepadControlAxis {
    /// Creates a [`GamepadControlAxis`] for continuous input from the given axis.
    /// No processing is applied to raw data from the gamepad.
    #[inline]
    pub const fn new(axis: GamepadAxis) -> Self {
        Self {
            axis,
            processors: Vec::new(),
        }
    }

    /// The horizontal axis (X-axis) of the left stick.
    /// No processing is applied to raw data from the gamepad.
    pub const LEFT_X: Self = Self::new(GamepadAxis::LeftStickX);

    /// The vertical axis (Y-axis) of the left stick.
    /// No processing is applied to raw data from the gamepad.
    pub const LEFT_Y: Self = Self::new(GamepadAxis::LeftStickY);

    /// The left `Z` button. No processing is applied to raw data from the gamepad.
    pub const LEFT_Z: Self = Self::new(GamepadAxis::LeftZ);

    /// The horizontal axis (X-axis) of the right stick.
    /// No processing is applied to raw data from the gamepad.
    pub const RIGHT_X: Self = Self::new(GamepadAxis::RightStickX);

    /// The vertical axis (Y-axis) of the right stick.
    /// No processing is applied to raw data from the gamepad.
    pub const RIGHT_Y: Self = Self::new(GamepadAxis::RightStickY);

    /// The right `Z` button. No processing is applied to raw data from the gamepad.
    pub const RIGHT_Z: Self = Self::new(GamepadAxis::RightZ);
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
    #[inline]
    fn value(&self, input_store: &CentralInputStore, gamepad: Entity) -> f32 {
        let value = read_axis_value(input_store, gamepad, self.axis);
        self.processors
            .iter()
            .fold(value, |value, processor| processor.process(value))
    }

    /// Sends a [`RawGamepadEvent::Axis`] event with the specified value on the provided gamepad.
    fn set_value_as_gamepad(&self, world: &mut World, value: f32, gamepad: Option<Entity>) {
        let mut query_state = SystemState::<Query<Entity, With<Gamepad>>>::new(world);
        let query = query_state.get(world);
        let gamepad = gamepad.unwrap_or(find_gamepad(Some(query)));

        let event = RawGamepadEvent::Axis(RawGamepadAxisChangedEvent {
            gamepad,
            axis: self.axis,
            value,
        });
        world.resource_mut::<Events<RawGamepadEvent>>().send(event);
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
    pub x: GamepadAxis,

    /// Vertical movement of the stick.
    pub y: GamepadAxis,

    /// A processing pipeline that handles input values.
    pub processors: Vec<DualAxisProcessor>,
}

impl GamepadStick {
    /// The left gamepad stick. No processing is applied to raw data from the gamepad.
    pub const LEFT: Self = Self {
        x: GamepadAxis::LeftStickX,
        y: GamepadAxis::LeftStickY,
        processors: Vec::new(),
    };

    /// The right gamepad stick. No processing is applied to raw data from the gamepad.
    pub const RIGHT: Self = Self {
        x: GamepadAxis::RightStickX,
        y: GamepadAxis::RightStickY,
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
    #[inline]
    fn axis_pair(&self, input_store: &CentralInputStore, gamepad: Entity) -> Vec2 {
        let x = read_axis_value(input_store, gamepad, self.x);
        let y = read_axis_value(input_store, gamepad, self.y);
        self.processors
            .iter()
            .fold(Vec2::new(x, y), |value, processor| processor.process(value))
    }

    /// Sends a [`RawGamepadEvent::Axis`] event with the specified values on the provided gamepad [`Entity`].
    fn set_axis_pair_as_gamepad(&self, world: &mut World, value: Vec2, gamepad: Option<Entity>) {
        let mut query_state = SystemState::<Query<Entity, With<Gamepad>>>::new(world);
        let query = query_state.get(world);
        let gamepad = gamepad.unwrap_or(find_gamepad(Some(query)));

        let event = RawGamepadEvent::Axis(RawGamepadAxisChangedEvent {
            gamepad,
            axis: self.x,
            value: value.x,
        });
        world.resource_mut::<Events<RawGamepadEvent>>().send(event);

        let event = RawGamepadEvent::Axis(RawGamepadAxisChangedEvent {
            gamepad,
            axis: self.y,
            value: value.y,
        });
        world.resource_mut::<Events<RawGamepadEvent>>().send(event);
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

/// Checks if the given [`GamepadButton`] is currently pressed.
#[must_use]
#[inline]
fn button_pressed(input_store: &CentralInputStore, gamepad: Entity, button: GamepadButton) -> bool {
    let button = SpecificGamepadButton::new(gamepad, button);
    input_store.pressed(&button)
}

/// Retrieves the current value of the given [`GamepadButton`].
///
/// This will be 0.0 if the button is released, and 1.0 if it is pressed.
/// Physically triggerlike buttons will return a value between 0.0 and 1.0,
/// depending on how far the button is pressed.
#[must_use]
#[inline]
fn button_value(input_store: &CentralInputStore, gamepad: Entity, button: GamepadButton) -> f32 {
    let button = SpecificGamepadButton::new(gamepad, button);
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
    type SourceData = SQuery<(Entity, Read<Gamepad>)>;

    fn compute(
        mut central_input_store: ResMut<CentralInputStore>,
        source_data: StaticSystemParam<Self::SourceData>,
    ) {
        for (gamepad_entity, gamepad) in source_data.iter() {
            for key in gamepad.get_pressed() {
                let specific_button = SpecificGamepadButton {
                    gamepad: gamepad_entity,
                    button: *key,
                };
                let value = gamepad.get(*key).unwrap_or(1.0);
                central_input_store
                    .update_buttonlike(specific_button, ButtonValue::new(true, value));
            }

            for key in gamepad.get_just_released() {
                let specific_button = SpecificGamepadButton {
                    gamepad: gamepad_entity,
                    button: *key,
                };
                let value = gamepad.get(*key).unwrap_or(0.0);
                central_input_store
                    .update_buttonlike(specific_button, ButtonValue::new(false, value));
            }
        }
    }
}

/// Unlike [`GamepadButton`], this struct represents a specific button on a specific gamepad.
///
/// In the majority of cases, [`GamepadButton`] should be used instead.
impl UserInput for SpecificGamepadButton {
    fn kind(&self) -> InputControlKind {
        InputControlKind::Button
    }

    fn decompose(&self) -> BasicInputs {
        BasicInputs::Simple(Box::new(*self))
    }
}

#[serde_typetag]
impl Buttonlike for SpecificGamepadButton {
    /// WARNING: The supplied gamepad is ignored, as the button is already specific to a gamepad.
    fn pressed(&self, input_store: &CentralInputStore, _gamepad: Entity) -> bool {
        button_pressed(input_store, self.gamepad, self.button)
    }

    /// WARNING: The supplied gamepad is ignored, as the button is already specific to a gamepad.
    fn value(&self, input_store: &CentralInputStore, _gamepad: Entity) -> f32 {
        button_value(input_store, self.gamepad, self.button)
    }

    fn press(&self, world: &mut World) {
        self.set_value(world, 1.0);
    }

    fn release(&self, world: &mut World) {
        self.set_value(world, 0.0);
    }

    fn set_value(&self, world: &mut World, value: f32) {
        let event = RawGamepadEvent::Button(RawGamepadButtonChangedEvent {
            gamepad: self.gamepad,
            button: self.button,
            value,
        });
        world.resource_mut::<Events<RawGamepadEvent>>().send(event);
    }
}

// Built-in support for Bevy's GamepadButton.
impl UserInput for GamepadButton {
    /// [`GamepadButton`] acts as a button.
    #[inline]
    fn kind(&self) -> InputControlKind {
        InputControlKind::Button
    }

    /// Creates a [`BasicInputs`] that only contains the [`GamepadButton`] itself,
    /// as it represents a simple physical button.
    #[inline]
    fn decompose(&self) -> BasicInputs {
        BasicInputs::Simple(Box::new(*self))
    }
}

#[serde_typetag]
impl Buttonlike for GamepadButton {
    /// Checks if the specified button is currently pressed down.
    #[inline]
    fn pressed(&self, input_store: &CentralInputStore, gamepad: Entity) -> bool {
        button_pressed(input_store, gamepad, *self)
    }

    /// Retrieves the current value of the specified button.
    ///
    /// This will be 0.0 if the button is released, and 1.0 if it is pressed.
    /// Physically triggerlike buttons will return a value between 0.0 and 1.0,
    /// depending on how far the button is pressed.
    #[inline]
    fn value(&self, input_store: &CentralInputStore, gamepad: Entity) -> f32 {
        button_value(input_store, gamepad, *self)
    }

    /// Sends a [`RawGamepadEvent::Button`] event with a magnitude of 1.0 in the direction defined by `self` on the provided gamepad [`Entity`].
    fn press_as_gamepad(&self, world: &mut World, gamepad: Option<Entity>) {
        self.set_value_as_gamepad(world, 1.0, gamepad);
    }

    /// Sends a [`RawGamepadEvent::Button`] event with a magnitude of 0.0 in the direction defined by `self` on the provided gamepad [`Entity`].
    fn release_as_gamepad(&self, world: &mut World, gamepad: Option<Entity>) {
        self.set_value_as_gamepad(world, 0.0, gamepad);
    }

    /// Sends a [`RawGamepadEvent::Button`] event with the specified value in the direction defined by `self` on the provided gamepad [`Entity`].
    #[inline]
    fn set_value_as_gamepad(&self, world: &mut World, value: f32, gamepad: Option<Entity>) {
        let mut query_state = SystemState::<Query<Entity, With<Gamepad>>>::new(world);
        let query = query_state.get(world);
        let gamepad = gamepad.unwrap_or(find_gamepad(Some(query)));

        let event = RawGamepadEvent::Button(RawGamepadButtonChangedEvent {
            gamepad,
            button: *self,
            value,
        });
        world.resource_mut::<Events<RawGamepadEvent>>().send(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::CentralInputStorePlugin;
    use bevy::input::gamepad::{GamepadConnection, GamepadConnectionEvent};
    use bevy::input::InputPlugin;
    use bevy::prelude::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins((InputPlugin, CentralInputStorePlugin));

        // WARNING: you MUST register your gamepad during tests,
        // or all gamepad input mocking actions will fail
        let gamepad = app.world_mut().spawn(()).id();
        let mut gamepad_connection_events = app
            .world_mut()
            .resource_mut::<Events<GamepadConnectionEvent>>();
        gamepad_connection_events.send(GamepadConnectionEvent {
            // This MUST be consistent with any other mocked events
            gamepad,
            connection: GamepadConnection::Connected {
                name: "TestController".into(),
                vendor_id: None,
                product_id: None,
            },
        });

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
        let gamepad = app
            .world_mut()
            .query_filtered::<Entity, With<Gamepad>>()
            .iter(app.world())
            .next()
            .unwrap();
        let inputs = app.world().resource::<CentralInputStore>();

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
        let gamepad = app
            .world_mut()
            .query_filtered::<Entity, With<Gamepad>>()
            .iter(app.world())
            .next()
            .unwrap();
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
        let gamepad = app
            .world_mut()
            .query_filtered::<Entity, With<Gamepad>>()
            .iter(app.world())
            .next()
            .unwrap();
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
        let gamepad = app
            .world_mut()
            .query_filtered::<Entity, With<Gamepad>>()
            .iter(app.world())
            .next()
            .unwrap();
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
        let up = GamepadButton::DPadUp;
        assert_eq!(up.kind(), InputControlKind::Button);

        let left = GamepadButton::DPadLeft;
        assert_eq!(left.kind(), InputControlKind::Button);

        let down = GamepadButton::DPadDown;
        assert_eq!(left.kind(), InputControlKind::Button);

        let right = GamepadButton::DPadRight;
        assert_eq!(left.kind(), InputControlKind::Button);

        // No inputs
        let mut app = test_app();
        app.update();
        let gamepad = app.world_mut().spawn(()).id();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(!up.pressed(inputs, gamepad));
        assert!(!left.pressed(inputs, gamepad));
        assert!(!down.pressed(inputs, gamepad));
        assert!(!right.pressed(inputs, gamepad));

        // Press DPadLeft
        let mut app = test_app();
        GamepadButton::DPadLeft.press(app.world_mut());
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(!up.pressed(inputs, gamepad));
        assert!(left.pressed(inputs, gamepad));
        assert!(!down.pressed(inputs, gamepad));
        assert!(!right.pressed(inputs, gamepad));
    }

    #[test]
    #[ignore = "Input mocking is subtly broken: https://github.com/Leafwing-Studios/leafwing-input-manager/issues/516"]
    fn test_gamepad_button_values() {
        let up = GamepadButton::DPadUp;
        assert_eq!(up.kind(), InputControlKind::Button);

        let left = GamepadButton::DPadLeft;
        assert_eq!(left.kind(), InputControlKind::Button);

        let down = GamepadButton::DPadDown;
        assert_eq!(down.kind(), InputControlKind::Button);

        let right = GamepadButton::DPadRight;
        assert_eq!(right.kind(), InputControlKind::Button);

        // No inputs
        let mut app = test_app();
        app.update();
        let gamepad = app.world_mut().spawn(()).id();
        let inputs = app.world().resource::<CentralInputStore>();

        assert_eq!(up.value(inputs, gamepad), 0.0);
        assert_eq!(left.value(inputs, gamepad), 0.0);
        assert_eq!(down.value(inputs, gamepad), 0.0);
        assert_eq!(right.value(inputs, gamepad), 0.0);

        // Press DPadLeft
        let mut app = test_app();
        GamepadButton::DPadLeft.press(app.world_mut());
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert_eq!(up.value(inputs, gamepad), 0.0);
        assert_eq!(left.value(inputs, gamepad), 1.0);
        assert_eq!(down.value(inputs, gamepad), 0.0);
        assert_eq!(right.value(inputs, gamepad), 0.0);
    }
}
