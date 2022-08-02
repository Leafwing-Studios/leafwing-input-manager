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
use crate::user_input::UserInput;

use bevy::app::App;
use bevy::ecs::event::Events;
use bevy::ecs::system::{ResMut, SystemState};
use bevy::ecs::world::World;
#[cfg(feature = "ui")]
use bevy::ecs::{component::Component, query::With, system::Query};
use bevy::input::gamepad::GamepadEventRaw;
use bevy::input::mouse::MouseScrollUnit;
use bevy::input::ButtonState;
use bevy::input::{
    gamepad::{Gamepad, GamepadButton, GamepadEvent, GamepadEventType},
    keyboard::{KeyCode, KeyboardInput},
    mouse::{MouseButton, MouseButtonInput, MouseMotion, MouseWheel},
    touch::{TouchInput, Touches},
    Input,
};
use bevy::math::Vec2;
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
/// app.add_plugin(InputPlugin);
///
/// // Pay respects!
/// app.send_input(KeyCode::F);
/// app.update();
/// ```
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy::input::InputPlugin;
/// use leafwing_input_manager::{input_mocking::MockInput, user_input::UserInput};
///
/// let mut app = App::new();
/// app.add_plugin(InputPlugin);
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
/// app.update();
/// ```
pub trait MockInput {
    /// Send the specified `user_input` directly
    ///
    /// These are sent as the raw input events, and do not set the value of [`Input`] or [`Axis`] directly.
    /// Note that inputs will continue to be pressed until explicitly released or [`MockInput::reset_inputs`] is called.
    ///
    /// To send specific values for axislike inputs, set their `value` field.
    ///
    /// Gamepad input will be sent by the first registed controller found.
    /// If none are found, gamepad input will be silently skipped.
    ///
    /// # Warning
    ///
    /// You *must* call `app.update()` at least once after sending input
    /// with `InputPlugin` included in your plugin set
    /// for the raw input events to be processed into [`Input`] and [`Axis`] data.
    fn send_input(&mut self, input: impl Into<UserInput>);

    /// Send the specified `user_input` directly, using the specified gamepad
    ///
    /// Note that inputs will continue to be pressed until explicitly released or [`MockInput::reset_inputs`] is called.
    ///
    /// Provide the [`Gamepad`] identifier to control which gamepad you are emulating.
    fn send_input_as_gamepad(&mut self, input: impl Into<UserInput>, gamepad: Option<Gamepad>);

    /// Releases the specified `user_input` directly
    ///
    /// Gamepad input will be released by the first registed controller found.
    /// If none are found, gamepad input will be silently skipped.
    fn release_input(&mut self, input: impl Into<UserInput>);

    /// Releases the specified `user_input` directly, using the specified gamepad
    ///
    /// Provide the [`Gamepad`] identifier to control which gamepad you are emulating.
    fn release_input_as_gamepad(&mut self, input: impl Into<UserInput>, gamepad: Option<Gamepad>);

    /// Is the provided `user_input` pressed?
    ///
    /// This method is intended as a convenience for testing; check the [`Input`] resource directly,
    /// or use an [`InputMap`](crate::input_map::InputMap) in real code.
    fn pressed(&self, input: impl Into<UserInput>) -> bool;

    /// Is the provided `user_input` pressed for the provided [`Gamepad`]?
    ///
    /// This method is intended as a convenience for testing; check the [`Input`] resource directly,
    /// or use an [`InputMap`](crate::input_map::InputMap) in real code.
    fn pressed_for_gamepad(&self, input: impl Into<UserInput>, gamepad: Option<Gamepad>) -> bool;

    /// Clears all user input streams, resetting them to their default state
    ///
    /// All buttons are released, and `just_pressed` and `just_released` information on the [`Input`] type are lost.
    /// `just_pressed` and `just_released` on the [`ActionState`](crate::action_state::ActionState) will be kept.
    ///
    /// This will clear all [`KeyCode`], [`GamepadButton`] and [`MouseButton`] input streams,
    /// as well as any [`Interaction`] components and all input [`Events`].
    fn reset_inputs(&mut self);

    /// Presses all `bevy::ui` buttons with the matching `Marker` component
    ///
    /// Changes their [`Interaction`] component to [`Interaction::Clicked`]
    #[cfg(feature = "ui")]
    fn click_button<Marker: Component>(&mut self);

    /// Hovers over all `bevy::ui` buttons with the matching `Marker` component
    ///
    /// Changes their [`Interaction`] component to [`Interaction::Clicked`]
    #[cfg(feature = "ui")]
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

        // Keyboard buttons
        for button in raw_inputs.keycodes {
            self.keyboard_events.send(KeyboardInput {
                scan_code: u32::MAX,
                key_code: Some(button),
                state: ButtonState::Pressed,
            });
        }

