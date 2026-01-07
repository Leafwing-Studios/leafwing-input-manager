//! Mouse inputs

use crate as leafwing_input_manager;
use crate::axislike::{DualAxisDirection, DualAxisType};
use crate::buttonlike::ButtonValue;
use crate::clashing_inputs::BasicInputs;
use crate::input_processing::*;
use crate::user_input::{InputControlKind, UserInput};
use bevy::ecs::message::Messages;
use bevy::ecs::system::StaticSystemParam;
use bevy::ecs::system::lifetimeless::SRes;
use bevy::input::mouse::{
    AccumulatedMouseMotion, AccumulatedMouseScroll, MouseButton, MouseButtonInput, MouseMotion,
    MouseScrollUnit, MouseWheel,
};
use bevy::input::{ButtonInput, ButtonState};
use bevy::math::FloatOrd;
use bevy::prelude::{Entity, Reflect, ResMut, Vec2, World};
use leafwing_input_manager_macros::serde_typetag;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

use super::updating::{CentralInputStore, UpdatableInput};
use super::{Axislike, Buttonlike, DualAxislike};

// Built-in support for Bevy's MouseButton
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
    type SourceData = SRes<ButtonInput<MouseButton>>;

    fn compute(
        mut central_input_store: ResMut<CentralInputStore>,
        source_data: StaticSystemParam<Self::SourceData>,
    ) {
        for button in source_data.get_pressed() {
            central_input_store.update_buttonlike(*button, ButtonValue::from_pressed(true));
        }

        for button in source_data.get_just_released() {
            central_input_store.update_buttonlike(*button, ButtonValue::from_pressed(false));
        }
    }
}

#[serde_typetag]
impl Buttonlike for MouseButton {
    /// Checks if the specified button is currently pressed down.
    #[inline]
    fn get_pressed(&self, input_store: &CentralInputStore, _gamepad: Entity) -> Option<bool> {
        input_store.pressed(self)
    }

    /// Sends a fake [`MouseButtonInput`] message to the world with [`ButtonState::Pressed`].
    ///
    /// # Note
    ///
    /// The `window` field will be filled with a placeholder value.
    fn press(&self, world: &mut World) {
        let mut messages = world.resource_mut::<Messages<MouseButtonInput>>();
        messages.write(MouseButtonInput {
            button: *self,
            state: ButtonState::Pressed,
            window: Entity::PLACEHOLDER,
        });
    }

    /// Sends a fake [`MouseButtonInput`] message to the world with [`ButtonState::Released`].
    ///
    /// # Note
    ///
    /// The `window` field will be filled with a placeholder value.
    fn release(&self, world: &mut World) {
        let mut messages = world.resource_mut::<Messages<MouseButtonInput>>();
        messages.write(MouseButtonInput {
            button: *self,
            state: ButtonState::Released,
            window: Entity::PLACEHOLDER,
        });
    }

    /// If the value is greater than `0.0`, press the key; otherwise release it.
    fn set_value(&self, world: &mut World, value: f32) {
        if value > 0.0 {
            self.press(world);
        } else {
            self.release(world);
        }
    }
}

/// Provides button-like behavior for mouse movement in cardinal directions.
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy::input::InputPlugin;
/// use leafwing_input_manager::plugin::CentralInputStorePlugin;
/// use leafwing_input_manager::prelude::*;
/// use leafwing_input_manager::user_input::testing_utils::FetchUserInput;
///
/// let mut app = App::new();
/// app.add_plugins((InputPlugin, CentralInputStorePlugin));
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
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct MouseMoveDirection {
    /// The direction to monitor (up, down, left, or right).
    pub direction: DualAxisDirection,

    /// The threshold value for the direction to be considered pressed.
    /// Must be non-negative.
    pub threshold: f32,
}

impl MouseMoveDirection {
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

    /// Movement in the upward direction.
    pub const UP: Self = Self {
        direction: DualAxisDirection::Up,
        threshold: 0.0,
    };

    /// Movement in the downward direction.
    pub const DOWN: Self = Self {
        direction: DualAxisDirection::Down,
        threshold: 0.0,
    };

    /// Movement in the leftward direction.
    pub const LEFT: Self = Self {
        direction: DualAxisDirection::Left,
        threshold: 0.0,
    };

    /// Movement in the rightward direction.
    pub const RIGHT: Self = Self {
        direction: DualAxisDirection::Right,
        threshold: 0.0,
    };
}

