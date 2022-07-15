//! Unified input streams for working with [`bevy_input`] data.

use bevy_input::{
    gamepad::{Gamepad, GamepadAxis, GamepadButton, Gamepads},
    keyboard::KeyCode,
    mouse::{MouseButton, MouseWheel},
    Axis, Input,
};
use petitset::PetitSet;

use bevy_ecs::prelude::{Events, Res, ResMut, World};
use bevy_ecs::system::SystemState;

use crate::axislike::{AxisType, DualAxisData, MouseWheelAxisType, VirtualDPad};
use crate::buttonlike::MouseWheelDirection;
use crate::user_input::{InputKind, UserInput};

/// A collection of [`Input`] structs, which can be used to update an [`InputMap`](crate::input_map::InputMap).
///
/// Each of these streams is optional; if a stream does not exist, it is treated as if it were entirely unpressed.
///
/// These are typically collected via a system from the [`World`](bevy::prelude::World) as resources.
#[derive(Debug, Clone)]
pub struct InputStreams<'a> {
    /// An optional [`GamepadButton`] [`Input`] stream
    pub gamepad_buttons: Option<&'a Input<GamepadButton>>,
    /// An optional [`GamepadButton`] [`Axis`] stream
    pub gamepad_button_axes: Option<&'a Axis<GamepadButton>>,
    /// An optional [`GamepadAxis`] [`Axis`] stream
    pub gamepad_axes: Option<&'a Axis<GamepadAxis>>,
    /// An optional list of registered gamepads
    pub gamepads: Option<&'a Gamepads>,
    /// An optional [`KeyCode`] [`Input`] stream
    pub keyboard: Option<&'a Input<KeyCode>>,
    /// An optional [`MouseButton`] [`Input`] stream
    pub mouse: Option<&'a Input<MouseButton>>,
    /// An optional [`MouseWheel`] event stream
    pub mouse_wheel: Option<&'a Events<MouseWheel>>,
    /// The [`Gamepad`] that this struct will detect inputs from
    pub associated_gamepad: Option<Gamepad>,
}

// Constructors
impl<'a> InputStreams<'a> {
    /// Construct an [`InputStreams`] from a [`World`]
    pub fn from_world(world: &'a mut World, gamepad: Option<Gamepad>) -> Self {
        let mut input_system_state: SystemState<(
            Option<Res<Input<GamepadButton>>>,
            Option<Res<Axis<GamepadButton>>>,
            Option<Res<Axis<GamepadAxis>>>,
            Option<Res<Gamepads>>,
            Option<Res<Input<KeyCode>>>,
            Option<Res<Input<MouseButton>>>,
            Option<Res<Events<MouseWheel>>>,
        )> = SystemState::new(world);

        let (
            maybe_gamepad_buttons,
            maybe_gamepad_button_axes,
            maybe_gamepad_axes,
            maybe_gamepads,
            maybe_keyboard,
            maybe_mouse,
            maybe_mouse_wheel,
        ) = input_system_state.get(world);

        InputStreams {
            gamepad_buttons: maybe_gamepad_buttons.map(|r| r.into_inner()),
            gamepad_button_axes: maybe_gamepad_button_axes.map(|r| r.into_inner()),
            gamepad_axes: maybe_gamepad_axes.map(|r| r.into_inner()),
            gamepads: maybe_gamepads.map(|r| r.into_inner()),
            keyboard: maybe_keyboard.map(|r| r.into_inner()),
            mouse: maybe_mouse.map(|r| r.into_inner()),
            mouse_wheel: maybe_mouse_wheel.map(|r| r.into_inner()),
            associated_gamepad: gamepad,
        }
    }

