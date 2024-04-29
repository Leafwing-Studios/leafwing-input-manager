//! Gamepad inputs

use bevy::prelude::{Gamepad, GamepadAxis, GamepadAxisType, GamepadButton, GamepadButtonType};
use leafwing_input_manager_macros::serde_typetag;

use crate as leafwing_input_manager;
use crate::input_streams::InputStreams;
use crate::user_inputs::UserInput;

// Built-in support for Bevy's GamepadButtonType.
#[serde_typetag]
impl UserInput for GamepadButtonType {
    /// Checks if the specified [`GamepadButtonType`] is currently pressed down.
    ///
    /// When a [`Gamepad`] is specified, only checks if the button is pressed on the gamepad.
    /// Otherwise, checks if the button is pressed on any connected gamepads.
    #[inline]
    fn is_active(&self, input_streams: &InputStreams) -> bool {
        let gamepad_pressed_self = |gamepad: Gamepad| -> bool {
            let button = GamepadButton::new(gamepad, *self);
            input_streams.gamepad_buttons.pressed(button)
        };

        if let Some(gamepad) = input_streams.associated_gamepad {
            gamepad_pressed_self(gamepad)
        } else {
            input_streams.gamepads.iter().any(gamepad_pressed_self)
        }
    }

    /// Retrieves the strength of the button press for the specified [`GamepadButtonType`].
    ///
    /// When a [`Gamepad`] is specified, only retrieves the value of the button on the gamepad.
    /// Otherwise, retrieves the value on any connected gamepads.
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        // This implementation differs from `is_active()` because the upstream bevy::input
        // still waffles about whether triggers are buttons or axes.
        // So, we consider the axes for consistency with other gamepad axes (e.g., thumbs ticks).

        let gamepad_value_self = |gamepad: Gamepad| -> Option<f32> {
            let button = GamepadButton::new(gamepad, *self);
            input_streams.gamepad_button_axes.get(button)
        };

        if let Some(gamepad) = input_streams.associated_gamepad {
            gamepad_value_self(gamepad).unwrap_or_else(|| f32::from(self.is_active(input_streams)))
        } else {
            input_streams
                .gamepads
                .iter()
                .map(gamepad_value_self)
                .flatten()
                .find(|value| *value != 0.0)
                .unwrap_or_default()
        }
    }
}

// Built-in support for Bevy's GamepadAxisType.
// #[serde_typetag]
impl UserInput for GamepadAxisType {
    /// Checks if the specified [`GamepadAxisType`] is currently active.
    ///
    /// When a [`Gamepad`] is specified, only checks if the axis is triggered on the gamepad.
    /// Otherwise, checks if the axis is triggered on any connected gamepads.
    #[inline]
    fn is_active(&self, input_streams: &InputStreams) -> bool {
        self.value(input_streams) != 0.0
    }

    /// Retrieves the strength of the specified [`GamepadAxisType`].
    ///
    /// When a [`Gamepad`] is specified, only retrieves the value of the axis on the gamepad.
    /// Otherwise, retrieves the axis on any connected gamepads.
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        let gamepad_value_self = |gamepad: Gamepad| -> Option<f32> {
            let axis = GamepadAxis::new(gamepad, *self);
            input_streams.gamepad_axes.get(axis)
        };

        if let Some(gamepad) = input_streams.associated_gamepad {
            gamepad_value_self(gamepad).unwrap_or_else(|| f32::from(self.is_active(input_streams)))
        } else {
            input_streams
                .gamepads
                .iter()
                .map(gamepad_value_self)
                .flatten()
                .find(|value| *value != 0.0)
                .unwrap_or_default()
        }
    }
}