impl UserInput for MouseMoveDirection {
    /// [`MouseMoveDirection`] acts as a virtual button.
    #[inline]
    fn kind(&self) -> InputControlKind {
        InputControlKind::Button
    }

    /// [`MouseMoveDirection`] represents a simple virtual button.
    #[inline]
    fn decompose(&self) -> BasicInputs {
        BasicInputs::Simple(Box::new((*self).threshold(0.0)))
    }
}

#[serde_typetag]
impl Buttonlike for MouseMoveDirection {
    /// Checks if there is any recent mouse movement along the specified direction.
    #[inline]
    fn get_pressed(&self, input_store: &CentralInputStore, _gamepad: Entity) -> Option<bool> {
        let mouse_movement = input_store.pair(&MouseMove::default());
        mouse_movement
            .map(|mouse_movement| self.direction.is_active(mouse_movement, self.threshold))
    }

    /// Sends a [`MouseMotion`] message with a magnitude of 1.0 in the direction defined by `self`.
    fn press(&self, world: &mut World) {
        world
            .resource_mut::<Messages<MouseMotion>>()
            .write(MouseMotion {
                delta: self.direction.full_active_value(),
            });
    }

    /// This method has no effect.
    ///
    /// As mouse movement directions are determined based on the recent change in mouse position,
    /// no action other than waiting for the next frame is necessary to release the input.
    fn release(&self, _world: &mut World) {}
}

impl Eq for MouseMoveDirection {}

impl Hash for MouseMoveDirection {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.direction.hash(state);
        FloatOrd(self.threshold).hash(state);
    }
}

/// Relative changes in position of mouse movement on a single axis (X or Y).
///
/// # Value Processing
///
/// You can customize how the values are processed using a pipeline of processors.
/// See [`WithAxisProcessingPipelineExt`] for details.
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy::input::InputPlugin;
/// use leafwing_input_manager::plugin::CentralInputStorePlugin;
/// use leafwing_input_manager::prelude::*;
/// use leafwing_input_manager::user_input::testing_utils::FetchUserInput;
///
/// let mut app = App::new();
/// app.add_plugins((InputPlugin, CentralInputStorePlugin));
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
    pub processors: Vec<AxisProcessor>,
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
            Box::new(MouseMoveDirection {
                direction: self.axis.negative(),
                threshold: 0.0,
            }),
            Box::new(MouseMoveDirection {
                direction: self.axis.positive(),
                threshold: 0.0,
            }),
        ])
    }
}

#[serde_typetag]
impl Axislike for MouseMoveAxis {
    /// Retrieves the amount of the mouse movement along the specified axis
    /// after processing by the associated processors.
    #[inline]
    fn get_value(&self, input_store: &CentralInputStore, _gamepad: Entity) -> Option<f32> {
        input_store.pair(&MouseMove::default()).map(|movement| {
            let value = self.axis.get_value(movement);

            self.processors
                .iter()
                .fold(value, |value, processor| processor.process(value))
        })
    }

    /// Sends a [`MouseMotion`] message along the appropriate axis with the specified value.
    fn set_value(&self, world: &mut World, value: f32) {
        let message = MouseMotion {
            delta: match self.axis {
                DualAxisType::X => Vec2::new(value, 0.0),
                DualAxisType::Y => Vec2::new(0.0, value),
            },
        };
        world.resource_mut::<Messages<MouseMotion>>().write(message);
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
/// # Value Processing
///
/// You can customize how the values are processed using a pipeline of processors.
/// See [`WithDualAxisProcessingPipelineExt`] for details.
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy::input::InputPlugin;
/// use leafwing_input_manager::plugin::CentralInputStorePlugin;
/// use leafwing_input_manager::prelude::*;
/// use leafwing_input_manager::user_input::testing_utils::FetchUserInput;
///
/// let mut app = App::new();
/// app.add_plugins((InputPlugin, CentralInputStorePlugin));
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
    pub processors: Vec<DualAxisProcessor>,
}

impl UpdatableInput for MouseMove {
    type SourceData = SRes<AccumulatedMouseMotion>;

    fn compute(
        mut central_input_store: ResMut<CentralInputStore>,
        source_data: StaticSystemParam<Self::SourceData>,
    ) {
        central_input_store.update_dualaxislike(Self::default(), source_data.delta);
    }
}

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

#[serde_typetag]
impl DualAxislike for MouseMove {
    /// Retrieves the mouse displacement after processing by the associated processors.
    #[inline]
    fn get_axis_pair(&self, input_store: &CentralInputStore, _gamepad: Entity) -> Option<Vec2> {
        input_store.pair(&MouseMove::default()).map(|movement| {
            self.processors
                .iter()
                .fold(movement, |value, processor| processor.process(value))
        })
    }

