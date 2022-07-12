//! Unified input streams for working with [`bevy_input`] data.

use bevy_input::{
    gamepad::{Gamepad, GamepadAxis, GamepadButton, Gamepads},
    keyboard::KeyCode,
    mouse::MouseButton,
    Axis, Input,
};
use bevy_math::Vec2;
use petitset::PetitSet;

use crate::axislike::{AxisPair, VirtualDPad};
use crate::user_input::{InputKind, UserInput};

/// A collection of [`Input`] structs, which can be used to update an [`InputMap`](crate::input_map::InputMap).
///
/// Each of these streams is optional; if a stream does not exist, it is treated as if it were entirely unpressed.
///
/// These are typically collected via a system from the [`World`](bevy::prelude::World) as resources.
#[derive(Clone)]
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
    /// The [`Gamepad`] that this struct will detect inputs from
    pub associated_gamepad: Option<Gamepad>,
}

// Constructors
impl<'a> InputStreams<'a> {
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
            associated_gamepad: None,
        }
    }

    /// Construct [`InputStreams`] with only a [`GamepadButton`] input stream
    pub fn from_mouse(mouse_input_stream: &'a Input<MouseButton>) -> Self {
        Self {
            gamepad_buttons: None,
            gamepad_button_axes: None,
            gamepad_axes: None,
            gamepads: None,
            keyboard: None,
            mouse: Some(mouse_input_stream),
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
            InputKind::DualAxis(axis) => {
                let axis_pair = self
                    .get_input_axis_pair(&UserInput::Single(button))
                    .unwrap();
                let x = axis_pair.x();
                let y = axis_pair.y();

                x > axis.x_positive_low
                    || x < axis.x_negative_low
                    || y > axis.y_positive_low
                    || y < axis.y_negative_low
            }
            InputKind::SingleAxis(axis) => {
                let value = self.get_input_value(&UserInput::Single(button));

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
    pub fn get_input_value(&self, input: &UserInput) -> f32 {
        let use_button_value = || -> f32 {
            if self.input_pressed(input) {
                1.0
            } else {
                0.0
            }
        };

        match input {
            UserInput::Single(InputKind::SingleAxis(single_axis)) => {
                if let Some(axes) = self.gamepad_axes {
                    if let Some(gamepad) = self.associated_gamepad {
                        axes.get(GamepadAxis {
                            gamepad,
                            axis_type: single_axis.axis_type,
                        })
                        .unwrap_or_default()
                    // If no gamepad is registered, return the first non-zero input found
                    } else if let Some(gamepads) = self.gamepads {
                        for &gamepad in gamepads.iter() {
                            let value = axes
                                .get(GamepadAxis {
                                    gamepad,
                                    axis_type: single_axis.axis_type,
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
            UserInput::Single(InputKind::DualAxis(_)) => {
                if self.gamepad_axes.is_some() {
                    self.get_input_axis_pair(input).unwrap_or_default().length()
                } else {
                    0.0
                }
            }
            UserInput::VirtualDPad { .. } => {
                self.get_input_axis_pair(input).unwrap_or_default().length()
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
    pub fn get_input_axis_pair(&self, input: &UserInput) -> Option<AxisPair> {
        match input {
            UserInput::Single(InputKind::DualAxis(dual_axis)) => {
                if let Some(axes) = self.gamepad_axes {
                    if let Some(gamepad) = self.associated_gamepad {
                        let x = axes
                            .get(GamepadAxis {
                                gamepad,
                                axis_type: dual_axis.x_axis_type,
                            })
                            .unwrap_or_default();
                        let y = axes
                            .get(GamepadAxis {
                                gamepad,
                                axis_type: dual_axis.y_axis_type,
                            })
                            .unwrap_or_default();
                        // The registered gampead had inputs
                        Some(AxisPair::new(Vec2::new(x, y)))
                    } else if let Some(gamepads) = self.gamepads {
                        for &gamepad in gamepads.iter() {
                            let x = axes
                                .get(GamepadAxis {
                                    gamepad,
                                    axis_type: dual_axis.x_axis_type,
                                })
                                .unwrap_or_default();
                            let y = axes
                                .get(GamepadAxis {
                                    gamepad,
                                    axis_type: dual_axis.y_axis_type,
                                })
                                .unwrap_or_default();

                            if (x != 0.0) | (y != 0.0) {
                                // At least one gamepad had inputs
                                return Some(AxisPair::new(Vec2::new(x, y)));
                            }
                        }

                        // No input from any gamepad
                        Some(AxisPair::new(Vec2::ZERO))
                    } else {
                        // No Gamepads resource
                        None
                    }
                } else {
                    // No Axis<GamepadAxis> resource
                    None
                }
            }
            UserInput::VirtualDPad(VirtualDPad {
                up,
                down,
                left,
                right,
            }) => {
                let x = self.get_input_value(&UserInput::Single(*right))
                    - self.get_input_value(&UserInput::Single(*left)).abs();
                let y = self.get_input_value(&UserInput::Single(*up))
                    - self.get_input_value(&UserInput::Single(*down)).abs();
                Some(AxisPair::new(Vec2::new(x, y)))
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
    /// The [`Gamepad`] that this struct will detect inputs from
    pub associated_gamepad: Option<Gamepad>,
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

        InputStreams {
            gamepad_buttons,
            gamepad_button_axes,
            gamepad_axes,
            gamepads,
            keyboard,
            mouse,
            associated_gamepad: mutable_streams.associated_gamepad,
        }
    }
}
