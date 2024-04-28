//! Mouse inputs

use bevy::prelude::{MouseButton, Reflect, Vec2};
use leafwing_input_manager_macros::serde_typetag;
use serde::{Deserialize, Serialize};

use crate as leafwing_input_manager;
use crate::input_processing::*;
use crate::input_streams::InputStreams;
use crate::user_inputs::axislike::{AxisInputMode, DualAxis, DualAxisData, DualAxisDirection};
use crate::user_inputs::UserInput;

// Built-in support for Bevy's MouseButton
impl UserInput for MouseButton {
    /// Checks if the specified [`MouseButton`] is currently pressed down.
    fn is_active(&self, input_streams: &InputStreams) -> bool {
        input_streams
            .mouse_buttons
            .is_some_and(|buttons| buttons.pressed(*self))
    }

    /// Retrieves the strength of the button press for the specified [`MouseButton`].
    ///
    /// # Returns
    ///
    /// - `1.0` if the button is currently pressed down, indicating an active input.
    /// - `0.0` if the button is not pressed, signifying no input.
    fn value(&self, input_streams: &InputStreams) -> f32 {
        f32::from(self.is_active(input_streams))
    }

    /// Always returns [`None`] as [`MouseButton`]s don't represent dual-axis input.
    fn axis_pair(&self, _input_streams: &InputStreams) -> Option<DualAxisData> {
        None
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

/// Input associated with mouse movement on both X and Y axes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct MouseMove {
    /// The [`AxisInputMode`] for both axes.
    pub(crate) input_mode: AxisInputMode,

    /// The [`DualAxisProcessor`] used to handle input values.
    pub(crate) processor: DualAxisProcessor,
}

impl MouseMove {
    /// Creates a [`MouseMove`] for continuous movement on X and Y axes without any processing applied.
    pub const fn analog() -> Self {
        Self {
            input_mode: AxisInputMode::Analog,
            processor: DualAxisProcessor::None,
        }
    }

    /// Creates a [`MouseMove`] for discrete movement on X and Y axes without any processing applied.
    pub const fn digital() -> Self {
        Self {
            input_mode: AxisInputMode::Digital,
            processor: DualAxisProcessor::None,
        }
    }

    /// Creates a [`MouseMove`] for continuous movement on X and Y axes with the specified [`DualAxisProcessor`].
    pub fn analog_using(processor: impl Into<DualAxisProcessor>) -> Self {
        Self {
            input_mode: AxisInputMode::Analog,
            processor: processor.into(),
        }
    }

    /// Creates a [`MouseMove`] for discrete movement on X and Y axes with the specified [`DualAxisProcessor`].
    pub fn digital_using(processor: impl Into<DualAxisProcessor>) -> Self {
        Self {
            input_mode: AxisInputMode::Analog,
            processor: processor.into(),
        }
    }

    /// Appends the given [`DualAxisProcessor`] as the next processing step.
    #[inline]
    pub fn with_processor(mut self, processor: impl Into<DualAxisProcessor>) -> Self {
        self.processor = self.processor.with_processor(processor);
        self
    }

    /// Replaces the current [`DualAxisProcessor`] with the specified `processor`.
    #[inline]
    pub fn replace_processor(mut self, processor: impl Into<DualAxisProcessor>) -> Self {
        self.processor = processor.into();
        self
    }

    /// Removes the current used [`DualAxisProcessor`].
    #[inline]
    pub fn no_processor(mut self) -> Self {
        self.processor = DualAxisProcessor::None;
        self
    }
}

#[serde_typetag]
impl UserInput for MouseMove {
    /// Checks if there is any recent mouse movement.
    #[must_use]
    #[inline]
    fn is_active(&self, input_streams: &InputStreams) -> bool {
        let movement = accumulate_mouse_movement(input_streams);
        self.processor.process(movement) != Vec2::ZERO
    }

    /// Retrieves the amount of the mouse movement
    /// after processing by the associated [`DualAxisProcessor`].
    #[must_use]
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        let movement = accumulate_mouse_movement(input_streams);
        let value = self.processor.process(movement);
        self.input_mode.dual_axis_magnitude(value)
    }

    /// Retrieves the mouse displacement
    /// after processing by the associated [`DualAxisProcessor`].
    #[must_use]
    #[inline]
    fn axis_pair(&self, input_streams: &InputStreams) -> Option<DualAxisData> {
        let movement = accumulate_mouse_movement(input_streams);
        let value = self.input_mode.dual_axis_value(movement);
        let value = self.processor.process(value);
        Some(DualAxisData::from_xy(value))
    }
}

/// Input associated with mouse movement on a specific axis.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct MouseMoveAxis {
    /// The axis that this input tracks.
    pub(crate) axis: DualAxis,

    /// The [`AxisInputMode`] for the axis.
    pub(crate) input_mode: AxisInputMode,

    /// The [`AxisProcessor`] used to handle input values.
    pub(crate) processor: AxisProcessor,
}

