//! Mouse inputs

use bevy::input::mouse::{MouseButtonInput, MouseMotion, MouseWheel};
use bevy::input::{ButtonInput, ButtonState};
use bevy::prelude::{
    Entity, Events, Gamepad, MouseButton, Reflect, Res, ResMut, Resource, Vec2, World,
};
use leafwing_input_manager_macros::serde_typetag;
use serde::{Deserialize, Serialize};

use crate as leafwing_input_manager;
use crate::axislike::{DualAxisDirection, DualAxisType};
use crate::clashing_inputs::BasicInputs;
use crate::input_processing::*;
use crate::user_input::{InputControlKind, UserInput};

use super::updating::{CentralInputStore, UpdatableInput};
use super::{Axislike, Buttonlike, DualAxislike};

// Built-in support for Bevy's MouseButton
#[serde_typetag]
impl UserInput for MouseButton {
    /// [`MouseButton`] acts as a button.
    #[inline]
    fn kind(&self) -> InputControlKind {
        InputControlKind::Button
    }

    /// Returns a [`BasicInputs`] that only contains the [`MouseButton`] itself,
    /// as it represents a simple physical button.
    #[inline]
    fn decompose(&self) -> BasicInputs {
        BasicInputs::Simple(Box::new(*self))
    }
}

impl UpdatableInput for MouseButton {
    type SourceData = ButtonInput<MouseButton>;

    fn compute(
        mut central_input_store: ResMut<CentralInputStore>,
        source_data: Res<Self::SourceData>,
    ) {
        for key in source_data.get_pressed() {
            central_input_store.update_buttonlike(*key, true);
        }

        for key in source_data.get_just_released() {
            central_input_store.update_buttonlike(*key, false);
        }
    }
}

impl Buttonlike for MouseButton {
    /// Checks if the specified button is currently pressed down.
    #[inline]
    fn pressed(&self, input_store: &CentralInputStore, _gamepad: Gamepad) -> bool {
        input_store.pressed(self)
    }

    /// Sends a fake [`MouseButtonInput`] event to the world with [`ButtonState::Pressed`].
    ///
    /// # Note
    ///
    /// The `window` field will be filled with a placeholder value.
    fn press(&self, world: &mut World) {
        let mut events = world.resource_mut::<Events<MouseButtonInput>>();
        events.send(MouseButtonInput {
            button: *self,
            state: ButtonState::Pressed,
            window: Entity::PLACEHOLDER,
        });
    }

    /// Sends a fake [`MouseButtonInput`] event to the world with [`ButtonState::Released`].
    ///
    /// # Note
    ///
    /// The `window` field will be filled with a placeholder value.
    fn release(&self, world: &mut World) {
        let mut events = world.resource_mut::<Events<MouseButtonInput>>();
        events.send(MouseButtonInput {
            button: *self,
            state: ButtonState::Released,
            window: Entity::PLACEHOLDER,
        });
    }
}

/// Provides button-like behavior for mouse movement in cardinal directions.
///
/// # Behaviors
///
/// - Activation: Only if the mouse moves in the chosen direction.
/// - Single-Axis Value:
///   - `1.0`: The input is currently active.
///   - `0.0`: The input is inactive.
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy::input::InputPlugin;
/// use leafwing_input_manager::plugin::{AccumulatorPlugin, CentralInputStorePlugin};
/// use leafwing_input_manager::prelude::*;
/// use leafwing_input_manager::user_input::testing_utils::FetchUserInput;
///
/// let mut app = App::new();
/// app.add_plugins((InputPlugin, AccumulatorPlugin, CentralInputStorePlugin));
///
/// // Positive Y-axis movement
/// let input = MouseMoveDirection::UP;
///
/// // Movement in the opposite direction doesn't activate the input
/// MouseMoveAxis::Y.set_value(app.world_mut(), -5.0);
/// app.update();
/// assert!(!app.read_pressed(input));
///
/// // Movement in the chosen direction activates the input
/// MouseMoveAxis::Y.set_value(app.world_mut(), 5.0);
/// app.update();
/// assert!(app.read_pressed(input));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct MouseMoveDirection(pub DualAxisDirection);

impl MouseMoveDirection {
    /// Movement in the upward direction.
    pub const UP: Self = Self(DualAxisDirection::Up);

    /// Movement in the downward direction.
    pub const DOWN: Self = Self(DualAxisDirection::Down);

