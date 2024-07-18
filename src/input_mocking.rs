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
/// use leafwing_input_manager::input_mocking::MockInput;
/// use leafwing_input_manager::prelude::*;
///
/// let mut app = App::new();
///
/// // This functionality requires Bevy's InputPlugin (included with DefaultPlugins)
/// app.add_plugins(InputPlugin);
///
/// // Press a key press directly.
/// app.press_input(KeyCode::KeyD);
///
/// // Or use chords to press multiple keys at the same time!
/// let bevy = [KeyCode::KeyB, KeyCode::KeyE, KeyCode::KeyV, KeyCode::KeyY];
/// app.press_input(ButtonlikeChord::new(bevy));
///
/// // Send values to an axis.
/// app.send_axis_values(MouseScrollAxis::Y, [5.0]);
///
/// // Send values to two axes.
/// app.send_axis_values(MouseMove::default(), [5.0, 8.0]);
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
    /// pressing all buttons and keys in the [`RawInputs`](crate::raw_inputs::RawInputs) of the `input`.
    ///
    /// To avoid confusing adjustments, it is best to stick with straightforward button-like inputs,
    /// like [`KeyCode`]s, [`ModifierKey`]s, and [`ButtonlikeChord`]s.
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
    fn press_input(&mut self, input: impl UserInput);

    /// Simulates an activated event for the given `input`, using the specified `gamepad`,
    /// pressing all buttons and keys in the [`RawInputs`](crate::raw_inputs::RawInputs) of the `input`.
    ///
    /// To avoid confusing adjustments, it is best to stick with straightforward button-like inputs,
    /// like [`KeyCode`]s, [`ModifierKey`]s, and [`ButtonlikeChord`]s.
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
    fn press_input_as_gamepad(&mut self, input: impl UserInput, gamepad: Option<Gamepad>);

    /// Simulates axis value changed events for the given `input`.
    /// Each value in the `values` iterator corresponds to an axis in the [`RawInputs`](crate::raw_inputs::RawInputs) of the `input`.
    /// Missing axis values default to `0.0`.
    ///
    /// To avoid confusing adjustments, it is best to stick with straightforward axis-like inputs
    /// like [`MouseScrollAxis::Y`], [`MouseMove`] and [`GamepadStick::LEFT`].
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
    fn send_axis_values(&mut self, input: impl UserInput, values: impl IntoIterator<Item = f32>);

    /// Simulates axis value changed events for the given `input`, using the specified `gamepad`.
    /// Each value in the `values` iterator corresponds to an axis in the [`RawInputs`](crate::raw_inputs::RawInputs) of the `input`.
    /// Missing axis values default to `0.0`.
    ///
    /// To avoid confusing adjustments, it is best to stick with straightforward axis-like inputs
    /// like [`MouseScrollAxis::Y`], [`MouseMove`] and [`GamepadStick::LEFT`].
    /// Non-axial inputs (e.g., keys and buttons) aren't affected;
    /// the current value will be retained for the next encountered axis.
    /// Use [`Self::press_input_as_gamepad`] for those.
    ///
    /// # Input State Persistence
    ///
    /// Each axis remains at the specified value until explicitly changed or by calling [`Self::reset_inputs`].
    fn send_axis_values_as_gamepad(
        &mut self,
        input: impl UserInput,
        values: impl IntoIterator<Item = f32>,
        gamepad: Option<Gamepad>,
    );

    /// Simulates a released or deactivated event for the given `input`.
    ///
    /// # Gamepad Input
    ///
    /// Gamepad input is sent by the first registered controller.
    /// If no controllers are found, it is silently ignored.
    fn release_input(&mut self, input: impl UserInput);

    /// Simulates a released or deactivated event for the given `input`, using the specified `gamepad`.
    fn release_input_as_gamepad(&mut self, input: impl UserInput, gamepad: Option<Gamepad>);

    /// Resets all inputs in the [`MutableInputStreams`] to their default state.
    ///
    /// All buttons are released, and `just_pressed` and `just_released` information on the [`ButtonInput`] type are lost.
    /// `just_pressed` and `just_released` on the [`ActionState`](crate::action_state::ActionState) will be kept.
    ///
    /// This will clear all [`KeyCode`], [`GamepadButton`] and [`MouseButton`] input streams,
    /// as well as any [`Interaction`] components and all input [`Events`].
    fn reset_inputs(&mut self);
}

