//! Helpful utilities for testing input management by sending mock input events
//!
//! The [`MockInput`] trait contains methods with the same API that operate at three levels:
//! [`App`], [`World`] and [`MutableInputStreams`], each passing down the supplied arguments to the next.
//!
//! Inputs are provided in the convenient, high-level [`UserInput`] form.
//! These are then parsed down to their [`UserInput::raw_inputs()`],
//! which are then sent as [`bevy::input`] events of the appropriate types.

use crate::axislike::{AxisType, MouseMotionAxisType, MouseWheelAxisType};
use crate::buttonlike::{MouseMotionDirection, MouseWheelDirection};
use crate::input_streams::{InputStreams, MutableInputStreams};
use crate::user_input::{RawInputs, UserInput};

use bevy::app::App;
use bevy::ecs::event::Events;
use bevy::ecs::system::{ResMut, SystemState};
use bevy::ecs::world::World;
#[cfg(feature = "ui")]
use bevy::ecs::{component::Component, query::With, system::Query};
use bevy::input::gamepad::{GamepadAxisChangedEvent, GamepadButtonChangedEvent};
use bevy::input::keyboard::{Key, NativeKey};
use bevy::input::mouse::MouseScrollUnit;
use bevy::input::ButtonState;
use bevy::input::{
    gamepad::{Gamepad, GamepadButton, GamepadEvent},
    keyboard::{KeyCode, KeyboardInput},
    mouse::{MouseButton, MouseButtonInput, MouseMotion, MouseWheel},
    touch::{TouchInput, Touches},
    ButtonInput,
};
use bevy::math::Vec2;
use bevy::prelude::Entity;
#[cfg(feature = "ui")]
use bevy::ui::Interaction;
use bevy::window::CursorMoved;

/// Send fake input events for testing purposes
///
/// In game code, you should (almost) always be setting the [`ActionState`](crate::action_state::ActionState)
/// directly instead.
///
/// # Examples
/// ```rust
/// use bevy::prelude::*;
/// use bevy::input::InputPlugin;
/// use leafwing_input_manager::input_mocking::MockInput;
///
/// // Remember to add InputPlugin so the resources will be there!
/// let mut app = App::new();
/// app.add_plugins(InputPlugin);
///
/// // Pay respects!
/// app.send_input(KeyCode::KeyF);
/// app.update();
/// ```
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy::input::InputPlugin;
/// use leafwing_input_manager::{input_mocking::MockInput, user_input::UserInput};
///
/// let mut app = App::new();
/// app.add_plugins(InputPlugin);
///
/// // Send inputs one at a time
/// let B_E_V_Y = [KeyCode::KeyB, KeyCode::KeyE, KeyCode::KeyV, KeyCode::KeyY];
///
/// for letter in B_E_V_Y {
///     app.send_input(letter);
/// }
///
/// // Or use chords!
/// app.send_input(UserInput::chord(B_E_V_Y));
/// app.update();
/// ```
pub trait MockInput {
    /// Send the specified `user_input` directly
    ///
    /// These are sent as the raw input events, and do not set the value of [`ButtonInput`] or [`Axis`](bevy::input::Axis) directly.
    /// Note that inputs will continue to be pressed until explicitly released or [`MockInput::reset_inputs`] is called.
    ///
    /// To send specific values for axislike inputs, set their `value` field.
    ///
    /// Gamepad input will be sent by the first registered controller found.
    /// If none are found, gamepad input will be silently skipped.
    ///
    /// # Warning
    ///
    /// You *must* call `app.update()` at least once after sending input
    /// with `InputPlugin` included in your plugin set
    /// for the raw input events to be processed into [`ButtonInput`] and [`Axis`](bevy::input::Axis) data.
    fn send_input(&mut self, input: impl Into<UserInput>);

    /// Send the specified `user_input` directly, using the specified gamepad
    ///
    /// Note that inputs will continue to be pressed until explicitly released or [`MockInput::reset_inputs`] is called.
    ///
    /// Provide the [`Gamepad`] identifier to control which gamepad you are emulating.
    fn send_input_as_gamepad(&mut self, input: impl Into<UserInput>, gamepad: Option<Gamepad>);

    /// Releases the specified `user_input` directly
    ///
    /// Gamepad input will be released by the first registered controller found.
    /// If none are found, gamepad input will be silently skipped.
    fn release_input(&mut self, input: impl Into<UserInput>);