    /// Movement in the leftward direction.
    pub const LEFT: Self = Self(DualAxisDirection::Left);

    /// Movement in the rightward direction.
    pub const RIGHT: Self = Self(DualAxisDirection::Right);
}

#[serde_typetag]
impl UserInput for MouseMoveDirection {
    /// [`MouseMoveDirection`] acts as a virtual button.
    #[inline]
    fn kind(&self) -> InputControlKind {
        InputControlKind::Button
    }

    /// [`MouseMoveDirection`] represents a simple virtual button.
    #[inline]
    fn decompose(&self) -> BasicInputs {
        BasicInputs::Simple(Box::new(*self))
    }
}

impl Buttonlike for MouseMoveDirection {
    /// Checks if there is any recent mouse movement along the specified direction.
    #[must_use]
    #[inline]
    fn pressed(&self, input_store: &CentralInputStore, _gamepad: Gamepad) -> bool {
        let mouse_movement = input_store.pair(&MouseMove::default());
        self.0.is_active(mouse_movement)
    }

    /// Sends a [`MouseMotion`] event with a magnitude of 1.0 in the direction defined by `self`.
    fn press(&self, world: &mut World) {
        world
            .resource_mut::<Events<MouseMotion>>()
            .send(MouseMotion {
                delta: self.0.full_active_value(),
            });
    }

    /// This method has no effect.
    ///
    /// As mouse movement directions are determined based on the recent change in mouse position,
    /// no action other than waiting for the next frame is necessary to release the input.
    fn release(&self, _world: &mut World) {}
}

/// Relative changes in position of mouse movement on a single axis (X or Y).
///
/// # Behaviors
///
/// - Raw Value: Captures the amount of movement on the chosen axis (X or Y).
/// - Value Processing: Configure a pipeline to modify the raw value before use,
///     see [`WithAxisProcessingPipelineExt`] for details.
/// - Activation: Only if its value is non-zero.
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy::input::InputPlugin;
/// use leafwing_input_manager::plugin::{AccumulatorPlugin, CentralInputStorePlugin};
/// use leafwing_input_manager::prelude::*;
/// use leafwing_input_manager::user_input::testing_utils::FetchUserInput;
///
/// let mut app = App::new();
/// app.add_plugins((InputPlugin, AccumulatorPlugin, CentralInputStorePlugin));
///
/// // Y-axis movement
/// let input = MouseMoveAxis::Y;
///
/// // Movement on the chosen axis activates the input
/// MouseMoveAxis::Y.set_value(app.world_mut(), 1.0);
/// app.update();
/// assert_eq!(app.read_axis_value(input), 1.0);
///
/// // You can configure a processing pipeline (e.g., doubling the value)
/// let doubled = MouseMoveAxis::Y.sensitivity(2.0);
/// assert_eq!(app.read_axis_value(doubled), 2.0);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct MouseMoveAxis {
    /// The specified axis that this input tracks.
    pub axis: DualAxisType,

    /// A processing pipeline that handles input values.
    pub(crate) processors: Vec<AxisProcessor>,
}

impl MouseMoveAxis {
    /// Movement on the X-axis. No processing is applied to raw data from the mouse.
    pub const X: Self = Self {
        axis: DualAxisType::X,
        processors: Vec::new(),
    };

    /// Movement on the Y-axis. No processing is applied to raw data from the mouse.
    pub const Y: Self = Self {
        axis: DualAxisType::Y,
        processors: Vec::new(),
    };
}

#[serde_typetag]
impl UserInput for MouseMoveAxis {
    /// [`MouseMoveAxis`] acts as an axis input.
    #[inline]
    fn kind(&self) -> InputControlKind {
        InputControlKind::Axis
    }

    /// [`MouseMoveAxis`] represents a composition of two [`MouseMoveDirection`]s.
    #[inline]
    fn decompose(&self) -> BasicInputs {
        BasicInputs::Composite(vec![
            Box::new(MouseMoveDirection(self.axis.negative())),
            Box::new(MouseMoveDirection(self.axis.positive())),
        ])
    }
}

impl Axislike for MouseMoveAxis {
    /// Retrieves the amount of the mouse movement along the specified axis
    /// after processing by the associated processors.
    #[must_use]
    #[inline]
    fn value(&self, input_store: &CentralInputStore, _gamepad: Gamepad) -> f32 {
        let movement = input_store.pair(&MouseMove::default());
        let value = self.axis.get_value(movement);
        self.processors
            .iter()
            .fold(value, |value, processor| processor.process(value))
    }

