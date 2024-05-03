//! Helpful utilities for testing input management by sending mock input events
//!
//! The [`MockInput`] trait contains methods with the same API that operate at three levels:
//!
//! 1. [`App`].
//! 2. [`World`].
//! 3. [`MutableInputStreams`].
//!
//! Each passing down the supplied arguments to the next.

use bevy::ecs::system::SystemState;
use bevy::input::gamepad::{Gamepad, GamepadButton, GamepadButtonType, GamepadEvent};
use bevy::input::gamepad::{GamepadAxisChangedEvent, GamepadButtonChangedEvent};
use bevy::input::keyboard::{Key, KeyCode, KeyboardInput, NativeKey};
use bevy::input::mouse::{MouseButton, MouseButtonInput, MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::input::touch::{TouchInput, Touches};
use bevy::input::{ButtonInput, ButtonState};
use bevy::prelude::{App, Entity, Events, GamepadAxisType, ResMut, Vec2, World};
use bevy::window::CursorMoved;

use crate::axislike::AxisType;
#[cfg(feature = "ui")]
use bevy::ecs::{component::Component, query::With, system::Query};
#[cfg(feature = "ui")]
use bevy::ui::Interaction;

use crate::input_streams::{InputStreams, MutableInputStreams};
use crate::user_input::*;

/// Send fake input events for testing purposes
///
/// In game code, you should (almost) always be setting the [`ActionState`](crate::action_state::ActionState)
/// directly instead.
///
/// # Warning
///
/// You *must* call [`app.update()`](App::update) at least once after sending input
/// with [`InputPlugin`](bevy::input::InputPlugin) included in your plugin set
/// for the raw input events to be processed into [`ButtonInput`] and [`Axis`](bevy::prelude::Axis) data.
///
/// # Examples
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy::input::InputPlugin;
/// use leafwing_input_manager::axislike::MouseMotionAxisType;
/// use leafwing_input_manager::input_mocking::MockInput;
/// use leafwing_input_manager::prelude::*;
///
/// let mut app = App::new();
///
/// // This functionality requires Bevy's InputPlugin to be present in your plugin set.
/// // If you included the `DefaultPlugins`, then InputPlugin is already included.
/// app.add_plugins(InputPlugin);
///
/// // Press a key press directly.
/// app.press_input(KeyCode::KeyD);
///
/// // Or use chords to press multiple keys at the same time!
/// let bevy = [KeyCode::KeyB, KeyCode::KeyE, KeyCode::KeyV, KeyCode::KeyY];
/// app.press_input(UserInput::chord(bevy));
///
/// // Send values to an axis.
/// app.send_axis_values(SingleAxis::mouse_wheel_y(), [5.0]);
///
/// // Send values to two axes.
/// app.send_axis_values(DualAxis::mouse_motion(), [5.0, 8.0]);
///
/// // Release or deactivate an input.
/// app.release_input(KeyCode::KeyR);
///
/// // Reset all inputs to their default state.
/// app.reset_inputs();
///
/// // Remember to call the update method at least once after sending input.
/// app.update();
/// ```
pub trait MockInput {
    /// Simulates an activated event for the given `input`,
    /// pressing all buttons and keys in the [`RawInputs`] of the `input`.
    ///
    /// To avoid confusing adjustments, it is best to stick with straightforward button-like inputs,
    /// like [`KeyCode`]s, [`Modifier`]s, and [`UserInput::Chord`]s.
    /// Axial inputs (e.g., analog thumb sticks) aren't affected.
    /// Use [`Self::send_axis_values`] for those.
    ///
    /// # Input State Persistence
    ///
    /// Pressed inputs remain active until explicitly released or by calling [`Self::reset_inputs`].
    ///
    /// # Gamepad Input
    ///
    /// Gamepad input is sent by the first registered controller.
    /// If no controllers are found, it is silently ignored.
    ///
    /// # Limitations
    ///
    /// Unfortunately, due to upstream constraints,
    /// pressing a [`GamepadButtonType`] has no effect
    /// because Bevy currently disregards all external [`GamepadButtonChangedEvent`] events.
    /// See <https://github.com/Leafwing-Studios/leafwing-input-manager/issues/516> for more details.
    fn press_input(&mut self, input: impl Into<UserInput>);

    /// Simulates an activated event for the given `input`, using the specified `gamepad`,
    /// pressing all buttons and keys in the [`RawInputs`] of the `input`.
    ///
    /// To avoid confusing adjustments, it is best to stick with straightforward button-like inputs,
    /// like [`KeyCode`]s, [`Modifier`]s, and [`UserInput::Chord`]s.
    /// Axial inputs (e.g., analog thumb sticks) aren't affected.
    /// Use [`Self::send_axis_values_as_gamepad`] for those.
    ///
    /// # Input State Persistence
    ///
    /// Pressed inputs remain active until explicitly released or by calling [`Self::reset_inputs`].
    ///
    /// # Limitations
    ///
    /// Unfortunately, due to upstream constraints,
    /// pressing a [`GamepadButtonType`] has no effect
    /// because Bevy currently disregards all external [`GamepadButtonChangedEvent`] events.
    /// See <https://github.com/Leafwing-Studios/leafwing-input-manager/issues/516> for more details.
    fn press_input_as_gamepad(&mut self, input: impl Into<UserInput>, gamepad: Option<Gamepad>);

    /// Simulates axis value changed events for the given `input`.
    /// Each value in the `values` iterator corresponds to an axis in the [`RawInputs`] of the `input`.
    /// Missing axis values default to `0.0`.
    ///
    /// To avoid confusing adjustments, it is best to stick with straightforward axis-like inputs
    /// like [`SingleAxis`](crate::axislike::SingleAxis) and [`DualAxis`](crate::axislike::DualAxis).
    /// Non-axial inputs (e.g., keys and buttons) aren't affected;
    /// the current value will be retained for the next encountered axis.
    /// Use [`Self::press_input`] for those.
    ///
    /// # Input State Persistence
    ///
    /// Each axis remains at the specified value until explicitly changed or by calling [`Self::reset_inputs`].
    ///
    /// # Gamepad Input
    ///
    /// Gamepad input is sent by the first registered controller.
    /// If no controllers are found, it is silently ignored.
    fn send_axis_values(
        &mut self,
        input: impl Into<UserInput>,
        values: impl IntoIterator<Item = f32>,
    );

    /// Simulates axis value changed events for the given `input`, using the specified `gamepad`.
    /// Each value in the `values` iterator corresponds to an axis in the [`RawInputs`] of the `input`.
    /// Missing axis values default to `0.0`.
    ///
    /// To avoid confusing adjustments, it is best to stick with straightforward axis-like inputs
    /// like [`SingleAxis`](crate::axislike::SingleAxis) and [`DualAxis`](crate::axislike::DualAxis).
    /// Non-axial inputs (e.g., keys and buttons) aren't affected;
    /// the current value will be retained for the next encountered axis.
    /// Use [`Self::press_input_as_gamepad`] for those.
    ///
    /// # Input State Persistence
    ///
    /// Each axis remains at the specified value until explicitly changed or by calling [`Self::reset_inputs`].
    fn send_axis_values_as_gamepad(
        &mut self,
        input: impl Into<UserInput>,
        values: impl IntoIterator<Item = f32>,
        gamepad: Option<Gamepad>,
    );

    /// Simulates a released or deactivated event for the given `input`.
    ///
    /// # Gamepad Input
    ///
    /// Gamepad input is sent by the first registered controller.
    /// If no controllers are found, it is silently ignored.
    fn release_input(&mut self, input: impl Into<UserInput>);

    /// Simulates a released or deactivated event for the given `input`, using the specified `gamepad`.
    fn release_input_as_gamepad(&mut self, input: impl Into<UserInput>, gamepad: Option<Gamepad>);

    /// Resets all inputs in the [`MutableInputStreams`] to their default state.
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
    /// Checks if the `input` is currently pressed.
    ///
    /// This method is intended as a convenience for testing; check the [`ButtonInput`] resource directly,
    /// or use an [`InputMap`](crate::input_map::InputMap) in real code.
    fn pressed(&self, input: impl Into<UserInput>) -> bool;

    /// Checks if the `input` is currently pressed the specified [`Gamepad`].
    ///
    /// This method is intended as a convenience for testing; check the [`ButtonInput`] resource directly,
    /// or use an [`InputMap`](crate::input_map::InputMap) in real code.
    fn pressed_on_gamepad(&self, input: impl Into<UserInput>, gamepad: Option<Gamepad>) -> bool;
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
    fn press_input(&mut self, input: impl Into<UserInput>) {
        self.press_input_as_gamepad(input, self.guess_gamepad());
    }

    fn press_input_as_gamepad(&mut self, input: impl Into<UserInput>, gamepad: Option<Gamepad>) {
        let raw_inputs = input.into().raw_inputs();

        // Press KeyCode
        for keycode in raw_inputs.keycodes.iter() {
            self.send_keycode_state(keycode, ButtonState::Pressed);
        }

        // Press MouseButton
        for button in raw_inputs.mouse_buttons.iter() {
            self.send_mouse_button_state(button, ButtonState::Pressed);
        }

        // Press MouseMotionDirection, discrete mouse motion events
        for direction in raw_inputs.mouse_motion.iter() {
            self.send_mouse_move(direction.direction().full_active_value());
        }

        // Press MouseWheelDirection, discrete mouse wheel events
        for direction in raw_inputs.mouse_wheel.iter() {
            self.send_mouse_wheel(direction.direction().full_active_value());
        }

        // Press GamepadButtonType.
        // Unfortunately, due to upstream constraints, this has no effect
        // because Bevy currently disregards all external GamepadButtonChangedEvent events.
        // See <https://github.com/Leafwing-Studios/leafwing-input-manager/issues/516> for more details.
        if let Some(gamepad) = gamepad {
            for button in raw_inputs.gamepad_buttons.iter() {
                self.send_gamepad_button_state(gamepad, button, ButtonState::Pressed);
            }
        }
    }

    fn send_axis_values(
        &mut self,
        input: impl Into<UserInput>,
        values: impl IntoIterator<Item = f32>,
    ) {
        self.send_axis_values_as_gamepad(input, values, self.guess_gamepad())
    }

    fn send_axis_values_as_gamepad(
        &mut self,
        input: impl Into<UserInput>,
        values: impl IntoIterator<Item = f32>,
        gamepad: Option<Gamepad>,
    ) {
        let raw_inputs = input.into().raw_inputs();
        let mut value_iter = values.into_iter();

        for axis_types in raw_inputs.axis_types.iter() {
            match axis_types {
                AxisType::Gamepad(axis) => {
                    if let Some(gamepad) = gamepad {
                        let value = value_iter.next().unwrap_or_default();
                        self.send_gamepad_axis_value(gamepad, axis, value);
                    }
                }
                AxisType::MouseMotion(axis) => {
                    let value = value_iter.next().unwrap_or_default();
                    let value = axis.axis().dual_axis_value(value);
                    self.send_mouse_move(value);
                }
                AxisType::MouseWheel(axis) => {
                    let value = value_iter.next().unwrap_or_default();
                    let value = axis.axis().dual_axis_value(value);
                    self.send_mouse_wheel(value);
                }
            }
        }
    }

    fn release_input(&mut self, input: impl Into<UserInput>) {
        self.release_input_as_gamepad(input, self.guess_gamepad())
    }

    fn release_input_as_gamepad(&mut self, input: impl Into<UserInput>, gamepad: Option<Gamepad>) {
        let raw_inputs = input.into().raw_inputs();

        // Release KeyCode
        for keycode in raw_inputs.keycodes.iter() {
            self.send_keycode_state(keycode, ButtonState::Released);
        }

        // Release MouseButton
        for button in raw_inputs.mouse_buttons.iter() {
            self.send_mouse_button_state(button, ButtonState::Released);
        }

        // Release GamepadButtonType.
        // Unfortunately, due to upstream constraints, this has no effect
        // because Bevy currently disregards all external GamepadButtonChangedEvent events.
        // See <https://github.com/Leafwing-Studios/leafwing-input-manager/issues/516> for more details.
        if let Some(gamepad) = gamepad {
            for button in raw_inputs.gamepad_buttons.iter() {
                self.send_gamepad_button_state(gamepad, button, ButtonState::Released);
            }
        }

        // Deactivate GamepadAxisType
        for axis_type in raw_inputs.axis_types.iter() {
            if let (Some(gamepad), AxisType::Gamepad(axis)) = (gamepad, axis_type) {
                self.send_gamepad_axis_value(gamepad, axis, 0.0);
            }
        }

        // Mouse axial inputs don't require an explicit deactivating,
        // as we directly check the state by reading the mouse input events.
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
    fn send_keycode_state(&mut self, keycode: &KeyCode, state: ButtonState) {
        self.keyboard_events.send(KeyboardInput {
            logical_key: Key::Unidentified(NativeKey::Unidentified),
            key_code: *keycode,
            state,
            window: Entity::PLACEHOLDER,
        });
    }

    fn send_mouse_button_state(&mut self, button: &MouseButton, state: ButtonState) {
        self.mouse_button_events.send(MouseButtonInput {
            button: *button,
            state,
            window: Entity::PLACEHOLDER,
        });
    }

    fn send_mouse_wheel(&mut self, delta: Vec2) {
        self.mouse_wheel.send(MouseWheel {
            x: delta.x,
            y: delta.y,
            // FIXME: MouseScrollUnit is not recorded and is always assumed to be Pixel
            unit: MouseScrollUnit::Pixel,
            window: Entity::PLACEHOLDER,
        });
    }

    fn send_mouse_move(&mut self, delta: Vec2) {
        self.mouse_motion.send(MouseMotion { delta });
    }

    fn send_gamepad_button_state(
        &mut self,
        gamepad: Gamepad,
        button_type: &GamepadButtonType,
        state: ButtonState,
    ) {
        let value = f32::from(state == ButtonState::Pressed);
        let event = GamepadButtonChangedEvent::new(gamepad, *button_type, value);
        self.gamepad_events.send(GamepadEvent::Button(event));
    }

    fn send_gamepad_axis_value(
        &mut self,
        gamepad: Gamepad,
        axis_type: &GamepadAxisType,
        value: f32,
    ) {
        let event = GamepadAxisChangedEvent::new(gamepad, *axis_type, value);
        self.gamepad_events.send(GamepadEvent::Axis(event));
    }
}

impl QueryInput for InputStreams<'_> {
    fn pressed(&self, input: impl Into<UserInput>) -> bool {
        self.input_pressed(&input.into())
    }

    fn pressed_on_gamepad(&self, input: impl Into<UserInput>, gamepad: Option<Gamepad>) -> bool {
        let mut input_streams = self.clone();
        input_streams.associated_gamepad = gamepad;

        input_streams.input_pressed(&input.into())
    }
}

impl MockInput for World {
    fn press_input(&mut self, input: impl Into<UserInput>) {
        let mut mutable_input_streams = MutableInputStreams::from_world(self, None);

        mutable_input_streams.press_input(input);
    }

    fn press_input_as_gamepad(&mut self, input: impl Into<UserInput>, gamepad: Option<Gamepad>) {
        let mut mutable_input_streams = MutableInputStreams::from_world(self, gamepad);

        mutable_input_streams.press_input_as_gamepad(input, gamepad);
    }

    fn send_axis_values(
        &mut self,
        input: impl Into<UserInput>,
        values: impl IntoIterator<Item = f32>,
    ) {
        let mut mutable_input_streams = MutableInputStreams::from_world(self, None);

        mutable_input_streams.send_axis_values(input, values);
    }

    fn send_axis_values_as_gamepad(
        &mut self,
        input: impl Into<UserInput>,
        values: impl IntoIterator<Item = f32>,
        gamepad: Option<Gamepad>,
    ) {
        let mut mutable_input_streams = MutableInputStreams::from_world(self, gamepad);

        mutable_input_streams.send_axis_values(input, values);
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
        self.pressed_on_gamepad(input, None)
    }

    fn pressed_on_gamepad(&self, input: impl Into<UserInput>, gamepad: Option<Gamepad>) -> bool {
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
    fn press_input(&mut self, input: impl Into<UserInput>) {
        self.world.press_input(input);
    }

    fn press_input_as_gamepad(&mut self, input: impl Into<UserInput>, gamepad: Option<Gamepad>) {
        self.world.press_input_as_gamepad(input, gamepad);
    }

    fn send_axis_values(
        &mut self,
        input: impl Into<UserInput>,
        values: impl IntoIterator<Item = f32>,
    ) {
        self.world.send_axis_values(input, values);
    }

    fn send_axis_values_as_gamepad(
        &mut self,
        input: impl Into<UserInput>,
        values: impl IntoIterator<Item = f32>,
        gamepad: Option<Gamepad>,
    ) {
        self.world
            .send_axis_values_as_gamepad(input, values, gamepad);
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

    fn pressed_on_gamepad(&self, input: impl Into<UserInput>, gamepad: Option<Gamepad>) -> bool {
        self.world.pressed_on_gamepad(input, gamepad)
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

        // Press buttons
        app.press_input(KeyCode::Space);
        app.press_input(MouseButton::Right);
        app.update();

        // Verify that checking the resource value directly works
        let keyboard_input = app.world.resource::<ButtonInput<KeyCode>>();
        assert!(keyboard_input.pressed(KeyCode::Space));

        // Test the convenient `pressed` API
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
        assert!(!app.pressed_on_gamepad(GamepadButtonType::North, Some(gamepad)));

        // Press buttons
        app.press_input_as_gamepad(GamepadButtonType::North, Some(gamepad));
        app.update();

        // Test that resetting inputs works
        app.reset_inputs();
        app.update();

        assert!(!app.pressed_on_gamepad(GamepadButtonType::North, Some(gamepad)));
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

        // Press buttons
        app.press_input(GamepadButtonType::North);
        app.update();

        // Test the convenient `pressed` API
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
