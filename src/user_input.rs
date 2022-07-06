//! Helpful abstractions over user inputs of all sorts

use bevy_input::{
    gamepad::{Gamepad, GamepadAxis, GamepadAxisType, GamepadButton, GamepadButtonType},
    keyboard::KeyCode,
    mouse::MouseButton,
    Axis, Input,
};

use bevy_utils::HashSet;
use petitset::PetitSet;
use serde::{Deserialize, Serialize};

/// Some combination of user input, which may cross [`Input`] boundaries
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
        }
    }

    /// The number of buttons in the [`UserInput`]
    pub fn len(&self) -> usize {
        match self {
            UserInput::Single(_) => 1,
            UserInput::Chord(button_set) => button_set.len(),
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
        }
    }

    /// Returns the raw inputs that make up this [`UserInput`]
    pub fn raw_inputs(&self) -> (Vec<GamepadButtonType>, Vec<KeyCode>, Vec<MouseButton>) {
        let mut gamepad_axes: Vec<GamepadAxisType> = Vec::default();
        let mut gamepad_buttons: Vec<GamepadButtonType> = Vec::default();
        let mut keyboard_buttons: Vec<KeyCode> = Vec::default();
        let mut mouse_buttons: Vec<MouseButton> = Vec::default();

        match self {
            UserInput::Single(button) => match *button {
                InputKind::GamepadAxis(variant) => gamepad_axes.push(variant.axis),
                InputKind::GamepadButton(variant) => gamepad_buttons.push(variant),
                InputKind::Keyboard(variant) => keyboard_buttons.push(variant),
                InputKind::Mouse(variant) => mouse_buttons.push(variant),
            },
            UserInput::Chord(button_set) => {
                for button in button_set.iter() {
                    match button {
                        InputKind::GamepadAxis(variant) => gamepad_axes.push(variant.axis),
                        InputKind::GamepadButton(variant) => gamepad_buttons.push(*variant),
                        InputKind::Keyboard(variant) => keyboard_buttons.push(*variant),
                        InputKind::Mouse(variant) => mouse_buttons.push(*variant),
                    }
                }
            }
        };

        (gamepad_buttons, keyboard_buttons, mouse_buttons)
    }
}

impl From<InputKind> for UserInput {
    fn from(input: InputKind) -> Self {
        UserInput::Single(input)
    }
}

impl From<GamepadAxisThreshold> for UserInput {
    fn from(input: GamepadAxisThreshold) -> Self {
        UserInput::Single(InputKind::GamepadAxis(input))
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

/// A button-like input type
///
/// See [`Button`] for the value-ful equivalent.
/// Use the [`From`] or [`Into`] traits to convert from a [`InputButton`] to a [`InputMode`].
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
            InputKind::GamepadButton(_) | InputKind::GamepadAxis(_) => InputMode::Gamepad,
            InputKind::Keyboard(_) => InputMode::Keyboard,
            InputKind::Mouse(_) => InputMode::Mouse,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputKind {
    /// A button on a gamepad
    GamepadButton(GamepadButtonType),
    /// An axis on a gamepad
    GamepadAxis(GamepadAxisThreshold),
    /// A button on a keyboard
    Keyboard(KeyCode),
    /// A button on a mouse
    Mouse(MouseButton),
}

/// Used to indicate at which point a gamepad axis event should trigger an action.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GamepadAxisThreshold {
    /// The axis that is being checked.
    pub axis: GamepadAxisType,
    /// Indicates whether the action should trigger when the axis value is greater, lesser, or equal
    /// to the `threshold`.
    pub comparison: GamepadAxisComparison,
    /// The threshold to compare the current axis value with.
    pub threshold: f32,
}

impl PartialEq for GamepadAxisThreshold {
    fn eq(&self, other: &Self) -> bool {
        use bevy_core::FloatOrd;
        self.axis == other.axis
            && self.comparison == other.comparison
            && FloatOrd(self.threshold) == FloatOrd(other.threshold)
    }
}
impl Eq for GamepadAxisThreshold {}
impl std::hash::Hash for GamepadAxisThreshold {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use bevy_core::FloatOrd;
        self.axis.hash(state);
        self.comparison.hash(state);
        FloatOrd(self.threshold).hash(state);
    }
}

/// Different possible ways to compare an axis threshold to the axis value.
///
/// See [`GamepadAxisThreshold`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GamepadAxisComparison {
    /// Value must be less than threshold
    Less,
    /// Value must be greater than threshold
    Greater,
    /// Value must be equal to threshold
    Equal,
}