    /// Sends a [`MouseMotion`] message with the specified displacement.
    fn set_axis_pair(&self, world: &mut World, value: Vec2) {
        world
            .resource_mut::<Messages<MouseMotion>>()
            .write(MouseMotion { delta: value });
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
/// ```rust
/// use bevy::prelude::*;
/// use bevy::input::InputPlugin;
/// use leafwing_input_manager::plugin::CentralInputStorePlugin;
/// use leafwing_input_manager::prelude::*;
/// use leafwing_input_manager::user_input::testing_utils::FetchUserInput;
///
/// let mut app = App::new();
/// app.add_plugins((InputPlugin, CentralInputStorePlugin));
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
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct MouseScrollDirection {
    /// The direction to monitor (up, down, left, or right).
    pub direction: DualAxisDirection,

    /// The threshold value for the direction to be considered pressed.
    /// Must be non-negative.
    pub threshold: f32,
}

impl MouseScrollDirection {
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

    /// Movement in the upward direction.
    pub const UP: Self = Self {
        direction: DualAxisDirection::Up,
        threshold: 0.0,
    };

    /// Movement in the downward direction.
    pub const DOWN: Self = Self {
        direction: DualAxisDirection::Down,
        threshold: 0.0,
    };

    /// Movement in the leftward direction.
    pub const LEFT: Self = Self {
        direction: DualAxisDirection::Left,
        threshold: 0.0,
    };

    /// Movement in the rightward direction.
    pub const RIGHT: Self = Self {
        direction: DualAxisDirection::Right,
        threshold: 0.0,
    };
}

impl UserInput for MouseScrollDirection {
    /// [`MouseScrollDirection`] acts as a virtual button.
    #[inline]
    fn kind(&self) -> InputControlKind {
        InputControlKind::Button
    }

    /// [`MouseScrollDirection`] represents a simple virtual button.
    #[inline]
    fn decompose(&self) -> BasicInputs {
        BasicInputs::Simple(Box::new((*self).threshold(0.0)))
    }
}

#[serde_typetag]
impl Buttonlike for MouseScrollDirection {
    /// Checks if there is any recent mouse wheel movement along the specified direction.
    #[inline]
    fn get_pressed(&self, input_store: &CentralInputStore, _gamepad: Entity) -> Option<bool> {
        input_store
            .pair(&MouseScroll::default())
            .map(|movement| self.direction.is_active(movement, self.threshold))
    }

    /// Sends a [`MouseWheel`] message with a magnitude of 1.0 px in the direction defined by `self`.
    ///
    /// # Note
    ///
    /// The `window` field will be filled with a placeholder value.
    fn press(&self, world: &mut World) {
        let vec = self.direction.full_active_value();

        world
            .resource_mut::<Messages<MouseWheel>>()
            .write(MouseWheel {
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

impl Eq for MouseScrollDirection {}

impl Hash for MouseScrollDirection {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.direction.hash(state);
        FloatOrd(self.threshold).hash(state);
    }
}

/// Amount of mouse wheel scrolling on a single axis (X or Y).
///
/// # Value Processing
///
/// You can customize how the values are processed using a pipeline of processors.
/// See [`WithAxisProcessingPipelineExt`] for details.
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy::input::InputPlugin;
/// use bevy::input::mouse::MouseScrollUnit;
/// use leafwing_input_manager::plugin::CentralInputStorePlugin;
/// use leafwing_input_manager::prelude::*;
/// use leafwing_input_manager::user_input::testing_utils::FetchUserInput;
///
/// let mut app = App::new();
/// app.add_plugins((InputPlugin, CentralInputStorePlugin));
///
/// // Y-axis movement
/// let mut input = MouseScrollAxis::Y;
///
/// // You can configure which [`MouseScrollUnit`] this Axis uses.
/// // The default is [`MouseScrollUnit::Pixel`]
/// input.mouse_scroll_unit = MouseScrollUnit::Line;
/// assert_eq!(input.mouse_scroll_unit, MouseScrollUnit::Line);
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

    /// The [`MouseScrollUnit`] this axis uses
    pub mouse_scroll_unit: MouseScrollUnit,

    /// A processing pipeline that handles input values.
    pub processors: Vec<AxisProcessor>,
}

impl MouseScrollAxis {
    /// Horizontal scrolling of the mouse wheel. No processing is applied to raw data from the mouse.
    pub const X: Self = Self {
        axis: DualAxisType::X,
        processors: Vec::new(),
        mouse_scroll_unit: MouseScrollUnit::Pixel,
    };

    /// Vertical scrolling of the mouse wheel. No processing is applied to raw data from the mouse.
    pub const Y: Self = Self {
        axis: DualAxisType::Y,
        processors: Vec::new(),
        mouse_scroll_unit: MouseScrollUnit::Pixel,
    };
}

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
            Box::new(MouseScrollDirection {
                direction: self.axis.negative(),
                threshold: 0.0,
            }),
            Box::new(MouseScrollDirection {
                direction: self.axis.positive(),
                threshold: 0.0,
            }),
        ])
    }
}

#[serde_typetag]
impl Axislike for MouseScrollAxis {
    /// Retrieves the amount of the mouse wheel movement along the specified axis
    /// after processing by the associated processors.
    #[inline]
    fn get_value(&self, input_store: &CentralInputStore, _gamepad: Entity) -> Option<f32> {
        input_store.pair(&MouseScroll::default()).map(|movement| {
            let value = self.axis.get_value(movement);

            self.processors
                .iter()
                .fold(value, |value, processor| processor.process(value))
        })
    }