        // Mouse buttons
        for button in raw_inputs.mouse_buttons {
            self.mouse_button_events.send(MouseButtonInput {
                button,
                state: ButtonState::Pressed,
            });
        }

        // Discrete mouse wheel events
        for mouse_wheel_direction in raw_inputs.mouse_wheel {
            match mouse_wheel_direction {
                MouseWheelDirection::Left => self.mouse_wheel.send(MouseWheel {
                    unit: MouseScrollUnit::Pixel,
                    x: -1.0,
                    y: 0.0,
                }),
                MouseWheelDirection::Right => self.mouse_wheel.send(MouseWheel {
                    unit: MouseScrollUnit::Pixel,
                    x: 1.0,
                    y: 0.0,
                }),
                MouseWheelDirection::Up => self.mouse_wheel.send(MouseWheel {
                    unit: MouseScrollUnit::Pixel,
                    x: 0.0,
                    y: 1.0,
                }),
                MouseWheelDirection::Down => self.mouse_wheel.send(MouseWheel {
                    unit: MouseScrollUnit::Pixel,
                    x: 0.0,
                    y: -1.0,
                }),
            }
        }

        // Discrete mouse motion event
        for mouse_motion_direction in raw_inputs.mouse_motion {
            match mouse_motion_direction {
                MouseMotionDirection::Up => self.mouse_motion.send(MouseMotion {
                    delta: Vec2 { x: 0.0, y: 1.0 },
                }),
                MouseMotionDirection::Down => self.mouse_motion.send(MouseMotion {
                    delta: Vec2 { x: 0.0, y: -1.0 },
                }),
                MouseMotionDirection::Right => self.mouse_motion.send(MouseMotion {
                    delta: Vec2 { x: 1.0, y: 0.0 },
                }),
                MouseMotionDirection::Left => self.mouse_motion.send(MouseMotion {
                    delta: Vec2 { x: -1.0, y: 0.0 },
                }),
            }
        }

        // Gamepad buttons
        for button_type in raw_inputs.gamepad_buttons {
            if let Some(gamepad) = gamepad {
                self.gamepad_events.send(GamepadEventRaw {
                    gamepad,
                    event_type: GamepadEventType::ButtonChanged(button_type, 1.0),
                });
            }
        }

