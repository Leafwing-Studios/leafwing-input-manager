//! Unified input streams for working with [`bevy::input`] data.

use bevy::ecs::prelude::{Events, ResMut, World};
use bevy::ecs::system::SystemState;
use bevy::input::{
    gamepad::{Gamepad, GamepadAxis, GamepadButton, GamepadEvent, Gamepads},
    keyboard::{KeyCode, KeyboardInput},
    mouse::{MouseButton, MouseButtonInput, MouseMotion, MouseWheel},
    Axis, ButtonInput,
};
use bevy::utils::HashSet;

use crate::axislike::{
    AxisType, DualAxisData, MouseMotionAxisType, MouseWheelAxisType, SingleAxis, VirtualAxis,
    VirtualDPad,
};
use crate::buttonlike::{MouseMotionDirection, MouseWheelDirection};
use crate::prelude::DualAxis;
use crate::user_input::{InputKind, UserInput};

/// A collection of [`ButtonInput`] structs, which can be used to update an [`InputMap`](crate::input_map::InputMap).
///
/// These are typically collected via a system from the [`World`] as resources.
#[derive(Debug, Clone)]
pub struct InputStreams<'a> {
    /// A [`GamepadButton`] [`Input`](ButtonInput) stream
    pub gamepad_buttons: &'a ButtonInput<GamepadButton>,
    /// A [`GamepadButton`] [`Axis`] stream
    pub gamepad_button_axes: &'a Axis<GamepadButton>,
    /// A [`GamepadAxis`] [`Axis`] stream
    pub gamepad_axes: &'a Axis<GamepadAxis>,
    /// A list of registered gamepads
    pub gamepads: &'a Gamepads,
    /// A [`KeyCode`] [`ButtonInput`] stream
    pub keycodes: Option<&'a ButtonInput<KeyCode>>,
    /// A [`MouseButton`] [`Input`](ButtonInput) stream
    pub mouse_buttons: Option<&'a ButtonInput<MouseButton>>,
    /// A [`MouseWheel`] event stream
    pub mouse_wheel: Option<Vec<MouseWheel>>,
    /// A [`MouseMotion`] event stream
    pub mouse_motion: Vec<MouseMotion>,
    /// The [`Gamepad`] that this struct will detect inputs from
    pub associated_gamepad: Option<Gamepad>,
}