/// Query input state directly for testing purposes.
///
/// In game code, you should (almost) always be using [`ActionState`](crate::action_state::ActionState)
/// methods instead.
///
/// # Examples
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy::input::InputPlugin;
/// use leafwing_input_manager::input_mocking::QueryInput;
/// use leafwing_input_manager::plugin::AccumulatorPlugin;
/// use leafwing_input_manager::prelude::*;
///
/// let mut app = App::new();
///
/// // This functionality requires Bevy's InputPlugin (included with DefaultPlugins)
/// app.add_plugins((InputPlugin, AccumulatorPlugin));
///
/// // Check if a key is currently pressed down.
/// let pressed = app.pressed(KeyCode::KeyB);
///
/// // Read the current vertical mouse scroll value.
/// let value = app.read_axis_value(MouseScrollAxis::Y);
///
/// // Read the current changes in relative mouse X and Y coordinates.
/// let values = app.read_dual_axis_values(MouseMove::default());
/// let x = values[0];
/// let y = values[1];
/// ```
pub trait QueryInput {
    /// Checks if the `input` is currently pressed or active.
    ///
    /// This method is intended as a convenience for testing;
    /// use an [`InputMap`](crate::input_map::InputMap) in real code.
    fn pressed(&self, input: impl Buttonlike) -> bool;

    /// Checks if the `input` is currently pressed or active on the specified [`Gamepad`].
    ///
    /// This method is intended as a convenience for testing;
    /// use an [`InputMap`](crate::input_map::InputMap) in real code.
    fn pressed_on_gamepad(&self, input: impl Buttonlike, gamepad: Option<Gamepad>) -> bool;

    /// Retrieves the value on the axis represented by the `input`.
    fn read_axis_value(&self, input: impl Axislike) -> f32;

    /// Retrieves the values on the axes represented by the `input`.
    ///
    /// This method is intended as a convenience for testing;
    /// use an [`InputMap`](crate::input_map::InputMap) in real code.
    fn read_dual_axis_values(&self, input: impl DualAxislike) -> Vec2;

    /// Retrieves the values on the axes represented by the `input` on the specified [`Gamepad`].
    ///
    /// This method is intended as a convenience for testing;
    /// use an [`InputMap`](crate::input_map::InputMap) in real code.
    fn read_dual_axis_values_on_gamepad(
        &self,
        input: impl DualAxislike,
        gamepad: Option<Gamepad>,
    ) -> Vec2;
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
    fn press_input(&mut self, input: impl UserInput) {
        self.press_input_as_gamepad(input, self.guess_gamepad());
    }

    fn press_input_as_gamepad(&mut self, input: impl UserInput, gamepad: Option<Gamepad>) {
        let raw_inputs = input.raw_inputs();

        // Press KeyCode
        for keycode in raw_inputs.keycodes.iter() {
            self.send_keycode_state(keycode, ButtonState::Pressed);
        }

        // Press MouseButton
        for button in raw_inputs.mouse_buttons.iter() {
            self.send_mouse_button_state(button, ButtonState::Pressed);
        }

        // Press MouseMoveDirection, discrete mouse motion events
        for direction in raw_inputs.mouse_move_directions.iter() {
            self.send_mouse_move(direction.0.full_active_value());
        }

        // Press MouseScrollDirection, discrete mouse wheel events
        for direction in raw_inputs.mouse_scroll_directions.iter() {
            self.send_mouse_scroll(direction.0.full_active_value());
        }

        if let Some(gamepad) = gamepad {
            for direction in raw_inputs.gamepad_control_directions.iter() {
                self.send_gamepad_axis_value(
                    gamepad,
                    &direction.axis,
                    direction.side.full_active_value(),
                );
            }

            // Press GamepadButtonType.
            // Unfortunately, due to upstream constraints, this has no effect
            // because Bevy currently disregards all external GamepadButtonChangedEvent events.
            // See <https://github.com/Leafwing-Studios/leafwing-input-manager/issues/516> for more details.
            for button in raw_inputs.gamepad_buttons.iter() {
                self.send_gamepad_button_state(gamepad, button, ButtonState::Pressed);
            }
        }
    }

    fn send_axis_values(&mut self, input: impl UserInput, values: impl IntoIterator<Item = f32>) {
        self.send_axis_values_as_gamepad(input, values, self.guess_gamepad())
    }

