//! Containment module for boring implementations of the [`Display`] trait

use crate::input_like::InputKind;
use std::fmt::Display;

impl Display for InputKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputKind::SingleAxis(axis) => write!(f, "{axis:?}"),
            InputKind::GamepadButton(button) => write!(f, "{button:?}"),
            InputKind::Mouse(button) => write!(f, "{button:?}"),
            InputKind::MouseWheel(button) => write!(f, "{button:?}"),
            InputKind::MouseMotion(button) => write!(f, "{button:?}"),
            InputKind::Keyboard(button) => write!(f, "{button:?}"),
            // TODO: We probably want to display the key on the currently active layout
            InputKind::KeyLocation(scan_code) => write!(f, "{scan_code:?}"),
            InputKind::Modifier(button) => write!(f, "{button:?}"),
        }
    }
}