    /// Sends a [`MouseMotion`] event along the appropriate axis with the specified value.
    fn set_value(&self, world: &mut World, value: f32) {
        let event = MouseMotion {
            delta: match self.axis {
                DualAxisType::X => Vec2::new(value, 0.0),
                DualAxisType::Y => Vec2::new(0.0, value),
            },
        };
        world.resource_mut::<Events<MouseMotion>>().send(event);
    }
}

impl WithAxisProcessingPipelineExt for MouseMoveAxis {
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

/// Relative changes in position of mouse movement on both axes.
///
/// # Behaviors
///
/// - Raw Value: Captures the amount of movement on both axes.
/// - Value Processing: Configure a pipeline to modify the raw value before use,
///     see [`WithDualAxisProcessingPipelineExt`] for details.
/// - Activation: Only if its processed value is non-zero on either axis.
/// - Single-Axis Value: Reports the magnitude of the processed value.
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy::input::InputPlugin;
/// use leafwing_input_manager::plugin::{AccumulatorPlugin, CentralInputStorePlugin};
/// use leafwing_input_manager::prelude::*;
/// use leafwing_input_manager::user_input::testing_utils::FetchUserInput;
///
/// let mut app = App::new();
/// app.add_plugins((InputPlugin, AccumulatorPlugin, CentralInputStorePlugin));
///
/// let input = MouseMove::default();
///
/// // Movement on either axis activates the input
/// MouseMoveAxis::Y.set_value(app.world_mut(), 3.0);
/// app.update();
/// assert_eq!(app.read_dual_axis_values(input), Vec2::new(0.0, 3.0));
///
/// // You can configure a processing pipeline (e.g., doubling the Y value)
/// let doubled = MouseMove::default().sensitivity_y(2.0);
/// assert_eq!(app.read_dual_axis_values(doubled), Vec2::new(0.0, 6.0));
/// ```
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct MouseMove {
    /// A processing pipeline that handles input values.
    pub(crate) processors: Vec<DualAxisProcessor>,
}

impl MouseMove {
    /// Retrieves the current X and Y values of the movement after processing by the associated processors.
    #[must_use]
    #[inline]
    fn processed_value(&self, input_store: &CentralInputStore) -> Vec2 {
        let movement = input_store.pair(&MouseMove::default());
        self.processors
            .iter()
            .fold(movement, |value, processor| processor.process(value))
    }
}

impl UpdatableInput for MouseMove {
    type SourceData = AccumulatedMouseMovement;

    fn compute(
        mut central_input_store: ResMut<CentralInputStore>,
        source_data: Res<Self::SourceData>,
    ) {
        central_input_store.update_dualaxislike(Self::default(), source_data.0);
    }
}

#[serde_typetag]
impl UserInput for MouseMove {
    /// [`MouseMove`] acts as a dual-axis input.
    #[inline]
    fn kind(&self) -> InputControlKind {
        InputControlKind::DualAxis
    }

    /// [`MouseMove`] represents a composition of four [`MouseMoveDirection`]s.
    #[inline]
    fn decompose(&self) -> BasicInputs {
        BasicInputs::Composite(vec![
            Box::new(MouseMoveDirection::UP),
            Box::new(MouseMoveDirection::DOWN),
            Box::new(MouseMoveDirection::LEFT),
            Box::new(MouseMoveDirection::RIGHT),
        ])
    }
}

impl DualAxislike for MouseMove {
    /// Retrieves the mouse displacement after processing by the associated processors.
    #[must_use]
    #[inline]
    fn axis_pair(&self, input_store: &CentralInputStore, _gamepad: Gamepad) -> Vec2 {
        self.processed_value(input_store)
    }