// Constructors
impl<'a> InputStreams<'a> {
    /// Construct an [`InputStreams`] from a [`World`]
    pub fn from_world(world: &'a World, gamepad: Option<Gamepad>) -> Self {
        let gamepad_buttons = world.resource::<ButtonInput<GamepadButton>>();
        let gamepad_button_axes = world.resource::<Axis<GamepadButton>>();
        let gamepad_axes = world.resource::<Axis<GamepadAxis>>();
        let gamepads = world.resource::<Gamepads>();
        let keycodes = world.get_resource::<ButtonInput<KeyCode>>();
        let mouse_buttons = world.get_resource::<ButtonInput<MouseButton>>();
        let mouse_wheel = world.resource::<Events<MouseWheel>>();
        let mouse_motion = world.resource::<Events<MouseMotion>>();

        let mouse_wheel: Vec<MouseWheel> = mouse_wheel
            .get_reader()
            .read(mouse_wheel)
            .cloned()
            .collect();
        let mouse_motion: Vec<MouseMotion> = mouse_motion
            .get_reader()
            .read(mouse_motion)
            .cloned()
            .collect();

        InputStreams {
            gamepad_buttons,
            gamepad_button_axes,
            gamepad_axes,
            gamepads,
            keycodes,
            mouse_buttons,
            mouse_wheel: Some(mouse_wheel),
            mouse_motion,
            associated_gamepad: gamepad,
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
            }) => [up, down, left, right]
                .into_iter()
                .any(|button| self.button_pressed(button.clone())),
            UserInput::VirtualAxis(VirtualAxis { negative, positive }) => {
                self.button_pressed(*negative) || self.button_pressed(*positive)
            }
        }
    }

    /// Is at least one of the `inputs` pressed?
    #[must_use]
    pub fn any_pressed(&self, inputs: &HashSet<UserInput>) -> bool {
        inputs.iter().any(|input| self.input_pressed(input))
    }

    /// Is the `button` pressed?
    #[must_use]
    pub fn button_pressed(&self, button: InputKind) -> bool {
        match button {
            InputKind::DualAxis(axis) => {
                let x_value =
                    self.input_value(&UserInput::Single(InputKind::SingleAxis(axis.x)), false);
                let y_value =
                    self.input_value(&UserInput::Single(InputKind::SingleAxis(axis.y)), false);

                axis.deadzone
                    .deadzone_input_value(x_value, y_value)
                    .is_some()
            }
            InputKind::SingleAxis(axis) => {
                let value = self.input_value(&UserInput::Single(button), false);

                value < axis.negative_low || value > axis.positive_low
            }
            InputKind::GamepadButton(button_type) => {
                if let Some(gamepad) = self.associated_gamepad {
                    self.gamepad_buttons.pressed(GamepadButton {
                        gamepad,
                        button_type,
                    })
                } else {
                    self.gamepads.iter().any(|gamepad| {
                        self.gamepad_buttons.pressed(GamepadButton {
                            gamepad,
                            button_type,
                        })
                    })
                }
            }
            InputKind::PhysicalKey(keycode) => {
                matches!(self.keycodes, Some(keycodes) if keycodes.pressed(keycode))
            }
            InputKind::Modifier(modifier) => {
                let key_codes = modifier.key_codes();
                // Short circuiting is probably not worth the branch here
                matches!(self.keycodes, Some(keycodes) if keycodes.pressed(key_codes[0]) | keycodes.pressed(key_codes[1]))
            }
            InputKind::Mouse(mouse_button) => {
                matches!(self.mouse_buttons, Some(mouse_buttons) if mouse_buttons.pressed(mouse_button))
            }
            InputKind::MouseWheel(mouse_wheel_direction) => {
                let Some(mouse_wheel) = &self.mouse_wheel else {
                    return false;
                };

                let mut total_mouse_wheel_movement = 0.0;

                // PERF: this summing is computed for every individual input
                // This should probably be computed once, and then cached / read
                // Fix upstream!
                for mouse_wheel_event in mouse_wheel {
                    total_mouse_wheel_movement += match mouse_wheel_direction {
                        MouseWheelDirection::Up | MouseWheelDirection::Down => mouse_wheel_event.y,
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
            }
            // CLEANUP: refactor to share code with MouseWheel
            InputKind::MouseMotion(mouse_motion_direction) => {
                let mut total_mouse_movement = 0.0;

                for mouse_motion_event in &self.mouse_motion {
                    total_mouse_movement += match mouse_motion_direction {
                        MouseMotionDirection::Up | MouseMotionDirection::Down => {
                            mouse_motion_event.delta.y
                        }
                        MouseMotionDirection::Left | MouseMotionDirection::Right => {
                            mouse_motion_event.delta.x
                        }
                    }
                }

                match mouse_motion_direction {
                    MouseMotionDirection::Up | MouseMotionDirection::Right => {
                        total_mouse_movement > 0.0
                    }
                    MouseMotionDirection::Down | MouseMotionDirection::Left => {
                        total_mouse_movement < 0.0
                    }
                }
            }
        }
    }

    /// Are all of the `buttons` pressed?
    #[must_use]
    pub fn all_buttons_pressed(&self, buttons: &[InputKind]) -> bool {
        buttons
            .iter()
            .all(|button| self.button_pressed(button.clone()))
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
    /// be sure to clamp the returned data.
    pub fn input_value(&self, input: &UserInput, include_deadzone: bool) -> f32 {
        let use_button_value = || -> f32 {
            if self.input_pressed(input) {
                1.0
            } else {
                0.0
            }
        };

        // Helper that takes the value returned by an axis and returns 0.0 if it is not within the
        // triggering range.
        let value_in_axis_range = |axis: &SingleAxis, mut value: f32| -> f32 {
            if include_deadzone {
                if value >= axis.negative_low && value <= axis.positive_low {
                    return 0.0;
                }

                let width = if value.is_sign_positive() {
                    axis.positive_low.abs()
                } else {
                    axis.negative_low.abs()
                };
                value = value.signum() * (value.abs() - width).max(0.0) / (1.0 - width);
            }
            if axis.inverted {
                value *= -1.0;
            }

            value * axis.sensitivity
        };

        match input {
            UserInput::Single(InputKind::SingleAxis(single_axis)) => {
                match single_axis.axis_type {
                    AxisType::Gamepad(axis_type) => {
                        if let Some(gamepad) = self.associated_gamepad {
                            let value = self
                                .gamepad_axes
                                .get(GamepadAxis { gamepad, axis_type })
                                .unwrap_or_default();

                            value_in_axis_range(single_axis, value)
                        } else {
                            for gamepad in self.gamepads.iter() {
                                let value = self
                                    .gamepad_axes
                                    .get(GamepadAxis { gamepad, axis_type })
                                    .unwrap_or_default();

                                // Return early if *any* gamepad is pressing this axis
                                if value != 0.0 {
                                    return value_in_axis_range(single_axis, value);
                                }
                            }

                            // If we don't have the required data, fall back to 0.0
                            0.0
                        }
                    }
                    AxisType::MouseWheel(axis_type) => {
                        let Some(mouse_wheel) = &self.mouse_wheel else {
                            return 0.0;
                        };

                        let mut total_mouse_wheel_movement = 0.0;

                        for mouse_wheel_event in mouse_wheel {
                            total_mouse_wheel_movement += match axis_type {
                                MouseWheelAxisType::X => mouse_wheel_event.x,
                                MouseWheelAxisType::Y => mouse_wheel_event.y,
                            }
                        }
                        value_in_axis_range(single_axis, total_mouse_wheel_movement)
                    }
                    // CLEANUP: deduplicate code with MouseWheel
                    AxisType::MouseMotion(axis_type) => {
                        let mut total_mouse_motion_movement = 0.0;

                        for mouse_wheel_event in &self.mouse_motion {
                            total_mouse_motion_movement += match axis_type {
                                MouseMotionAxisType::X => mouse_wheel_event.delta.x,
                                MouseMotionAxisType::Y => mouse_wheel_event.delta.y,
                            }
                        }
                        value_in_axis_range(single_axis, total_mouse_motion_movement)
                    }
                }
            }
            UserInput::VirtualAxis(VirtualAxis { negative, positive }) => {
                self.input_value(&UserInput::Single(*positive), true).abs()
                    - self.input_value(&UserInput::Single(*negative), true).abs()
            }
            UserInput::Single(InputKind::DualAxis(_)) => {
                self.input_axis_pair(input).unwrap_or_default().length()
            }
            UserInput::VirtualDPad { .. } => {
                self.input_axis_pair(input).unwrap_or_default().length()
            }
            UserInput::Chord(inputs) => {
                let mut value = 0.0;
                let mut has_axis = false;

                // Prioritize axis over button input values
                for input in inputs.iter() {
                    value += match input {
                        InputKind::SingleAxis(axis) => {
                            has_axis = true;
                            self.input_value(&UserInput::Single(InputKind::SingleAxis(*axis)), true)
                        }
                        InputKind::MouseWheel(axis) => {
                            has_axis = true;
                            self.input_value(&UserInput::Single(InputKind::MouseWheel(*axis)), true)
                        }
                        InputKind::MouseMotion(axis) => {
                            has_axis = true;
                            self.input_value(
                                &UserInput::Single(InputKind::MouseMotion(*axis)),
                                true,
                            )
                        }
                        _ => 0.0,
                    }
                }

                if has_axis {
                    return value;
                }

                use_button_value()
            }
            // This is required because upstream bevy::input still waffles about whether triggers are buttons or axes
            UserInput::Single(InputKind::GamepadButton(button_type)) => {
                if let Some(gamepad) = self.associated_gamepad {
                    // Get the value from the registered gamepad
                    self.gamepad_button_axes
                        .get(GamepadButton {
                            gamepad,
                            button_type: *button_type,
                        })
                        .unwrap_or_else(use_button_value)
                } else {
                    for gamepad in self.gamepads.iter() {
                        let value = self
                            .gamepad_button_axes
                            .get(GamepadButton {
                                gamepad,
                                button_type: *button_type,
                            })
                            .unwrap_or_else(use_button_value);

                        // Return early if *any* gamepad is pressing this button
                        if value != 0.0 {
                            return value;
                        }
                    }

                    // If we don't have the required data, fall back to 0.0
                    0.0
                }
            }
            _ => use_button_value(),
        }
    }

    /// Get the axis pair associated to the user input.
    ///
    /// If `input` is a chord, returns result of the first dual axis in the chord.

    /// If `input` is not a [`DualAxis`] or [`VirtualDPad`], returns [`None`].
    ///
    /// # Warning
    ///
    /// If you need to ensure that this value is always in the range `[-1., 1.]`,
    /// be sure to clamp the returned data.
    pub fn input_axis_pair(&self, input: &UserInput) -> Option<DualAxisData> {
        match input {
            UserInput::Chord(inputs) => {
                for input_kind in inputs.iter() {
                    // Exclude chord combining both button-like and axis-like inputs unless all buttons are pressed.
                    if !self.button_pressed(*input_kind) {
                        return None;
                    }

                    // Return result of the first dual axis in the chord.
                    if let InputKind::DualAxis(dual_axis) = input_kind {
                        return Some(self.extract_dual_axis_data(dual_axis).unwrap_or_default());
                    }
                }
                None
            }
            UserInput::Single(InputKind::DualAxis(dual_axis)) => {
                Some(self.extract_dual_axis_data(dual_axis).unwrap_or_default())
            }
            UserInput::VirtualDPad(VirtualDPad {
                up,
                down,
                left,
                right,
            }) => {
                let x = self.input_value(&UserInput::Single(*right), true).abs()
                    - self.input_value(&UserInput::Single(*left), true).abs();
                let y = self.input_value(&UserInput::Single(*up), true).abs()
                    - self.input_value(&UserInput::Single(*down), true).abs();
                Some(DualAxisData::new(x, y))
            }
            _ => None,
        }
    }

    fn extract_dual_axis_data(&self, dual_axis: &DualAxis) -> Option<DualAxisData> {
        let x = self.input_value(
            &UserInput::Single(InputKind::SingleAxis(dual_axis.x)),
            false,
        );
        let y = self.input_value(
            &UserInput::Single(InputKind::SingleAxis(dual_axis.y)),
            false,
        );

        dual_axis.deadzone.deadzone_input_value(x, y)
    }
}

/// A mutable collection of [`ButtonInput`] structs, which can be used for mocking user inputs.
///
/// These are typically collected via a system from the [`World`] as resources.
// WARNING: If you update the fields of this type, you must also remember to update `InputMocking::reset_inputs`.
#[derive(Debug)]
pub struct MutableInputStreams<'a> {
    /// A [`GamepadButton`] [`Input`](ButtonInput) stream
    pub gamepad_buttons: &'a mut ButtonInput<GamepadButton>,
    /// A [`GamepadButton`] [`Axis`] stream
    pub gamepad_button_axes: &'a mut Axis<GamepadButton>,
    /// A [`GamepadAxis`] [`Axis`] stream
    pub gamepad_axes: &'a mut Axis<GamepadAxis>,
    /// A list of registered [`Gamepads`]
    pub gamepads: &'a mut Gamepads,
    /// Events used for mocking gamepad-related inputs
    pub gamepad_events: &'a mut Events<GamepadEvent>,

    /// A [`KeyCode`] [`ButtonInput`] stream
    pub keycodes: &'a mut ButtonInput<KeyCode>,
    /// Events used for mocking keyboard-related inputs
    pub keyboard_events: &'a mut Events<KeyboardInput>,

    /// A [`MouseButton`] [`Input`](ButtonInput) stream
    pub mouse_buttons: &'a mut ButtonInput<MouseButton>,
    /// Events used for mocking [`MouseButton`] inputs
    pub mouse_button_events: &'a mut Events<MouseButtonInput>,
    /// A [`MouseWheel`] event stream
    pub mouse_wheel: &'a mut Events<MouseWheel>,
    /// A [`MouseMotion`] event stream
    pub mouse_motion: &'a mut Events<MouseMotion>,

    /// The [`Gamepad`] that this struct will detect inputs from
    pub associated_gamepad: Option<Gamepad>,
}

impl<'a> MutableInputStreams<'a> {
    /// Construct a [`MutableInputStreams`] from the [`World`]
    pub fn from_world(world: &'a mut World, gamepad: Option<Gamepad>) -> Self {
        let mut input_system_state: SystemState<(
            ResMut<ButtonInput<GamepadButton>>,
            ResMut<Axis<GamepadButton>>,
            ResMut<Axis<GamepadAxis>>,
            ResMut<Gamepads>,
            ResMut<Events<GamepadEvent>>,
            ResMut<ButtonInput<KeyCode>>,
            ResMut<Events<KeyboardInput>>,
            ResMut<ButtonInput<MouseButton>>,
            ResMut<Events<MouseButtonInput>>,
            ResMut<Events<MouseWheel>>,
            ResMut<Events<MouseMotion>>,
        )> = SystemState::new(world);

        let (
            gamepad_buttons,
            gamepad_button_axes,
            gamepad_axes,
            gamepads,
            gamepad_events,
            keycodes,
            keyboard_events,
            mouse_buttons,
            mouse_button_events,
            mouse_wheel,
            mouse_motion,
        ) = input_system_state.get_mut(world);

        MutableInputStreams {
            gamepad_buttons: gamepad_buttons.into_inner(),
            gamepad_button_axes: gamepad_button_axes.into_inner(),
            gamepad_axes: gamepad_axes.into_inner(),
            gamepads: gamepads.into_inner(),
            gamepad_events: gamepad_events.into_inner(),
            keycodes: keycodes.into_inner(),
            keyboard_events: keyboard_events.into_inner(),
            mouse_buttons: mouse_buttons.into_inner(),
            mouse_button_events: mouse_button_events.into_inner(),
            mouse_wheel: mouse_wheel.into_inner(),
            mouse_motion: mouse_motion.into_inner(),
            associated_gamepad: gamepad,
        }
    }

    /// Guess which registered [`Gamepad`] should be used.
    ///
    /// If an associated gamepad is set, use that.
    /// Otherwise use the first registered gamepad, if any.
    pub fn guess_gamepad(&self) -> Option<Gamepad> {
        match self.associated_gamepad {
            Some(gamepad) => Some(gamepad),
            None => self.gamepads.iter().next(),
        }
    }
}

impl<'a> From<MutableInputStreams<'a>> for InputStreams<'a> {
    fn from(mutable_streams: MutableInputStreams<'a>) -> Self {
        InputStreams {
            gamepad_buttons: mutable_streams.gamepad_buttons,
            gamepad_button_axes: mutable_streams.gamepad_button_axes,
            gamepad_axes: mutable_streams.gamepad_axes,
            gamepads: mutable_streams.gamepads,
            keycodes: Some(mutable_streams.keycodes),
            mouse_buttons: Some(mutable_streams.mouse_buttons),
            mouse_wheel: Some(
                mutable_streams
                    .mouse_wheel
                    .get_reader()
                    .read(mutable_streams.mouse_wheel)
                    .cloned()
                    .collect(),
            ),
            mouse_motion: mutable_streams
                .mouse_motion
                .get_reader()
                .read(mutable_streams.mouse_motion)
                .cloned()
                .collect(),
            associated_gamepad: mutable_streams.associated_gamepad,
        }
    }
}