    /// Construct [`InputStreams`] with only a [`GamepadButton`] input stream
    pub fn from_gamepad(
        gamepad_button_stream: &'a Input<GamepadButton>,
        gamepad_button_axis_stream: &'a Axis<GamepadButton>,
        gamepad_axis_stream: &'a Axis<GamepadAxis>,
        associated_gamepad: Gamepad,
    ) -> Self {
        Self {
            gamepad_buttons: Some(gamepad_button_stream),
            gamepad_button_axes: Some(gamepad_button_axis_stream),
            gamepad_axes: Some(gamepad_axis_stream),
            gamepads: None,
            keyboard: None,
            mouse: None,
            mouse_wheel: None,
            associated_gamepad: Some(associated_gamepad),
        }
    }

    /// Construct [`InputStreams`] with only a [`KeyCode`] input stream
    pub fn from_keyboard(keyboard_input_stream: &'a Input<KeyCode>) -> Self {
        Self {
            gamepad_buttons: None,
            gamepad_button_axes: None,
            gamepad_axes: None,
            gamepads: None,
            keyboard: Some(keyboard_input_stream),
            mouse: None,
            mouse_wheel: None,
            associated_gamepad: None,
        }
    }

    /// Construct [`InputStreams`] with only a [`GamepadButton`] input stream
    pub fn from_mouse(
        mouse_input_stream: &'a Input<MouseButton>,
        mouse_wheel_events: &'a Events<MouseWheel>,
    ) -> Self {
        Self {
            gamepad_buttons: None,
            gamepad_button_axes: None,
            gamepad_axes: None,
            gamepads: None,
            keyboard: None,
            mouse: Some(mouse_input_stream),
            mouse_wheel: Some(mouse_wheel_events),
            associated_gamepad: None,
        }
    }
}

// Input checking
impl<'a> InputStreams<'a> {
    /// Is the `input` matched by the [`InputStreams`]?
    pub fn input_pressed(&self, input: &UserInput) -> bool {
        match input {
            UserInput::Single(button) => self.button_pressed(*button),
            UserInput::Chord(buttons) => self.all_buttons_pressed(buttons),
            UserInput::VirtualDPad(VirtualDPad {
                up,
                down,
                left,
                right,
            }) => {
                for button in [up, down, left, right] {
                    if self.button_pressed(*button) {
                        return true;
                    }
                }
                false
            }
        }
    }

    /// Is at least one of the `inputs` pressed?
    #[must_use]
    pub fn any_pressed(&self, inputs: &PetitSet<UserInput, 16>) -> bool {
        for input in inputs.iter() {
            if self.input_pressed(input) {
                return true;
            }
        }
        // If none of the inputs matched, return false
        false
    }

