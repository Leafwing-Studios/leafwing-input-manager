//! Utilities for handling keyboard inputs.
//!
//! This module provides utilities for working with keyboard inputs in the applications.
//! It includes support for querying the state of individual keys,
//! as well as modifiers like Alt, Control, Shift, and Super (OS symbol key).
//!
//! # Usage
//!
//! The [`UserInput`] trait is implemented for [`KeyCode`],
//! allowing you to easily query the state of specific keys.
//!
//! Additionally, the [`ModifierKey`] enum represents keyboard modifiers
//! and provides methods for querying their state.

use bevy::prelude::{KeyCode, Reflect};
use serde::{Deserialize, Serialize};

use crate::input_streams::InputStreams;
use crate::user_inputs::UserInput;

/// Built-in support for Bevy's [`KeyCode`].
impl UserInput<'_> for KeyCode {
    /// Retrieves the strength of the key press.
    ///
    /// # Returns
    ///
    /// - `None` if the keyboard input tracking is unavailable.
    /// - `Some(0.0)` if this tracked key isn't pressed.
    /// - `Some(1.0)` if this tracked key is currently pressed.
    fn value(&self, input_query: InputStreams<'_>) -> Option<f32> {
        let keycode_stream = input_query;
        keycode_stream
            .keycodes
            .map(|keyboard| keyboard.pressed(*self))
            .map(f32::from)
    }

    /// Checks if this tracked key is being pressed down during the current tick.
    fn started(&self, input_query: InputStreams<'_>) -> bool {
        let keycode_stream = input_query;
        keycode_stream
            .keycodes
            .is_some_and(|keyboard| keyboard.just_pressed(*self))
    }

    /// Checks if this tracked key is being released during the current tick.
    fn finished(&self, input_query: InputStreams<'_>) -> bool {
        let keycode_stream = input_query;
        keycode_stream
            .keycodes
            .is_some_and(|keyboard| keyboard.just_released(*self))
    }
}

/// The keyboard modifier combining two [`KeyCode`] values into one representation.
///
/// Each variant corresponds to a pair of [`KeyCode`] modifiers,
/// such as Alt, Control, Shift, or Super (OS symbol key),
/// one for the left and one for the right key,
/// indicating the modifier's activation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum ModifierKey {
    /// The Alt key, corresponds to [`KeyCode::AltLeft`] and [`KeyCode::AltRight`].
    Alt,
    /// The Control key, corresponds to [`KeyCode::ControlLeft`] and [`KeyCode::ControlRight`].
    Control,
    /// The Shift key, corresponds to [`KeyCode::ShiftLeft`] and [`KeyCode::ShiftRight`].
    Shift,
    /// The Super (OS symbol) key, corresponds to [`KeyCode::SuperLeft`] and [`KeyCode::SuperRight`].
    Super,
}

impl ModifierKey {
    /// Returns the [`KeyCode`] corresponding to the left key of the modifier.
    pub const fn left(&self) -> KeyCode {
        match self {
            ModifierKey::Alt => KeyCode::AltLeft,
            ModifierKey::Control => KeyCode::ControlLeft,
            ModifierKey::Shift => KeyCode::ShiftLeft,
            ModifierKey::Super => KeyCode::SuperLeft,
        }
    }

    /// Returns the [`KeyCode`] corresponding to the right key of the modifier.
    pub const fn right(&self) -> KeyCode {
        match self {
            ModifierKey::Alt => KeyCode::AltRight,
            ModifierKey::Control => KeyCode::ControlRight,
            ModifierKey::Shift => KeyCode::ShiftRight,
            ModifierKey::Super => KeyCode::SuperRight,
        }
    }
}

impl UserInput<'_> for ModifierKey {
    /// Retrieves the strength of the key press.
    ///
    /// # Returns
    ///
    /// - `None` if the keyboard input tracking is unavailable.
    /// - `Some(0.0)` if these tracked keys aren't pressed.
    /// - `Some(1.0)` if these tracked keys are currently pressed.
    fn value(&self, input_query: InputStreams<'_>) -> Option<f32> {
        let keycode_stream = input_query;
        keycode_stream
            .keycodes
            .map(|keyboard| keyboard.pressed(self.left()) | keyboard.pressed(self.right()))
            .map(f32::from)
    }

    /// Checks if these tracked keys are being pressed down during the current tick.
    fn started(&self, input_query: InputStreams<'_>) -> bool {
        let keycode_stream = input_query;
        keycode_stream.keycodes.is_some_and(|keyboard| {
            keyboard.just_pressed(self.left()) | keyboard.just_pressed(self.right())
        })
    }

    /// Checks if these tracked keys are being released during the current tick.
    fn finished(&self, input_query: InputStreams<'_>) -> bool {
        let keycode_stream = input_query;
        keycode_stream.keycodes.is_some_and(|keyboard| {
            keyboard.just_released(self.left()) | keyboard.just_released(self.right())
        })
    }
}