    /// Sends a [`MouseMotion`] event with the specified displacement.
    fn set_axis_pair(&self, world: &mut World, value: Vec2) {
        world
            .resource_mut::<Events<MouseMotion>>()
            .send(MouseMotion { delta: value });
    }
}

impl WithDualAxisProcessingPipelineExt for MouseMove {
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

/// Provides button-like behavior for mouse wheel scrolling in cardinal directions.
///
/// # Behaviors
///
/// - Activation: Only if the mouse wheel is scrolling in the chosen direction.
/// - Single-Axis Value:
///   - `1.0`: The input is currently active.
///   - `0.0`: The input is inactive.
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy::input::InputPlugin;
/// use leafwing_input_manager::plugin::{AccumulatorPlugin, CentralInputStorePlugin};
/// use leafwing_input_manager::prelude::*;
/// use leafwing_input_manager::user_input::testing_utils::FetchUserInput;
///
/// let mut app = App::new();
/// app.add_plugins((InputPlugin, AccumulatorPlugin, CentralInputStorePlugin));
///
/// // Positive Y-axis scrolling
/// let input = MouseScrollDirection::UP;
///
/// // Scrolling in the opposite direction doesn't activate the input
/// MouseScrollAxis::Y.set_value(app.world_mut(), -5.0);
/// app.update();
/// assert!(!app.read_pressed(input));
///
/// // Scrolling in the chosen direction activates the input
/// MouseScrollAxis::Y.set_value(app.world_mut(), 5.0);
/// app.update();
/// assert!(app.read_pressed(input));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct MouseScrollDirection(pub DualAxisDirection);

impl MouseScrollDirection {
    /// Movement in the upward direction.
    pub const UP: Self = Self(DualAxisDirection::Up);

    /// Movement in the downward direction.
    pub const DOWN: Self = Self(DualAxisDirection::Down);

    /// Movement in the leftward direction.
    pub const LEFT: Self = Self(DualAxisDirection::Left);

    /// Movement in the rightward direction.
    pub const RIGHT: Self = Self(DualAxisDirection::Right);
}

#[serde_typetag]
impl UserInput for MouseScrollDirection {
    /// [`MouseScrollDirection`] acts as a virtual button.
    #[inline]
    fn kind(&self) -> InputControlKind {
        InputControlKind::Button
    }

    /// [`MouseScrollDirection`] represents a simple virtual button.
    #[inline]
    fn decompose(&self) -> BasicInputs {
        BasicInputs::Simple(Box::new(*self))
    }
}

impl Buttonlike for MouseScrollDirection {
    /// Checks if there is any recent mouse wheel movement along the specified direction.
    #[must_use]
    #[inline]
    fn pressed(&self, input_store: &CentralInputStore, _gamepad: Gamepad) -> bool {
        let movement = input_store.pair(&MouseScroll::default());
        self.0.is_active(movement)
    }

    /// Sends a [`MouseWheel`] event with a magnitude of 1.0 px in the direction defined by `self`.
    ///
    /// # Note
    ///
    /// The `window` field will be filled with a placeholder value.
    fn press(&self, world: &mut World) {
        let vec = self.0.full_active_value();

        world.resource_mut::<Events<MouseWheel>>().send(MouseWheel {
            unit: bevy::input::mouse::MouseScrollUnit::Pixel,
            x: vec.x,
            y: vec.y,
            window: Entity::PLACEHOLDER,
        });
    }

    /// This method has no effect.
    ///
    /// As mouse scroll directions are determined based on the recent change in mouse scrolling,
    /// no action other than waiting for the next frame is necessary to release the input.
    fn release(&self, _world: &mut World) {}
}

/// Amount of mouse wheel scrolling on a single axis (X or Y).
///
/// # Behaviors
///
/// - Raw Value: Captures the amount of scrolling on the chosen axis (X or Y).
/// - Value Processing: [`WithAxisProcessingPipelineExt`] offers methods
///     for managing a processing pipeline that can be applied to the raw value before use.
/// - Activation: Only if its value is non-zero.
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy::input::InputPlugin;
/// use leafwing_input_manager::plugin::{AccumulatorPlugin, CentralInputStorePlugin};
/// use leafwing_input_manager::prelude::*;
/// use leafwing_input_manager::user_input::testing_utils::FetchUserInput;
///
/// let mut app = App::new();
/// app.add_plugins((InputPlugin, AccumulatorPlugin, CentralInputStorePlugin));
///
/// // Y-axis movement
/// let input = MouseScrollAxis::Y;
///
/// // Scrolling on the chosen axis activates the input
/// MouseScrollAxis::Y.set_value(app.world_mut(), 1.0);
/// app.update();
/// assert_eq!(app.read_axis_value(input), 1.0);
///
/// // You can configure a processing pipeline (e.g., doubling the value)
/// let doubled = MouseScrollAxis::Y.sensitivity(2.0);
/// assert_eq!(app.read_axis_value(doubled), 2.0);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct MouseScrollAxis {
    /// The axis that this input tracks.
    pub axis: DualAxisType,

