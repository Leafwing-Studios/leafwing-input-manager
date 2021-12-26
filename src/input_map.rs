use crate::Actionlike;
use bevy::prelude::*;
use bevy::utils::HashSet;
use core::fmt::Debug;
use multimap::MultiMap;

/// Maps from raw inputs to an input-method agnostic representation
///
/// Multiple inputs can be mapped to the same action,
/// and each input can be mapped to multiple actions.
///
/// The provided input types must be one of [GamepadButtonType], [KeyCode] or [MouseButton].
#[derive(Component, Debug)]
pub struct InputMap<A: Actionlike> {
    pub map: MultiMap<A, UserInput>,
    associated_gamepad: Option<Gamepad>,
}

impl<A: Actionlike> InputMap<A> {
    /// Is at least one of the corresponding inputs for `action` found in the provided `input` stream?
    pub fn pressed(
        &self,
        action: A,
        gamepad_input_stream: &Input<GamepadButton>,
        keyboard_input_stream: &Input<KeyCode>,
        mouse_input_stream: &Input<MouseButton>,
    ) -> bool {
        if let Some(matching_inputs) = self.map.get_vec(&action) {
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
    pub fn any_pressed(
        &self,
        inputs: &Vec<UserInput>,
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
                UserInput::Combination(buttons) => self.all_buttons_pressed(
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
    pub fn all_buttons_pressed(
        &self,
        buttons: &HashSet<Button>,
        gamepad_input_stream: &Input<GamepadButton>,
        keyboard_input_stream: &Input<KeyCode>,
        mouse_input_stream: &Input<MouseButton>,
    ) -> bool {
        for &button in buttons {
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
    /// Existing mappings for that action will not be overwritten
    pub fn insert(&mut self, action: A, input: impl Into<UserInput>) {
        self.map.insert(action, input.into());
    }

    /// Insert a mapping between `action` and the combination of `buttons` provided
    ///
    /// Existing mappings for that action will not be overwritten.
    /// Any iterator of [Button] can be supplied, but will be converted into a [HashSet] for storage and use.
    pub fn combo(&mut self, action: A, buttons: impl IntoIterator<Item = Button>) {
        self.map.insert(action, UserInput::combo(buttons));
    }

    /// Clears all inputs registered for the `action`
    ///
    /// Returns all previously registered inputs, if any
    pub fn remove(&mut self, action: A) -> Option<Vec<UserInput>> {
        self.map.remove(&action)
    }

    /// Assigns a particular [Gamepad] to the entity controlled by this input map
    pub fn assign_gamepad(&mut self, gamepad: Gamepad) {
        self.associated_gamepad = Some(gamepad);
    }

    /// Fetches the [Gamepad] associated with the entity controlled by this entity map
    pub fn gamepad(&self) -> Option<Gamepad> {
        self.associated_gamepad
    }
}

impl<A: Actionlike> Default for InputMap<A> {
    fn default() -> Self {
        Self {
            map: MultiMap::default(),
            associated_gamepad: None,
        }
    }
}

/// Some combination of user input, which may cross [Input] boundaries
///
/// Suitable for use in an [InputMap]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserInput {
    Single(Button),
    Combination(HashSet<Button>),
}

impl UserInput {
    pub fn combo(buttons: impl IntoIterator<Item = Button>) -> Self {
        let mut set: HashSet<Button> = HashSet::default();
        for button in buttons {
            set.insert(button);
        }

        UserInput::Combination(set)
    }
}

/// A button-like input type
///
/// Commonly stored in the [UserInput] enum.
///
/// Unfortunately we cannot use a trait object here, as the types used by `Input`
/// require traits that are not object-safe.
///
/// Please contact the maintainers if you need support for another type!
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Button {
    Gamepad(GamepadButtonType),
    Keyboard(KeyCode),
    Mouse(MouseButton),
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
