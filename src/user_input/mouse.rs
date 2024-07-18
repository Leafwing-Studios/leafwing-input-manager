//! Mouse inputs

use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::{MouseButton, Reflect, Resource, Vec2};
use leafwing_input_manager_macros::serde_typetag;
use serde::{Deserialize, Serialize};

use crate as leafwing_input_manager;
use crate::axislike::{DualAxisDirection, DualAxisType};
use crate::clashing_inputs::BasicInputs;
use crate::input_processing::*;
use crate::input_streams::InputStreams;
use crate::raw_inputs::RawInputs;
use crate::user_input::{InputControlKind, UserInput};

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

    /// Creates a [`RawInputs`] from the button directly.
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        RawInputs::from_mouse_buttons([*self])
    }
}

impl Buttonlike for MouseButton {
    /// Checks if the specified button is currently pressed down.
    #[inline]
    fn pressed(&self, input_streams: &InputStreams) -> bool {
        input_streams
            .mouse_buttons
            .is_some_and(|buttons| buttons.pressed(*self))
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
/// use leafwing_input_manager::plugin::AccumulatorPlugin;
/// use leafwing_input_manager::prelude::*;
///
/// let mut app = App::new();
/// app.add_plugins((InputPlugin, AccumulatorPlugin));
///
/// // Positive Y-axis movement
/// let input = MouseMoveDirection::UP;
///
/// // Movement in the opposite direction doesn't activate the input
/// app.send_axis_values(MouseMoveAxis::Y, [-5.0]);
/// app.update();
/// assert!(!app.pressed(input));
///
/// // Movement in the chosen direction activates the input
/// app.send_axis_values(MouseMoveAxis::Y, [5.0]);
/// app.update();
/// assert!(app.pressed(input));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct MouseMoveDirection(pub(crate) DualAxisDirection);

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

    /// Creates a [`RawInputs`] from the direction directly.
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        RawInputs::from_mouse_move_directions([*self])
    }
}

impl Buttonlike for MouseMoveDirection {
    /// Checks if there is any recent mouse movement along the specified direction.
    #[must_use]
    #[inline]
    fn pressed(&self, input_streams: &InputStreams) -> bool {
        let mouse_movement = input_streams.mouse_motion.0;
        self.0.is_active(mouse_movement)
    }
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
/// use leafwing_input_manager::plugin::AccumulatorPlugin;
/// use leafwing_input_manager::prelude::*;
///
/// let mut app = App::new();
/// app.add_plugins((InputPlugin, AccumulatorPlugin));
///
/// // Y-axis movement
/// let input = MouseMoveAxis::Y;
///
/// // Movement on the chosen axis activates the input
/// app.send_axis_values(MouseMoveAxis::Y, [1.0]);
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
    pub(crate) axis: DualAxisType,

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

    /// Creates a [`RawInputs`] from the [`DualAxisType`] used by the axis.
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        RawInputs::from_mouse_move_axes([self.axis])
    }
}

impl Axislike for MouseMoveAxis {
    /// Retrieves the amount of the mouse movement along the specified axis
    /// after processing by the associated processors.
    #[must_use]
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        let movement = input_streams.mouse_motion.0;
        let value = self.axis.get_value(movement);
        self.processors
            .iter()
            .fold(value, |value, processor| processor.process(value))
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
/// use leafwing_input_manager::plugin::AccumulatorPlugin;
/// use leafwing_input_manager::prelude::*;
///
/// let mut app = App::new();
/// app.add_plugins((InputPlugin, AccumulatorPlugin));
///
/// let input = MouseMove::default();
///
/// // Movement on either axis activates the input
/// app.send_axis_values(MouseMoveAxis::Y, [3.0]);
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
    fn processed_value(&self, input_streams: &InputStreams) -> Vec2 {
        let movement = input_streams.mouse_motion.0;
        self.processors
            .iter()
            .fold(movement, |value, processor| processor.process(value))
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

    /// Creates a [`RawInputs`] from two [`DualAxisType`]s used by the input.
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        RawInputs::from_mouse_move_axes(DualAxisType::axes())
    }
}