    /// A processing pipeline that handles input values.
    pub(crate) processors: Vec<AxisProcessor>,
}

impl MouseScrollAxis {
    /// Horizontal scrolling of the mouse wheel. No processing is applied to raw data from the mouse.
    pub const X: Self = Self {
        axis: DualAxisType::X,
        processors: Vec::new(),
    };

    /// Vertical scrolling of the mouse wheel. No processing is applied to raw data from the mouse.
    pub const Y: Self = Self {
        axis: DualAxisType::Y,
        processors: Vec::new(),
    };
}

#[serde_typetag]
impl UserInput for MouseScrollAxis {
    /// [`MouseScrollAxis`] acts as an axis input.
    #[inline]
    fn kind(&self) -> InputControlKind {
        InputControlKind::Axis
    }

    /// [`MouseScrollAxis`] represents a composition of two [`MouseScrollDirection`]s.
    #[inline]
    fn decompose(&self) -> BasicInputs {
        BasicInputs::Composite(vec![
            Box::new(MouseScrollDirection(self.axis.negative())),
            Box::new(MouseScrollDirection(self.axis.positive())),
        ])
    }
}

impl Axislike for MouseScrollAxis {
    /// Retrieves the amount of the mouse wheel movement along the specified axis
    /// after processing by the associated processors.
    #[must_use]
    #[inline]
    fn value(&self, input_store: &CentralInputStore, _gamepad: Gamepad) -> f32 {
        let movement = input_store.pair(&MouseScroll::default());
        let value = self.axis.get_value(movement);
        self.processors
            .iter()
            .fold(value, |value, processor| processor.process(value))
    }

    /// Sends a [`MouseWheel`] event along the appropriate axis with the specified value in pixels.
    ///
    /// # Note
    /// The `window` field will be filled with a placeholder value.
    fn set_value(&self, world: &mut World, value: f32) {
        let event = MouseWheel {
            unit: bevy::input::mouse::MouseScrollUnit::Pixel,
            x: if self.axis == DualAxisType::X {
                value
            } else {
                0.0
            },
            y: if self.axis == DualAxisType::Y {
                value
            } else {
                0.0
            },
            window: Entity::PLACEHOLDER,
        };
        world.resource_mut::<Events<MouseWheel>>().send(event);
    }
}

impl WithAxisProcessingPipelineExt for MouseScrollAxis {
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

/// Amount of mouse wheel scrolling on both axes.
///
/// # Behaviors
///
/// - Raw Value: Captures the amount of scrolling on the chosen axis (X or Y).
/// - Value Processing: [`WithAxisProcessingPipelineExt`] offers methods
///     for managing a processing pipeline that can be applied to the raw value before use.
/// - Activation: Only if its value is non-zero.
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy::input::InputPlugin;
/// use leafwing_input_manager::plugin::{AccumulatorPlugin, CentralInputStorePlugin};
/// use leafwing_input_manager::prelude::*;
/// use leafwing_input_manager::user_input::testing_utils::FetchUserInput;
///
/// let mut app = App::new();
/// app.add_plugins((InputPlugin, AccumulatorPlugin, CentralInputStorePlugin));
///
/// let input = MouseScroll::default();
///
/// // Scrolling on either axis activates the input
/// MouseScrollAxis::Y.set_value(app.world_mut(), 3.0);
/// app.update();
/// assert_eq!(app.read_dual_axis_values(input), Vec2::new(0.0, 3.0));
///
/// // You can configure a processing pipeline (e.g., doubling the Y value)
/// let doubled = MouseScroll::default().sensitivity_y(2.0);
/// assert_eq!(app.read_dual_axis_values(doubled), Vec2::new(0.0, 6.0));
/// ```
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct MouseScroll {
    /// A processing pipeline that handles input values.
    pub(crate) processors: Vec<DualAxisProcessor>,
}

impl MouseScroll {
    /// Retrieves the current X and Y values of the movement after processing by the associated processors.
    #[must_use]
    #[inline]
    fn processed_value(&self, input_store: &CentralInputStore) -> Vec2 {
        let movement = input_store.pair(&MouseScroll::default());
        self.processors
            .iter()
            .fold(movement, |value, processor| processor.process(value))
    }
}

impl UpdatableInput for MouseScroll {
    type SourceData = AccumulatedMouseScroll;

