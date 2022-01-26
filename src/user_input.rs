//! Helpful abstractions over user input

use bevy::input::{
    gamepad::{GamepadButton, GamepadButtonType},
    keyboard::KeyCode,
    mouse::MouseButton,
    Input,
};
use bevy::utils::HashSet;
use petitset::PetitSet;
use strum::EnumIter;

/// Some combination of user input, which may cross [`Input`] boundaries
///
/// Suitable for use in an [`InputMap`]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UserInput {
    /// A null user input, used for a safe default and error-handling
    ///
    /// This input can never be pressed.
    Null,
    /// A single button
    Single(InputButton),
    /// A combination of buttons, pressed simultaneously
    ///
    /// Up to 8 (!!) buttons can be chorded together at once.
    /// Chords are considered to belong to all of the [InputMode]s of their constituent buttons.
    Chord(PetitSet<InputButton, 8>),
}

impl Default for UserInput {
    fn default() -> Self {
        UserInput::Null
    }
}

impl UserInput {
    /// Creates a [`UserInput::Chord`] from an iterator of [`Button`]s
    ///
    /// If `buttons` has a length of 1, a [`UserInput::Single`] variant will be returned instead.
    /// If `buttons` has a length of 0, a [`UserInput::Null`] variant will be returned instead.
    pub fn chord(buttons: impl IntoIterator<Item = impl Into<InputButton>>) -> Self {
        // We can't just check the length unless we add an ExactSizeIterator bound :(
        let mut length: u8 = 0;

        let mut set: PetitSet<InputButton, 8> = PetitSet::default();
        for button in buttons {
            length += 1;
            set.insert(button.into());
        }

        match length {
            0 => UserInput::Null,
            1 => UserInput::Single(set.into_iter().next().unwrap()),
            _ => UserInput::Chord(set),
        }
    }

    /// Which [`InputMode`]s does this input contain?
    pub fn input_modes(&self) -> PetitSet<InputMode, 3> {
        let mut set = PetitSet::default();
        match self {
            UserInput::Null => (),
            UserInput::Single(button) => {
                set.insert((*button).into());
            }
            UserInput::Chord(buttons) => {
                for &button in buttons.iter() {
                    set.insert(button.into());
                }
            }
        }
        set
    }

    /// Does this [`UserInput`] match the provided [`InputMode`]?
    ///
    /// For [`UserInput::Chord`], this will be true if any of the buttons in the combination match.
    pub fn matches_input_mode(&self, input_mode: InputMode) -> bool {
        // This is slightly faster than using Self::input_modes
        // As we can return early
        match self {
            UserInput::Single(button) => {
                let button_mode: InputMode = (*button).into();
                button_mode == input_mode
            }
            UserInput::Chord(set) => {
                for button in set.iter() {
                    let button_mode: InputMode = (*button).into();
                    if button_mode == input_mode {
                        return true;
                    }
                }
                false
            }
            UserInput::Null => false,
        }
    }

    /// The number of buttons in the [`UserInput`]
    pub fn len(&self) -> u8 {
        match self {
            UserInput::Null => 0,
            UserInput::Single(_) => 1,
            UserInput::Chord(button_set) => button_set.len().try_into().unwrap(),
        }
    }

    /// Is the number of buttons in the [`UserInput`] 0?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// How many of the provided `buttons` are found in the [`UserInput`]
    pub fn n_matching(&self, buttons: &HashSet<InputButton>) -> u8 {
        match self {
            UserInput::Null => 0,
            UserInput::Single(button) => {
                if buttons.contains(button) {
                    1
                } else {
                    0
                }
            }
            UserInput::Chord(chord_buttons) => {
                let mut n_matching = 0;
                for button in buttons.iter() {
                    if chord_buttons.contains(button) {
                        n_matching += 1;
                    }
                }

                n_matching
            }
        }
    }
}

impl From<InputButton> for UserInput {
    fn from(input: InputButton) -> Self {
        UserInput::Single(input)
    }
}

impl From<GamepadButtonType> for UserInput {
    fn from(input: GamepadButtonType) -> Self {
        UserInput::Single(InputButton::Gamepad(input))
    }
}

impl From<KeyCode> for UserInput {
    fn from(input: KeyCode) -> Self {
        UserInput::Single(InputButton::Keyboard(input))
    }
}

impl From<MouseButton> for UserInput {
    fn from(input: MouseButton) -> Self {
        UserInput::Single(InputButton::Mouse(input))
    }
}

/// A button-like input type
///
/// See [`Button`] for the value-ful equivalent.
/// Use the [`From`] or [`Into`] traits to convert from a [`Button`] to a [`InputMode`].
///
/// Unfortunately we cannot use a trait object here, as the types used by `Input`
/// require traits that are not object-safe.
///
/// Please contact the maintainers if you need support for another type!
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum InputMode {
    /// A gamepad
    Gamepad,
    /// A keyboard
    Keyboard,
    /// A mouse
    Mouse,
}

impl From<InputButton> for InputMode {
    fn from(button: InputButton) -> Self {
        match button {
            InputButton::Gamepad(_) => InputMode::Gamepad,
            InputButton::Keyboard(_) => InputMode::Keyboard,
            InputButton::Mouse(_) => InputMode::Mouse,
        }
    }
}

/// The values of a button-like input type
///
/// See [`InputMode`] for the value-less equivalent. Commonly stored in the [`UserInput`] enum.
///
/// Unfortunately we cannot use a trait object here, as the types used by `Input`
/// require traits that are not object-safe.
///
/// Please contact the maintainers if you need support for another type!
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputButton {
    /// A button on a gamepad
    Gamepad(GamepadButtonType),
    /// A button on a keyboard
    Keyboard(KeyCode),
    /// A button on a mouse
    Mouse(MouseButton),
}

impl From<GamepadButtonType> for InputButton {
    fn from(input: GamepadButtonType) -> Self {
        InputButton::Gamepad(input)
    }
}

impl From<KeyCode> for InputButton {
    fn from(input: KeyCode) -> Self {
        InputButton::Keyboard(input)
    }
}

impl From<MouseButton> for InputButton {
    fn from(input: MouseButton) -> Self {
        InputButton::Mouse(input)
    }
}

/// A collection of [`Input`] structs, which can be used to update an [`InputMap`](crate::input_map::InputMap).
///
/// Each of these streams is optional; if a stream does not exist, it is treated as if it were entirely unpressed.
///
/// These are typically collected via a system from the [`World`](bevy::prelude::World) as resources.
#[derive(Debug, Clone)]
pub struct InputStreams<'a> {
    /// An optional [`GamepadButton`] [`Input`] stream
    pub gamepad: Option<&'a Input<GamepadButton>>,
    /// An optional [`KeyCode`] [`Input`] stream
    pub keyboard: Option<&'a Input<KeyCode>>,
    /// An optional [`MouseButton`] [`Input`] stream
    pub mouse: Option<&'a Input<MouseButton>>,
}
