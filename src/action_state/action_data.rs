//! Contains types used to store the state of the actions held in an [`ActionState`](super::ActionState).

use bevy::{math::Vec2, reflect::Reflect};
use serde::{Deserialize, Serialize};

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
            },
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
}

/// Metadata about an [`Buttonlike`](crate::user_input::Buttonlike) action
///
/// If a button is released, its `reasons_pressed` should be empty.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct ButtonData {
    /// Is the action pressed or released?
    pub state: ButtonState,
    /// The `state` of the action in the `Main` schedule
    pub update_state: ButtonState,
    /// The `state` of the action in the `FixedMain` schedule
    pub fixed_update_state: ButtonState,
    /// When was the button pressed / released, and how long has it been held for?
    #[cfg(feature = "timing")]
    pub timing: Timing,
    /// Was this action consumed by [`ActionState::consume`](super::ActionState::consume)?
    ///
    /// Actions that are consumed cannot be pressed again until they are explicitly released.
    /// This ensures that consumed actions are not immediately re-pressed by continued inputs.
    pub consumed: bool,
}

impl ButtonData {
    /// The default data for a button that was just pressed.
    pub const JUST_PRESSED: Self = Self {
        state: ButtonState::JustPressed,
        update_state: ButtonState::JustPressed,
        fixed_update_state: ButtonState::JustPressed,
        #[cfg(feature = "timing")]
        timing: Timing::NEW,
        consumed: false,
    };

    /// The default data for a button that was just released.
    pub const JUST_RELEASED: Self = Self {
        state: ButtonState::JustReleased,
        update_state: ButtonState::JustReleased,
        fixed_update_state: ButtonState::JustReleased,
        #[cfg(feature = "timing")]
        timing: Timing::NEW,
        consumed: false,
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
        #[cfg(feature = "timing")]
        timing: Timing::NEW,
        consumed: false,
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

/// The raw data for an [`ActionState`](super::ActionState)  corresponding to a pair of virtual axes.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize, Reflect)]
pub struct DualAxisData {
    /// The XY coordinates of the axis
    pub pair: Vec2,
    /// The `pair` of the action in the `Main` schedule
    pub update_pair: Vec2,
    /// The `value` of the action in the `FixedMain` schedule
    pub fixed_update_pair: Vec2,
}