impl<'a> From<&'a MutableInputStreams<'a>> for InputStreams<'a> {
    fn from(mutable_streams: &'a MutableInputStreams<'a>) -> Self {
        InputStreams {
            gamepad_buttons: mutable_streams.gamepad_buttons,
            gamepad_button_axes: mutable_streams.gamepad_button_axes,
            gamepad_axes: mutable_streams.gamepad_axes,
            gamepads: mutable_streams.gamepads,
            keycodes: Some(mutable_streams.keycodes),
            mouse_buttons: Some(mutable_streams.mouse_buttons),
            mouse_wheel: Some(
                mutable_streams
                    .mouse_wheel
                    .get_reader()
                    .read(mutable_streams.mouse_wheel)
                    .cloned()
                    .collect(),
            ),
            mouse_motion: mutable_streams
                .mouse_motion
                .get_reader()
                .read(mutable_streams.mouse_motion)
                .cloned()
                .collect(),
            associated_gamepad: mutable_streams.associated_gamepad,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{InputStreams, MutableInputStreams};
    use crate::prelude::{MockInput, QueryInput};
    use bevy::input::InputPlugin;
    use bevy::prelude::*;

    #[test]
    fn modifier_key_triggered_by_either_input() {
        use crate::user_input::Modifier;
        let mut app = App::new();
        app.add_plugins(InputPlugin);

        let mut input_streams = MutableInputStreams::from_world(&mut app.world, None);
        assert!(!InputStreams::from(&input_streams).pressed(Modifier::Control));

        input_streams.send_input(KeyCode::ControlLeft);
        app.update();

        let mut input_streams = MutableInputStreams::from_world(&mut app.world, None);
        assert!(InputStreams::from(&input_streams).pressed(Modifier::Control));

        input_streams.reset_inputs();
        app.update();

        let mut input_streams = MutableInputStreams::from_world(&mut app.world, None);
        assert!(!InputStreams::from(&input_streams).pressed(Modifier::Control));

        input_streams.send_input(KeyCode::ControlRight);
        app.update();

        let input_streams = MutableInputStreams::from_world(&mut app.world, None);
        assert!(InputStreams::from(&input_streams).pressed(Modifier::Control));
    }
}
