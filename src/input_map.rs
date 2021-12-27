//! This module contains [InputMap] and its supporting methods and impls.

use crate::smallset::SmallSet;
use crate::Actionlike;
use bevy::prelude::*;
use bevy::utils::HashMap;
use core::fmt::Debug;

/// Maps from raw inputs to an input-method agnostic representation
///
/// Multiple inputs can be mapped to the same action,
/// and each input can be mapped to multiple actions.
///
/// The provided input types must be one of [GamepadButtonType], [KeyCode] or [MouseButton].
///
/// A maximum of 32(!) bindings can be registered to each action.
/// If there is any chance of hitting this limit,
/// check the current number registered using the [n_registered](Self::n_registered) method.
///
/// # Example
/// ```rust
/// use bevy::prelude::*;
/// use leafwing_input_manager::prelude::*;
/// use leafwing_input_manager::input_map::Button;
/// use strum_macros::EnumIter;
///
/// // You can Run!
/// #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
/// enum Action {
///     Run,
///     Hide,
/// }
///
/// let mut input_map: InputMap<Action> = InputMap::default();
///
/// // Basic insertion
/// input_map.insert(Action::Run, GamepadButtonType::South);
/// input_map.insert(Action::Run, MouseButton::Left);
/// input_map.insert(Action::Run, KeyCode::LShift);
/// input_map.insert_multiple(Action::Hide, [GamepadButtonType::LeftTrigger, GamepadButtonType::RightTrigger]);
///
/// // Combinations!
/// input_map.insert_chord(Action::Run, [KeyCode::LControl, KeyCode::R]);
/// input_map.insert_chord(Action::Hide, [Button::Keyboard(KeyCode::H), Button::Gamepad(GamepadButtonType::South), Button::Mouse(MouseButton::Middle)]);
///
/// // But you can't Hide :(
/// input_map.clear_action(Action::Hide);
///```
#[derive(Component, Debug)]
pub struct InputMap<A: Actionlike> {
    /// The raw [HashMap] [SmallSet] used to store the input mapping
    pub map: HashMap<A, SmallSet<UserInput, 32>>,
    associated_gamepad: Option<Gamepad>,
}

impl<A: Actionlike> InputMap<A> {
    /// Is at least one of the corresponding inputs for `action` found in the provided `input` stream?
    #[must_use]
    pub fn pressed(
        &self,
        action: A,
        gamepad_input_stream: &Input<GamepadButton>,
        keyboard_input_stream: &Input<KeyCode>,
        mouse_input_stream: &Input<MouseButton>,
    ) -> bool {
        if let Some(matching_inputs) = self.map.get(&action) {
            self.any_pressed(
                matching_inputs,
                gamepad_input_stream,
                keyboard_input_stream,
                mouse_input_stream,
            )
        } else {
            // No matches can be found if no inputs are registred for that action
            false
        }
    }

    /// Is at least one of the `inputs` pressed?
    #[must_use]
    pub fn any_pressed(
        &self,
        inputs: &SmallSet<UserInput, 32>,
        gamepad_input_stream: &Input<GamepadButton>,
        keyboard_input_stream: &Input<KeyCode>,
        mouse_input_stream: &Input<MouseButton>,
    ) -> bool {
        for input in inputs.clone() {
            if match input {
                UserInput::Single(button) => self.button_pressed(
                    button,
                    gamepad_input_stream,
                    keyboard_input_stream,
                    mouse_input_stream,
                ),
                UserInput::Chord(buttons) => self.all_buttons_pressed(
                    &buttons,
                    gamepad_input_stream,
                    keyboard_input_stream,
                    mouse_input_stream,
                ),
            } {
                // If any of the appropriate inputs match, the action is considered pressed
                return true;
            }
        }
        // If none of the inputs matched, return false
        false
    }

    /// Is the `button` pressed?
    #[must_use]
    pub fn button_pressed(
        &self,
        button: Button,
        gamepad_input_stream: &Input<GamepadButton>,
        keyboard_input_stream: &Input<KeyCode>,
        mouse_input_stream: &Input<MouseButton>,
    ) -> bool {
        match button {
            Button::Gamepad(gamepad_button) => {
                // If no gamepad is registered, we know for sure that no match was found
                if let Some(gamepad) = self.associated_gamepad {
                    gamepad_input_stream.pressed(GamepadButton(gamepad, gamepad_button))
                } else {
                    false
                }
            }
            Button::Keyboard(keycode) => keyboard_input_stream.pressed(keycode),
            Button::Mouse(mouse_button) => mouse_input_stream.pressed(mouse_button),
        }
    }

    /// Are all of the `buttons` pressed?
    #[must_use]
    pub fn all_buttons_pressed(
        &self,
        buttons: &SmallSet<Button, 8>,
        gamepad_input_stream: &Input<GamepadButton>,
        keyboard_input_stream: &Input<KeyCode>,
        mouse_input_stream: &Input<MouseButton>,
    ) -> bool {
        for button in buttons.clone() {
            // If any of the appropriate inputs failed to match, the action is considered pressed
            if !self.button_pressed(
                button,
                gamepad_input_stream,
                keyboard_input_stream,
                mouse_input_stream,
            ) {
                return false;
            }
        }
        // If none of the inputs failed to match, return true
        true
    }

