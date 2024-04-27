//! Keyboard inputs

use bevy::prelude::{KeyCode, Reflect, Vec2};
use leafwing_input_manager_macros::serde_typetag;
use serde::{Deserialize, Serialize};

use crate::input_streams::InputStreams;
use crate::user_inputs::UserInput;

// Built-in support for KeyCode
#[serde_typetag]
impl UserInput for KeyCode {
    /// Checks if the specified [`KeyCode`] is currently pressed down.
    #[inline]
    fn is_active(&self, input_streams: &InputStreams) -> bool {
        input_streams
            .keycodes
            .is_some_and(|keys| keys.pressed(*self))
    }

    /// Retrieves the strength of the key press for the specified [`KeyCode`].
    ///
    /// # Returns
    ///
    /// - `1.0`: The key is currently pressed down, indicating an active input.
    /// - `0.0`: The key is not pressed, signifying no input.
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        f32::from(self.is_active(input_streams))
    }

    /// Always returns [`None`] as [`KeyCode`]s don't represent dual-axis input.
    #[inline]
    fn axis_pair(&self, _input_streams: &InputStreams) -> Option<Vec2> {
        None
    }
}

/// Defines different keyboard modifiers like Alt, Control, Shift, and Super (OS symbol key).
///
/// Each variant represents a pair of [`KeyCode`]s, the left and right version of the modifier key,
/// allowing for handling modifiers regardless of which side is pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub enum ModifierKey {
    /// The Alt key, representing either [`KeyCode::AltLeft`] or [`KeyCode::AltRight`].
    Alt,

    /// The Control key, representing either [`KeyCode::ControlLeft`] or [`KeyCode::ControlRight`].
    Control,

    /// The Shift key, representing either [`KeyCode::ShiftLeft`] or [`KeyCode::ShiftRight`].
    Shift,

    /// The Super (OS symbol) key, representing either [`KeyCode::SuperLeft`] or [`KeyCode::SuperRight`].
    Super,
}

impl ModifierKey {
    /// Returns a pair of [`KeyCode`]s corresponding to both modifier keys.
    pub const fn keys(&self) -> [KeyCode; 2] {
        [self.left(), self.right()]
    }

    /// Returns the [`KeyCode`] corresponding to the left modifier key.
    pub const fn left(&self) -> KeyCode {
        match self {
            ModifierKey::Alt => KeyCode::AltLeft,
            ModifierKey::Control => KeyCode::ControlLeft,
            ModifierKey::Shift => KeyCode::ShiftLeft,
            ModifierKey::Super => KeyCode::SuperLeft,
        }
    }

    /// Returns the [`KeyCode`] corresponding to the right modifier key.
    pub const fn right(&self) -> KeyCode {
        match self {
            ModifierKey::Alt => KeyCode::AltRight,
            ModifierKey::Control => KeyCode::ControlRight,
            ModifierKey::Shift => KeyCode::ShiftRight,
            ModifierKey::Super => KeyCode::SuperRight,
        }
    }
}

#[serde_typetag]
impl UserInput for ModifierKey {
    /// Checks if the specified [`ModifierKey`] is currently pressed down.
    ///
    /// # Returns
    ///
    /// - `true`: The key is currently pressed down, indicating an active input.
    /// - `false`: The key is not pressed, signifying no input.
    #[inline]
    fn is_active(&self, input_streams: &InputStreams) -> bool {
        let modifiers = self.keys();
        input_streams
            .keycodes
            .is_some_and(|keys| keys.pressed(modifiers[0]) | keys.pressed(modifiers[1]))
    }

    /// Gets the strength of the key press for the specified [`ModifierKey`].
    ///
    /// # Returns
    ///
    /// - `1.0`: The key is currently pressed down, indicating an active input.
    /// - `0.0`: The key is not pressed, signifying no input.
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        f32::from(self.is_active(input_streams))
    }

    /// Always returns [`None`] as [`ModifierKey`]s don't represent dual-axis input.
    #[inline]
    fn axis_pair(&self, _input_streams: &InputStreams) -> Option<Vec2> {
        None
    }
}