    /// Releases the specified `user_input` directly, using the specified gamepad
    ///
    /// Provide the [`Gamepad`] identifier to control which gamepad you are emulating.
    fn release_input_as_gamepad(&mut self, input: impl Into<UserInput>, gamepad: Option<Gamepad>);

    /// Clears all user input streams, resetting them to their default state
    ///
    /// All buttons are released, and `just_pressed` and `just_released` information on the [`ButtonInput`] type are lost.
    /// `just_pressed` and `just_released` on the [`ActionState`](crate::action_state::ActionState) will be kept.
    ///
    /// This will clear all [`KeyCode`], [`GamepadButton`] and [`MouseButton`] input streams,
    /// as well as any [`Interaction`] components and all input [`Events`].
    fn reset_inputs(&mut self);
}

/// Query [`ButtonInput`] state directly for testing purposes.
///
/// In game code, you should (almost) always be using [`ActionState`](crate::action_state::ActionState)
/// methods instead.
pub trait QueryInput {
    /// Is the provided `user_input` pressed?
    ///
    /// This method is intended as a convenience for testing; check the [`ButtonInput`] resource directly,
    /// or use an [`InputMap`](crate::input_map::InputMap) in real code.
    fn pressed(&self, input: impl Into<UserInput>) -> bool;

    /// Is the provided `user_input` pressed for the provided [`Gamepad`]?
    ///
    /// This method is intended as a convenience for testing; check the [`ButtonInput`] resource directly,
    /// or use an [`InputMap`](crate::input_map::InputMap) in real code.
    fn pressed_for_gamepad(&self, input: impl Into<UserInput>, gamepad: Option<Gamepad>) -> bool;
}

/// Send fake UI interaction for testing purposes.
#[cfg(feature = "ui")]
pub trait MockUIInteraction {
    /// Presses all `bevy::ui` buttons with the matching `Marker` component
    ///
    /// Changes their [`Interaction`] component to [`Interaction::Pressed`]
    fn click_button<Marker: Component>(&mut self);

    /// Hovers over all `bevy::ui` buttons with the matching `Marker` component
    ///
    /// Changes their [`Interaction`] component to [`Interaction::Pressed`]
    fn hover_button<Marker: Component>(&mut self);
}