        // Axis data
        for (outer_axis_type, maybe_position_data) in raw_inputs.axis_data {
            if let Some(position_data) = maybe_position_data {
                match outer_axis_type {
                    AxisType::Gamepad(axis_type) => {
                        if let Some(gamepad) = gamepad {
                            self.gamepad_events.send(GamepadEventRaw {
                                gamepad,
                                event_type: GamepadEventType::AxisChanged(axis_type, position_data),
                            });
                        }
                    }
                    AxisType::MouseWheel(axis_type) => {
                        match axis_type {
                            // FIXME: MouseScrollUnit is not recorded and is always assumed to be Pixel
                            MouseWheelAxisType::X => self.mouse_wheel.send(MouseWheel {
                                unit: MouseScrollUnit::Pixel,
                                x: position_data,
                                y: 0.0,
                            }),
                            MouseWheelAxisType::Y => self.mouse_wheel.send(MouseWheel {
                                unit: MouseScrollUnit::Pixel,
                                x: 0.0,
                                y: position_data,
                            }),
                        }
                    }
                    AxisType::MouseMotion(axis_type) => match axis_type {
                        MouseMotionAxisType::X => self.mouse_motion.send(MouseMotion {
                            delta: Vec2 {
                                x: position_data,
                                y: 0.0,
                            },
                        }),
                        MouseMotionAxisType::Y => self.mouse_motion.send(MouseMotion {
                            delta: Vec2 {
                                x: 0.0,
                                y: position_data,
                            },
                        }),
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

        for button_type in raw_inputs.gamepad_buttons {
            if let Some(gamepad) = gamepad {
                self.gamepad_events.send(GamepadEventRaw {
                    gamepad,
                    event_type: GamepadEventType::ButtonChanged(button_type, 1.0),
                });
            }
        }

        for button in raw_inputs.keycodes {
            self.keyboard_events.send(KeyboardInput {
                scan_code: u32::MAX,
                key_code: Some(button),
                state: ButtonState::Released,
            });
        }

        for button in raw_inputs.mouse_buttons {
            self.mouse_button_events.send(MouseButtonInput {
                button,
                state: ButtonState::Released,
            });
        }
    }

    fn pressed(&self, input: impl Into<UserInput>) -> bool {
        let input_streams: InputStreams = self.into();
        input_streams.input_pressed(&input.into())
    }

    fn pressed_for_gamepad(&self, input: impl Into<UserInput>, gamepad: Option<Gamepad>) -> bool {
        let mut input_streams: InputStreams = self.into();
        input_streams.associated_gamepad = gamepad;

        input_streams.input_pressed(&input.into())
    }

    fn reset_inputs(&mut self) {
        // WARNING: this *must* be updated when MutableInputStreams's fields change
        // Note that we deliberately are not resetting either Gamepads or associated_gamepad
        // as they are not actually input data
        *self.gamepad_buttons = Default::default();
        *self.gamepad_axes = Default::default();
        *self.keycode = Default::default();
        *self.mouse_button = Default::default();
        *self.mouse_wheel = Default::default();
        *self.mouse_motion = Default::default();
    }

    #[cfg(feature = "ui")]
    fn click_button<Marker: Component>(&mut self) {
        panic!("Cannot use bevy_ui input mocking from `MutableInputStreams`, use an `App` or `World` instead.")
    }

    #[cfg(feature = "ui")]
    fn hover_button<Marker: Component>(&mut self) {
        panic!("Cannot use bevy_ui input mocking from `MutableInputStreams`, use an `App` or `World` instead.")
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

    fn pressed(&self, input: impl Into<UserInput>) -> bool {
        self.pressed_for_gamepad(input, None)
    }

    fn pressed_for_gamepad(&self, input: impl Into<UserInput>, gamepad: Option<Gamepad>) -> bool {
        let input_streams = InputStreams::from_world(self, gamepad);

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

    fn send_input_as_gamepad(&mut self, input: impl Into<UserInput>, gamepad: Option<Gamepad>) {
        self.world.send_input_as_gamepad(input, gamepad);
    }

    fn release_input(&mut self, input: impl Into<UserInput>) {
        self.world.release_input(input);
    }

    fn release_input_as_gamepad(&mut self, input: impl Into<UserInput>, gamepad: Option<Gamepad>) {
        self.world.release_input_as_gamepad(input, gamepad);
    }

    fn pressed(&self, input: impl Into<UserInput>) -> bool {
        self.world.pressed(input)
    }

    fn pressed_for_gamepad(&self, input: impl Into<UserInput>, gamepad: Option<Gamepad>) -> bool {
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
    use crate::input_mocking::MockInput;
    use bevy::{input::InputPlugin, prelude::*};

    #[test]
    fn ordinary_button_inputs() {
        let mut app = App::new();
        app.add_plugin(InputPlugin);

        // Test that buttons are unpressed by default
        assert!(!app.pressed(KeyCode::Space));
        assert!(!app.pressed(MouseButton::Right));

        // Send inputs
        app.send_input(KeyCode::Space);
        app.send_input(MouseButton::Right);
        app.update();

        // Verify that checking the resource value directly works
        let keyboard_input: &Input<KeyCode> = app.world.resource();
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
        app.add_plugin(InputPlugin);

        let gamepad = Gamepad { id: 0 };
        let mut gamepad_events = app.world.resource_mut::<Events<GamepadEvent>>();
        gamepad_events.send(GamepadEvent {
            gamepad,
            event_type: GamepadEventType::Connected,
        });
        app.update();

        // Test that buttons are unpressed by default
        assert!(!app.pressed_for_gamepad(GamepadButtonType::North, Some(gamepad)));

        // Send inputs
        app.send_input_as_gamepad(GamepadButtonType::North, Some(gamepad));
        app.update();

        // Checking the old-fashioned way
        // FIXME: put this in a gamepad_button.rs integration test.
        let gamepad_input = app.world.resource::<Input<GamepadButton>>();
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
        app.add_plugin(InputPlugin);

        let gamepad = Gamepad { id: 0 };
        let mut gamepad_events = app.world.resource_mut::<Events<GamepadEvent>>();
        gamepad_events.send(GamepadEvent {
            gamepad,
            event_type: GamepadEventType::Connected,
        });
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
        use bevy::ecs::prelude::*;
        use bevy::ui::Interaction;

        #[derive(Component)]
        struct ButtonMarker;

        let mut app = App::new();
        app.add_plugin(InputPlugin);

        // Marked button
        app.world
            .spawn()
            .insert(Interaction::None)
            .insert(ButtonMarker);
        // Unmarked button
        app.world.spawn().insert(Interaction::None);

        // Click the button
        app.world.click_button::<ButtonMarker>();
        app.update();

        let mut interaction_query = app.world.query::<(&Interaction, Option<&ButtonMarker>)>();
        for (interaction, maybe_marker) in interaction_query.iter(&app.world) {
            match maybe_marker {
                Some(_) => assert_eq!(*interaction, Interaction::Clicked),
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