impl DualAxislike for MouseMove {
    /// Retrieves the mouse displacement after processing by the associated processors.
    #[must_use]
    #[inline]
    fn axis_pair(&self, input_streams: &InputStreams) -> Vec2 {
        self.processed_value(input_streams)
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
/// use leafwing_input_manager::plugin::AccumulatorPlugin;
/// use leafwing_input_manager::prelude::*;
///
/// let mut app = App::new();
/// app.add_plugins((InputPlugin, AccumulatorPlugin));
///
/// // Positive Y-axis scrolling
/// let input = MouseScrollDirection::UP;
///
/// // Scrolling in the opposite direction doesn't activate the input
/// app.send_axis_values(MouseScrollAxis::Y, [-5.0]);
/// app.update();
/// assert!(!app.pressed(input));
///
/// // Scrolling in the chosen direction activates the input
/// app.send_axis_values(MouseScrollAxis::Y, [5.0]);
/// app.update();
/// assert!(app.pressed(input));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct MouseScrollDirection(pub(crate) DualAxisDirection);

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

    /// Creates a [`RawInputs`] from the direction directly.
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        RawInputs::from_mouse_scroll_directions([*self])
    }
}

impl Buttonlike for MouseScrollDirection {
    /// Checks if there is any recent mouse wheel movement along the specified direction.
    #[must_use]
    #[inline]
    fn pressed(&self, input_streams: &InputStreams) -> bool {
        let movement = input_streams.mouse_scroll.0;
        self.0.is_active(movement)
    }
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
/// use leafwing_input_manager::plugin::AccumulatorPlugin;
/// use leafwing_input_manager::prelude::*;
///
/// let mut app = App::new();
/// app.add_plugins((InputPlugin, AccumulatorPlugin));
///
/// // Y-axis movement
/// let input = MouseScrollAxis::Y;
///
/// // Scrolling on the chosen axis activates the input
/// app.send_axis_values(MouseScrollAxis::Y, [1.0]);
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
    pub(crate) axis: DualAxisType,

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

    /// Creates a [`RawInputs`] from the [`DualAxisType`] used by the axis.
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        RawInputs::from_mouse_scroll_axes([self.axis])
    }
}

impl Axislike for MouseScrollAxis {
    /// Retrieves the amount of the mouse wheel movement along the specified axis
    /// after processing by the associated processors.
    #[must_use]
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        let movement = input_streams.mouse_scroll.0;
        let value = self.axis.get_value(movement);
        self.processors
            .iter()
            .fold(value, |value, processor| processor.process(value))
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
/// use leafwing_input_manager::plugin::AccumulatorPlugin;
/// use leafwing_input_manager::prelude::*;
///
/// let mut app = App::new();
/// app.add_plugins((InputPlugin, AccumulatorPlugin));
///
/// let input = MouseScroll::default();
///
/// // Scrolling on either axis activates the input
/// app.send_axis_values(MouseScrollAxis::Y, [3.0]);
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
    fn processed_value(&self, input_streams: &InputStreams) -> Vec2 {
        let movement = input_streams.mouse_scroll.0;
        self.processors
            .iter()
            .fold(movement, |value, processor| processor.process(value))
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

    /// Creates a [`RawInputs`] from two [`DualAxisType`] used by the input.
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        RawInputs::from_mouse_scroll_axes(DualAxisType::axes())
    }
}

impl DualAxislike for MouseScroll {
    /// Retrieves the mouse scroll movement on both axes after processing by the associated processors.
    #[must_use]
    #[inline]
    fn axis_pair(&self, input_streams: &InputStreams) -> Vec2 {
        self.processed_value(input_streams)
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
    use crate::input_mocking::MockInput;
    use crate::plugin::AccumulatorPlugin;
    use bevy::input::InputPlugin;
    use bevy::prelude::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(InputPlugin).add_plugins(AccumulatorPlugin);
        app
    }

    #[test]
    fn test_mouse_button() {
        let left = MouseButton::Left;
        assert_eq!(left.kind(), InputControlKind::Button);
        assert_eq!(left.raw_inputs(), RawInputs::from_mouse_buttons([left]));

        let middle = MouseButton::Middle;
        assert_eq!(middle.kind(), InputControlKind::Button);
        assert_eq!(middle.raw_inputs(), RawInputs::from_mouse_buttons([middle]));

        let right = MouseButton::Right;
        assert_eq!(right.kind(), InputControlKind::Button);
        assert_eq!(right.raw_inputs(), RawInputs::from_mouse_buttons([right]));

        // No inputs
        let mut app = test_app();
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);