    /// Sends a [`MouseWheel`] message along the appropriate axis with the specified value in pixels.
    ///
    /// # Note
    ///
    /// The `window` field will be filled with a placeholder value.
    fn set_value(&self, world: &mut World, value: f32) {
        let message = MouseWheel {
            unit: self.mouse_scroll_unit,
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
        world.resource_mut::<Messages<MouseWheel>>().write(message);
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
/// # Value Processing
///
/// You can customize how the values are processed using a pipeline of processors.
/// See [`WithDualAxisProcessingPipelineExt`] for details.
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy::input::InputPlugin;
/// use bevy::input::mouse::MouseScrollUnit;
/// use leafwing_input_manager::plugin::CentralInputStorePlugin;
/// use leafwing_input_manager::prelude::*;
/// use leafwing_input_manager::user_input::testing_utils::FetchUserInput;
///
/// let mut app = App::new();
/// app.add_plugins((InputPlugin, CentralInputStorePlugin));
///
/// let mut input = MouseScroll::default();
///
/// // You can configure which [`MouseScrollUnit`] this Axis uses.
/// // The default is [`MouseScrollUnit::Pixel`]
/// input.mouse_scroll_unit = MouseScrollUnit::Line;
/// assert_eq!(input.mouse_scroll_unit, MouseScrollUnit::Line);
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct MouseScroll {
    /// The [`MouseScrollUnit`] this axis uses
    pub mouse_scroll_unit: MouseScrollUnit,

    /// A processing pipeline that handles input values.
    pub processors: Vec<DualAxisProcessor>,
}

impl Default for MouseScroll {
    fn default() -> Self {
        Self {
            mouse_scroll_unit: MouseScrollUnit::Pixel,
            processors: Vec::new(),
        }
    }
}

impl UpdatableInput for MouseScroll {
    type SourceData = SRes<AccumulatedMouseScroll>;

    fn compute(
        mut central_input_store: ResMut<CentralInputStore>,
        source_data: StaticSystemParam<Self::SourceData>,
    ) {
        central_input_store.update_dualaxislike(Self::default(), source_data.delta);
    }
}

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

#[serde_typetag]
impl DualAxislike for MouseScroll {
    /// Retrieves the mouse scroll movement on both axes after processing by the associated processors.
    #[inline]
    fn get_axis_pair(&self, input_store: &CentralInputStore, _gamepad: Entity) -> Option<Vec2> {
        input_store.pair(&MouseScroll::default()).map(|movement| {
            self.processors
                .iter()
                .fold(movement, |value, processor| processor.process(value))
        })
    }

    /// Sends a [`MouseWheel`] message with the specified displacement in pixels.
    ///
    /// # Note
    /// The `window` field will be filled with a placeholder value.
    fn set_axis_pair(&self, world: &mut World, value: Vec2) {
        world
            .resource_mut::<Messages<MouseWheel>>()
            .write(MouseWheel {
                unit: self.mouse_scroll_unit,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::CentralInputStorePlugin;
    use bevy::input::InputPlugin;
    use bevy::prelude::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(InputPlugin)
            .add_plugins(CentralInputStorePlugin);
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
        let gamepad = app.world_mut().spawn(()).id();
        let inputs = app.world().resource::<CentralInputStore>();

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
    fn test_user_input_mouse_button_kind() {
        let mouse_button = MouseButton::Left;
        assert_eq!(mouse_button.kind(), InputControlKind::Button);
    }

    #[test]
    fn test_mouse_button_set_value_press() {
        let mut app = test_app();
        let mouse_button = MouseButton::Left;
        mouse_button.set_value(app.world_mut(), 1.0);

        let mut mouse_motion_messages = app
            .world_mut()
            .get_resource_mut::<Messages<MouseButtonInput>>()
            .unwrap();

        assert_eq!(mouse_motion_messages.len(), 1);
        assert_eq!(
            mouse_motion_messages.drain().next().unwrap(),
            MouseButtonInput {
                button: MouseButton::Left,
                state: ButtonState::Pressed,
                window: Entity::PLACEHOLDER,
            }
        );
    }

    #[test]
    fn test_mouse_button_set_value_release() {
        let mut app = test_app();
        let mouse_button = MouseButton::Left;
        mouse_button.set_value(app.world_mut(), 0.0);

        let mut mouse_motion_messages = app
            .world_mut()
            .get_resource_mut::<Messages<MouseButtonInput>>()
            .unwrap();

        assert_eq!(mouse_motion_messages.len(), 1);
        assert_eq!(
            mouse_motion_messages.drain().next().unwrap(),
            MouseButtonInput {
                button: MouseButton::Left,
                state: ButtonState::Released,
                window: Entity::PLACEHOLDER,
            }
        );
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
        let gamepad = app.world_mut().spawn(()).id();
        let inputs = app.world().resource::<CentralInputStore>();

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
    fn test_mouse_move_direction_set_valid_threshold() {
        let mouse_move_direction = MouseMoveDirection::UP.threshold(0.5);

        assert_eq!(mouse_move_direction.threshold, 0.5);
    }

    #[test]
    #[should_panic = "assertion failed: threshold >= 0.0"]
    fn test_mouse_move_direction_set_invalid_threshold() {
        let _ = MouseMoveDirection::UP.threshold(-0.5);
    }

    #[test]
    fn test_mouse_move_direction_kind() {
        let mouse_move_direction = MouseMoveDirection::UP;

        assert_eq!(mouse_move_direction.kind(), InputControlKind::Button);
    }

    #[test]
    fn test_mouse_move_axis_processing_pipeline() {
        let mouse_move_axis = MouseMoveAxis::X.with_processor(AxisProcessor::Inverted);

        assert_eq!(mouse_move_axis.processors.len(), 1);
        assert_eq!(
            mouse_move_axis.processors.first().unwrap(),
            &AxisProcessor::Inverted
        );

        let mouse_move_axis =
            mouse_move_axis.replace_processing_pipeline(vec![AxisProcessor::Digital]);

        assert_eq!(mouse_move_axis.processors.len(), 1);
        assert_eq!(
            mouse_move_axis.processors.first().unwrap(),
            &AxisProcessor::Digital
        );

        let mouse_move_axis = mouse_move_axis.reset_processing_pipeline();

        assert_eq!(mouse_move_axis.processors.len(), 0);
    }

    #[test]
    fn test_mouse_move_processing_pipeline() {
        let mouse_move =
            MouseMove::default().with_processor(DualAxisProcessor::Inverted(DualAxisInverted::ALL));

        assert_eq!(mouse_move.processors.len(), 1);
        assert_eq!(
            mouse_move.processors.first().unwrap(),
            &DualAxisProcessor::Inverted(DualAxisInverted::ALL),
        );

        let mouse_move = mouse_move.replace_processing_pipeline(vec![DualAxisProcessor::Digital]);

        assert_eq!(mouse_move.processors.len(), 1);
        assert_eq!(
            mouse_move.processors.first().unwrap(),
            &DualAxisProcessor::Digital
        );

        let mouse_move = mouse_move.reset_processing_pipeline();

        assert_eq!(mouse_move.processors.len(), 0);
    }

    #[test]
    fn test_mouse_scroll_processing_pipeline() {
        let mouse_scroll = MouseScroll::default()
            .with_processor(DualAxisProcessor::Inverted(DualAxisInverted::ALL));

        assert_eq!(mouse_scroll.processors.len(), 1);
        assert_eq!(
            mouse_scroll.processors.first().unwrap(),
            &DualAxisProcessor::Inverted(DualAxisInverted::ALL),
        );

        let mouse_scroll =
            mouse_scroll.replace_processing_pipeline(vec![DualAxisProcessor::Digital]);

        assert_eq!(mouse_scroll.processors.len(), 1);
        assert_eq!(
            mouse_scroll.processors.first().unwrap(),
            &DualAxisProcessor::Digital
        );

        let mouse_scroll = mouse_scroll.reset_processing_pipeline();

        assert_eq!(mouse_scroll.processors.len(), 0);
    }

    #[test]
    fn test_mouse_scroll_axis_processing_pipeline() {
        let mouse_scroll_axis = MouseScrollAxis::X.with_processor(AxisProcessor::Inverted);

        assert_eq!(mouse_scroll_axis.processors.len(), 1);
        assert_eq!(
            mouse_scroll_axis.processors.first().unwrap(),
            &AxisProcessor::Inverted,
        );

        let mouse_scroll_axis =
            mouse_scroll_axis.replace_processing_pipeline(vec![AxisProcessor::Digital]);

        assert_eq!(mouse_scroll_axis.processors.len(), 1);
        assert_eq!(
            mouse_scroll_axis.processors.first().unwrap(),
            &AxisProcessor::Digital
        );

        let mouse_scroll_axis = mouse_scroll_axis.reset_processing_pipeline();

        assert_eq!(mouse_scroll_axis.processors.len(), 0);
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
        let gamepad = app.world_mut().spawn(()).id();
        let inputs = app.world().resource::<CentralInputStore>();

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

        // Set changes in scrolling `to (2.0, 3.0)
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
    fn test_mouse_scroll_direction_set_valid_threshold() {
        let mouse_scroll_direction = MouseScrollDirection::UP.threshold(0.5);

        assert_eq!(mouse_scroll_direction.threshold, 0.5);
    }

    #[test]
    #[should_panic = "assertion failed: threshold >= 0.0"]
    fn test_mouse_scroll_direction_set_invalid_threshold() {
        let _ = MouseScrollDirection::UP.threshold(-0.5);
    }

    #[test]
    fn one_frame_accumulate_mouse_movement() {
        let mut app = test_app();
        MouseMoveAxis::Y.set_value(app.world_mut(), 3.0);
        MouseMoveAxis::Y.set_value(app.world_mut(), 2.0);

        let mouse_motion_messages = app.world().get_resource::<Messages<MouseMotion>>().unwrap();
        for message in mouse_motion_messages.iter_current_update_messages() {
            dbg!("Message sent: {:?}", message);
        }

        // The haven't been processed yet
        let accumulated_mouse_movement = app.world().resource::<AccumulatedMouseMotion>();
        assert_eq!(accumulated_mouse_movement.delta, Vec2::new(0.0, 0.0));

        app.update();

        // Now the messages should be processed
        let accumulated_mouse_movement = app.world().resource::<AccumulatedMouseMotion>();
        assert_eq!(accumulated_mouse_movement.delta, Vec2::new(0.0, 5.0));

        let inputs = app.world().resource::<CentralInputStore>();
        assert_eq!(
            inputs.pair(&MouseMove::default()).unwrap(),
            Vec2::new(0.0, 5.0)
        );
    }

    #[test]
    fn multiple_frames_accumulate_mouse_movement() {
        let mut app = test_app();
        let inputs = app.world().resource::<CentralInputStore>();
        // Starts at None
        assert_eq!(
            inputs.pair(&MouseMove::default()),
            None,
            "Initial movement is not None."
        );

        // Send some data
        MouseMoveAxis::Y.set_value(app.world_mut(), 3.0);
        // Wait for the messages to be processed
        app.update();

        let inputs = app.world().resource::<CentralInputStore>();
        // Data is read
        assert_eq!(
            inputs.pair(&MouseMove::default()).unwrap(),
            Vec2::new(0.0, 3.0),
            "Movement sent was not read correctly."
        );

        // Do nothing
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();
        // Back to 0 for this frame
        assert_eq!(
            inputs.pair(&MouseMove::default()).unwrap(),
            Vec2::ZERO,
            "No movement was expected. Is the position in the message stream being cleared properly?"
        );
    }
}