    fn compute(
        mut central_input_store: ResMut<CentralInputStore>,
        source_data: Res<Self::SourceData>,
    ) {
        central_input_store.update_dualaxislike(Self::default(), source_data.0);
    }
}

#[serde_typetag]
impl UserInput for MouseScroll {
    /// [`MouseScroll`] acts as an axis input.
    #[inline]
    fn kind(&self) -> InputControlKind {
        InputControlKind::DualAxis
    }

    /// [`MouseScroll`] represents a composition of four [`MouseScrollDirection`]s.
    #[inline]
    fn decompose(&self) -> BasicInputs {
        BasicInputs::Composite(vec![
            Box::new(MouseScrollDirection::UP),
            Box::new(MouseScrollDirection::DOWN),
            Box::new(MouseScrollDirection::LEFT),
            Box::new(MouseScrollDirection::RIGHT),
        ])
    }
}

impl DualAxislike for MouseScroll {
    /// Retrieves the mouse scroll movement on both axes after processing by the associated processors.
    #[must_use]
    #[inline]
    fn axis_pair(&self, input_store: &CentralInputStore, _gamepad: Gamepad) -> Vec2 {
        self.processed_value(input_store)
    }

    /// Sends a [`MouseWheel`] event with the specified displacement in pixels.
    ///
    /// # Note
    /// The `window` field will be filled with a placeholder value.
    fn set_axis_pair(&self, world: &mut World, value: Vec2) {
        world.resource_mut::<Events<MouseWheel>>().send(MouseWheel {
            unit: bevy::input::mouse::MouseScrollUnit::Pixel,
            x: value.x,
            y: value.y,
            window: Entity::PLACEHOLDER,
        });
    }
}

impl WithDualAxisProcessingPipelineExt for MouseScroll {
    #[inline]
    fn reset_processing_pipeline(mut self) -> Self {
        self.processors.clear();
        self
    }

    #[inline]
    fn replace_processing_pipeline(
        mut self,
        processors: impl IntoIterator<Item = DualAxisProcessor>,
    ) -> Self {
        self.processors = processors.into_iter().collect();
        self
    }

    #[inline]
    fn with_processor(mut self, processor: impl Into<DualAxisProcessor>) -> Self {
        self.processors.push(processor.into());
        self
    }
}

/// A resource that records the accumulated mouse movement for the frame.
///
/// These values are computed by summing the [`MouseMotion`] events.
///
/// This resource is automatically added by [`InputManagerPlugin`](crate::plugin::InputManagerPlugin).
/// Its value is updated during [`InputManagerSystem::Update`](crate::plugin::InputManagerSystem::Update).
#[derive(Debug, Default, Resource, Reflect, Serialize, Deserialize, Clone, PartialEq)]
pub struct AccumulatedMouseMovement(pub Vec2);

impl AccumulatedMouseMovement {
    /// Resets the accumulated mouse movement to zero.
    #[inline]
    pub fn reset(&mut self) {
        self.0 = Vec2::ZERO;
    }

    /// Accumulates the specified mouse movement.
    #[inline]
    pub fn accumulate(&mut self, event: &MouseMotion) {
        self.0 += event.delta;
    }
}

/// A resource that records the accumulated mouse wheel (scrolling) movement for the frame.
///
/// These values are computed by summing the [`MouseWheel`] events.
///
/// This resource is automatically added by [`InputManagerPlugin`](crate::plugin::InputManagerPlugin).
/// Its value is updated during [`InputManagerSystem::Update`](crate::plugin::InputManagerSystem::Update).
#[derive(Debug, Default, Resource, Reflect, Serialize, Deserialize, Clone, PartialEq)]
pub struct AccumulatedMouseScroll(pub Vec2);

impl AccumulatedMouseScroll {
    /// Resets the accumulated mouse scroll to zero.
    #[inline]
    pub fn reset(&mut self) {
        self.0 = Vec2::ZERO;
    }

    /// Accumulates the specified mouse wheel movement.
    ///
    /// # Warning
    ///
    /// This ignores the mouse scroll unit: all values are treated as equal.
    /// All scrolling, no matter what window it is on, is added to the same total.
    #[inline]
    pub fn accumulate(&mut self, event: &MouseWheel) {
        self.0.x += event.x;
        self.0.y += event.y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::{AccumulatorPlugin, CentralInputStorePlugin};
    use bevy::input::InputPlugin;
    use bevy::prelude::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(InputPlugin)
            .add_plugins((AccumulatorPlugin, CentralInputStorePlugin));
        app
    }