impl From<GamepadAxisThreshold> for InputKind {
    fn from(input: GamepadAxisThreshold) -> Self {
        InputKind::GamepadAxis(input)
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

/// A collection of [`Input`] structs, which can be used to update an [`InputMap`](crate::input_map::InputMap).
///
/// Each of these streams is optional; if a stream does not exist, it is treated as if it were entirely unpressed.
///
/// These are typically collected via a system from the [`World`](bevy::prelude::World) as resources.
#[derive(Debug, Clone)]
pub struct InputStreams<'a> {
    /// An optional [`GamepadButton`] [`Input`] stream
    pub gamepad_buttons: Option<&'a Input<GamepadButton>>,
    /// An optional [`GamepadButton`] [`Axis`] stream
    pub gamepad_button_axes: Option<&'a Axis<GamepadButton>>,
    /// An optional [`GamepadAxis`] [`Axis`] stream
    pub gamepad_axes: Option<&'a Axis<GamepadAxis>>,
    /// An optional [`KeyCode`] [`Input`] stream
    pub keyboard: Option<&'a Input<KeyCode>>,
    /// An optional [`MouseButton`] [`Input`] stream
    pub mouse: Option<&'a Input<MouseButton>>,
    /// The [`Gamepad`] that this struct will detect inputs from
    pub associated_gamepad: Option<Gamepad>,
}

// Constructors
impl<'a> InputStreams<'a> {
    /// Construct [`InputStreams`] with only a [`GamepadButton`] input stream
    pub fn from_gamepad(
        gamepad_button_stream: &'a Input<GamepadButton>,
        gamepad_button_axis_stream: &'a Axis<GamepadButton>,
        gamepad_axis_stream: &'a Axis<GamepadAxis>,
        associated_gamepad: Gamepad,
    ) -> Self {
        Self {
            gamepad_buttons: Some(gamepad_button_stream),
            gamepad_button_axes: Some(gamepad_button_axis_stream),
            gamepad_axes: Some(gamepad_axis_stream),
            keyboard: None,
            mouse: None,
            associated_gamepad: Some(associated_gamepad),
        }
    }

    /// Construct [`InputStreams`] with only a [`KeyCode`] input stream
    pub fn from_keyboard(keyboard_input_stream: &'a Input<KeyCode>) -> Self {
        Self {
            gamepad_buttons: None,
            gamepad_button_axes: None,
            gamepad_axes: None,
            keyboard: Some(keyboard_input_stream),
            mouse: None,
            associated_gamepad: None,
        }
    }

    /// Construct [`InputStreams`] with only a [`GamepadButton`] input stream
    pub fn from_mouse(mouse_input_stream: &'a Input<MouseButton>) -> Self {
        Self {
            gamepad_buttons: None,
            gamepad_button_axes: None,
            gamepad_axes: None,
            keyboard: None,
            mouse: Some(mouse_input_stream),
            associated_gamepad: None,
        }
    }
}

// Input checking
impl<'a> InputStreams<'a> {
    /// Is the `input` matched by the [`InputStreams`]?
    pub fn input_pressed(&self, input: &UserInput) -> bool {
        match input {
            UserInput::Single(button) => self.button_pressed(*button),
            UserInput::Chord(buttons) => self.all_buttons_pressed(buttons),
        }
    }

    /// Is at least one of the `inputs` pressed?
    #[must_use]
    pub fn any_pressed(&self, inputs: &PetitSet<UserInput, 16>) -> bool {
        for input in inputs.iter() {
            if self.input_pressed(input) {
                return true;
            }
        }
        // If none of the inputs matched, return false
        false
    }

