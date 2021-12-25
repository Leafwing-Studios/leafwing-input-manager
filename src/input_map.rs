use crate::Actionlike;
use bevy::prelude::*;
use core::fmt::Debug;
use downcast_rs::Downcast;
use multimap::MultiMap;
use std::any::TypeId;
use std::hash::Hash;

/// Maps from raw inputs to an input-method agnostic representation
///
/// Multiple inputs of the same type can be mapped to the same action.
/// A seperate resource of this type will be required for each input method you wish to support.
///
/// In almost all cases, the `InputType` type parameter (e.g. `Keycode`) will be the same as the
/// `InputVariant` type parameter: gamepads are the only common exception.
#[derive(Component, Debug)]
pub struct InputMap<A: Actionlike> {
    mouse: MultiMap<A, MouseButton>,
    keyboard: MultiMap<A, KeyCode>,
    gamepad: MultiMap<A, GamepadButton>,
}

impl<A: Actionlike> InputMap<A> {
    /// Is at least one of the corresponding inputs for `action` found in the provided `input` stream?
    pub fn pressed_by<I: Inputlike>(&self, action: A, input_stream: &Input<I>) -> bool {
        // Mouse
        if TypeId::of::<I>() == TypeId::of::<MouseButton>() {
            let input_stream = input_stream
                .as_any()
                .downcast_ref::<Input<MouseButton>>()
                .unwrap();
            if let Some(inputs) = self.mouse.get_vec(&action) {
                for input in inputs {
                    if input_stream.pressed(*input) {
                        // If any of the matching inputs are pressed, return true
                        return true;
                    }
                }
                // If none of the matching inputs inputs are pressed, return false
                return false;
            } else {
                return false;
            }
        }

        // Keyboard
        if TypeId::of::<I>() == TypeId::of::<KeyCode>() {
            let input_stream = input_stream
                .as_any()
                .downcast_ref::<Input<KeyCode>>()
                .unwrap();
            if let Some(inputs) = self.keyboard.get_vec(&action) {
                for input in inputs {
                    if input_stream.pressed(*input) {
                        // If any of the matching inputs are pressed, return true
                        return true;
                    }
                }
                // If none of the matching inputs inputs are pressed, return false
                return false;
            } else {
                return false;
            }
        }

        // Gamepad
        if TypeId::of::<I>() == TypeId::of::<GamepadButton>() {
            let input_stream = input_stream
                .as_any()
                .downcast_ref::<Input<GamepadButton>>()
                .unwrap();
            if let Some(inputs) = self.gamepad.get_vec(&action) {
                for input in inputs {
                    if input_stream.pressed(*input) {
                        // If any of the matching inputs are pressed, return true
                        return true;
                    }
                }
                // If none of the matching inputs inputs are pressed, return false
                return false;
            } else {
                return false;
            }
        }

        // If an invalid type is provided, return false
        false
    }

    pub fn insert<I: Inputlike>(&mut self, action: A, input: I) {
        if TypeId::of::<I>() == TypeId::of::<MouseButton>() {
            let input = input.as_any().downcast_ref::<MouseButton>().unwrap();
            self.mouse.insert(action, *input);
        }

        if TypeId::of::<I>() == TypeId::of::<KeyCode>() {
            let input = input.as_any().downcast_ref::<KeyCode>().unwrap();
            self.keyboard.insert(action, *input);
        }

        if TypeId::of::<I>() == TypeId::of::<GamepadButton>() {
            let input = input.as_any().downcast_ref::<GamepadButton>().unwrap();
            self.gamepad.insert(action, *input);
        }
    }

    /// Extracts all input mappings of type I
    pub fn get_all<I: Inputlike>(&self) -> Self {
        todo!()
    }

    /// Sets all mappings of type I to the provide mapping set
    pub fn set_all<I: Inputlike>(&mut self, mapping: MultiMap<A, I>) {
        todo!()
    }

    /// Clears all input mappings of type I
    pub fn clear_all<I: Inputlike>(&mut self) {
        todo!()
    }
}

impl<A: Actionlike> Default for InputMap<A> {
    fn default() -> Self {
        Self {
            mouse: MultiMap::default(),
            keyboard: MultiMap::default(),
            gamepad: MultiMap::default(),
        }
    }
}

// BLOCKED: Replace with Bevy standard once https://github.com/bevyengine/bevy/pull/3419 is merged
/// A type that can be used as a button-like input
pub trait Inputlike: Send + Sync + Debug + Copy + Hash + Eq + 'static {}

impl Inputlike for KeyCode {}

impl Inputlike for MouseButton {}

impl Inputlike for GamepadButton {}
