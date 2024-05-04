//! Mouse inputs

use bevy::prelude::{MouseButton, Reflect, Vec2};
use leafwing_input_manager_macros::serde_typetag;
use serde::{Deserialize, Serialize};

use crate as leafwing_input_manager;
use crate::axislike::{AxisInputMode, DualAxisData, DualAxisDirection, DualAxisType};
use crate::clashing_inputs::BasicInputs;
use crate::input_processing::*;
use crate::input_streams::InputStreams;
use crate::raw_inputs::RawInputs;
use crate::user_input::{InputKind, UserInput};

// Built-in support for Bevy's MouseButton
#[serde_typetag]
impl UserInput for MouseButton {
    /// [`MouseButton`] always acts as a button.
    #[inline]
    fn kind(&self) -> InputKind {
        InputKind::Button
    }

    /// Checks if the specified button is currently pressed down.
    #[inline]
    fn pressed(&self, input_streams: &InputStreams) -> bool {
        input_streams
            .mouse_buttons
            .is_some_and(|buttons| buttons.pressed(*self))
    }

    /// Retrieves the strength of the button press for the specified button.
    ///
    /// # Returns
    ///
    /// - `1.0` if the button is currently pressed down, indicating an active input.
    /// - `0.0` if the button is not pressed, signifying no input.
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        f32::from(self.pressed(input_streams))
    }

    /// Always returns [`None`] as [`MouseButton`] doesn't represent dual-axis input.
    #[must_use]
    #[inline]
    fn axis_pair(&self, _input_streams: &InputStreams) -> Option<DualAxisData> {
        None
    }

    /// Returns a [`BasicInputs`] that only contains the [`MouseButton`] itself,
    /// as it represents a simple physical button.
    #[inline]
    fn basic_inputs(&self) -> BasicInputs {
        BasicInputs::Single(Box::new(*self))
    }

    /// Creates a [`RawInputs`] from the button directly.
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        RawInputs::from_mouse_buttons([*self])
    }
}

/// Retrieves the total mouse displacement.
#[must_use]
#[inline]
fn accumulate_mouse_movement(input_streams: &InputStreams) -> Vec2 {
    // PERF: this summing is computed for every individual input
    // This should probably be computed once, and then cached / read
    // Fix upstream!
    input_streams
        .mouse_motion
        .iter()
        .map(|event| event.delta)
        .sum()
}

/// Captures ongoing mouse movement on a specific direction, treated as a button press.
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
    /// [`MouseMoveDirection`] always acts as a virtual button.
    #[inline]
    fn kind(&self) -> InputKind {
        InputKind::Button
    }

    /// Checks if there is any recent mouse movement along the specified direction.
    #[must_use]
    #[inline]
    fn pressed(&self, input_streams: &InputStreams) -> bool {
        let movement = accumulate_mouse_movement(input_streams);
        self.0.is_active(movement)
    }

    /// Retrieves the amount of the mouse movement along the specified direction,
    /// returning `0.0` for no movement and `1.0` for full movement.
    #[must_use]
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        f32::from(self.pressed(input_streams))
    }

    /// Always returns [`None`] as [`MouseMoveDirection`] doesn't represent dual-axis input.
    #[must_use]
    #[inline]
    fn axis_pair(&self, _input_streams: &InputStreams) -> Option<DualAxisData> {
        None
    }

    /// Returns a [`BasicInputs`] that only contains the [`MouseMoveDirection`] itself,
    /// as it represents a simple virtual button.
    #[inline]
    fn basic_inputs(&self) -> BasicInputs {
        BasicInputs::Single(Box::new(*self))
    }

    /// Creates a [`RawInputs`] from the direction directly.
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        RawInputs::from_mouse_move_directions([*self])
    }
}

/// Captures ongoing mouse movement on a specific axis.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct MouseMoveAxis {
    /// The axis that this input tracks.
    pub(crate) axis: DualAxisType,

    /// The method to interpret values on the axis,
    /// either [`AxisInputMode::Analog`] or [`AxisInputMode::Digital`].
    pub(crate) input_mode: AxisInputMode,

    /// The [`AxisProcessor`] used to handle input values.
    pub(crate) processor: AxisProcessor,
}

