use crate::InputActionEnum;
use bevy::prelude::*;
use core::{hash::Hash, marker::PhantomData};
use multimap::MultiMap;
/// Maps from raw inputs to an input-method agnostic representation
///
/// Multiple inputs of the same type can be mapped to the same action.
/// A seperate resource of this type will be required for each input method you wish to support.
///
/// In almost all cases, the `InputType` type parameter (e.g. `Keycode`) will be the same as the
/// `InputVariant` type parameter: gamepads are the only common exception.
#[derive(Component)]
pub struct InputMap<InputAction, InputType, InputVariant = InputType>
where
    InputAction: InputActionEnum,
    InputVariant: Copy + Hash + Eq,
{
    mmap: MultiMap<InputAction, InputVariant>,
    _phantom: PhantomData<InputType>,
}

impl<InputAction, InputType, InputVariant> Default
    for InputMap<InputAction, InputType, InputVariant>
where
    InputAction: InputActionEnum,
    InputVariant: Copy + Hash + Eq,
{
    fn default() -> Self {
        Self {
            mmap: MultiMap::default(),
            _phantom: PhantomData::default(),
        }
    }
}

impl<InputAction, InputType, InputVariant> InputMap<InputAction, InputType, InputVariant>
where
    InputAction: InputActionEnum,
    InputVariant: Copy + Hash + Eq,
{
    /// Maps a particular `input` to the provided `action`
    ///
    /// This is commonly used to configure new inputs.
    pub fn insert(&mut self, action: InputAction, input: InputVariant) {
        self.mmap.insert(action, input);
    }

    /// Removes an 'action' from the map, returning the vector of 'input' at the key if the key was previously in the map.
    ///
    /// This can be used to reset keybindings in a granular fashion.
    pub fn remove(&mut self, action: InputAction) {
        self.mmap.remove(&action);
    }
}

// This handles the simple case, where InputVariant == InputType
// See https://github.com/bevyengine/bevy/issues/3224 for why these aren't always the same
impl<InputAction, InputType> InputMap<InputAction, InputType>
where
    InputAction: InputActionEnum,
    InputType: Copy + Hash + Eq,
{
    pub fn pressed(&self, action: InputAction, input: &Input<InputType>) -> bool {
        let presses = self
            .mmap
            .get_vec(&action)
            .unwrap_or_else(|| panic!("No bindings found for {}", action));

        for press in presses {
            if input.pressed(*press) {
                return true;
            }
        }
        false
    }

    pub fn just_pressed(&self, action: InputAction, input: &Input<InputType>) -> bool {
        let presses = self
            .mmap
            .get_vec(&action)
            .unwrap_or_else(|| panic!("No bindings found for {}", action));

        for press in presses {
            if input.just_pressed(*press) {
                return true;
            }
        }
        false
    }
}

// Special-cased impl required due to https://github.com/bevyengine/bevy/issues/3224
impl<InputAction: InputActionEnum> InputMap<InputAction, GamepadButton, GamepadButtonType> {
    pub fn pressed(
        &self,
        action: InputAction,
        input: &Input<GamepadButton>,
        gamepad: Gamepad,
    ) -> bool {
        let button_types = self
            .mmap
            .get_vec(&action)
            .unwrap_or_else(|| panic!("No bindings found for {}", action));

        for &button_type in button_types {
            let gamepad_button = GamepadButton(gamepad, button_type);

            if input.pressed(gamepad_button) {
                return true;
            }
        }
        false
    }

    pub fn just_pressed(
        &self,
        action: InputAction,
        input: &Input<GamepadButton>,
        gamepad: Gamepad,
    ) -> bool {
        let button_types = self
            .mmap
            .get_vec(&action)
            .unwrap_or_else(|| panic!("No bindings found for {}", action));

        for &button_type in button_types {
            let gamepad_button = GamepadButton(gamepad, button_type);

            if input.just_pressed(gamepad_button) {
                return true;
            }
        }
        false
    }
}