    /// Is the `button` pressed?
    #[must_use]
    pub fn button_pressed(&self, button: InputKind) -> bool {
        match button {
            InputKind::GamepadAxis(axis) => {
                let value = self.get_input_value(&UserInput::Single(button));

                match axis.comparison {
                    GamepadAxisComparison::Less => value < axis.threshold,
                    GamepadAxisComparison::Greater => value > axis.threshold,
                    GamepadAxisComparison::Equal => {
                        (axis.threshold.abs() - value.abs()) > f32::EPSILON
                    }
                }
            }
            InputKind::GamepadButton(gamepad_button) => {
                // If no gamepad is registered, we know for sure that no match was found
                if let Some(gamepad) = self.associated_gamepad {
                    if let Some(gamepad_buttons) = self.gamepad_buttons {
                        gamepad_buttons.pressed(GamepadButton(gamepad, gamepad_button))
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            InputKind::Keyboard(keycode) => {
                if let Some(keyboard_stream) = self.keyboard {
                    keyboard_stream.pressed(keycode)
                } else {
                    false
                }
            }
            InputKind::Mouse(mouse_button) => {
                if let Some(mouse_stream) = self.mouse {
                    mouse_stream.pressed(mouse_button)
                } else {
                    false
                }
            }
        }
    }

    /// Are all of the `buttons` pressed?
    #[must_use]
    pub fn all_buttons_pressed(&self, buttons: &PetitSet<InputKind, 8>) -> bool {
        for &button in buttons.iter() {
            // If any of the appropriate inputs failed to match, the action is considered pressed
            if !self.button_pressed(button) {
                return false;
            }
        }
        // If none of the inputs failed to match, return true
        true
    }

    /// Get the "value" of the input.
    ///
    /// For binary inputs such as buttons, this will always be either `0.0` or `1.0`. For analog
    /// inputs such as axes, this will be the axis value.
    ///
    /// [`UserInput::Chord`] inputs are also considered binary and will return `0.0` or `1.0` based
    /// on whether the chord has been pressed.
    pub fn get_input_value(&self, input: &UserInput) -> f32 {
        let use_button_value = || {
            if self.input_pressed(input) {
                1.0
            } else {
                0.0
            }
        };

        if let Some(gamepad) = self.associated_gamepad {
            match input {
                UserInput::Single(InputKind::GamepadAxis(threshold)) => {
                    if let Some(axes) = self.gamepad_axes {
                        axes.get(GamepadAxis(gamepad, threshold.axis))
                            .unwrap_or_default()
                    } else {
                        0.0
                    }
                }
                UserInput::Single(InputKind::GamepadButton(button_type)) => {
                    if let Some(button_axes) = self.gamepad_button_axes {
                        button_axes
                            .get(GamepadButton(gamepad, *button_type))
                            .unwrap_or_default()
                    } else {
                        0.0
                    }
                }
                _ => use_button_value(),
            }

        // If there is no gamepad
        } else {
            match input {
                UserInput::Single(InputKind::GamepadAxis(_) | InputKind::GamepadButton(_)) => 0.0,
                _ => use_button_value(),
            }
        }
    }
}

/// A mutable collection of [`Input`] structs, which can be used for mocking user inputs.
///
/// Each of these streams is optional; if a stream does not exist, inputs sent to them will be ignored.
///
/// These are typically collected via a system from the [`World`](bevy::prelude::World) as resources.
#[derive(Debug)]
pub struct MutableInputStreams<'a> {
    /// An optional [`GamepadButton`] [`Input`] stream
    pub gamepad_buttons: Option<&'a mut Input<GamepadButton>>,
    /// An optional [`GamepadButton`] [`Axis`] stream
    pub gamepad_button_axes: Option<&'a mut Axis<GamepadButton>>,
    /// An optional [`GamepadAxis`] [`Axis`] stream
    pub gamepad_axes: Option<&'a mut Axis<GamepadAxis>>,
    /// An optional [`KeyCode`] [`Input`] stream
    pub keyboard: Option<&'a mut Input<KeyCode>>,
    /// An optional [`MouseButton`] [`Input`] stream
    pub mouse: Option<&'a mut Input<MouseButton>>,
    /// The [`Gamepad`] that this struct will detect inputs from
    pub associated_gamepad: Option<Gamepad>,
}

impl<'a> From<MutableInputStreams<'a>> for InputStreams<'a> {
    fn from(mutable_streams: MutableInputStreams<'a>) -> Self {
        let gamepad_buttons = mutable_streams
            .gamepad_buttons
            .map(|mutable_ref| &*mutable_ref);
        let gamepad_button_axes = mutable_streams
            .gamepad_button_axes
            .map(|mutable_ref| &*mutable_ref);
        let gamepad_axes = mutable_streams
            .gamepad_axes
            .map(|mutable_ref| &*mutable_ref);

        let keyboard = mutable_streams.keyboard.map(|mutable_ref| &*mutable_ref);

        let mouse = mutable_streams.mouse.map(|mutable_ref| &*mutable_ref);

        InputStreams {
            gamepad_buttons,
            gamepad_button_axes,
            gamepad_axes,
            keyboard,
            mouse,
            associated_gamepad: mutable_streams.associated_gamepad,
        }
    }
}
