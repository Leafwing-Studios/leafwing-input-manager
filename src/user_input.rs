//! Helpful abstractions over user inputs of all sorts

use bevy_input::{gamepad::GamepadButtonType, keyboard::KeyCode, mouse::MouseButton};

use bevy_utils::HashSet;
use petitset::PetitSet;
use serde::{Deserialize, Serialize};

use crate::axislike::{AxisType, DualAxis, SingleAxis, VirtualDPad};

/// Some combination of user input, which may cross [`Input`]-mode boundaries
///
/// Suitable for use in an [`InputMap`](crate::input_map::InputMap)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserInput {
    /// A single button
    Single(InputKind),
    /// A combination of buttons, pressed simultaneously
    ///
    /// Up to 8 (!!) buttons can be chorded together at once.
    /// Chords are considered to belong to all of the [InputMode]s of their constituent buttons.
    Chord(PetitSet<InputKind, 8>),
    /// A virtual DPad that you can get an [`AxisPair`] from
    VirtualDPad(VirtualDPad),
}

impl UserInput {
    /// Creates a [`UserInput::Chord`] from an iterator of [`Button`]s
    ///
    /// If `buttons` has a length of 1, a [`UserInput::Single`] variant will be returned instead.
    pub fn chord(buttons: impl IntoIterator<Item = impl Into<InputKind>>) -> Self {
        // We can't just check the length unless we add an ExactSizeIterator bound :(
        let mut length: u8 = 0;

        let mut set: PetitSet<InputKind, 8> = PetitSet::default();
        for button in buttons {
            length += 1;
            set.insert(button.into());
        }

        match length {
            1 => UserInput::Single(set.into_iter().next().unwrap()),
            _ => UserInput::Chord(set),
        }
    }