    #[test]
    fn test_mouse_button() {
        let left = MouseButton::Left;
        assert_eq!(left.kind(), InputControlKind::Button);

        let middle = MouseButton::Middle;
        assert_eq!(middle.kind(), InputControlKind::Button);

        let right = MouseButton::Right;
        assert_eq!(right.kind(), InputControlKind::Button);

        // No inputs
        let mut app = test_app();
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        let gamepad = Gamepad::new(0);

        assert!(!left.pressed(inputs, gamepad));
        assert!(!middle.pressed(inputs, gamepad));
        assert!(!right.pressed(inputs, gamepad));

        // Press left
        let mut app = test_app();
        MouseButton::Left.press(app.world_mut());
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(left.pressed(inputs, gamepad));
        assert!(!middle.pressed(inputs, gamepad));
        assert!(!right.pressed(inputs, gamepad));

        // Press middle
        let mut app = test_app();
        MouseButton::Middle.press(app.world_mut());
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(!left.pressed(inputs, gamepad));
        assert!(middle.pressed(inputs, gamepad));
        assert!(!right.pressed(inputs, gamepad));

        // Press right
        let mut app = test_app();
        MouseButton::Right.press(app.world_mut());
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(!left.pressed(inputs, gamepad));
        assert!(!middle.pressed(inputs, gamepad));
        assert!(right.pressed(inputs, gamepad));
    }

    #[test]
    fn test_mouse_move() {
        let mouse_move_up = MouseMoveDirection::UP;
        assert_eq!(mouse_move_up.kind(), InputControlKind::Button);

        let mouse_move_y = MouseMoveAxis::Y;
        assert_eq!(mouse_move_y.kind(), InputControlKind::Axis);

        let mouse_move = MouseMove::default();
        assert_eq!(mouse_move.kind(), InputControlKind::DualAxis);

        // No inputs
        let mut app = test_app();
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        let gamepad = Gamepad::new(0);

        assert!(!mouse_move_up.pressed(inputs, gamepad));
        assert_eq!(mouse_move_y.value(inputs, gamepad), 0.0);
        assert_eq!(mouse_move.axis_pair(inputs, gamepad), Vec2::new(0.0, 0.0));

        // Move left
        let data = Vec2::new(-1.0, 0.0);
        let mut app = test_app();
        MouseMoveDirection::LEFT.press(app.world_mut());
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(!mouse_move_up.pressed(inputs, gamepad));
        assert_eq!(mouse_move_y.value(inputs, gamepad), 0.0);
        assert_eq!(mouse_move.axis_pair(inputs, gamepad), data);

        // Move up
        let data = Vec2::new(0.0, 1.0);
        let mut app = test_app();
        MouseMoveDirection::UP.press(app.world_mut());
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(mouse_move_up.pressed(inputs, gamepad));
        assert_eq!(mouse_move_y.value(inputs, gamepad), data.y);
        assert_eq!(mouse_move.axis_pair(inputs, gamepad), data);

        // Move down
        let data = Vec2::new(0.0, -1.0);
        let mut app = test_app();
        MouseMoveDirection::DOWN.press(app.world_mut());
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(!mouse_move_up.pressed(inputs, gamepad));
        assert_eq!(mouse_move_y.value(inputs, gamepad), data.y);
        assert_eq!(mouse_move.axis_pair(inputs, gamepad), data);

        // Set changes in movement on the Y-axis to 3.0
        let data = Vec2::new(0.0, 3.0);
        let mut app = test_app();
        MouseMoveAxis::Y.set_value(app.world_mut(), data.y);
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(mouse_move_up.pressed(inputs, gamepad));
        assert_eq!(mouse_move_y.value(inputs, gamepad), data.y);
        assert_eq!(mouse_move.axis_pair(inputs, gamepad), data);

        // Set changes in movement to (2.0, 3.0)
        let data = Vec2::new(2.0, 3.0);
        let mut app = test_app();
        MouseMove::default().set_axis_pair(app.world_mut(), data);
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(mouse_move_up.pressed(inputs, gamepad));
        assert_eq!(mouse_move_y.value(inputs, gamepad), data.y);
        assert_eq!(mouse_move.axis_pair(inputs, gamepad), data);
    }