impl MouseMoveAxis {
    /// The horizontal [`MouseMoveAxis`] for continuous input.
    /// No processing is applied to raw data from the mouse.
    pub const X: Self = Self {
        axis: DualAxisType::X,
        input_mode: AxisInputMode::Analog,
        processor: AxisProcessor::None,
    };

    /// The vertical [`MouseMoveAxis`] for continuous input.
    /// No processing is applied to raw data from the mouse.
    pub const Y: Self = Self {
        axis: DualAxisType::Y,
        input_mode: AxisInputMode::Analog,
        processor: AxisProcessor::None,
    };

    /// The horizontal [`MouseMoveAxis`] for discrete input.
    /// No processing is applied to raw data from the mouse.
    pub const X_DIGITAL: Self = Self {
        axis: DualAxisType::X,
        input_mode: AxisInputMode::Digital,
        processor: AxisProcessor::None,
    };

    /// The vertical [`MouseMoveAxis`] for discrete input.
    /// No processing is applied to raw data from the mouse.
    pub const Y_DIGITAL: Self = Self {
        axis: DualAxisType::Y,
        input_mode: AxisInputMode::Digital,
        processor: AxisProcessor::None,
    };
}

#[serde_typetag]
impl UserInput for MouseMoveAxis {
    /// [`MouseMoveAxis`] always acts as an axis input.
    #[inline]
    fn kind(&self) -> InputKind {
        InputKind::Axis
    }

    /// Checks if there is any recent mouse movement along the specified axis.
    #[must_use]
    #[inline]
    fn pressed(&self, input_streams: &InputStreams) -> bool {
        self.value(input_streams) != 0.0
    }

