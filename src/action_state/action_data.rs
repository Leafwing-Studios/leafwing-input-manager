//! Contains types used to store the state of the actions held in an [`ActionState`](super::ActionState).

use bevy::{
    math::{Vec2, Vec3},
    platform_support::time::Instant,
    reflect::Reflect,
};
use serde::{Deserialize, Serialize};

use crate::buttonlike::ButtonValue;
#[cfg(feature = "timing")]
use crate::timing::Timing;
use crate::{buttonlike::ButtonState, InputControlKind};

/// Data about the state of an action.
///
/// Universal data about the state of the data is stored directly in this struct,
/// while data for each kind of action (buttonlike, axislike...) is stored in the `kind_data` field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct ActionData {
    /// Whether or not the action is disabled.
    ///
    /// While disabled, buttons will always report as released, and axes will always report as 0.
    pub disabled: bool,
    /// The data for the action.
    pub kind_data: ActionKindData,
}

impl ActionData {
    /// Constructs a new `ActionData` with default values corresponding to the given `kind_data`.
    pub fn from_kind(input_control_kind: InputControlKind) -> Self {
        Self {
            disabled: false,
            kind_data: match input_control_kind {
                InputControlKind::Button => ActionKindData::Button(ButtonData::default()),
                InputControlKind::Axis => ActionKindData::Axis(AxisData::default()),
                InputControlKind::DualAxis => ActionKindData::DualAxis(DualAxisData::default()),
                InputControlKind::TripleAxis => {
                    ActionKindData::TripleAxis(TripleAxisData::default())
                }
            },
        }
    }

    /// Ticks the action data, updating the state of the action.
    pub fn tick(&mut self, _current_instant: Instant, _previous_instant: Instant) {
        match self.kind_data {
            ActionKindData::Button(ref mut data) => {
                data.state.tick();

                #[cfg(feature = "timing")]
                data.timing.tick(_current_instant, _previous_instant);
            }
            ActionKindData::Axis(ref mut _data) => {}
            ActionKindData::DualAxis(ref mut _data) => {}
            ActionKindData::TripleAxis(ref mut _data) => {}
        }
    }
}

/// A wrapper over the various forms of data that an action can take.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub enum ActionKindData {
    /// The data for a button-like action.
    Button(ButtonData),
    /// The data for an axis-like action.
    Axis(AxisData),
    /// The data for a dual-axis-like action.
    DualAxis(DualAxisData),
    /// The data for a triple-axis-like action.
    TripleAxis(TripleAxisData),
}

impl ActionKindData {
    pub(super) fn swap_to_update_state(&mut self) {
        // save the changes applied to `state` into `fixed_update_state`
        // switch to loading the `update_state` into `state`
        match self {
            Self::Button(data) => {
                data.fixed_update_state = data.state;
                data.fixed_update_value = data.value;
                data.state = data.update_state;
                data.value = data.update_value;
            }
            Self::Axis(data) => {
                data.fixed_update_value = data.value;
                data.value = data.update_value;
            }
            Self::DualAxis(data) => {
                data.fixed_update_pair = data.pair;
                data.pair = data.update_pair;
            }
            Self::TripleAxis(data) => {
                data.fixed_update_triple = data.triple;
                data.triple = data.update_triple;
            }
        }
    }

    pub(super) fn swap_to_fixed_update_state(&mut self) {
        // save the changes applied to `state` into `update_state`
        // switch to loading the `fixed_update_state` into `state`
        match self {
            Self::Button(data) => {
                data.update_state = data.state;
                data.update_value = data.value;
                data.state = data.fixed_update_state;
                data.value = data.fixed_update_value;
            }
            Self::Axis(data) => {
                data.update_value = data.value;
                data.value = data.fixed_update_value;
            }
            Self::DualAxis(data) => {
                data.update_pair = data.pair;
                data.pair = data.fixed_update_pair;
            }
            Self::TripleAxis(data) => {
                data.update_triple = data.triple;
                data.triple = data.fixed_update_triple;
            }
        }
    }
}