    /// Which [`InputMode`]s does this input contain?
    pub fn input_modes(&self) -> PetitSet<InputMode, 3> {
        let mut set = PetitSet::default();
        match self {
            UserInput::Single(button) => {
                set.insert((*button).into());
            }
            UserInput::Chord(buttons) => {
                for &button in buttons.iter() {
                    set.insert(button.into());
                }
            }
            UserInput::VirtualDPad(VirtualDPad {
                up,
                down,
                left,
                right,
            }) => {
                set.insert((*up).into());
                set.insert((*down).into());
                set.insert((*left).into());
                set.insert((*right).into());
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
            UserInput::VirtualDPad(VirtualDPad {
                up,
                down,
                left,
                right,
            }) => {
                for button in [up, down, left, right] {
                    let button_mode: InputMode = (*button).into();
                    if button_mode == input_mode {
                        return true;
                    }
                }
                false
            }
        }
    }

    /// The number of logical inputs that make up the [`UserInput`].
    ///
    /// - A [`Single`][UserInput::Single] input returns 1
    /// - A [`Chord`][UserInput::Chord] returns the number of buttons in the chord
    /// - A [`VirtualDPad`][UserInput::VirtualDPad] returns 1
    pub fn len(&self) -> usize {
        match self {
            UserInput::Single(_) => 1,
            UserInput::Chord(button_set) => button_set.len(),
            UserInput::VirtualDPad { .. } => 1,
        }
    }

    /// Is the number of buttons in the [`UserInput`] 0?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// How many of the provided `buttons` are found in the [`UserInput`]
    ///
    /// # Example
    /// ```rust
    /// use bevy_input::keyboard::KeyCode::*;
    /// use bevy_utils::HashSet;
    /// use leafwing_input_manager::user_input::UserInput;
    ///
    /// let buttons = HashSet::from_iter([LControl.into(), LAlt.into()]);
    /// let a: UserInput  = A.into();
    /// let ctrl_a = UserInput::chord([LControl, A]);
    /// let ctrl_alt_a = UserInput::chord([LControl, LAlt, A]);
    ///
    /// assert_eq!(a.n_matching(&buttons), 0);
    /// assert_eq!(ctrl_a.n_matching(&buttons), 1);
    /// assert_eq!(ctrl_alt_a.n_matching(&buttons), 2);
    /// ```
    pub fn n_matching(&self, buttons: &HashSet<InputKind>) -> usize {
        match self {
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
            UserInput::VirtualDPad(VirtualDPad {
                up,
                down,
                left,
                right,
            }) => {
                let mut n_matching = 0;
                for button in buttons.iter() {
                    for dpad_button in [up, down, left, right] {
                        if button == dpad_button {
                            n_matching += 1;
                        }
                    }
                }

                n_matching
            }
        }
    }

    /// Returns the raw inputs that make up this [`UserInput`]
    pub fn raw_inputs(
        &self,
    ) -> (
        Vec<GamepadButtonType>,
        Vec<(AxisType, Option<f32>)>,
        Vec<KeyCode>,
        Vec<MouseButton>,
    ) {
        let mut gamepad_axes: Vec<(AxisType, Option<f32>)> = Vec::default();
        let mut gamepad_buttons: Vec<GamepadButtonType> = Vec::default();
        let mut keyboard_buttons: Vec<KeyCode> = Vec::default();
        let mut mouse_buttons: Vec<MouseButton> = Vec::default();

        match self {
            UserInput::Single(button) => match *button {
                InputKind::DualAxis(variant) => {
                    let (x_value, y_value) = match variant.value {
                        Some(vec) => (Some(vec.x), Some(vec.y)),
                        None => (None, None),
                    };

                    gamepad_axes.push((variant.x_axis_type, x_value));
                    gamepad_axes.push((variant.y_axis_type, y_value));
                }
                InputKind::SingleAxis(variant) => {
                    gamepad_axes.push((variant.axis_type, variant.value))
                }
                InputKind::GamepadButton(variant) => gamepad_buttons.push(variant),
                InputKind::Keyboard(variant) => keyboard_buttons.push(variant),
                InputKind::Mouse(variant) => mouse_buttons.push(variant),
            },
            UserInput::Chord(button_set) => {
                for button in button_set.iter() {
                    match button {
                        InputKind::DualAxis(variant) => {
                            let (x_value, y_value) = match variant.value {
                                Some(vec) => (Some(vec.x), Some(vec.y)),
                                None => (None, None),
                            };

                            gamepad_axes.push((variant.x_axis_type, x_value));
                            gamepad_axes.push((variant.y_axis_type, y_value));
                        }
                        InputKind::SingleAxis(variant) => {
                            gamepad_axes.push((variant.axis_type, variant.value))
                        }
                        InputKind::GamepadButton(variant) => gamepad_buttons.push(*variant),
                        InputKind::Keyboard(variant) => keyboard_buttons.push(*variant),
                        InputKind::Mouse(variant) => mouse_buttons.push(*variant),
                    }
                }
            }
            UserInput::VirtualDPad(VirtualDPad {
                up,
                down,
                left,
                right,
            }) => {
                for button in [up, down, left, right] {
                    match *button {
                        InputKind::DualAxis(variant) => {
                            let (x_value, y_value) = match variant.value {
                                Some(vec) => (Some(vec.x), Some(vec.y)),
                                None => (None, None),
                            };

                            gamepad_axes.push((variant.x_axis_type, x_value));
                            gamepad_axes.push((variant.y_axis_type, y_value));
                        }
                        InputKind::SingleAxis(variant) => {
                            gamepad_axes.push((variant.axis_type, variant.value))
                        }
                        InputKind::GamepadButton(variant) => gamepad_buttons.push(variant),
                        InputKind::Keyboard(variant) => keyboard_buttons.push(variant),
                        InputKind::Mouse(variant) => mouse_buttons.push(variant),
                    }
                }
            }
        };

        (
            gamepad_buttons,
            gamepad_axes,
            keyboard_buttons,
            mouse_buttons,
        )
    }
}

impl From<InputKind> for UserInput {
    fn from(input: InputKind) -> Self {
        UserInput::Single(input)
    }
}

impl From<DualAxis> for UserInput {
    fn from(input: DualAxis) -> Self {
        UserInput::Single(InputKind::DualAxis(input))
    }
}

impl From<SingleAxis> for UserInput {
    fn from(input: SingleAxis) -> Self {
        UserInput::Single(InputKind::SingleAxis(input))
    }
}

impl From<VirtualDPad> for UserInput {
    fn from(input: VirtualDPad) -> Self {
        UserInput::VirtualDPad(input)
    }
}

impl From<GamepadButtonType> for UserInput {
    fn from(input: GamepadButtonType) -> Self {
        UserInput::Single(InputKind::GamepadButton(input))
    }
}

impl From<KeyCode> for UserInput {
    fn from(input: KeyCode) -> Self {
        UserInput::Single(InputKind::Keyboard(input))
    }
}

impl From<MouseButton> for UserInput {
    fn from(input: MouseButton) -> Self {
        UserInput::Single(InputKind::Mouse(input))
    }
}

/// What mode (sort of device) an [`InputKind`] originated from.
///
/// Use the [`From`] or [`Into`] traits to convert from a [`InputKind`] to a [`InputMode`].
///
/// Unfortunately we cannot use a trait object here, as the types used by `Input`
/// require traits that are not object-safe.
///
/// Please contact the maintainers if you need support for another type!
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputMode {
    /// A gamepad
    Gamepad,
    /// A keyboard
    Keyboard,
    /// A mouse
    Mouse,
}

impl InputMode {
    /// Iterates over the possible [`InputModes`](InputMode)
    pub fn iter() -> InputModeIter {
        InputModeIter::default()
    }
}

/// An iterator of [`InputModes`](InputMode)
///
/// Created by calling [`InputMode::iter`]
#[derive(Debug, Clone, Default)]
pub struct InputModeIter {
    cursor: u8,
}

impl Iterator for InputModeIter {
    type Item = InputMode;