    /// Is the `button` pressed?
    #[must_use]
    pub fn button_pressed(&self, button: InputKind) -> bool {
        match button {
            InputKind::DualAxis(_) => {
                let axis_pair = self.input_axis_pair(&UserInput::Single(button)).unwrap();

                axis_pair.length() != 0.0
            }
            InputKind::SingleAxis(axis) => {
                let value = self.input_value(&UserInput::Single(button));

                value < axis.negative_low || value > axis.positive_low
            }
            InputKind::GamepadButton(gamepad_button) => {
                // If a gamepad was registered, just check that one
                if let Some(gamepad) = self.associated_gamepad {
                    if let Some(gamepad_buttons) = self.gamepad_buttons {
                        gamepad_buttons.pressed(GamepadButton {
                            gamepad,
                            button_type: gamepad_button,
                        })
                    } else {
                        false
                    }
                // If no gamepad is registered, scan the list of available gamepads
                } else {
                    // Verify that both gamepads and gamepad_buttons exists
                    if let (Some(gamepads), Some(gamepad_buttons)) =
                        (self.gamepads, self.gamepad_buttons)
                    {
                        for &gamepad in gamepads.iter() {
                            if gamepad_buttons.pressed(GamepadButton {
                                gamepad,
                                button_type: gamepad_button,
                            }) {
                                // Return early if *any* gamepad is pressing this button
                                return true;
                            }
                        }
                        // If none of the available gamepads pressed this button, return false
                        false

                    // If we don't have the required data, fall back to false
                    } else {
                        false
                    }
                }
            }
            InputKind::Keyboard(keycode) => {
                if let Some(keyboard_stream) = self.keyboard {
                    keyboard_stream.pressed(keycode)
                } else {
                    false
                }
            }
            InputKind::Mouse(mouse_button) => {
                if let Some(mouse_stream) = self.mouse {
                    mouse_stream.pressed(mouse_button)
                } else {
                    false
                }
            }
            InputKind::MouseWheel(mouse_wheel_direction) => {
                if let Some(mouse_wheel_events) = self.mouse_wheel {
                    let mut total_mouse_wheel_movement = 0.0;

                    // FIXME: verify that this works and doesn't double count events
                    let mut event_reader = mouse_wheel_events.get_reader();

                    // PERF: this summing is computed for every individual input
                    // This should probably be computed once, and then cached / read
                    // Fix upstream!
                    for mouse_wheel_event in event_reader.iter(mouse_wheel_events) {
                        total_mouse_wheel_movement += match mouse_wheel_direction {
                            MouseWheelDirection::Up | MouseWheelDirection::Down => {
                                mouse_wheel_event.y
                            }
                            MouseWheelDirection::Left | MouseWheelDirection::Right => {
                                mouse_wheel_event.x
                            }
                        }
                    }

                    match mouse_wheel_direction {
                        MouseWheelDirection::Up | MouseWheelDirection::Right => {
                            total_mouse_wheel_movement > 0.0
                        }
                        MouseWheelDirection::Down | MouseWheelDirection::Left => {
                            total_mouse_wheel_movement < 0.0
                        }
                    }
                } else {
                    false
                }
            }
        }
    }

    /// Are all of the `buttons` pressed?
    #[must_use]
    pub fn all_buttons_pressed(&self, buttons: &PetitSet<InputKind, 8>) -> bool {
        for &button in buttons.iter() {
            // If any of the appropriate inputs failed to match, the action is considered pressed
            if !self.button_pressed(button) {
                return false;
            }
        }
        // If none of the inputs failed to match, return true
        true
    }

