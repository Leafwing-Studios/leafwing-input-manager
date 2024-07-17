//! Contains types used to store the state of the actions held in an [`ActionState`](super::ActionState).

use bevy::{math::Vec2, reflect::Reflect};
use serde::{Deserialize, Serialize};

use crate::buttonlike::ButtonState;
#[cfg(feature = "timing")]
use crate::timing::Timing;

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
    /// Is the action disabled?
    ///
    /// While disabled, an action will always report as released, regardless of its actual state.
    pub disabled: bool,
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
        disabled: false,
    };

    /// The default data for a button that was just released.
    pub const JUST_RELEASED: Self = Self {
        state: ButtonState::JustReleased,
        update_state: ButtonState::JustReleased,
        fixed_update_state: ButtonState::JustReleased,
        #[cfg(feature = "timing")]
        timing: Timing::NEW,
        consumed: false,
        disabled: false,
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
        disabled: false,
    };

    /// Is the action currently pressed?
    #[inline]
    #[must_use]
    pub fn pressed(&self) -> bool {
        !self.disabled && self.state.pressed()
    }

    /// Was the action pressed since the last time it was ticked?
    #[inline]
    #[must_use]
    pub fn just_pressed(&self) -> bool {
        !self.disabled && self.state.just_pressed()
    }

    /// Is the action currently released?
    #[inline]
    #[must_use]
    pub fn released(&self) -> bool {
        self.disabled || self.state.released()
    }

    /// Was the action released since the last time it was ticked?
    #[inline]
    #[must_use]
    pub fn just_released(&self) -> bool {
        !self.disabled && self.state.just_released()
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
    /// Is the action disabled?
    ///
    /// While disabled, an action will always return 0, regardless of its actual state.
    pub disabled: bool,
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
    /// Is the action disabled?
    ///
    /// While disabled, an action will always return 0, regardless of its actual state.
    pub disabled: bool,
}