    fn next(&mut self) -> Option<InputMode> {
        let item = match self.cursor {
            0 => Some(InputMode::Gamepad),
            1 => Some(InputMode::Keyboard),
            2 => Some(InputMode::Mouse),
            _ => None,
        };
        if self.cursor <= 2 {
            self.cursor += 1;
        }

        item
    }
}

impl From<InputKind> for InputMode {
    fn from(button: InputKind) -> Self {
        match button {
            InputKind::GamepadButton(_) | InputKind::SingleAxis(_) | InputKind::DualAxis(_) => {
                InputMode::Gamepad
            }
            InputKind::Keyboard(_) => InputMode::Keyboard,
            InputKind::Mouse(_) => InputMode::Mouse,
        }
    }
}

/// The different kinds of supported input bindings.
///
/// See [`InputMode`] for the value-less equivalent. Commonly stored in the [`UserInput`] enum.
///
/// Unfortunately we cannot use a trait object here, as the types used by `Input`
/// require traits that are not object-safe.
///
/// Please contact the maintainers if you need support for another type!
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputKind {
    /// A button on a gamepad
    GamepadButton(GamepadButtonType),
    /// A single axis of continous motion
    SingleAxis(SingleAxis),
    /// Two paired axes of continous motion
    DualAxis(DualAxis),
    /// A button on a keyboard
    Keyboard(KeyCode),
    /// A button on a mouse
    Mouse(MouseButton),
}

impl From<DualAxis> for InputKind {
    fn from(input: DualAxis) -> Self {
        InputKind::DualAxis(input)
    }
}

impl From<SingleAxis> for InputKind {
    fn from(input: SingleAxis) -> Self {
        InputKind::SingleAxis(input)
    }
}

impl From<GamepadButtonType> for InputKind {
    fn from(input: GamepadButtonType) -> Self {
        InputKind::GamepadButton(input)
    }
}

impl From<KeyCode> for InputKind {
    fn from(input: KeyCode) -> Self {
        InputKind::Keyboard(input)
    }
}

impl From<MouseButton> for InputKind {
    fn from(input: MouseButton) -> Self {
        InputKind::Mouse(input)
    }
}