    fn send_axis_values_as_gamepad(
        &mut self,
        input: impl UserInput,
        values: impl IntoIterator<Item = f32>,
        gamepad: Option<Gamepad>,
    ) {
        let raw_inputs = input.raw_inputs();
        let mut value_iter = values.into_iter();

        if let Some(gamepad) = gamepad {
            for axis in raw_inputs.gamepad_axes.iter() {
                let value = value_iter.next().unwrap_or_default();
                self.send_gamepad_axis_value(gamepad, axis, value);
            }
        }

        for axis in raw_inputs.mouse_move_axes.iter() {
            let value = value_iter.next().unwrap_or_default();
            let value = axis.dual_axis_value(value);
            self.send_mouse_move(value);
        }

        for axis in raw_inputs.mouse_scroll_axes.iter() {
            let value = value_iter.next().unwrap_or_default();
            let value = axis.dual_axis_value(value);
            self.send_mouse_scroll(value);
        }
    }

    fn release_input(&mut self, input: impl UserInput) {
        self.release_input_as_gamepad(input, self.guess_gamepad())
    }

    fn release_input_as_gamepad(&mut self, input: impl UserInput, gamepad: Option<Gamepad>) {
        let raw_inputs = input.raw_inputs();

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
        if let Some(gamepad) = gamepad {
            for direction in raw_inputs.gamepad_control_directions.iter() {
                self.send_gamepad_axis_value(gamepad, &direction.axis, 0.0);
            }

            for axis in raw_inputs.gamepad_axes.iter() {
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
        *self.mouse_scroll = Default::default();
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

    fn send_mouse_scroll(&mut self, delta: Vec2) {
        let event = MouseWheel {
            unit: MouseScrollUnit::Pixel,
            x: delta.x,
            y: delta.y,
            window: Entity::PLACEHOLDER,
        };

        self.mouse_scroll_events.send(event);
    }

    fn send_mouse_move(&mut self, delta: Vec2) {
        let event = MouseMotion { delta };

        self.mouse_motion_events.send(event);
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
    #[inline]
    fn pressed(&self, input: impl Buttonlike) -> bool {
        input.pressed(self)
    }

    #[inline]
    fn pressed_on_gamepad(&self, input: impl Buttonlike, gamepad: Option<Gamepad>) -> bool {
        let mut input_streams = self.clone();
        input_streams.associated_gamepad = gamepad;

        input_streams.pressed(input)
    }

    fn read_axis_value(&self, input: impl Axislike) -> f32 {
        input.value(self)
    }

    #[inline]
    fn read_dual_axis_values(&self, input: impl DualAxislike) -> Vec2 {
        input.axis_pair(self)
    }

    fn read_dual_axis_values_on_gamepad(
        &self,
        input: impl DualAxislike,
        gamepad: Option<Gamepad>,
    ) -> Vec2 {
        let mut input_streams = self.clone();
        input_streams.associated_gamepad = gamepad;

        input_streams.read_dual_axis_values(input)
    }
}

impl MockInput for World {
    fn press_input(&mut self, input: impl UserInput) {
        let mut mutable_input_streams = MutableInputStreams::from_world(self, None);

        mutable_input_streams.press_input(input);
    }

    fn press_input_as_gamepad(&mut self, input: impl UserInput, gamepad: Option<Gamepad>) {
        let mut mutable_input_streams = MutableInputStreams::from_world(self, gamepad);

        mutable_input_streams.press_input_as_gamepad(input, gamepad);
    }

    fn send_axis_values(&mut self, input: impl UserInput, values: impl IntoIterator<Item = f32>) {
        let mut mutable_input_streams = MutableInputStreams::from_world(self, None);

        mutable_input_streams.send_axis_values(input, values);
    }

    fn send_axis_values_as_gamepad(
        &mut self,
        input: impl UserInput,
        values: impl IntoIterator<Item = f32>,
        gamepad: Option<Gamepad>,
    ) {
        let mut mutable_input_streams = MutableInputStreams::from_world(self, gamepad);

        mutable_input_streams.send_axis_values(input, values);
    }

    fn release_input(&mut self, input: impl UserInput) {
        let mut mutable_input_streams = MutableInputStreams::from_world(self, None);

        mutable_input_streams.release_input(input);
    }

    fn release_input_as_gamepad(&mut self, input: impl UserInput, gamepad: Option<Gamepad>) {
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
    fn pressed(&self, input: impl Buttonlike) -> bool {
        self.pressed_on_gamepad(input, None)
    }

    fn pressed_on_gamepad(&self, input: impl Buttonlike, gamepad: Option<Gamepad>) -> bool {
        let input_streams = InputStreams::from_world(self, gamepad);

        input_streams.pressed(input)
    }

    fn read_axis_value(&self, input: impl Axislike) -> f32 {
        let input_streams = InputStreams::from_world(self, None);

        input_streams.read_axis_value(input)
    }

    fn read_dual_axis_values(&self, input: impl DualAxislike) -> Vec2 {
        self.read_dual_axis_values_on_gamepad(input, None)
    }

    fn read_dual_axis_values_on_gamepad(
        &self,
        input: impl DualAxislike,
        gamepad: Option<Gamepad>,
    ) -> Vec2 {
        let input_streams = InputStreams::from_world(self, gamepad);

        input_streams.read_dual_axis_values(input)
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
    fn press_input(&mut self, input: impl UserInput) {
        self.world_mut().press_input(input);
    }

    fn press_input_as_gamepad(&mut self, input: impl UserInput, gamepad: Option<Gamepad>) {
        self.world_mut().press_input_as_gamepad(input, gamepad);
    }

    fn send_axis_values(&mut self, input: impl UserInput, values: impl IntoIterator<Item = f32>) {
        self.world_mut().send_axis_values(input, values);
    }

    fn send_axis_values_as_gamepad(
        &mut self,
        input: impl UserInput,
        values: impl IntoIterator<Item = f32>,
        gamepad: Option<Gamepad>,
    ) {
        self.world_mut()
            .send_axis_values_as_gamepad(input, values, gamepad);
    }

    fn release_input(&mut self, input: impl UserInput) {
        self.world_mut().release_input(input);
    }

    fn release_input_as_gamepad(&mut self, input: impl UserInput, gamepad: Option<Gamepad>) {
        self.world_mut().release_input_as_gamepad(input, gamepad);
    }

    fn reset_inputs(&mut self) {
        self.world_mut().reset_inputs();
    }
}

impl QueryInput for App {
    fn pressed(&self, input: impl Buttonlike) -> bool {
        self.world().pressed(input)
    }

    fn pressed_on_gamepad(&self, input: impl Buttonlike, gamepad: Option<Gamepad>) -> bool {
        self.world().pressed_on_gamepad(input, gamepad)
    }

    fn read_axis_value(&self, input: impl Axislike) -> f32 {
        self.world().read_axis_value(input)
    }

    fn read_dual_axis_values(&self, input: impl DualAxislike) -> Vec2 {
        self.world().read_dual_axis_values(input)
    }

    fn read_dual_axis_values_on_gamepad(
        &self,
        input: impl DualAxislike,
        gamepad: Option<Gamepad>,
    ) -> Vec2 {
        self.world()
            .read_dual_axis_values_on_gamepad(input, gamepad)
    }
}

#[cfg(feature = "ui")]
impl MockUIInteraction for App {
    fn click_button<Marker: Component>(&mut self) {
        self.world_mut().click_button::<Marker>();
    }

    fn hover_button<Marker: Component>(&mut self) {
        self.world_mut().hover_button::<Marker>();
    }
}

#[cfg(test)]
mod test {
    use crate::input_mocking::{MockInput, QueryInput};
    use crate::plugin::AccumulatorPlugin;
    use crate::user_input::*;
    use bevy::input::gamepad::{
        GamepadConnection, GamepadConnectionEvent, GamepadEvent, GamepadInfo,
    };
    use bevy::input::InputPlugin;
    use bevy::prelude::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(InputPlugin).add_plugins(AccumulatorPlugin);

        let gamepad = Gamepad::new(0);
        let mut gamepad_events = app.world_mut().resource_mut::<Events<GamepadEvent>>();
        gamepad_events.send(GamepadEvent::Connection(GamepadConnectionEvent {
            gamepad,
            connection: GamepadConnection::Connected(GamepadInfo {
                name: "TestController".into(),
            }),
        }));
        app.update();
        app.update();

        app
    }

    #[test]
    fn ordinary_button_inputs() {
        let mut app = test_app();

        // Test that buttons are unpressed by default
        assert!(!app.pressed(KeyCode::Space));
        assert!(!app.pressed(MouseButton::Right));

        // Press buttons
        app.press_input(KeyCode::Space);
        app.press_input(MouseButton::Right);
        app.update();

        // Verify that checking the resource value directly works
        let keyboard_input = app.world().resource::<ButtonInput<KeyCode>>();
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
        let mut app = test_app();
        let gamepad = Some(Gamepad::new(0));

        // Test that buttons are unpressed by default
        assert!(!app.pressed_on_gamepad(GamepadButtonType::North, gamepad));

        // Press buttons
        app.press_input_as_gamepad(GamepadButtonType::North, gamepad);
        app.update();

        // Verify the button are pressed
        assert!(app.pressed_on_gamepad(GamepadButtonType::North, gamepad));

        // Test that resetting inputs works
        app.reset_inputs();
        app.update();

        // Verify the button are released
        assert!(!app.pressed_on_gamepad(GamepadButtonType::North, gamepad));
    }

    #[test]
    fn implicit_gamepad_button_inputs() {
        let mut app = test_app();

        // Test that buttons are unpressed by default
        assert!(!app.pressed(GamepadButtonType::North));

        // Press buttons
        app.press_input(GamepadButtonType::North);
        app.update();

        // Verify the button are pressed
        assert!(app.pressed(GamepadButtonType::North));

        // Test that resetting inputs works
        app.reset_inputs();
        app.update();

        // Verify the button are released
        assert!(!app.pressed(GamepadButtonType::North));
    }

    #[test]
    fn mouse_inputs() {
        let mut app = test_app();

        // Mouse axes should be inactive by default (no scroll or movement)
        assert_eq!(
            app.read_dual_axis_values(MouseMove::default()),
            Vec2::default()
        );
        assert_eq!(
            app.read_dual_axis_values(MouseScroll::default()),
            Vec2::default()
        );

        // Send a simulated mouse scroll event with a value of 3 (positive for up)
        app.send_axis_values(MouseScrollAxis::Y, [3.0]);
        app.update();

        // Verify the mouse wheel Y axis reflects the simulated scroll
        // and the other axis isn't affected
        assert_eq!(app.read_axis_value(MouseScrollAxis::X), 0.0);
        assert_eq!(app.read_axis_value(MouseScrollAxis::Y), 3.0);
        assert_eq!(
            app.read_dual_axis_values(MouseScroll::default()),
            Vec2::new(0.0, 3.0),
        );

        // Send a simulated mouse movement event with a delta of (3.0, 2.0)
        app.send_axis_values(MouseScroll::default(), [3.0, 2.0]);
        app.update();

        // Verify the mouse motion axes reflects the simulated movement
        assert_eq!(
            app.read_dual_axis_values(MouseScroll::default()),
            Vec2::new(3.0, 2.0),
        );

        // Mouse input data is typically reset every frame
        // Verify other axes aren't affected
        assert_eq!(
            app.read_dual_axis_values(MouseMove::default()),
            Vec2::default()
        );

        // Test that resetting inputs works
        app.reset_inputs();
        app.update();

        // Verify all axes have no value after reset
        assert_eq!(app.read_axis_value(MouseScrollAxis::Y), 0.0);
        assert_eq!(
            app.read_dual_axis_values(MouseScroll::default()),
            Vec2::ZERO,
        );
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
        app.world_mut().spawn((Interaction::None, ButtonMarker));

        // Unmarked button
        app.world_mut().spawn(Interaction::None);

        // Click the button
        app.world_mut().click_button::<ButtonMarker>();
        app.update();

        let mut interaction_query = app
            .world_mut()
            .query::<(&Interaction, Option<&ButtonMarker>)>();
        for (interaction, maybe_marker) in interaction_query.iter(app.world()) {
            match maybe_marker {
                Some(_) => assert_eq!(*interaction, Interaction::Pressed),
                None => assert_eq!(*interaction, Interaction::None),
            }
        }

        // Reset inputs
        app.world_mut().reset_inputs();

        let mut interaction_query = app.world_mut().query::<&Interaction>();
        for interaction in interaction_query.iter(app.world()) {
            assert_eq!(*interaction, Interaction::None)
        }

        // Hover over the button
        app.hover_button::<ButtonMarker>();
        app.update();

        let mut interaction_query = app
            .world_mut()
            .query::<(&Interaction, Option<&ButtonMarker>)>();
        for (interaction, maybe_marker) in interaction_query.iter(app.world()) {
            match maybe_marker {
                Some(_) => assert_eq!(*interaction, Interaction::Hovered),
                None => assert_eq!(*interaction, Interaction::None),
            }
        }

        // Reset inputs
        app.world_mut().reset_inputs();

        let mut interaction_query = app.world_mut().query::<&Interaction>();
        for interaction in interaction_query.iter(app.world()) {
            assert_eq!(*interaction, Interaction::None)
        }
    }
}