    /// Retrieves the amount of the mouse movement along the specified axis
    /// after processing by the associated processor.
    #[must_use]
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        let movement = accumulate_mouse_movement(input_streams);
        let value = self.axis.get_value(movement);
        let value = self.input_mode.axis_value(value);
        self.processor.process(value)
    }

    /// Always returns [`None`] as [`MouseMoveAxis`] doesn't represent dual-axis input.
    #[must_use]
    #[inline]
    fn axis_pair(&self, _input_streams: &InputStreams) -> Option<DualAxisData> {
        None
    }

    /// Returns both negative and positive [`MouseMoveDirection`] to represent the movement.
    #[inline]
    fn basic_inputs(&self) -> BasicInputs {
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

impl WithAxisProcessingPipelineExt for MouseMoveAxis {
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

/// Captures ongoing mouse movement on both X and Y axes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct MouseMove {
    /// The method to interpret values on both axes,
    /// either [`AxisInputMode::Analog`] or [`AxisInputMode::Digital`].
    pub(crate) input_mode: AxisInputMode,

    /// The [`DualAxisProcessor`] used to handle input values.
    pub(crate) processor: DualAxisProcessor,
}

impl MouseMove {
    /// Continuous [`MouseMove`] for input on both X and Y axes.
    /// No processing is applied to raw data from the mouse.
    pub const RAW: Self = Self {
        input_mode: AxisInputMode::Analog,
        processor: DualAxisProcessor::None,
    };

    /// Discrete [`MouseMove`] for input on both X and Y axes.
    /// No processing is applied to raw data from the mouse.
    pub const DIGITAL: Self = Self {
        input_mode: AxisInputMode::Digital,
        processor: DualAxisProcessor::None,
    };
}

#[serde_typetag]
impl UserInput for MouseMove {
    /// [`MouseMove`] always acts as a dual-axis input.
    #[inline]
    fn kind(&self) -> InputKind {
        InputKind::DualAxis
    }

    /// Checks if there is any recent mouse movement.
    #[must_use]
    #[inline]
    fn pressed(&self, input_streams: &InputStreams) -> bool {
        let movement = accumulate_mouse_movement(input_streams);
        let value = self.input_mode.dual_axis_value(movement);
        let value = self.processor.process(value);
        value != Vec2::ZERO
    }

    /// Retrieves the amount of the mouse movement after processing by the associated processor.
    #[must_use]
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        let movement = accumulate_mouse_movement(input_streams);
        let value = self.input_mode.dual_axis_value(movement);
        let value = self.processor.process(value);
        self.input_mode.dual_axis_magnitude(value)
    }

    /// Retrieves the mouse displacement after processing by the associated processor.
    #[must_use]
    #[inline]
    fn axis_pair(&self, input_streams: &InputStreams) -> Option<DualAxisData> {
        let movement = accumulate_mouse_movement(input_streams);
        let value = self.input_mode.dual_axis_value(movement);
        let value = self.processor.process(value);
        Some(DualAxisData::from_xy(value))
    }

    /// Returns four [`MouseMoveDirection`]s to represent the movement.
    #[inline]
    fn basic_inputs(&self) -> BasicInputs {
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

impl WithDualAxisProcessingPipelineExt for MouseMove {
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

/// Accumulates the mouse wheel movement.
#[must_use]
#[inline]
fn accumulate_wheel_movement(input_streams: &InputStreams) -> Vec2 {
    let Some(wheel) = &input_streams.mouse_wheel else {
        return Vec2::ZERO;
    };

    // PERF: this summing is computed for every individual input
    // This should probably be computed once, and then cached / read
    // Fix upstream!
    wheel.iter().map(|event| Vec2::new(event.x, event.y)).sum()
}

/// Captures ongoing mouse wheel movement on a specific direction, treated as a button press.
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
    /// [`MouseScrollDirection`] always acts as a virtual button.
    #[inline]
    fn kind(&self) -> InputKind {
        InputKind::Button
    }

    /// Checks if there is any recent mouse wheel movement along the specified direction.
    #[must_use]
    #[inline]
    fn pressed(&self, input_streams: &InputStreams) -> bool {
        let movement = accumulate_wheel_movement(input_streams);
        self.0.is_active(movement)
    }

    /// Retrieves the magnitude of the mouse wheel movement along the specified direction,
    /// returning `0.0` for no movement and `1.0` for full movement.
    #[must_use]
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        f32::from(self.pressed(input_streams))
    }

    /// Always returns [`None`] as [`MouseScrollDirection`] doesn't represent dual-axis input.
    #[must_use]
    #[inline]
    fn axis_pair(&self, _input_streams: &InputStreams) -> Option<DualAxisData> {
        None
    }

    /// Returns a [`BasicInputs`] that only contains the [`MouseScrollDirection`] itself,
    /// as it represents a simple virtual button.
    #[inline]
    fn basic_inputs(&self) -> BasicInputs {
        BasicInputs::Single(Box::new(*self))
    }

    /// Creates a [`RawInputs`] from the direction directly.
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        RawInputs::from_mouse_scroll_directions([*self])
    }
}

/// Captures ongoing mouse wheel movement on a specific axis.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct MouseScrollAxis {
    /// The axis that this input tracks.
    pub(crate) axis: DualAxisType,

    /// The method to interpret values on the axis,
    /// either [`AxisInputMode::Analog`] or [`AxisInputMode::Digital`].
    pub(crate) input_mode: AxisInputMode,

    /// The [`AxisProcessor`] used to handle input values.
    pub(crate) processor: AxisProcessor,
}

impl MouseScrollAxis {
    /// The horizontal [`MouseScrollAxis`] for continuous input.
    /// No processing is applied to raw data from the mouse.
    pub const X: Self = Self {
        axis: DualAxisType::X,
        input_mode: AxisInputMode::Analog,
        processor: AxisProcessor::None,
    };

    /// The vertical [`MouseScrollAxis`] for continuous input.
    /// No processing is applied to raw data from the mouse.
    pub const Y: Self = Self {
        axis: DualAxisType::Y,
        input_mode: AxisInputMode::Analog,
        processor: AxisProcessor::None,
    };

