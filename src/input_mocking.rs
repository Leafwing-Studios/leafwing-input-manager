//! Helpful utilities for testing input management by sending mock input events

use crate::user_input::{InputStreams, MutableInputStreams, UserInput};
use bevy_app::App;
use bevy_ecs::event::Events;
use bevy_ecs::system::{Res, ResMut, SystemState};
use bevy_ecs::world::World;
#[cfg(feature = "ui")]
use bevy_ecs::{component::Component, query::With, system::Query};
use bevy_input::{
    gamepad::{Gamepad, GamepadButton, GamepadEvent, Gamepads},
    keyboard::{KeyCode, KeyboardInput},
    mouse::{MouseButton, MouseButtonInput, MouseWheel},
    touch::{TouchInput, Touches},
    Input,
};
#[cfg(feature = "ui")]
use bevy_ui::Interaction;
use bevy_window::CursorMoved;

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
    /// Note that inputs will continue to be pressed until explicitly released or [`MockInput::reset_inputs`] is called.
    ///
    /// Gamepad input will be sent by the first registed controller found.
    /// If none are found, gamepad input will be silently skipped.
    fn send_input(&mut self, input: impl Into<UserInput>);

    /// Send the specified `user_input` directly, using the specified gamepad
    ///
    /// Note that inputs will continue to be pressed until explicitly released or [`MockInput::reset_inputs`] is called.
    ///
    /// Provide the [`Gamepad`] identifier to control which gamepad you are emulating inputs from
    fn send_input_to_gamepad(&mut self, input: impl Into<UserInput>, gamepad: Option<Gamepad>);

    /// Releases the specified `user_input` directly
    ///
    /// Gamepad input will be released by the first registed controller found.
    /// If none are found, gamepad input will be silently skipped.
    fn release_input(&mut self, input: impl Into<UserInput>);

    /// Releases the specified `user_input` directly, using the specified gamepad
    ///
    /// Provide the [`Gamepad`] identifier to control which gamepad you are emulating inputs from
    fn release_input_for_gamepad(&mut self, input: impl Into<UserInput>, gamepad: Option<Gamepad>);

    /// Is the provided `user_input` pressed?
    ///
    /// This method is intended as a convenience for testing; check the [`Input`] resource directly,
    /// or use an [`InputMap`](crate::input_map::InputMap) in real code.
    fn pressed(&mut self, input: impl Into<UserInput>) -> bool;

    /// Is the provided `user_input` pressed for the provided [`Gamepad`]?
    ///
    /// This method is intended as a convenience for testing; check the [`Input`] resource directly,
    /// or use an [`InputMap`](crate::input_map::InputMap) in real code.
    fn pressed_for_gamepad(
        &mut self,
        input: impl Into<UserInput>,
        gamepad: Option<Gamepad>,
    ) -> bool;

    /// Clears all user input streams, resetting them to their default state
    ///
    /// All buttons are released, and `just_pressed` and `just_released` information on the [`Input`] type are lost.
    /// `just_pressed` and `just_released` on the [`ActionState`](crate::action_state::ActionState) will be kept.
    ///
    /// This will clear all [`KeyCode`], [`GamepadButton`] and [`MouseButton`] input streams,
    /// as well as any [`Interaction`] components and all input [`Events`].
    fn reset_inputs(&mut self);

    /// Presses all `bevy_ui` buttons with the matching `Marker` component
    ///
    /// Changes their [`Interaction`] component to [`Interaction::Clicked`]
    #[cfg(feature = "ui")]
    fn click_button<Marker: Component>(&mut self);

    /// Hovers over all `bevy_ui` buttons with the matching `Marker` component
    ///
    /// Changes their [`Interaction`] component to [`Interaction::Clicked`]
    #[cfg(feature = "ui")]
    fn hover_button<Marker: Component>(&mut self);
}