impl MockInput for MutableInputStreams<'_> {
    fn send_input(&mut self, input: impl Into<UserInput>) {
        self.send_input_as_gamepad(input, self.guess_gamepad());
    }

    fn send_input_as_gamepad(&mut self, input: impl Into<UserInput>, gamepad: Option<Gamepad>) {
        let input_to_send: UserInput = input.into();
        // Extract the raw inputs
        let raw_inputs = input_to_send.raw_inputs();

        self.send_keyboard_input(ButtonState::Pressed, &raw_inputs);

        // Mouse buttons
        for button in raw_inputs.mouse_buttons.iter() {
            self.mouse_button_events.send(MouseButtonInput {
                button: *button,
                state: ButtonState::Pressed,
                window: Entity::PLACEHOLDER,
            });
        }

        // Discrete mouse wheel events
        for mouse_wheel_direction in raw_inputs.mouse_wheel.iter() {
            match *mouse_wheel_direction {
                MouseWheelDirection::Left => self.send_mouse_wheel(-1.0, 0.0),
                MouseWheelDirection::Right => self.send_mouse_wheel(1.0, 0.0),
                MouseWheelDirection::Up => self.send_mouse_wheel(0.0, 1.0),
                MouseWheelDirection::Down => self.send_mouse_wheel(0.0, -1.0),
            };
        }

        // Discrete mouse motion event
        for mouse_motion_direction in raw_inputs.mouse_motion.iter() {
            match *mouse_motion_direction {
                MouseMotionDirection::Up => self.send_mouse_motion(0.0, 1.0),
                MouseMotionDirection::Down => self.send_mouse_motion(0.0, -1.0),
                MouseMotionDirection::Right => self.send_mouse_motion(1.0, 0.0),
                MouseMotionDirection::Left => self.send_mouse_motion(-1.0, 0.0),
            };
        }

        self.send_gamepad_button_changed(gamepad, &raw_inputs);

        // Axis data
        for (outer_axis_type, maybe_position_data) in raw_inputs.axis_data.iter() {
            if let Some(position_data) = *maybe_position_data {
                match outer_axis_type {
                    AxisType::Gamepad(axis_type) => {
                        if let Some(gamepad) = gamepad {
                            self.gamepad_events
                                .send(GamepadEvent::Axis(GamepadAxisChangedEvent {
                                    gamepad,
                                    axis_type: *axis_type,
                                    value: position_data,
                                }));
                        }
                    }
                    AxisType::MouseWheel(axis_type) => match *axis_type {
                        MouseWheelAxisType::X => self.send_mouse_wheel(position_data, 0.0),
                        MouseWheelAxisType::Y => self.send_mouse_wheel(0.0, position_data),
                    },
                    AxisType::MouseMotion(axis_type) => match *axis_type {
                        MouseMotionAxisType::X => self.send_mouse_motion(position_data, 0.0),
                        MouseMotionAxisType::Y => self.send_mouse_motion(0.0, position_data),
                    },
                }
            }
        }
    }

    fn release_input(&mut self, input: impl Into<UserInput>) {
        self.release_input_as_gamepad(input, self.guess_gamepad())
    }

    fn release_input_as_gamepad(&mut self, input: impl Into<UserInput>, gamepad: Option<Gamepad>) {
        // Releasing axis-like inputs deliberately has no effect; it's unclear what this would do

        let input_to_release: UserInput = input.into();
        let raw_inputs = input_to_release.raw_inputs();

        self.send_gamepad_button_changed(gamepad, &raw_inputs);

        self.send_keyboard_input(ButtonState::Released, &raw_inputs);

        for button in raw_inputs.mouse_buttons {
            self.mouse_button_events.send(MouseButtonInput {
                button,
                state: ButtonState::Released,
                window: Entity::PLACEHOLDER,
            });
        }
    }

    fn reset_inputs(&mut self) {
        // WARNING: this *must* be updated when MutableInputStreams''s fields change
        // Note that we deliberately are not resetting either Gamepads or associated_gamepad
        // as they are not actual input data
        *self.gamepad_buttons = Default::default();
        *self.gamepad_axes = Default::default();
        *self.keycodes = Default::default();
        *self.mouse_buttons = Default::default();
        *self.mouse_wheel = Default::default();
        *self.mouse_motion = Default::default();
    }
}

impl MutableInputStreams<'_> {
    fn send_keyboard_input(&mut self, button_state: ButtonState, raw_inputs: &RawInputs) {
        for key_code in raw_inputs.keycodes.iter() {
            self.keyboard_events.send(KeyboardInput {
                logical_key: Key::Unidentified(NativeKey::Unidentified),
                key_code: *key_code,
                state: button_state,
                window: Entity::PLACEHOLDER,
            });
        }
    }

    fn send_mouse_wheel(&mut self, x: f32, y: f32) {
        // FIXME: MouseScrollUnit is not recorded and is always assumed to be Pixel
        let unit = MouseScrollUnit::Pixel;
        let window = Entity::PLACEHOLDER;
        self.mouse_wheel.send(MouseWheel { unit, x, y, window });
    }

    fn send_mouse_motion(&mut self, x: f32, y: f32) {
        let delta = Vec2::new(x, y);
        self.mouse_motion.send(MouseMotion { delta });
    }

    fn send_gamepad_button_changed(&mut self, gamepad: Option<Gamepad>, raw_inputs: &RawInputs) {
        if let Some(gamepad) = gamepad {
            for button_type in raw_inputs.gamepad_buttons.iter() {
                self.gamepad_events
                    .send(GamepadEvent::Button(GamepadButtonChangedEvent {
                        gamepad,
                        button_type: *button_type,
                        value: 1.0,
                    }));
            }
        }
    }
}

impl QueryInput for InputStreams<'_> {
    fn pressed(&self, input: impl Into<UserInput>) -> bool {
        self.input_pressed(&input.into())
    }

    fn pressed_for_gamepad(&self, input: impl Into<UserInput>, gamepad: Option<Gamepad>) -> bool {
        let mut input_streams = self.clone();
        input_streams.associated_gamepad = gamepad;

        input_streams.input_pressed(&input.into())
    }
}

impl MockInput for World {
    fn send_input(&mut self, input: impl Into<UserInput>) {
        let mut mutable_input_streams = MutableInputStreams::from_world(self, None);

        mutable_input_streams.send_input(input);
    }