impl MouseMoveAxis {
    /// Creates a [`MouseMoveAxis`] for continuous movement on the X-axis without any processing applied.
    pub const fn analog_x() -> Self {
        Self {
            axis: DualAxis::X,
            input_mode: AxisInputMode::Analog,
            processor: AxisProcessor::None,
        }
    }

    /// Creates a [`MouseMoveAxis`] for continuous movement on the Y-axis without any processing applied.
    pub const fn analog_y() -> Self {
        Self {
            axis: DualAxis::Y,
            input_mode: AxisInputMode::Analog,
            processor: AxisProcessor::None,
        }
    }

    /// Creates a [`MouseMoveAxis`] for discrete movement on the X-axis without any processing applied.
    pub const fn digital_x() -> Self {
        Self {
            axis: DualAxis::X,
            input_mode: AxisInputMode::Digital,
            processor: AxisProcessor::None,
        }
    }

    /// Creates a [`MouseMoveAxis`] for discrete movement on the Y-axis without any processing applied.
    pub const fn digital_y() -> Self {
        Self {
            axis: DualAxis::Y,
            input_mode: AxisInputMode::Digital,
            processor: AxisProcessor::None,
        }
    }
    /// Creates a [`MouseMoveAxis`] for continuous movement on the X-axis with the specified [`AxisProcessor`].
    pub fn analog_x_using(processor: impl Into<AxisProcessor>) -> Self {
        Self {
            axis: DualAxis::X,
            input_mode: AxisInputMode::Analog,
            processor: processor.into(),
        }
    }

    /// Creates a [`MouseMoveAxis`] for continuous movement on the Y-axis with the specified [`AxisProcessor`].
    pub fn analog_y_using(processor: impl Into<AxisProcessor>) -> Self {
        Self {
            axis: DualAxis::Y,
            input_mode: AxisInputMode::Analog,
            processor: processor.into(),
        }
    }

    /// Creates a [`MouseMoveAxis`] for discrete movement on the X-axis with the specified [`AxisProcessor`].
    pub fn digital_x_using(processor: impl Into<AxisProcessor>) -> Self {
        Self {
            axis: DualAxis::X,
            input_mode: AxisInputMode::Analog,
            processor: processor.into(),
        }
    }

    /// Creates a [`MouseMoveAxis`] for discrete movement on the Y-axis with the specified [`AxisProcessor`].
    pub fn digital_y_using(processor: impl Into<AxisProcessor>) -> Self {
        Self {
            axis: DualAxis::Y,
            input_mode: AxisInputMode::Analog,
            processor: processor.into(),
        }
    }

    /// Appends the given [`AxisProcessor`] as the next processing step.
    #[inline]
    pub fn with_processor(mut self, processor: impl Into<AxisProcessor>) -> Self {
        self.processor = self.processor.with_processor(processor);
        self
    }

    /// Replaces the current [`AxisProcessor`] with the specified `processor`.
    #[inline]
    pub fn replace_processor(mut self, processor: impl Into<AxisProcessor>) -> Self {
        self.processor = processor.into();
        self
    }

    /// Removes the current used [`AxisProcessor`].
    #[inline]
    pub fn no_processor(mut self) -> Self {
        self.processor = AxisProcessor::None;
        self
    }
}

#[serde_typetag]
impl UserInput for MouseMoveAxis {
    /// Checks if there is any recent mouse movement along the specified axis.
    #[must_use]
    #[inline]
    fn is_active(&self, input_streams: &InputStreams) -> bool {
        let movement = accumulate_mouse_movement(input_streams);
        let value = self.axis.value(movement);
        self.processor.process(value) != 0.0
    }

    /// Retrieves the amount of the mouse movement along the specified axis
    /// after processing by the associated [`AxisProcessor`].
    #[must_use]
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        let movement = accumulate_mouse_movement(input_streams);
        let value = self.axis.value(movement);
        let value = self.processor.process(value);
        self.input_mode.axis_value(value)
    }
}

/// Input associated with mouse movement on a specific direction, treated as a button press.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct MouseMoveDirection(DualAxisDirection);

impl MouseMoveDirection {
    /// Movement in the upward direction.
    const UP: Self = Self(DualAxisDirection::Up);

    /// Movement in the downward direction.
    const DOWN: Self = Self(DualAxisDirection::Down);

    /// Movement in the leftward direction.
    const LEFT: Self = Self(DualAxisDirection::Left);

    /// Movement in the rightward direction.
    const RIGHT: Self = Self(DualAxisDirection::Right);
}