    #[test]
    fn test_mouse_scroll() {
        let mouse_scroll_up = MouseScrollDirection::UP;
        assert_eq!(mouse_scroll_up.kind(), InputControlKind::Button);

        let mouse_scroll_y = MouseScrollAxis::Y;
        assert_eq!(mouse_scroll_y.kind(), InputControlKind::Axis);
        let mouse_scroll = MouseScroll::default();
        assert_eq!(mouse_scroll.kind(), InputControlKind::DualAxis);

        // No inputs
        let mut app = test_app();
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        let gamepad = Gamepad::new(0);

        assert!(!mouse_scroll_up.pressed(inputs, gamepad));
        assert_eq!(mouse_scroll_y.value(inputs, gamepad), 0.0);
        assert_eq!(mouse_scroll.axis_pair(inputs, gamepad), Vec2::new(0.0, 0.0));

        // Move up
        let data = Vec2::new(0.0, 1.0);
        let mut app = test_app();
        MouseScrollDirection::UP.press(app.world_mut());
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(mouse_scroll_up.pressed(inputs, gamepad));
        assert_eq!(mouse_scroll_y.value(inputs, gamepad), data.y);
        assert_eq!(mouse_scroll.axis_pair(inputs, gamepad), data);

        // Scroll down
        let data = Vec2::new(0.0, -1.0);
        let mut app = test_app();
        MouseScrollDirection::DOWN.press(app.world_mut());
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(!mouse_scroll_up.pressed(inputs, gamepad));
        assert_eq!(mouse_scroll_y.value(inputs, gamepad), data.y);
        assert_eq!(mouse_scroll.axis_pair(inputs, gamepad), data);

        // Set changes in scrolling on the Y-axis to 3.0
        let data = Vec2::new(0.0, 3.0);
        let mut app = test_app();
        MouseScrollAxis::Y.set_value(app.world_mut(), data.y);
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(mouse_scroll_up.pressed(inputs, gamepad));
        assert_eq!(mouse_scroll_y.value(inputs, gamepad), data.y);
        assert_eq!(mouse_scroll.axis_pair(inputs, gamepad), data);

        // Set changes in scrolling to (2.0, 3.0)
        let data = Vec2::new(2.0, 3.0);
        let mut app = test_app();
        MouseScroll::default().set_axis_pair(app.world_mut(), data);
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(mouse_scroll_up.pressed(inputs, gamepad));
        assert_eq!(mouse_scroll_y.value(inputs, gamepad), data.y);
        assert_eq!(mouse_scroll.axis_pair(inputs, gamepad), data);
    }

    #[test]
    fn one_frame_accumulate_mouse_movement() {
        let mut app = test_app();
        MouseMoveAxis::Y.set_value(app.world_mut(), 3.0);
        MouseMoveAxis::Y.set_value(app.world_mut(), 2.0);

        let mouse_motion_events = app.world().get_resource::<Events<MouseMotion>>().unwrap();
        for event in mouse_motion_events.iter_current_update_events() {
            dbg!("Event sent: {:?}", event);
        }

        // The haven't been processed yet
        let accumulated_mouse_movement = app.world().resource::<AccumulatedMouseMovement>();
        assert_eq!(accumulated_mouse_movement.0, Vec2::new(0.0, 0.0));

        app.update();

        // Now the events should be processed
        let accumulated_mouse_movement = app.world().resource::<AccumulatedMouseMovement>();
        assert_eq!(accumulated_mouse_movement.0, Vec2::new(0.0, 5.0));

        let inputs = app.world().resource::<CentralInputStore>();
        assert_eq!(inputs.pair(&MouseMove::default()), Vec2::new(0.0, 5.0));
    }

    #[test]
    fn multiple_frames_accumulate_mouse_movement() {
        let mut app = test_app();
        let inputs = app.world().resource::<CentralInputStore>();
        // Starts at 0
        assert_eq!(
            inputs.pair(&MouseMove::default()),
            Vec2::ZERO,
            "Initial movement is not zero."
        );

        // Send some data
        MouseMoveAxis::Y.set_value(app.world_mut(), 3.0);
        // Wait for the events to be processed
        app.update();

        let inputs = app.world().resource::<CentralInputStore>();
        // Data is read
        assert_eq!(
            inputs.pair(&MouseMove::default()),
            Vec2::new(0.0, 3.0),
            "Movement sent was not read correctly."
        );

        // Do nothing
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();
        // Back to 0 for this frame
        assert_eq!(
            inputs.pair(&MouseMove::default()),
            Vec2::ZERO,
            "No movement was expected. Is the position in the event stream being cleared properly?"
        );
    }
}
