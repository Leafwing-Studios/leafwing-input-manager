//! Containment module for boring implementations of the [`Display`] trait

use crate::axislike::{VirtualAxis, VirtualDPad};
use crate::user_input::{InputKind, UserInput};
use itertools::Itertools;
use std::fmt::Display;

impl Display for UserInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // The representation of the button
            UserInput::Single(button) => write!(f, "{button}"),
            // The representation of each button, separated by "+"
            UserInput::Chord(button_set) => f.write_str(&button_set.iter().join("+")),
            UserInput::VirtualDPad(VirtualDPad {
                up,
                down,
                left,
                right,
                ..
            }) => {
                write!(
                    f,
                    "VirtualDPad(up: {up}, down: {down}, left: {left}, right: {right})"
                )
            }
            UserInput::VirtualAxis(VirtualAxis {
                negative, positive, ..
            }) => {
                write!(f, "VirtualDPad(negative: {negative}, positive: {positive})")
            }
        }
    }
}

impl Display for InputKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputKind::SingleAxis(axis) => write!(f, "{axis:?}"),
            InputKind::DualAxis(axis) => write!(f, "{axis:?}"),
            InputKind::GamepadButton(button) => write!(f, "{button:?}"),
            InputKind::Mouse(button) => write!(f, "{button:?}"),
            InputKind::MouseWheel(button) => write!(f, "{button:?}"),
            InputKind::MouseMotion(button) => write!(f, "{button:?}"),
            // TODO: We probably want to display the key on the currently active layout
            InputKind::PhysicalKey(key_code) => write!(f, "{key_code:?}"),
            InputKind::Modifier(button) => write!(f, "{button:?}"),
        }
    }
}