/// Metadata about an [`Buttonlike`](crate::user_input::Buttonlike) action
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct ButtonData {
    /// Is the action pressed or released?
    pub state: ButtonState,
    /// The `state` of the action in the `Main` schedule
    pub update_state: ButtonState,
    /// The `state` of the action in the `FixedMain` schedule
    pub fixed_update_state: ButtonState,
    /// How far has the button been pressed
    pub value: f32,
    /// The `value` of the action in the `Main` schedule
    pub update_value: f32,
    /// The `value` of the action in the `FixedMain` schedule
    pub fixed_update_value: f32,
    /// When was the button pressed / released, and how long has it been held for?
    #[cfg(feature = "timing")]
    pub timing: Timing,
}

impl ButtonData {
    /// The default data for a button that was just pressed.
    pub const JUST_PRESSED: Self = Self {
        state: ButtonState::JustPressed,
        update_state: ButtonState::JustPressed,
        fixed_update_state: ButtonState::JustPressed,
        value: 1.0,
        update_value: 1.0,
        fixed_update_value: 1.0,
        #[cfg(feature = "timing")]
        timing: Timing::NEW,
    };

    /// The default data for a button that was just released.
    pub const JUST_RELEASED: Self = Self {
        state: ButtonState::JustReleased,
        update_state: ButtonState::JustReleased,
        fixed_update_state: ButtonState::JustReleased,
        value: 0.0,
        update_value: 0.0,
        fixed_update_value: 0.0,
        #[cfg(feature = "timing")]
        timing: Timing::NEW,
    };

    /// The default data for a button that is released,
    /// but was not just released.
    ///
    /// This is the default state for a button,
    /// as it avoids surprising behavior when the button is first created.
    pub const RELEASED: Self = Self {
        state: ButtonState::Released,
        update_state: ButtonState::Released,
        fixed_update_state: ButtonState::Released,
        value: 0.0,
        update_value: 0.0,
        fixed_update_value: 0.0,
        #[cfg(feature = "timing")]
        timing: Timing::NEW,
    };

    /// Is the action currently pressed?
    #[inline]
    #[must_use]
    pub fn pressed(&self) -> bool {
        self.state.pressed()
    }

    /// Was the action pressed since the last time it was ticked?
    #[inline]
    #[must_use]
    pub fn just_pressed(&self) -> bool {
        self.state.just_pressed()
    }

    /// Is the action currently released?
    #[inline]
    #[must_use]
    pub fn released(&self) -> bool {
        self.state.released()
    }

    /// Was the action released since the last time it was ticked?
    #[inline]
    #[must_use]
    pub fn just_released(&self) -> bool {
        self.state.just_released()
    }

    /// Convert `self` to a [`ButtonValue`].
    #[inline]
    #[must_use]
    pub fn to_button_value(&self) -> ButtonValue {
        ButtonValue::new(self.state.pressed(), self.value)
    }
}

/// The raw data for an [`ActionState`](super::ActionState) corresponding to a single virtual axis.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize, Reflect)]
pub struct AxisData {
    /// How far the axis is currently pressed
    pub value: f32,
    /// The `value` of the action in the `Main` schedule
    pub update_value: f32,
    /// The `value` of the action in the `FixedMain` schedule
    pub fixed_update_value: f32,
}

/// The raw data for an [`ActionState`](super::ActionState) corresponding to a pair of virtual axes.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize, Reflect)]
pub struct DualAxisData {
    /// The XY coordinates of the axis
    pub pair: Vec2,
    /// The `pair` of the action in the `Main` schedule
    pub update_pair: Vec2,
    /// The `pair` of the action in the `FixedMain` schedule
    pub fixed_update_pair: Vec2,
}

/// The raw data for an [`ActionState`](super::ActionState) corresponding to a triple of virtual axes.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize, Reflect)]
pub struct TripleAxisData {
    /// The XYZ coordinates of the axis
    pub triple: Vec3,
    /// The `triple` of the action in the `Main` schedule
    pub update_triple: Vec3,
    /// The `triple` of the action in the `FixedMain` schedule
    pub fixed_update_triple: Vec3,
}