    /// The horizontal [`MouseScrollAxis`] for discrete input.
    /// No processing is applied to raw data from the mouse.
    pub const X_DIGITAL: Self = Self {
        axis: DualAxisType::X,
        input_mode: AxisInputMode::Digital,
        processor: AxisProcessor::None,
    };

    /// The vertical [`MouseScrollAxis`] for discrete input.
    /// No processing is applied to raw data from the mouse.
    pub const Y_DIGITAL: Self = Self {
        axis: DualAxisType::Y,
        input_mode: AxisInputMode::Digital,
        processor: AxisProcessor::None,
    };
}

#[serde_typetag]
impl UserInput for MouseScrollAxis {
    /// [`MouseScrollAxis`] always acts as an axis input.
    #[inline]
    fn kind(&self) -> InputKind {
        InputKind::Axis
    }

    /// Checks if there is any recent mouse wheel movement along the specified axis.
    #[must_use]
    #[inline]
    fn pressed(&self, input_streams: &InputStreams) -> bool {
        self.value(input_streams) != 0.0
    }

    /// Retrieves the amount of the mouse wheel movement along the specified axis
    /// after processing by the associated processor.
    #[must_use]
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        let movement = accumulate_wheel_movement(input_streams);
        let value = self.axis.get_value(movement);
        let value = self.input_mode.axis_value(value);
        self.processor.process(value)
    }

    /// Always returns [`None`] as [`MouseScrollAxis`] doesn't represent dual-axis input.
    #[must_use]
    #[inline]
    fn axis_pair(&self, _input_streams: &InputStreams) -> Option<DualAxisData> {
        None
    }

    /// Returns both positive and negative [`MouseScrollDirection`]s to represent the movement.
    #[inline]
    fn basic_inputs(&self) -> BasicInputs {
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

impl WithAxisProcessingPipelineExt for MouseScrollAxis {
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

/// Captures ongoing mouse wheel movement on both X and Y axes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct MouseScroll {
    /// The method to interpret values on both axes,
    /// either [`AxisInputMode::Analog`] or [`AxisInputMode::Digital`].
    pub(crate) input_mode: AxisInputMode,

    /// The [`DualAxisProcessor`] used to handle input values.
    pub(crate) processor: DualAxisProcessor,
}

impl MouseScroll {
    /// Continuous [`MouseScroll`] for input on both X and Y axes.
    /// No processing is applied to raw data from the mouse.
    pub const RAW: Self = Self {
        input_mode: AxisInputMode::Analog,
        processor: DualAxisProcessor::None,
    };

    /// Discrete [`MouseScroll`] for input on both X and Y axes.
    /// No processing is applied to raw data from the mouse.
    pub const DIGITAL: Self = Self {
        input_mode: AxisInputMode::Digital,
        processor: DualAxisProcessor::None,
    };
}

#[serde_typetag]
impl UserInput for MouseScroll {
    /// [`MouseScroll`] always acts as an axis input.
    #[inline]
    fn kind(&self) -> InputKind {
        InputKind::DualAxis
    }

    /// Checks if there is any recent mouse wheel movement.
    #[must_use]
    #[inline]
    fn pressed(&self, input_streams: &InputStreams) -> bool {
        let movement = accumulate_wheel_movement(input_streams);
        let value = self.input_mode.dual_axis_value(movement);
        let value = self.processor.process(value);
        value != Vec2::ZERO
    }

    /// Retrieves the amount of the mouse wheel movement on both axes after processing by the associated processor.
    #[must_use]
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        let movement = accumulate_wheel_movement(input_streams);
        let value = self.input_mode.dual_axis_value(movement);
        let value = self.processor.process(value);
        self.input_mode.dual_axis_magnitude(value)
    }

    /// Retrieves the mouse scroll movement on both axes after processing by the associated processor.
    #[must_use]
    #[inline]
    fn axis_pair(&self, input_streams: &InputStreams) -> Option<DualAxisData> {
        let movement = accumulate_wheel_movement(input_streams);
        let value = self.input_mode.dual_axis_value(movement);
        let value = self.processor.process(value);
        Some(DualAxisData::from_xy(value))
    }

    /// Returns four [`MouseScrollDirection`]s to represent the movement.
    #[inline]
    fn basic_inputs(&self) -> BasicInputs {
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

impl WithDualAxisProcessingPipelineExt for MouseScroll {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input_mocking::MockInput;
    use bevy::input::InputPlugin;
    use bevy::prelude::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(InputPlugin);
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
    fn test_mouse_button() {
        let left = MouseButton::Left;
        assert_eq!(left.kind(), InputKind::Button);
        assert_eq!(left.raw_inputs(), RawInputs::from_mouse_buttons([left]));

        let middle = MouseButton::Middle;
        assert_eq!(middle.kind(), InputKind::Button);
        assert_eq!(middle.raw_inputs(), RawInputs::from_mouse_buttons([middle]));

        let right = MouseButton::Right;
        assert_eq!(right.kind(), InputKind::Button);
        assert_eq!(right.raw_inputs(), RawInputs::from_mouse_buttons([right]));

        // No inputs
        let mut app = test_app();
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        released(&left, &inputs);
        released(&middle, &inputs);
        released(&right, &inputs);

        // Press left
        let mut app = test_app();
        app.press_input(MouseButton::Left);
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        pressed(&left, &inputs);
        released(&middle, &inputs);
        released(&right, &inputs);

        // Press middle
        let mut app = test_app();
        app.press_input(MouseButton::Middle);
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        released(&left, &inputs);
        pressed(&middle, &inputs);
        released(&right, &inputs);

        // Press right
        let mut app = test_app();
        app.press_input(MouseButton::Right);
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        released(&left, &inputs);
        released(&middle, &inputs);
        pressed(&right, &inputs);
    }

    #[test]
    fn test_mouse_move() {
        let mouse_move_up = MouseMoveDirection::UP;
        assert_eq!(mouse_move_up.kind(), InputKind::Button);
        let raw_inputs = RawInputs::from_mouse_move_directions([mouse_move_up]);
        assert_eq!(mouse_move_up.raw_inputs(), raw_inputs);

        let mouse_move_y = MouseMoveAxis::Y;
        assert_eq!(mouse_move_y.kind(), InputKind::Axis);
        let raw_inputs = RawInputs::from_mouse_move_axes([DualAxisType::Y]);
        assert_eq!(mouse_move_y.raw_inputs(), raw_inputs);

        let mouse_move = MouseMove::RAW;
        assert_eq!(mouse_move.kind(), InputKind::DualAxis);
        let raw_inputs = RawInputs::from_mouse_move_axes([DualAxisType::X, DualAxisType::Y]);
        assert_eq!(mouse_move.raw_inputs(), raw_inputs);

        // No inputs
        let zeros = Some(DualAxisData::ZERO);
        let mut app = test_app();
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        released(&mouse_move_up, &inputs);
        released(&mouse_move_y, &inputs);
        check(&mouse_move, &inputs, false, 0.0, zeros);

        // Move left
        let data = DualAxisData::new(-1.0, 0.0);
        let mut app = test_app();
        app.press_input(MouseMoveDirection::LEFT);
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        released(&mouse_move_up, &inputs);
        released(&mouse_move_y, &inputs);
        check(&mouse_move, &inputs, true, data.length(), Some(data));

        // Move up
        let data = DualAxisData::new(0.0, 1.0);
        let mut app = test_app();
        app.press_input(MouseMoveDirection::UP);
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        pressed(&mouse_move_up, &inputs);
        check(&mouse_move_y, &inputs, true, data.y(), None);
        check(&mouse_move, &inputs, true, data.length(), Some(data));

        // Move down
        let data = DualAxisData::new(0.0, -1.0);
        let mut app = test_app();
        app.press_input(MouseMoveDirection::DOWN);
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        released(&mouse_move_up, &inputs);
        check(&mouse_move_y, &inputs, true, data.y(), None);
        check(&mouse_move, &inputs, true, data.length(), Some(data));

        // Set changes in movement on the Y-axis to 3.0
        let data = DualAxisData::new(0.0, 3.0);
        let mut app = test_app();
        app.send_axis_values(MouseMoveAxis::Y, [data.y()]);
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        pressed(&mouse_move_up, &inputs);
        check(&mouse_move_y, &inputs, true, data.y(), None);
        check(&mouse_move, &inputs, true, data.length(), Some(data));

        // Set changes in movement to (2.0, 3.0)
        let data = DualAxisData::new(2.0, 3.0);
        let mut app = test_app();
        app.send_axis_values(MouseMove::RAW, [data.x(), data.y()]);
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        pressed(&mouse_move_up, &inputs);
        check(&mouse_move_y, &inputs, true, data.y(), None);
        check(&mouse_move, &inputs, true, data.length(), Some(data));
    }

    #[test]
    fn test_mouse_scroll() {
        let mouse_scroll_up = MouseScrollDirection::UP;
        assert_eq!(mouse_scroll_up.kind(), InputKind::Button);
        let raw_inputs = RawInputs::from_mouse_scroll_directions([mouse_scroll_up]);
        assert_eq!(mouse_scroll_up.raw_inputs(), raw_inputs);

        let mouse_scroll_y = MouseScrollAxis::Y;
        assert_eq!(mouse_scroll_y.kind(), InputKind::Axis);
        let raw_inputs = RawInputs::from_mouse_scroll_axes([DualAxisType::Y]);
        assert_eq!(mouse_scroll_y.raw_inputs(), raw_inputs);

        let mouse_scroll = MouseScroll::RAW;
        assert_eq!(mouse_scroll.kind(), InputKind::DualAxis);
        let raw_inputs = RawInputs::from_mouse_scroll_axes([DualAxisType::X, DualAxisType::Y]);
        assert_eq!(mouse_scroll.raw_inputs(), raw_inputs);

        // No inputs
        let zeros = Some(DualAxisData::ZERO);
        let mut app = test_app();
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        released(&mouse_scroll_up, &inputs);
        released(&mouse_scroll_y, &inputs);
        check(&mouse_scroll, &inputs, false, 0.0, zeros);

        // Move up
        let data = DualAxisData::new(0.0, 1.0);
        let mut app = test_app();
        app.press_input(MouseScrollDirection::UP);
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        pressed(&mouse_scroll_up, &inputs);
        check(&mouse_scroll_y, &inputs, true, data.y(), None);
        check(&mouse_scroll, &inputs, true, data.length(), Some(data));

        // Scroll down
        let data = DualAxisData::new(0.0, -1.0);
        let mut app = test_app();
        app.press_input(MouseScrollDirection::DOWN);
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        released(&mouse_scroll_up, &inputs);
        check(&mouse_scroll_y, &inputs, true, data.y(), None);
        check(&mouse_scroll, &inputs, true, data.length(), Some(data));

        // Set changes in scrolling on the Y-axis to 3.0
        let data = DualAxisData::new(0.0, 3.0);
        let mut app = test_app();
        app.send_axis_values(MouseScrollAxis::Y, [data.y()]);
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        pressed(&mouse_scroll_up, &inputs);
        check(&mouse_scroll_y, &inputs, true, data.y(), None);
        check(&mouse_scroll, &inputs, true, data.length(), Some(data));

        // Set changes in scrolling to (2.0, 3.0)
        let data = DualAxisData::new(2.0, 3.0);
        let mut app = test_app();
        app.send_axis_values(MouseScroll::RAW, [data.x(), data.y()]);
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        pressed(&mouse_scroll_up, &inputs);
        check(&mouse_scroll_y, &inputs, true, data.y(), None);
        check(&mouse_scroll, &inputs, true, data.length(), Some(data));
    }
}
