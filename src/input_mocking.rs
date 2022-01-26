//! Helpful utilities for testing input management by sending mock input events

use crate::user_input::{InputButton, MutableInputStreams, UserInput};
use bevy::app::App;
use bevy::ecs::system::{ResMut, SystemState};
use bevy::ecs::world::World;
use bevy::input::{
    gamepad::{Gamepad, GamepadButton, Gamepads},
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
///
/// let world = World::new();
///
/// // Pay respects!
/// world.send_input(KeyCode::F);
/// ```
///
/// ```rust
/// /// use bevy::prelude::*;
/// use leafwing_input_manager::MockInput;
/// let app = App::new();
///
/// // Send inputs one at a time
/// let B_E_V_Y = [KeyCode::B, KeyCode::E, KeyCode::V, KeyCode::Y];
///
/// for letter in B_E_V_Y {
/// 	app.send_input(letter);
/// }
///
/// // Or use chords!
/// app.send_input(UserInput::chord(B_E_V_Y));
/// ```
pub trait MockInput {
    /// Send the specified `user_input` directly
    ///
    /// Gamepad input will be sent by the first registed controller found.
    /// If none are found, gamepad input will be silently skipped.
    fn send_user_input(&mut self, input: impl Into<UserInput>);

    /// Send the specified `user_input` directly, using the specified gamepad
    ///
    /// Provide the `Gamepad` identifier to control which gamepad you are emulating inputs from
    fn send_user_input_to_gamepad(&mut self, input: impl Into<UserInput>, gamepad: Option<Gamepad>);
}

impl<'a> MutableInputStreams<'a> {
    /// Send the specified `user_input` directly, using the specified gamepad
    ///
    /// Called by the methods of [`MockInput`].
    pub fn send_user_input_to_gamepad(
        &mut self,
        input: impl Into<UserInput>,
        gamepad: Option<Gamepad>,
    ) {
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
    fn send_user_input(&mut self, input: impl Into<UserInput>) {
        let gamepad = if let Some(gamepads) = self.get_resource::<Gamepads>() {
            gamepads.iter().next().copied()
        } else {
            None
        };

        self.send_user_input_to_gamepad(input, gamepad);
    }

    fn send_user_input_to_gamepad(
        &mut self,
        input: impl Into<UserInput>,
        gamepad: Option<Gamepad>,
    ) {
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

        mutable_input_streams.send_user_input_to_gamepad(input, gamepad);
    }
}

impl MockInput for App {
    fn send_user_input(&mut self, input: impl Into<UserInput>) {
        self.world.send_user_input(input);
    }

    fn send_user_input_to_gamepad(
        &mut self,
        input: impl Into<UserInput>,
        gamepad: Option<Gamepad>,
    ) {
        self.world.send_user_input_to_gamepad(input, gamepad);
    }
}
