use crate::Actionlike;
use bevy::prelude::*;
use core::fmt::Debug;
use multimap::MultiMap;

/// Maps from raw inputs to an input-method agnostic representation
///
/// Multiple inputs of the same type can be mapped to the same action.
/// A seperate resource of this type will be required for each input method you wish to support.
///
/// In almost all cases, the `InputType` type parameter (e.g. `Keycode`) will be the same as the
/// `InputVariant` type parameter: gamepads are the only common exception.
#[derive(Component, Debug)]
pub struct InputMap<A: Actionlike> {
    pub map: MultiMap<A, Buttonlike>,
    pub associated_gamepad: Option<Gamepad>,
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
            for &input in matching_inputs {
                // Check the corresponding input stream
                let matches = match input {
                    Buttonlike::Gamepad(gamepad_button) => {
                        // If no gamepad is registered, we know for sure that no match was found
                        if let Some(gamepad) = self.associated_gamepad {
                            gamepad_input_stream.pressed(GamepadButton(gamepad, gamepad_button))
                        } else {
                            false
                        }
                    }
                    Buttonlike::Keyboard(keycode) => keyboard_input_stream.pressed(keycode),
                    Buttonlike::Mouse(mouse_button) => mouse_input_stream.pressed(mouse_button),
                };

                // If any of the appropriate inputs match, the action is considered pressed
                if matches {
                    return true;
                }
            }
            // If none of the inputs matched, return false
            false
        } else {
            // No matches can be found if no inputs are registred for that action
            false
        }
    }

    pub fn insert(&mut self, action: A, input: impl Into<Buttonlike>) {
        self.map.insert(action, input.into());
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

/// A button-like input type
///
/// Unfortunately we cannot use a trait object here, as the types used by `Input`
/// require traits that are not object-safe.
///
/// Please contact the maintainers if you need support for another type!
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Buttonlike {
    Gamepad(GamepadButtonType),
    Keyboard(KeyCode),
    Mouse(MouseButton),
}

impl From<GamepadButtonType> for Buttonlike {
    fn from(input: GamepadButtonType) -> Self {
        Buttonlike::Gamepad(input)
    }
}

impl From<KeyCode> for Buttonlike {
    fn from(input: KeyCode) -> Self {
        Buttonlike::Keyboard(input)
    }
}

impl From<MouseButton> for Buttonlike {
    fn from(input: MouseButton) -> Self {
        Buttonlike::Mouse(input)
    }
}