    /// Get the "value" of the input.
    ///
    /// For binary inputs such as buttons, this will always be either `0.0` or `1.0`. For analog
    /// inputs such as axes, this will be the axis value.
    ///
    /// [`UserInput::Chord`] inputs are also considered binary and will return `0.0` or `1.0` based
    /// on whether the chord has been pressed.
    ///
    /// # Warning
    ///
    /// If you need to ensure that this value is always in the range `[-1., 1.]`,
    /// be sure to clamp the reutrned data.
    pub fn input_value(&self, input: &UserInput) -> f32 {
        let use_button_value = || -> f32 {
            if self.input_pressed(input) {
                1.0
            } else {
                0.0
            }
        };

        match input {
            UserInput::Single(InputKind::SingleAxis(single_axis)) => {
                match single_axis.axis_type {
                    AxisType::Gamepad(axis_type) => {
                        if let Some(axes) = self.gamepad_axes {
                            if let Some(gamepad) = self.associated_gamepad {
                                axes.get(GamepadAxis { gamepad, axis_type })
                                    .unwrap_or_default()
                            // If no gamepad is registered, return the first non-zero input found
                            } else if let Some(gamepads) = self.gamepads {
                                for &gamepad in gamepads.iter() {
                                    let value = axes
                                        .get(GamepadAxis {
                                            gamepad,
                                            axis_type: single_axis.axis_type.try_into().unwrap(),
                                        })
                                        .unwrap_or_default();

                                    if value != 0.0 {
                                        // A matching input was pressed on a gamepad
                                        return value;
                                    }
                                }

                                // No input was pressed on any gamepad
                                0.0
                            } else {
                                // No Gamepads resource found and no gamepad was registered
                                0.0
                            }
                        } else {
                            // No Axis<GamepadButton> was found
                            use_button_value()
                        }
                    }
                    AxisType::MouseWheel(axis_type) => {
                        // Mouse wheel events are summed to get the total movement this frame
                        let mut total_mouse_wheel_movement = 0.0;
                        if let Some(mouse_wheel_events) = self.mouse_wheel {
                            // FIXME: verify that this works and doesn't double count events
                            let mut event_reader = mouse_wheel_events.get_reader();

                            for mouse_wheel_event in event_reader.iter(mouse_wheel_events) {
                                total_mouse_wheel_movement += match axis_type {
                                    MouseWheelAxisType::X => mouse_wheel_event.x,
                                    MouseWheelAxisType::Y => mouse_wheel_event.y,
                                }
                            }
                        }
                        total_mouse_wheel_movement
                    }
                }
            }
            UserInput::Single(InputKind::DualAxis(_)) => {
                if self.gamepad_axes.is_some() {
                    self.input_axis_pair(input).unwrap_or_default().length()
                } else {
                    0.0
                }
            }
            UserInput::VirtualDPad { .. } => {
                self.input_axis_pair(input).unwrap_or_default().length()
            }
            // This is required because upstream bevy_input still waffles about whether triggers are buttons or axes
            UserInput::Single(InputKind::GamepadButton(button_type)) => {
                if let Some(button_axes) = self.gamepad_button_axes {
                    if let Some(gamepad) = self.associated_gamepad {
                        // Get the value from the registered gamepad
                        button_axes
                            .get(GamepadButton {
                                gamepad,
                                button_type: *button_type,
                            })
                            .unwrap_or_else(use_button_value)
                    } else if let Some(gamepads) = self.gamepads {
                        for &gamepad in gamepads.iter() {
                            let value = button_axes
                                .get(GamepadButton {
                                    gamepad,
                                    button_type: *button_type,
                                })
                                .unwrap_or_else(use_button_value);

                            if value != 0.0 {
                                // A matching input was pressed on a gamepad
                                return value;
                            }
                        }

                        // No input was pressed on any gamepad
                        0.0
                    } else {
                        // No Gamepads resource found and no gamepad was registered
                        0.0
                    }
                } else {
                    // No Axis<GamepadButton> was found
                    use_button_value()
                }
            }
            _ => use_button_value(),
        }
    }

    /// Get the axis pair associated to the user input.
    ///
    /// If `input` is not a [`DualAxis`] or [`VirtualDPad`], returns [`None`].
    ///
    /// See [`ActionState::action_axis_pair()`] for usage.
    ///
    /// # Warning
    ///
    /// If you need to ensure that this value is always in the range `[-1., 1.]`,
    /// be sure to clamp the returned data.
    pub fn input_axis_pair(&self, input: &UserInput) -> Option<DualAxisData> {
        match input {
            UserInput::Single(InputKind::DualAxis(dual_axis)) => {
                let x = self.input_value(&UserInput::Single(InputKind::SingleAxis(dual_axis.x)));
                let y = self.input_value(&UserInput::Single(InputKind::SingleAxis(dual_axis.y)));

                if x > dual_axis.x.positive_low
                    || x < dual_axis.x.negative_low
                    || y > dual_axis.y.positive_low
                    || y < dual_axis.y.negative_low
                {
                    Some(DualAxisData::new(x, y))
                } else {
                    Some(DualAxisData::new(0.0, 0.0))
                }
            }
            UserInput::VirtualDPad(VirtualDPad {
                up,
                down,
                left,
                right,
            }) => {
                let x = self.input_value(&UserInput::Single(*right))
                    - self.input_value(&UserInput::Single(*left)).abs();
                let y = self.input_value(&UserInput::Single(*up))
                    - self.input_value(&UserInput::Single(*down)).abs();
                Some(DualAxisData::new(x, y))
            }
            _ => None,
        }
    }
}