#[serde_typetag]
impl UserInput for MouseMoveDirection {
    /// Checks if there is any recent mouse movement along the specified direction.
    #[must_use]
    #[inline]
    fn is_active(&self, input_streams: &InputStreams) -> bool {
        let movement = accumulate_mouse_movement(input_streams);
        self.0.is_active(movement)
    }

    /// Retrieves the amount of the mouse movement along the specified direction,
    /// returning `0.0` for no movement and `1.0` for a currently active direction.
    #[must_use]
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        f32::from(self.is_active(input_streams))
    }
}

/// Accumulates the mouse wheel movement.
#[must_use]
#[inline]
fn accumulate_wheel_movement(input_streams: &InputStreams) -> Vec2 {
    let Some(wheel) = &input_streams.mouse_wheel else {
        return Vec2::ZERO;
    };

    wheel.iter().map(|event| Vec2::new(event.x, event.y)).sum()
}

/// Input associated with mouse wheel movement on both X and Y axes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct MouseScroll {
    /// The [`AxisInputMode`] for both axes.
    pub(crate) input_mode: AxisInputMode,

    /// The [`DualAxisProcessor`] used to handle input values.
    pub(crate) processor: DualAxisProcessor,
}

impl MouseScroll {
    /// Creates a [`MouseScroll`] for continuous movement on X and Y axes without any processing applied.
    pub const fn analog() -> Self {
        Self {
            input_mode: AxisInputMode::Analog,
            processor: DualAxisProcessor::None,
        }
    }

    /// Creates a [`MouseScroll`] for discrete movement on X and Y axes without any processing applied.
    pub const fn digital() -> Self {
        Self {
            input_mode: AxisInputMode::Digital,
            processor: DualAxisProcessor::None,
        }
    }

    /// Creates a [`MouseScroll`] for continuous movement on X and Y axes with the specified [`DualAxisProcessor`].
    pub fn analog_using(processor: impl Into<DualAxisProcessor>) -> Self {
        Self {
            input_mode: AxisInputMode::Analog,
            processor: processor.into(),
        }
    }

    /// Creates a [`MouseScroll`] for discrete movement on X and Y axes with the specified [`DualAxisProcessor`].
    pub fn digital_using(processor: impl Into<DualAxisProcessor>) -> Self {
        Self {
            input_mode: AxisInputMode::Analog,
            processor: processor.into(),
        }
    }

    /// Appends the given [`DualAxisProcessor`] as the next processing step.
    #[inline]
    pub fn with_processor(mut self, processor: impl Into<DualAxisProcessor>) -> Self {
        self.processor = self.processor.with_processor(processor);
        self
    }

    /// Replaces the current [`DualAxisProcessor`] with the specified `processor`.
    #[inline]
    pub fn replace_processor(mut self, processor: impl Into<DualAxisProcessor>) -> Self {
        self.processor = processor.into();
        self
    }

    /// Removes the current used [`DualAxisProcessor`].
    #[inline]
    pub fn no_processor(mut self) -> Self {
        self.processor = DualAxisProcessor::None;
        self
    }
}

#[serde_typetag]
impl UserInput for MouseScroll {
    /// Checks if there is any recent mouse wheel movement.
    #[must_use]
    #[inline]
    fn is_active(&self, input_streams: &InputStreams) -> bool {
        let movement = accumulate_wheel_movement(input_streams);
        self.processor.process(movement) != Vec2::ZERO
    }

    /// Retrieves the amount of the mouse wheel movement on both axes
    /// after processing by the associated [`DualAxisProcessor`].
    #[must_use]
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        let movement = accumulate_wheel_movement(input_streams);
        let value = self.processor.process(movement);
        self.input_mode.dual_axis_magnitude(value)
    }

    /// Retrieves the mouse scroll movement on both axes
    /// after processing by the associated [`DualAxisProcessor`].
    #[must_use]
    #[inline]
    fn axis_pair(&self, input_streams: &InputStreams) -> Option<DualAxisData> {
        let movement = accumulate_wheel_movement(input_streams);
        let value = self.input_mode.dual_axis_value(movement);
        let value = self.processor.process(value);
        Some(DualAxisData::from_xy(value))
    }
}

/// Input associated with mouse wheel movement on a specific axis.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct MouseScrollAxis {
    /// The axis that this input tracks.
    pub(crate) axis: DualAxis,

    /// The [`AxisInputMode`] for the axis.
    pub(crate) input_mode: AxisInputMode,

    /// The [`AxisProcessor`] used to handle input values.
    pub(crate) processor: AxisProcessor,
}

impl MouseScrollAxis {
    /// Creates a [`MouseScrollAxis`] for continuous movement on the X-axis without any processing applied.
    pub const fn analog_x() -> Self {
        Self {
            axis: DualAxis::X,
            input_mode: AxisInputMode::Analog,
            processor: AxisProcessor::None,
        }
    }

