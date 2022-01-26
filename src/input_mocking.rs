//! Helpful utilities for testing input management by sending mock input events

use crate::user_input::{InputButton, MutableInputStreams, UserInput};
use bevy::app::App;
use bevy::ecs::system::{ResMut, SystemState};
use bevy::ecs::world::World;
use bevy::input::{
    gamepad::{Gamepad, GamepadButton},
    keyboard::KeyCode,
    mouse::MouseButton,
    Input,
};

/// Send fake input events for testing purposes
///
/// In game code, you should (almost) always be setting the [`ActionState`](crate::action_state::ActionState)
/// directly instead.
///
/// # Examples
/// ```rust
/// use bevy::prelude::*;
/// use leafwing_input_manager::MockInput;
/// let world = World::new();
///
///
/// ```
///
/// ```rust
///
///
/// ```
pub trait MockInput {
    /// Send the specified `user_input` directly
    fn send_user_input(&mut self, input: impl Into<UserInput>, gamepad: Option<Gamepad>);
}

impl<'a> MockInput for MutableInputStreams<'a> {
    fn send_user_input(&mut self, input: impl Into<UserInput>, gamepad: Option<Gamepad>) {
        let input_to_send: UserInput = input.into();

        let mut gamepad_buttons: Vec<GamepadButton> = Vec::default();

        let mut keyboard_buttons: Vec<KeyCode> = Vec::default();
        let mut mouse_buttons: Vec<MouseButton> = Vec::default();

        match input_to_send {
            UserInput::Null => (),
            UserInput::Single(button) => match button {
                InputButton::Gamepad(gamepad_buttontype) => {
                    if let Some(gamepad) = gamepad {
                        gamepad_buttons.push(GamepadButton(gamepad, gamepad_buttontype));
                    }
                }
                InputButton::Keyboard(keycode) => keyboard_buttons.push(keycode),
                InputButton::Mouse(mouse_button) => mouse_buttons.push(mouse_button),
            },
            UserInput::Chord(button_set) => {
                for button in button_set {
                    match button {
                        InputButton::Gamepad(gamepad_buttontype) => {
                            if let Some(gamepad) = gamepad {
                                gamepad_buttons.push(GamepadButton(gamepad, gamepad_buttontype));
                            }
                        }
                        InputButton::Keyboard(keycode) => keyboard_buttons.push(keycode),
                        InputButton::Mouse(mouse_button) => mouse_buttons.push(mouse_button),
                    }
                }
            }
        };

        if let Some(gamepad_input) = self.gamepad.as_deref_mut() {
            for button in gamepad_buttons {
                gamepad_input.press(button);
            }
        }

        if let Some(keyboard_input) = self.keyboard.as_deref_mut() {
            for button in keyboard_buttons {
                keyboard_input.press(button);
            }
        }

        if let Some(mouse_input) = self.mouse.as_deref_mut() {
            for button in mouse_buttons {
                mouse_input.press(button);
            }
        }
    }
}

impl MockInput for World {
    fn send_user_input(&mut self, input: impl Into<UserInput>, gamepad: Option<Gamepad>) {
        // You can make a system with this type signature if you'd like to mock user input
        // in a non-exclusive system
        let mut input_system_state: SystemState<(
            Option<ResMut<Input<GamepadButton>>>,
            Option<ResMut<Input<KeyCode>>>,
            Option<ResMut<Input<MouseButton>>>,
        )> = SystemState::new(self);

        let (mut maybe_gamepad, mut maybe_keyboard, mut maybe_mouse) =
            input_system_state.get_mut(self);

        let mut mutable_input_streams = MutableInputStreams {
            gamepad: maybe_gamepad.as_deref_mut(),
            keyboard: maybe_keyboard.as_deref_mut(),
            mouse: maybe_mouse.as_deref_mut(),
        };

        mutable_input_streams.send_user_input(input, gamepad);
    }
}

impl MockInput for App {
    fn send_user_input(&mut self, input: impl Into<UserInput>, gamepad: Option<Gamepad>) {
        self.world.send_user_input(input, gamepad);
    }
}