/// A mutable collection of [`Input`] structs, which can be used for mocking user inputs.
///
/// Each of these streams is optional; if a stream does not exist, inputs sent to them will be ignored.
///
/// These are typically collected via a system from the [`World`](bevy::prelude::World) as resources.
#[derive(Debug)]
pub struct MutableInputStreams<'a> {
    /// An optional [`GamepadButton`] [`Input`] stream
    pub gamepad_buttons: Option<&'a mut Input<GamepadButton>>,
    /// An optional [`GamepadButton`] [`Axis`] stream
    pub gamepad_button_axes: Option<&'a mut Axis<GamepadButton>>,
    /// An optional [`GamepadAxis`] [`Axis`] stream
    pub gamepad_axes: Option<&'a mut Axis<GamepadAxis>>,
    /// An optional list of registered [`Gamepads`]
    pub gamepads: Option<&'a mut Gamepads>,
    /// An optional [`KeyCode`] [`Input`] stream
    pub keyboard: Option<&'a mut Input<KeyCode>>,
    /// An optional [`MouseButton`] [`Input`] stream
    pub mouse: Option<&'a mut Input<MouseButton>>,
    /// An optional [`MouseWheel`] event stream
    pub mouse_wheel: Option<&'a mut Events<MouseWheel>>,
    /// The [`Gamepad`] that this struct will detect inputs from
    pub associated_gamepad: Option<Gamepad>,
}

impl<'a> MutableInputStreams<'a> {
    /// Construct a [`MutableInputStreams`] from the [`World`]
    pub fn from_world(world: &'a mut World, gamepad: Option<Gamepad>) -> Self {
        let mut input_system_state: SystemState<(
            Option<ResMut<Input<GamepadButton>>>,
            Option<ResMut<Axis<GamepadButton>>>,
            Option<ResMut<Axis<GamepadAxis>>>,
            Option<ResMut<Gamepads>>,
            Option<ResMut<Input<KeyCode>>>,
            Option<ResMut<Input<MouseButton>>>,
            Option<ResMut<Events<MouseWheel>>>,
        )> = SystemState::new(world);

        let (
            maybe_gamepad_buttons,
            maybe_gamepad_button_axes,
            maybe_gamepad_axes,
            maybe_gamepads,
            maybe_keyboard,
            maybe_mouse,
            maybe_mouse_wheel,
        ) = input_system_state.get_mut(world);

        MutableInputStreams {
            gamepad_buttons: maybe_gamepad_buttons.map(|r| r.into_inner()),
            gamepad_button_axes: maybe_gamepad_button_axes.map(|r| r.into_inner()),
            gamepad_axes: maybe_gamepad_axes.map(|r| r.into_inner()),
            gamepads: maybe_gamepads.map(|r| r.into_inner()),
            keyboard: maybe_keyboard.map(|r| r.into_inner()),
            mouse: maybe_mouse.map(|r| r.into_inner()),
            mouse_wheel: maybe_mouse_wheel.map(|r| r.into_inner()),
            associated_gamepad: gamepad,
        }
    }
}

impl<'a> From<MutableInputStreams<'a>> for InputStreams<'a> {
    fn from(mutable_streams: MutableInputStreams<'a>) -> Self {
        let gamepad_buttons = mutable_streams
            .gamepad_buttons
            .map(|mutable_ref| &*mutable_ref);
        let gamepad_button_axes = mutable_streams
            .gamepad_button_axes
            .map(|mutable_ref| &*mutable_ref);
        let gamepad_axes = mutable_streams
            .gamepad_axes
            .map(|mutable_ref| &*mutable_ref);
        let gamepads = mutable_streams.gamepads.map(|mutable_ref| &*mutable_ref);

        let keyboard = mutable_streams.keyboard.map(|mutable_ref| &*mutable_ref);

        let mouse = mutable_streams.mouse.map(|mutable_ref| &*mutable_ref);
        let mouse_wheel = mutable_streams.mouse_wheel.map(|mutable_ref| &*mutable_ref);

        InputStreams {
            gamepad_buttons,
            gamepad_button_axes,
            gamepad_axes,
            gamepads,
            keyboard,
            mouse,
            mouse_wheel,
            associated_gamepad: mutable_streams.associated_gamepad,
        }
    }
}