        assert!(!left.pressed(&inputs));
        assert!(!middle.pressed(&inputs));
        assert!(!right.pressed(&inputs));

        // Press left
        let mut app = test_app();
        app.press_input(MouseButton::Left);
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);

        assert!(left.pressed(&inputs));
        assert!(!middle.pressed(&inputs));
        assert!(!right.pressed(&inputs));

        // Press middle
        let mut app = test_app();
        app.press_input(MouseButton::Middle);
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);

        assert!(!left.pressed(&inputs));
        assert!(middle.pressed(&inputs));
        assert!(!right.pressed(&inputs));

        // Press right
        let mut app = test_app();
        app.press_input(MouseButton::Right);
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);

        assert!(!left.pressed(&inputs));
        assert!(!middle.pressed(&inputs));
        assert!(right.pressed(&inputs));
    }

    #[test]
    fn test_mouse_move() {
        let mouse_move_up = MouseMoveDirection::UP;
        assert_eq!(mouse_move_up.kind(), InputControlKind::Button);
        let raw_inputs = RawInputs::from_mouse_move_directions([mouse_move_up]);
        assert_eq!(mouse_move_up.raw_inputs(), raw_inputs);

        let mouse_move_y = MouseMoveAxis::Y;
        assert_eq!(mouse_move_y.kind(), InputControlKind::Axis);
        let raw_inputs = RawInputs::from_mouse_move_axes([DualAxisType::Y]);
        assert_eq!(mouse_move_y.raw_inputs(), raw_inputs);

        let mouse_move = MouseMove::default();
        assert_eq!(mouse_move.kind(), InputControlKind::DualAxis);
        let raw_inputs = RawInputs::from_mouse_move_axes([DualAxisType::X, DualAxisType::Y]);
        assert_eq!(mouse_move.raw_inputs(), raw_inputs);

        // No inputs
        let mut app = test_app();
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);

        assert!(!mouse_move_up.pressed(&inputs));
        assert_eq!(mouse_move_y.value(&inputs), 0.0);
        assert_eq!(mouse_move.axis_pair(&inputs), Vec2::new(0.0, 0.0));

        // Move left
        let data = Vec2::new(-1.0, 0.0);
        let mut app = test_app();
        app.press_input(MouseMoveDirection::LEFT);
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);

        assert!(!mouse_move_up.pressed(&inputs));
        assert_eq!(mouse_move_y.value(&inputs), 0.0);
        assert_eq!(mouse_move.axis_pair(&inputs), data);

        // Move up
        let data = Vec2::new(0.0, 1.0);
        let mut app = test_app();
        app.press_input(MouseMoveDirection::UP);
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);

        assert!(mouse_move_up.pressed(&inputs));
        assert_eq!(mouse_move_y.value(&inputs), data.y);
        assert_eq!(mouse_move.axis_pair(&inputs), data);

        // Move down
        let data = Vec2::new(0.0, -1.0);
        let mut app = test_app();
        app.press_input(MouseMoveDirection::DOWN);
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);

        assert!(!mouse_move_up.pressed(&inputs));
        assert_eq!(mouse_move_y.value(&inputs), data.y);
        assert_eq!(mouse_move.axis_pair(&inputs), data);

        // Set changes in movement on the Y-axis to 3.0
        let data = Vec2::new(0.0, 3.0);
        let mut app = test_app();
        app.send_axis_values(MouseMoveAxis::Y, [data.y]);
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);

        assert!(mouse_move_up.pressed(&inputs));
        assert_eq!(mouse_move_y.value(&inputs), data.y);
        assert_eq!(mouse_move.axis_pair(&inputs), data);

        // Set changes in movement to (2.0, 3.0)
        let data = Vec2::new(2.0, 3.0);
        let mut app = test_app();
        app.send_axis_values(MouseMove::default(), [data.x, data.y]);
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);

        assert!(mouse_move_up.pressed(&inputs));
        assert_eq!(mouse_move_y.value(&inputs), data.y);
        assert_eq!(mouse_move.axis_pair(&inputs), data);
    }

    #[test]
    fn test_mouse_scroll() {
        let mouse_scroll_up = MouseScrollDirection::UP;
        assert_eq!(mouse_scroll_up.kind(), InputControlKind::Button);
        let raw_inputs = RawInputs::from_mouse_scroll_directions([mouse_scroll_up]);
        assert_eq!(mouse_scroll_up.raw_inputs(), raw_inputs);

        let mouse_scroll_y = MouseScrollAxis::Y;
        assert_eq!(mouse_scroll_y.kind(), InputControlKind::Axis);
        let raw_inputs = RawInputs::from_mouse_scroll_axes([DualAxisType::Y]);
        assert_eq!(mouse_scroll_y.raw_inputs(), raw_inputs);

        let mouse_scroll = MouseScroll::default();
        assert_eq!(mouse_scroll.kind(), InputControlKind::DualAxis);
        let raw_inputs = RawInputs::from_mouse_scroll_axes([DualAxisType::X, DualAxisType::Y]);
        assert_eq!(mouse_scroll.raw_inputs(), raw_inputs);

        // No inputs
        let mut app = test_app();
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);

        assert!(!mouse_scroll_up.pressed(&inputs));
        assert_eq!(mouse_scroll_y.value(&inputs), 0.0);
        assert_eq!(mouse_scroll.axis_pair(&inputs), Vec2::new(0.0, 0.0));

        // Move up
        let data = Vec2::new(0.0, 1.0);
        let mut app = test_app();
        app.press_input(MouseScrollDirection::UP);
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);

        assert!(mouse_scroll_up.pressed(&inputs));
        assert_eq!(mouse_scroll_y.value(&inputs), data.y);
        assert_eq!(mouse_scroll.axis_pair(&inputs), data);

        // Scroll down
        let data = Vec2::new(0.0, -1.0);
        let mut app = test_app();
        app.press_input(MouseScrollDirection::DOWN);
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);

        assert!(!mouse_scroll_up.pressed(&inputs));
        assert_eq!(mouse_scroll_y.value(&inputs), data.y);
        assert_eq!(mouse_scroll.axis_pair(&inputs), data);

        // Set changes in scrolling on the Y-axis to 3.0
        let data = Vec2::new(0.0, 3.0);
        let mut app = test_app();
        app.send_axis_values(MouseScrollAxis::Y, [data.y]);
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);

        assert!(mouse_scroll_up.pressed(&inputs));
        assert_eq!(mouse_scroll_y.value(&inputs), data.y);
        assert_eq!(mouse_scroll.axis_pair(&inputs), data);

        // Set changes in scrolling to (2.0, 3.0)
        let data = Vec2::new(2.0, 3.0);
        let mut app = test_app();
        app.send_axis_values(MouseScroll::default(), [data.x, data.y]);
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);

        assert!(mouse_scroll_up.pressed(&inputs));
        assert_eq!(mouse_scroll_y.value(&inputs), data.y);
        assert_eq!(mouse_scroll.axis_pair(&inputs), data);
    }

    #[test]
    fn one_frame_accumulate_mouse_movement() {
        let mut app = test_app();
        app.send_axis_values(MouseMoveAxis::Y, [3.0]);
        app.send_axis_values(MouseMoveAxis::Y, [2.0]);

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

        let inputs = InputStreams::from_world(app.world(), None);
        assert_eq!(inputs.mouse_motion.0, Vec2::new(0.0, 5.0));
    }

    #[test]
    fn multiple_frames_accumulate_mouse_movement() {
        let mut app = test_app();
        let inputs = InputStreams::from_world(app.world(), None);
        // Starts at 0
        assert_eq!(
            inputs.mouse_motion.0,
            Vec2::ZERO,
            "Initial movement is not zero."
        );

        // Send some data
        app.send_axis_values(MouseMoveAxis::Y, [3.0]);
        // Wait for the events to be processed
        app.update();

        let inputs = InputStreams::from_world(app.world(), None);
        // Data is read
        assert_eq!(
            inputs.mouse_motion.0,
            Vec2::new(0.0, 3.0),
            "Movement sent was not read correctly."
        );

        // Do nothing
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);
        // Back to 0 for this frame
        assert_eq!(
            inputs.mouse_motion.0,
            Vec2::ZERO,
            "No movement was expected. Is the position in the event stream being cleared properly?"
        );
    }
}
