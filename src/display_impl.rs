//! Containment module for boring implmentations of the [`Display`] trait

use crate::buttonlike_user_input::{InputButton, UserInput};
use std::fmt::Display;

impl Display for UserInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // The representation of the button
            UserInput::Single(button) => write!(f, "{button}"),
            // The representation of each button, seperated by "+"
            UserInput::Chord(button_set) => {
                let mut string = String::default();
                for button in button_set.iter() {
                    string.push('+');
                    string.push_str(&button.to_string());
                }
                write!(f, "{string}")
            }
        }
    }
}

impl Display for InputButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputButton::Gamepad(button) => write!(f, "{button:?}"),
            InputButton::Mouse(button) => write!(f, "{button:?}"),
            InputButton::Keyboard(button) => write!(f, "{button:?}"),
        }
    }
}
