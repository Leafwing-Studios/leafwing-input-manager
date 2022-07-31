//! Helpful utilities for testing input management by sending mock input events
//!
//! The [`MockInput`] trait contains methods with the same API that operate at three levels:
//! [`App`], [`World`] and [`MutableInputStreams`], each passing down the supplied arguments to the next.
//!
//! Inputs are provided in the convenient, high-level [`UserInput`] form.
//! These are then parsed down to their [`UserInput::raw_inputs()`],
//! which are then sent as [`bevy::input`] events of the appropriate types.

use crate::axislike::{AxisType, MouseMotionAxisType, MouseWheelAxisType};
use crate::input_streams::{InputStreams, MutableInputStreams};
use crate::user_input::UserInput;

use bevy::app::App;
use bevy::ecs::event::Events;
use bevy::ecs::system::{ResMut, SystemState};
use bevy::ecs::world::World;
#[cfg(feature = "ui")]
use bevy::ecs::{component::Component, query::With, system::Query};
use bevy::input::mouse::MouseScrollUnit;
use bevy::input::{
    gamepad::{Gamepad, GamepadAxis, GamepadButton, GamepadEvent, Gamepads},
    keyboard::{KeyCode, KeyboardInput},
    mouse::{MouseButton, MouseButtonInput, MouseMotion, MouseWheel},
    touch::{TouchInput, Touches},
    {Axis, Input},
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
/// use leafwing_input_manager::input_mocking::{MockInput, mockable_world};
///
/// // A `World` that contains all of the appropriate input resources
/// let mut world = mockable_world();
///
/// // Pay respects!
/// world.send_input(KeyCode::F);
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
/// ```
pub trait MockInput {
    /// Send the specified `user_input` directly
    ///
    /// Note that inputs will continue to be pressed until explicitly released or [`MockInput::reset_inputs`] is called.
    ///
    /// To send specific values for axislike inputs, set their `value` field.
    ///
    /// Gamepad input will be sent by the first registed controller found.
    /// If none are found, gamepad input will be silently skipped.
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
        let (gamepad_buttons, axis_data, keyboard_buttons, mouse_buttons) =
            input_to_send.raw_inputs();

        for button_type in gamepad_buttons {
            if let Some(gamepad) = gamepad {
                let gamepad_button = GamepadButton {
                    gamepad,
                    button_type,
                };
                self.gamepad_buttons.press(gamepad_button);
            }
        }

        for (outer_axis_type, maybe_position_data) in axis_data {
            if let Some(position_data) = maybe_position_data {
                match outer_axis_type {
                    AxisType::Gamepad(axis_type) => {
                        if let Some(gamepad) = gamepad {
                            self.gamepad_axes
                                .set(GamepadAxis { gamepad, axis_type }, position_data);
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

        for button in keyboard_buttons {
            self.keyboard.press(button);
        }

        for button in mouse_buttons {
            self.mouse.press(button);
        }
    }

    fn release_input(&mut self, input: impl Into<UserInput>) {
        self.release_input_as_gamepad(input, self.guess_gamepad())
    }

    fn release_input_as_gamepad(&mut self, input: impl Into<UserInput>, gamepad: Option<Gamepad>) {
        let input_to_release: UserInput = input.into();
        let (gamepad_buttons, axis_data, keyboard_buttons, mouse_buttons) =
            input_to_release.raw_inputs();

        for button_type in gamepad_buttons {
            if let Some(gamepad) = gamepad {
                let gamepad_button = GamepadButton {
                    gamepad,
                    button_type,
                };
                self.gamepad_buttons.release(gamepad_button);
            }
        }

        for (outer_axis_type, _maybe_position_data) in axis_data {
            match outer_axis_type {
                AxisType::Gamepad(axis_type) => {
                    if let Some(gamepad) = gamepad {
                        self.gamepad_axes.remove(GamepadAxis { gamepad, axis_type });
                    }
                }
                // Releasing event-like input should have no effect;
                // they are automatically cleared as time elapses
                AxisType::MouseWheel(_) => {}
                AxisType::MouseMotion(_) => {}
            }
        }

        for button in keyboard_buttons {
            self.keyboard.release(button);
        }

        for button in mouse_buttons {
            self.mouse.release(button);
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
        *self.keyboard = Default::default();
        *self.mouse = Default::default();
        *self.mouse_wheel = Default::default();
        *self.mouse_motion = Default::default();
    }

    fn click_button<Marker: Component>(&mut self) {
        panic!("Cannot use bevy_ui input mocking from `MutableInputStreams`, use an `App` or `World` instead.")
    }

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

        mutable_input_streams.send_input(input);
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
    use crate::input_mocking::{mockable_world, MockInput};
    use bevy::prelude::*;

    #[test]
    fn button_inputs() {
        let mut world = mockable_world();
        world.insert_resource(Input::<KeyCode>::default());
        world.insert_resource(Input::<MouseButton>::default());
        world.insert_resource(Input::<GamepadButton>::default());

        // BLOCKED: cannot use the less artifical APIs due to
        // https://github.com/bevyengine/bevy/issues/3808
        let gamepad = Some(Gamepad { id: 0 });

        // Test that buttons are unpressed by default
        assert!(!world.pressed(KeyCode::Space));
        assert!(!world.pressed(MouseButton::Right));
        assert!(!world.pressed_for_gamepad(GamepadButtonType::North, gamepad));

        // Send inputs
        world.send_input(KeyCode::Space);
        world.send_input(MouseButton::Right);
        world.send_input_as_gamepad(GamepadButtonType::North, gamepad);

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
        use bevy::ecs::prelude::*;
        use bevy::ui::Interaction;

        #[derive(Component)]
        struct ButtonMarker;

        let mut world = mockable_world();
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

/// Generates a [`World`] with the resources required by [`InputStreams`]
///
/// This is exclusively useful for testing.
/// When working with [`App`], add [`InputPlugin`](bevy::input::InputPlugin) instead.
pub fn mockable_world() -> World {
    let mut world = World::new();
    world.init_resource::<Input<GamepadButton>>();
    world.init_resource::<Axis<GamepadButton>>();
    world.init_resource::<Axis<GamepadAxis>>();
    world.init_resource::<Gamepads>();
    world.init_resource::<Input<KeyCode>>();
    world.init_resource::<Input<MouseButton>>();
    world.init_resource::<Events<MouseWheel>>();
    world.init_resource::<Events<MouseMotion>>();
    world
}
