//! Helpful utilities for testing input management by sending mock input events

use crate::user_input::{InputButton, MutableInputStreams, UserInput};
use bevy::app::App;
use bevy::ecs::component::Component;
use bevy::ecs::query::With;
use bevy::ecs::system::{Query, ResMut, SystemState};
use bevy::ecs::world::World;
use bevy::input::{
    gamepad::{Gamepad, GamepadButton, Gamepads},
    keyboard::KeyCode,
    mouse::MouseButton,
    Input,
};
use bevy::ui::Interaction;

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
/// let mut world = World::new();
///
/// // Pay respects!
/// world.send_input(KeyCode::F);
/// ```
///
/// ```rust
/// use bevy::prelude::*;
/// use leafwing_input_manager::{MockInput, user_input::UserInput};
///
/// let mut app = App::new();
///
/// // Send inputs one at a time
/// let B_E_V_Y = [KeyCode::B, KeyCode::E, KeyCode::V, KeyCode::Y];
///
/// for letter in B_E_V_Y {
///     app.send_input(letter);
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
    fn send_input(&mut self, input: impl Into<UserInput>);

    /// Send the specified `user_input` directly, using the specified gamepad
    ///
    /// Provide the `Gamepad` identifier to control which gamepad you are emulating inputs from
    fn send_input_to_gamepad(&mut self, input: impl Into<UserInput>, gamepad: Option<Gamepad>);

    /// Clears all user input streams, resetting them to their default state
    ///
    /// All buttons are released, and `just_pressed` and `just_released` information on the [`Input`] type are lost.
    /// `just_pressed` and `just_released` on the [`ActionState`](crate::action_state::ActionState) will be kept.
    ///
    /// This will clear all [`KeyCode`], [`GamepadButton`] and [`MouseButton`] input streams,
    /// as well as any [`Interaction`] components
    fn reset_inputs(&mut self);

    /// Presses all `bevy_ui` buttons with the matching `Marker` component
    ///
    /// Changes their [`Interaction`] component to [`Interaction::Clicked`]
    fn press_button<Marker: Component>(&mut self);
}

impl<'a> MutableInputStreams<'a> {
    /// Send the specified `user_input` directly, using the specified gamepad
    ///
    /// Called by the methods of [`MockInput`].
    pub fn send_user_input(&mut self, input: impl Into<UserInput>) {
        let input_to_send: UserInput = input.into();

        let mut gamepad_buttons: Vec<GamepadButton> = Vec::default();

        let mut keyboard_buttons: Vec<KeyCode> = Vec::default();
        let mut mouse_buttons: Vec<MouseButton> = Vec::default();

        match input_to_send {
            UserInput::Null => (),
            UserInput::Single(button) => match button {
                InputButton::Gamepad(gamepad_buttontype) => {
                    if let Some(gamepad) = self.associated_gamepad {
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
                            if let Some(gamepad) = self.associated_gamepad {
                                gamepad_buttons.push(GamepadButton(gamepad, gamepad_buttontype));
                            }
                        }
                        InputButton::Keyboard(keycode) => keyboard_buttons.push(keycode),
                        InputButton::Mouse(mouse_button) => mouse_buttons.push(mouse_button),
                    }
                }
            }
        };

        if let Some(ref mut gamepad_input) = self.gamepad {
            for button in gamepad_buttons {
                gamepad_input.press(button);
            }
        }

        if let Some(ref mut keyboard_input) = self.keyboard {
            for button in keyboard_buttons {
                keyboard_input.press(button);
            }
        }

        if let Some(ref mut mouse_input) = self.mouse {
            for button in mouse_buttons {
                mouse_input.press(button);
            }
        }
    }
}

impl MockInput for World {
    fn send_input(&mut self, input: impl Into<UserInput>) {
        let gamepad = if let Some(gamepads) = self.get_resource::<Gamepads>() {
            gamepads.iter().next().copied()
        } else {
            None
        };

        self.send_input_to_gamepad(input, gamepad);
    }

    fn send_input_to_gamepad(&mut self, input: impl Into<UserInput>, gamepad: Option<Gamepad>) {
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
            associated_gamepad: gamepad,
        };

        mutable_input_streams.send_user_input(input);
    }

    fn reset_inputs(&mut self) {
        let mut input_system_state: SystemState<(
            Query<&mut Interaction>,
            Option<ResMut<Input<GamepadButton>>>,
            Option<ResMut<Input<KeyCode>>>,
            Option<ResMut<Input<MouseButton>>>,
        )> = SystemState::new(self);

        let (mut interaction_query, maybe_gamepad, maybe_keyboard, maybe_mouse) =
            input_system_state.get_mut(self);

        for mut interaction in interaction_query.iter_mut() {
            *interaction = Interaction::None;
        }

        if let Some(mut gamepad) = maybe_gamepad {
            *gamepad = Default::default();
        }

        if let Some(mut keyboard) = maybe_keyboard {
            *keyboard = Default::default();
        }

        if let Some(mut mouse) = maybe_mouse {
            *mouse = Default::default();
        }
    }

    fn press_button<Marker: Component>(&mut self) {
        let mut button_query = self.query_filtered::<&mut Interaction, With<Marker>>();

        for mut interaction in button_query.iter_mut(self) {
            *interaction = Interaction::Clicked;
        }
    }
}

impl MockInput for App {
    fn send_input(&mut self, input: impl Into<UserInput>) {
        self.world.send_input(input);
    }

    fn send_input_to_gamepad(&mut self, input: impl Into<UserInput>, gamepad: Option<Gamepad>) {
        self.world.send_input_to_gamepad(input, gamepad);
    }

    fn reset_inputs(&mut self) {
        self.world.reset_inputs();
    }

    fn press_button<Marker: Component>(&mut self) {
        self.world.press_button::<Marker>();
    }
}