impl<'a> MutableInputStreams<'a> {
    /// Send the specified `user_input` directly, using the specified gamepad
    ///
    /// Called by the methods of [`MockInput`].
    pub fn send_user_input(&mut self, input: impl Into<UserInput>) {
        let input_to_send: UserInput = input.into();
        let (gamepad_buttons, keyboard_buttons, mouse_buttons) = input_to_send.raw_inputs();

        if let Some(ref mut gamepad_input) = self.gamepad {
            for button in gamepad_buttons {
                if let Some(associated_gamepad) = self.associated_gamepad {
                    let gamepad_button = GamepadButton{gamepad: associated_gamepad, button_type: button};
                    gamepad_input.press(gamepad_button);
                }
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

    /// Releases the specified `user_input` directly, using the specified gamepad
    ///
    /// Called by the methods of [`MockInput`].
    pub fn release_user_input(&mut self, input: impl Into<UserInput>) {
        let input_to_release: UserInput = input.into();
        let (gamepad_buttons, keyboard_buttons, mouse_buttons) = input_to_release.raw_inputs();

        if let Some(ref mut gamepad_input) = self.gamepad {
            for button in gamepad_buttons {
                if let Some(associated_gamepad) = self.associated_gamepad {
                    let gamepad_button = GamepadButton{gamepad: associated_gamepad, button_type: button};
                    gamepad_input.release(gamepad_button);
                }
            }
        }

        if let Some(ref mut keyboard_input) = self.keyboard {
            for button in keyboard_buttons {
                keyboard_input.release(button);
            }
        }

        if let Some(ref mut mouse_input) = self.mouse {
            for button in mouse_buttons {
                mouse_input.release(button);
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

    fn release_input(&mut self, input: impl Into<UserInput>) {
        let gamepad = if let Some(gamepads) = self.get_resource::<Gamepads>() {
            gamepads.iter().next().copied()
        } else {
            None
        };

        self.release_input_for_gamepad(input, gamepad);
    }

    fn release_input_for_gamepad(&mut self, input: impl Into<UserInput>, gamepad: Option<Gamepad>) {
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

        mutable_input_streams.release_user_input(input);
    }

    fn pressed(&mut self, input: impl Into<UserInput>) -> bool {
        let gamepad = if let Some(gamepads) = self.get_resource::<Gamepads>() {
            gamepads.iter().next().copied()
        } else {
            None
        };

        self.pressed_for_gamepad(input, gamepad)
    }

    fn pressed_for_gamepad(
        &mut self,
        input: impl Into<UserInput>,
        gamepad: Option<Gamepad>,
    ) -> bool {
        let mut input_system_state: SystemState<(
            Option<Res<Input<GamepadButton>>>,
            Option<Res<Input<KeyCode>>>,
            Option<Res<Input<MouseButton>>>,
        )> = SystemState::new(self);

        let (maybe_gamepad, maybe_keyboard, maybe_mouse) = input_system_state.get(self);

        let input_streams = InputStreams {
            gamepad: maybe_gamepad.as_deref(),
            keyboard: maybe_keyboard.as_deref(),
            mouse: maybe_mouse.as_deref(),
            associated_gamepad: gamepad,
        };

        input_streams.input_pressed(&input.into())
    }

    fn reset_inputs(&mut self) {
        #[cfg(feature = "ui")]
        {
            let mut interraction_system_state: SystemState<Query<&mut Interaction>> =
                SystemState::new(self);
            let mut interaction_query = interraction_system_state.get_mut(self);

            for mut interaction in interaction_query.iter_mut() {
                *interaction = Interaction::None;
            }
        }

        let mut input_system_state: SystemState<(
            Option<ResMut<Input<GamepadButton>>>,
            Option<ResMut<Input<KeyCode>>>,
            Option<ResMut<Input<MouseButton>>>,
        )> = SystemState::new(self);

        let (maybe_gamepad, maybe_keyboard, maybe_mouse) = input_system_state.get_mut(self);

        if let Some(mut gamepad) = maybe_gamepad {
            *gamepad = Default::default();
        }

        if let Some(mut keyboard) = maybe_keyboard {
            *keyboard = Default::default();
        }

        if let Some(mut mouse) = maybe_mouse {
            *mouse = Default::default();
        }

        self.insert_resource(Events::<GamepadEvent>::default());

        self.insert_resource(Events::<KeyboardInput>::default());

        self.insert_resource(Events::<MouseButtonInput>::default());
        self.insert_resource(Events::<MouseWheel>::default());
        self.insert_resource(Events::<CursorMoved>::default());

        self.insert_resource(Touches::default());
        self.insert_resource(Events::<TouchInput>::default());
    }

    #[cfg(feature = "ui")]
    fn click_button<Marker: Component>(&mut self) {
        let mut button_query = self.query_filtered::<&mut Interaction, With<Marker>>();

        for mut interaction in button_query.iter_mut(self) {
            *interaction = Interaction::Clicked;
        }
    }

    #[cfg(feature = "ui")]
    fn hover_button<Marker: Component>(&mut self) {
        let mut button_query = self.query_filtered::<&mut Interaction, With<Marker>>();

        for mut interaction in button_query.iter_mut(self) {
            *interaction = Interaction::Hovered;
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

    fn release_input(&mut self, input: impl Into<UserInput>) {
        self.world.release_input(input);
    }

    fn release_input_for_gamepad(&mut self, input: impl Into<UserInput>, gamepad: Option<Gamepad>) {
        self.world.release_input_for_gamepad(input, gamepad);
    }

    fn pressed(&mut self, input: impl Into<UserInput>) -> bool {
        self.world.pressed(input)
    }

    fn pressed_for_gamepad(
        &mut self,
        input: impl Into<UserInput>,
        gamepad: Option<Gamepad>,
    ) -> bool {
        self.world.pressed_for_gamepad(input, gamepad)
    }

    fn reset_inputs(&mut self) {
        self.world.reset_inputs();
    }

    #[cfg(feature = "ui")]
    fn click_button<Marker: Component>(&mut self) {
        self.world.click_button::<Marker>();
    }

    #[cfg(feature = "ui")]
    fn hover_button<Marker: Component>(&mut self) {
        self.world.hover_button::<Marker>();
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn button_inputs() {
        use crate::input_mocking::MockInput;
        use bevy::prelude::*;

        let mut world = World::new();
        world.insert_resource(Input::<KeyCode>::default());
        world.insert_resource(Input::<MouseButton>::default());
        world.insert_resource(Input::<GamepadButton>::default());

        // BLOCKED: cannot use the less artifical APIs due to
        // https://github.com/bevyengine/bevy/issues/3808
        let gamepad = Some(Gamepad(0));

        // Test that buttons are unpressed by default
        assert!(!world.pressed(KeyCode::Space));
        assert!(!world.pressed(MouseButton::Right));
        assert!(!world.pressed_for_gamepad(GamepadButtonType::North, gamepad));

        // Send inputs
        world.send_input(KeyCode::Space);
        world.send_input(MouseButton::Right);
        world.send_input_to_gamepad(GamepadButtonType::North, gamepad);

        // Verify that checking the resource value directly works
        let keyboard_input: &Input<KeyCode> = world.resource();
        assert!(keyboard_input.pressed(KeyCode::Space));

        // Test the convenient .pressed API
        assert!(world.pressed(KeyCode::Space));
        assert!(world.pressed(MouseButton::Right));
        assert!(world.pressed_for_gamepad(GamepadButtonType::North, gamepad));

        // Test that resetting inputs works
        world.reset_inputs();

        assert!(!world.pressed(KeyCode::Space));
        assert!(!world.pressed(MouseButton::Right));
        assert!(!world.pressed_for_gamepad(GamepadButtonType::North, gamepad));
    }

    #[test]
    #[cfg(feature = "ui")]
    fn ui_inputs() {
        use crate::input_mocking::MockInput;
        use bevy_ecs::prelude::*;
        use bevy_ui::Interaction;

        #[derive(Component)]
        struct ButtonMarker;

        let mut world = World::new();
        // Marked button
        world.spawn().insert(Interaction::None).insert(ButtonMarker);
        // Unmarked button
        world.spawn().insert(Interaction::None);

        // Click the button
        world.click_button::<ButtonMarker>();

        let mut interaction_query = world.query::<(&Interaction, Option<&ButtonMarker>)>();
        for (interaction, maybe_marker) in interaction_query.iter(&world) {
            match maybe_marker {
                Some(_) => assert_eq!(*interaction, Interaction::Clicked),
                None => assert_eq!(*interaction, Interaction::None),
            }
        }

        // Reset inputs
        world.reset_inputs();

        let mut interaction_query = world.query::<&Interaction>();
        for interaction in interaction_query.iter(&world) {
            assert_eq!(*interaction, Interaction::None)
        }

        // Hover over the button
        world.hover_button::<ButtonMarker>();

        let mut interaction_query = world.query::<(&Interaction, Option<&ButtonMarker>)>();
        for (interaction, maybe_marker) in interaction_query.iter(&world) {
            match maybe_marker {
                Some(_) => assert_eq!(*interaction, Interaction::Hovered),
                None => assert_eq!(*interaction, Interaction::None),
            }
        }

        // Reset inputs
        world.reset_inputs();

        let mut interaction_query = world.query::<&Interaction>();
        for interaction in interaction_query.iter(&world) {
            assert_eq!(*interaction, Interaction::None)
        }
    }
}