    fn send_input_as_gamepad(&mut self, input: impl Into<UserInput>, gamepad: Option<Gamepad>) {
        let mut mutable_input_streams = MutableInputStreams::from_world(self, gamepad);

        mutable_input_streams.send_input_as_gamepad(input, gamepad);
    }

    fn release_input(&mut self, input: impl Into<UserInput>) {
        let mut mutable_input_streams = MutableInputStreams::from_world(self, None);

        mutable_input_streams.release_input(input);
    }

    fn release_input_as_gamepad(&mut self, input: impl Into<UserInput>, gamepad: Option<Gamepad>) {
        let mut mutable_input_streams = MutableInputStreams::from_world(self, gamepad);

        mutable_input_streams.release_input_as_gamepad(input, gamepad);
    }

    fn reset_inputs(&mut self) {
        #[cfg(feature = "ui")]
        {
            let mut interaction_system_state: SystemState<Query<&mut Interaction>> =
                SystemState::new(self);
            let mut interaction_query = interaction_system_state.get_mut(self);

            for mut interaction in interaction_query.iter_mut() {
                *interaction = Interaction::None;
            }
        }

        let mut input_system_state: SystemState<(
            Option<ResMut<ButtonInput<GamepadButton>>>,
            Option<ResMut<ButtonInput<KeyCode>>>,
            Option<ResMut<ButtonInput<MouseButton>>>,
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
}

impl QueryInput for World {
    fn pressed(&self, input: impl Into<UserInput>) -> bool {
        self.pressed_for_gamepad(input, None)
    }

    fn pressed_for_gamepad(&self, input: impl Into<UserInput>, gamepad: Option<Gamepad>) -> bool {
        let input_streams = InputStreams::from_world(self, gamepad);

        input_streams.input_pressed(&input.into())
    }
}

#[cfg(feature = "ui")]
impl MockUIInteraction for World {
    fn click_button<Marker: Component>(&mut self) {
        let mut button_query = self.query_filtered::<&mut Interaction, With<Marker>>();

        for mut interaction in button_query.iter_mut(self) {
            *interaction = Interaction::Pressed;
        }
    }

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

    fn send_input_as_gamepad(&mut self, input: impl Into<UserInput>, gamepad: Option<Gamepad>) {
        self.world.send_input_as_gamepad(input, gamepad);
    }

    fn release_input(&mut self, input: impl Into<UserInput>) {
        self.world.release_input(input);
    }

    fn release_input_as_gamepad(&mut self, input: impl Into<UserInput>, gamepad: Option<Gamepad>) {
        self.world.release_input_as_gamepad(input, gamepad);
    }

    fn reset_inputs(&mut self) {
        self.world.reset_inputs();
    }
}

impl QueryInput for App {
    fn pressed(&self, input: impl Into<UserInput>) -> bool {
        self.world.pressed(input)
    }

    fn pressed_for_gamepad(&self, input: impl Into<UserInput>, gamepad: Option<Gamepad>) -> bool {
        self.world.pressed_for_gamepad(input, gamepad)
    }
}

#[cfg(feature = "ui")]
impl MockUIInteraction for App {
    fn click_button<Marker: Component>(&mut self) {
        self.world.click_button::<Marker>();
    }

    fn hover_button<Marker: Component>(&mut self) {
        self.world.hover_button::<Marker>();
    }
}

#[cfg(test)]
mod test {
    use crate::input_mocking::{MockInput, QueryInput};
    use bevy::{
        input::{
            gamepad::{GamepadConnection, GamepadConnectionEvent, GamepadEvent, GamepadInfo},
            InputPlugin,
        },
        prelude::*,
    };

    #[test]
    fn ordinary_button_inputs() {
        let mut app = App::new();
        app.add_plugins(InputPlugin);

        // Test that buttons are unpressed by default
        assert!(!app.pressed(KeyCode::Space));
        assert!(!app.pressed(MouseButton::Right));

        // Send inputs
        app.send_input(KeyCode::Space);
        app.send_input(MouseButton::Right);
        app.update();

        // Verify that checking the resource value directly works
        let keyboard_input: &ButtonInput<KeyCode> = app.world.resource();
        assert!(keyboard_input.pressed(KeyCode::Space));

        // Test the convenient .pressed API
        assert!(app.pressed(KeyCode::Space));
        assert!(app.pressed(MouseButton::Right));

        // Test that resetting inputs works
        app.reset_inputs();
        app.update();

        assert!(!app.pressed(KeyCode::Space));
        assert!(!app.pressed(MouseButton::Right));
    }

    #[test]
    fn explicit_gamepad_button_inputs() {
        let mut app = App::new();
        app.add_plugins(InputPlugin);

        let gamepad = Gamepad { id: 0 };
        let mut gamepad_events = app.world.resource_mut::<Events<GamepadEvent>>();
        gamepad_events.send(GamepadEvent::Connection(GamepadConnectionEvent {
            gamepad,
            connection: GamepadConnection::Connected(GamepadInfo {
                name: "TestController".into(),
            }),
        }));
        app.update();

        // Test that buttons are unpressed by default
        assert!(!app.pressed_for_gamepad(GamepadButtonType::North, Some(gamepad)));

        // Send inputs
        app.send_input_as_gamepad(GamepadButtonType::North, Some(gamepad));
        app.update();

        // Checking the old-fashioned way
        // FIXME: put this in a gamepad_button.rs integration test.
        let gamepad_input = app.world.resource::<ButtonInput<GamepadButton>>();
        assert!(gamepad_input.pressed(GamepadButton {
            gamepad,
            button_type: GamepadButtonType::North,
        }));

        // Test the convenient .pressed API
        assert!(app.pressed_for_gamepad(GamepadButtonType::North, Some(gamepad)));

        // Test that resetting inputs works
        app.reset_inputs();
        app.update();

        assert!(!app.pressed_for_gamepad(GamepadButtonType::North, Some(gamepad)));
    }

    #[test]
    fn implicit_gamepad_button_inputs() {
        let mut app = App::new();
        app.add_plugins(InputPlugin);

        let gamepad = Gamepad { id: 0 };
        let mut gamepad_events = app.world.resource_mut::<Events<GamepadEvent>>();
        gamepad_events.send(GamepadEvent::Connection(GamepadConnectionEvent {
            gamepad,
            connection: GamepadConnection::Connected(GamepadInfo {
                name: "TestController".into(),
            }),
        }));
        app.update();

        // Test that buttons are unpressed by default
        assert!(!app.pressed(GamepadButtonType::North));

        // Send inputs
        app.send_input(GamepadButtonType::North);
        app.update();

        // Test the convenient .pressed API
        assert!(app.pressed(GamepadButtonType::North));

        // Test that resetting inputs works
        app.reset_inputs();
        app.update();

        assert!(!app.pressed(GamepadButtonType::North));
    }

    #[test]
    #[cfg(feature = "ui")]
    fn ui_inputs() {
        use crate::input_mocking::MockUIInteraction;
        use bevy::ecs::prelude::*;
        use bevy::ui::Interaction;

        #[derive(Component)]
        struct ButtonMarker;

        let mut app = App::new();
        app.add_plugins(InputPlugin);

        // Marked button
        app.world.spawn((Interaction::None, ButtonMarker));
        // Unmarked button
        app.world.spawn(Interaction::None);

        // Click the button
        app.world.click_button::<ButtonMarker>();
        app.update();

        let mut interaction_query = app.world.query::<(&Interaction, Option<&ButtonMarker>)>();
        for (interaction, maybe_marker) in interaction_query.iter(&app.world) {
            match maybe_marker {
                Some(_) => assert_eq!(*interaction, Interaction::Pressed),
                None => assert_eq!(*interaction, Interaction::None),
            }
        }

        // Reset inputs
        app.world.reset_inputs();

        let mut interaction_query = app.world.query::<&Interaction>();
        for interaction in interaction_query.iter(&app.world) {
            assert_eq!(*interaction, Interaction::None)
        }

        // Hover over the button
        app.hover_button::<ButtonMarker>();
        app.update();

        let mut interaction_query = app.world.query::<(&Interaction, Option<&ButtonMarker>)>();
        for (interaction, maybe_marker) in interaction_query.iter(&app.world) {
            match maybe_marker {
                Some(_) => assert_eq!(*interaction, Interaction::Hovered),
                None => assert_eq!(*interaction, Interaction::None),
            }
        }

        // Reset inputs
        app.world.reset_inputs();

        let mut interaction_query = app.world.query::<&Interaction>();
        for interaction in interaction_query.iter(&app.world) {
            assert_eq!(*interaction, Interaction::None)
        }
    }
}