    /// Creates a [`MouseScrollAxis`] for continuous movement on the Y-axis without any processing applied.
    pub const fn analog_y() -> Self {
        Self {
            axis: DualAxis::Y,
            input_mode: AxisInputMode::Analog,
            processor: AxisProcessor::None,
        }
    }

    /// Creates a [`MouseScrollAxis`] for discrete movement on the X-axis without any processing applied.
    pub const fn digital_x() -> Self {
        Self {
            axis: DualAxis::X,
            input_mode: AxisInputMode::Digital,
            processor: AxisProcessor::None,
        }
    }

    /// Creates a [`MouseScrollAxis`] for discrete movement on the Y-axis without any processing applied.
    pub const fn digital_y() -> Self {
        Self {
            axis: DualAxis::Y,
            input_mode: AxisInputMode::Digital,
            processor: AxisProcessor::None,
        }
    }
    /// Creates a [`MouseScrollAxis`] for continuous movement on the X-axis with the specified [`AxisProcessor`].
    pub fn analog_x_using(processor: impl Into<AxisProcessor>) -> Self {
        Self {
            axis: DualAxis::X,
            input_mode: AxisInputMode::Analog,
            processor: processor.into(),
        }
    }

    /// Creates a [`MouseScrollAxis`] for continuous movement on the Y-axis with the specified [`AxisProcessor`].
    pub fn analog_y_using(processor: impl Into<AxisProcessor>) -> Self {
        Self {
            axis: DualAxis::Y,
            input_mode: AxisInputMode::Analog,
            processor: processor.into(),
        }
    }

    /// Creates a [`MouseScrollAxis`] for discrete movement on the X-axis with the specified [`AxisProcessor`].
    pub fn digital_x_using(processor: impl Into<AxisProcessor>) -> Self {
        Self {
            axis: DualAxis::X,
            input_mode: AxisInputMode::Analog,
            processor: processor.into(),
        }
    }

    /// Creates a [`MouseScrollAxis`] for discrete movement on the Y-axis with the specified [`AxisProcessor`].
    pub fn digital_y_using(processor: impl Into<AxisProcessor>) -> Self {
        Self {
            axis: DualAxis::Y,
            input_mode: AxisInputMode::Analog,
            processor: processor.into(),
        }
    }

    /// Appends the given [`AxisProcessor`] as the next processing step.
    #[inline]
    pub fn with_processor(mut self, processor: impl Into<AxisProcessor>) -> Self {
        self.processor = self.processor.with_processor(processor);
        self
    }

    /// Replaces the current [`AxisProcessor`] with the specified `processor`.
    #[inline]
    pub fn replace_processor(mut self, processor: impl Into<AxisProcessor>) -> Self {
        self.processor = processor.into();
        self
    }

    /// Removes the current used [`AxisProcessor`].
    #[inline]
    pub fn no_processor(mut self) -> Self {
        self.processor = AxisProcessor::None;
        self
    }
}

#[serde_typetag]
impl UserInput for MouseScrollAxis {
    /// Checks if there is any recent mouse wheel movement along the specified axis.
    #[must_use]
    #[inline]
    fn is_active(&self, input_streams: &InputStreams) -> bool {
        let movement = accumulate_wheel_movement(input_streams);
        let value = self.axis.value(movement);
        self.processor.process(value) != 0.0
    }

    /// Retrieves the amount of the mouse wheel movement along the specified axis
    /// after processing by the associated [`AxisProcessor`].
    #[must_use]
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        let movement = accumulate_wheel_movement(input_streams);
        let value = self.axis.value(movement);
        let value = self.input_mode.axis_value(value);
        self.processor.process(value)
    }
}

/// Input associated with mouse wheel movement on a specific direction, treated as a button press.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct MouseScrollDirection(DualAxisDirection);

impl MouseScrollDirection {
    /// Movement in the upward direction.
    const UP: Self = Self(DualAxisDirection::Up);

    /// Movement in the downward direction.
    const DOWN: Self = Self(DualAxisDirection::Down);

    /// Movement in the leftward direction.
    const LEFT: Self = Self(DualAxisDirection::Left);

    /// Movement in the rightward direction.
    const RIGHT: Self = Self(DualAxisDirection::Right);
}

#[serde_typetag]
impl UserInput for MouseMoveDirection {
    /// Checks if there is any recent mouse wheel movement along the specified direction.
    #[must_use]
    #[inline]
    fn is_active(&self, input_streams: &InputStreams) -> bool {
        let movement = accumulate_wheel_movement(input_streams);
        self.0.is_active(movement)
    }

    /// Retrieves the magnitude of the mouse wheel movement along the specified direction,
    /// returning `0.0` for no movement and `1.0` for a currently active direction.
    #[must_use]
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        f32::from(self.is_active(input_streams))
    }
}