    /// Insert a mapping between `action` and `input`
    ///
    /// Existing mappings for that action will not be overwritten.
    pub fn insert(&mut self, action: A, input: impl Into<UserInput>) {
        if let Some(existing_set) = self.map.get_mut(&action) {
            existing_set.insert(input.into());
        } else {
            let mut new_set = SmallSet::new();
            new_set.insert(input.into());
            self.map.insert(action, new_set);
        }
    }

    /// Insert a mapping between `action` and the provided `inputs`
    ///
    /// This method creates multiple distinct bindings.
    /// If you want to require multiple buttons to be pressed at once, use [insert_chord](Self::insert_chord).
    /// Any iterator that can be converted into a [UserInput] can be supplied.
    ///
    /// Existing mappings for that action will not be overwritten.
    pub fn insert_multiple(
        &mut self,
        action: A,
        inputs: impl IntoIterator<Item = impl Into<UserInput>>,
    ) {
        for input in inputs {
            self.insert(action, input);
        }
    }

    /// Insert a mapping between `action` and the simultaneous combination of `buttons` provided
    ///
    /// Any iterator that can be converted into a [Button] can be supplied, but will be converted into a [HashSet] for storage and use.
    /// Chords can also be added with the [insert](Self::insert) method, if the [UserInput::Chord] variant is constructed explicitly.
    ///
    /// Existing mappings for that action will not be overwritten.
    pub fn insert_chord(
        &mut self,
        action: A,
        buttons: impl IntoIterator<Item = impl Into<Button>>,
    ) {
        self.insert(action, UserInput::combo(buttons));
    }

    /// Merges two [InputMap]s, adding both of their bindings to the resulting [InputMap]
    ///
    /// If the associated gamepads do not match, the resulting associated gamepad will be set to `None`.
    #[must_use]
    pub fn merge(&self, other: &InputMap<A>) -> InputMap<A> {
        let associated_gamepad = if self.associated_gamepad == other.associated_gamepad {
            self.associated_gamepad
        } else {
            None
        };

        let mut new_map = InputMap {
            map: HashMap::default(),
            associated_gamepad,
        };

        for action in A::iter() {
            if let Some(self_bindings) = self.get(action, None) {
                new_map.insert_multiple(action, self_bindings);
            }

            if let Some(other_bindings) = other.get(action, None) {
                new_map.insert_multiple(action, other_bindings);
            }
        }

        new_map
    }

    /// Returns how many bindings are currently registered for the provided action
    ///
    /// A maximum of 32 bindings across all input modes can be stored for each action,
    /// and insert operations will panic if used when 32 bindings already exist.
    #[must_use]
    pub fn n_registered(&self, action: A) -> usize {
        if let Some(set) = self.get(action, None) {
            set.len()
        } else {
            0
        }
    }

    /// Returns the mapping between the `action` that uses the supplied `input_mode`
    ///
    /// If `input_mode` is `None`, all inputs will be returned regardless of input mode.
    ///
    /// For chords, an input will be returned if any of the contained buttons use that input mode.
    ///
    /// A copy of the values are returned, rather than a reference to them.
    /// Use `self.map.get` or `self.map.get_mut` if you require a reference.
    #[must_use]
    pub fn get(&self, action: A, input_mode: Option<InputMode>) -> Option<SmallSet<UserInput, 32>> {
        if let Some(full_set) = self.map.get(&action) {
            if let Some(input_mode) = input_mode {
                let mut matching_set = SmallSet::new();
                for input in full_set.clone() {
                    if input.matches_input_mode(input_mode) {
                        matching_set.insert(input.clone());
                    }
                }

                if matching_set.is_empty() {
                    None
                } else {
                    Some(matching_set)
                }
            } else {
                Some(full_set.clone())
            }
        } else {
            None
        }
    }

    /// Clears all inputs registered for the `action` that use the supplied `input_mode`
    ///
    /// If `input_mode` is `None`, all inputs will be cleared regardless of input mode.
    ///
    /// For chords, an input will be removed if any of the contained buttons use that input mode.
    ///
    /// Returns all previously registered inputs, if any
    pub fn clear_action(
        &mut self,
        action: A,
        input_mode: Option<InputMode>,
    ) -> Option<SmallSet<UserInput, 32>> {
        if let Some(input_mode) = input_mode {
            // Pull out all the matching inputs
            if let Some(full_set) = self.map.remove(&action) {
                let mut retained_set: SmallSet<UserInput, 32> = SmallSet::new();
                let mut removed_set: SmallSet<UserInput, 32> = SmallSet::new();

                for input in full_set {
                    if input.matches_input_mode(input_mode) {
                        removed_set.insert(input);
                    } else {
                        retained_set.insert(input);
                    }
                }

                // Put back the ones that didn't match
                self.insert_multiple(action, retained_set);

                // Return the items that matched
                if removed_set.is_empty() {
                    None
                } else {
                    Some(removed_set)
                }
            } else {
                None
            }
        } else {
            self.map.remove(&action)
        }
    }

    /// Clears all inputs that use the supplied `input_mode`
    ///
    /// If `input_mode` is `None`, all inputs will be cleared regardless of input mode.
    ///
    /// For chords, an input will be removed if any of the contained buttons use that input mode.
    ///
    /// Returns the subset of the action map that was removed
    #[allow(clippy::return_self_not_must_use)]
    pub fn clear_input_mode(&mut self, input_mode: Option<InputMode>) -> InputMap<A> {
        let mut cleared_input_map = InputMap {
            map: HashMap::default(),
            associated_gamepad: self.associated_gamepad,
        };

        for action in A::iter() {
            if let Some(removed_inputs) = self.clear_action(action, input_mode) {
                cleared_input_map.insert_multiple(action, removed_inputs);
            }
        }

        cleared_input_map
    }

    /// Assigns a particular [Gamepad] to the entity controlled by this input map
    pub fn assign_gamepad(&mut self, gamepad: Gamepad) {
        self.associated_gamepad = Some(gamepad);
    }

    /// Clears any [Gamepad] associated with the entity controlled by this input map
    pub fn clear_gamepad(&mut self) {
        self.associated_gamepad = None;
    }

    /// Fetches the [Gamepad] associated with the entity controlled by this entity map
    #[must_use]
    pub fn gamepad(&self) -> Option<Gamepad> {
        self.associated_gamepad
    }
}

impl<A: Actionlike> Default for InputMap<A> {
    fn default() -> Self {
        Self {
            map: HashMap::default(),
            associated_gamepad: None,
        }
    }
}

/// Some combination of user input, which may cross [Input] boundaries
///
/// Suitable for use in an [InputMap]
#[derive(Debug, Clone, PartialEq)]
pub enum UserInput {
    /// A single button
    Single(Button),
    /// A combination of buttons, pressed simultaneously
    /// Up to 8 (!!) buttons can be chorded together at once
    Chord(SmallSet<Button, 8>),
}

impl UserInput {
    /// Creates a [UserInput::Chord] from an iterator of [Button]s
    pub fn combo(buttons: impl IntoIterator<Item = impl Into<Button>>) -> Self {
        let mut set: SmallSet<Button, 8> = SmallSet::new();
        for button in buttons {
            set.insert(button.into());
        }

        UserInput::Chord(set)
    }

    /// Does this [UserInput] match the provided [InputMode]?
    ///
    /// For [UserInput::Chord], this will be true if any of the buttons in the combination match.
    pub fn matches_input_mode(&self, input_mode: InputMode) -> bool {
        match self {
            UserInput::Single(button) => {
                let button_mode: InputMode = (*button).into();
                button_mode == input_mode
            }
            UserInput::Chord(set) => {
                for button in set.clone() {
                    let button_mode: InputMode = button.into();
                    if button_mode == input_mode {
                        return true;
                    }
                }
                false
            }
        }
    }
}

impl From<GamepadButtonType> for UserInput {
    fn from(input: GamepadButtonType) -> Self {
        UserInput::Single(Button::Gamepad(input))
    }
}

impl From<KeyCode> for UserInput {
    fn from(input: KeyCode) -> Self {
        UserInput::Single(Button::Keyboard(input))
    }
}

impl From<MouseButton> for UserInput {
    fn from(input: MouseButton) -> Self {
        UserInput::Single(Button::Mouse(input))
    }
}

/// A button-like input type
///
/// See [Button] for the value-ful equivalent.
/// Use the [From] or [Into] traits to convert from a [Button] to a [InputMode].
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

impl From<Button> for InputMode {
    fn from(button: Button) -> Self {
        match button {
            Button::Gamepad(_) => InputMode::Gamepad,
            Button::Keyboard(_) => InputMode::Keyboard,
            Button::Mouse(_) => InputMode::Mouse,
        }
    }
}

/// The values of a button-like input type
///
/// See [InputMode] for the value-less equivalent. Commonly stored in the [UserInput] enum.
///
/// Unfortunately we cannot use a trait object here, as the types used by `Input`
/// require traits that are not object-safe.
///
/// Please contact the maintainers if you need support for another type!
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Button {
    /// A button on a gamepad
    Gamepad(GamepadButtonType),
    /// A button on a keyboard
    Keyboard(KeyCode),
    /// A button on a mouse
    Mouse(MouseButton),
}

impl From<GamepadButtonType> for Button {
    fn from(input: GamepadButtonType) -> Self {
        Button::Gamepad(input)
    }
}

impl From<KeyCode> for Button {
    fn from(input: KeyCode) -> Self {
        Button::Keyboard(input)
    }
}

impl From<MouseButton> for Button {
    fn from(input: MouseButton) -> Self {
        Button::Mouse(input)
    }
}
