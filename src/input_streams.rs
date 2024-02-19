//! Unified input streams for working with [`bevy::input`] data.

use bevy::ecs::prelude::{Event, Events, ResMut, World};
use bevy::ecs::system::SystemState;
use bevy::input::{
    gamepad::{Gamepad, GamepadAxis, GamepadButton, GamepadEvent, Gamepads},
    keyboard::{KeyCode, KeyboardInput, ScanCode},
    mouse::{MouseButton, MouseButtonInput, MouseMotion, MouseWheel},
    Axis, Input,
};
use bevy::math::Vec2;
use bevy::utils::HashSet;

use crate::axislike::{
    deadzone_axis_value, AxisType, DualAxisData, MouseMotionAxisType, MouseWheelAxisType,
    SingleAxis, VirtualAxis,
};
use crate::buttonlike::{MouseMotionDirection, MouseWheelDirection};
use crate::prelude::DualAxis;
use crate::user_input::{InputKind, UserInput};

/// A collection of [`Input`] structs, which can be used to update an [`InputMap`](crate::input_map::InputMap).
///
/// These are typically collected via a system from the [`World`] as resources.
#[derive(Debug, Clone)]
pub struct InputStreams<'a> {
    /// A [`GamepadButton`] [`Input`] stream
    pub gamepad_buttons: &'a Input<GamepadButton>,
    /// A [`GamepadButton`] [`Axis`] stream
    pub gamepad_button_axes: &'a Axis<GamepadButton>,
    /// A [`GamepadAxis`] [`Axis`] stream
    pub gamepad_axes: &'a Axis<GamepadAxis>,
    /// A list of registered gamepads
    pub gamepads: &'a Gamepads,
    /// A [`KeyCode`] [`Input`] stream
    pub keycodes: Option<&'a Input<KeyCode>>,
    /// A [`ScanCode`] [`Input`] stream
    pub scan_codes: Option<&'a Input<ScanCode>>,
    /// A [`MouseButton`] [`Input`] stream
    pub mouse_buttons: Option<&'a Input<MouseButton>>,
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
        let gamepad_buttons = world.resource::<Input<GamepadButton>>();
        let gamepad_button_axes = world.resource::<Axis<GamepadButton>>();
        let gamepad_axes = world.resource::<Axis<GamepadAxis>>();
        let gamepads = world.resource::<Gamepads>();
        let keycodes = world.get_resource::<Input<KeyCode>>();
        let scan_codes = world.get_resource::<Input<ScanCode>>();
        let mouse_buttons = world.get_resource::<Input<MouseButton>>();
        let mouse_wheel = world.resource::<Events<MouseWheel>>();
        let mouse_motion = world.resource::<Events<MouseMotion>>();

        let mouse_wheel: Vec<MouseWheel> = collect_events_cloned(mouse_wheel);
        let mouse_motion: Vec<MouseMotion> = collect_events_cloned(mouse_motion);

        InputStreams {
            gamepad_buttons,
            gamepad_button_axes,
            gamepad_axes,
            gamepads,
            keycodes,
            scan_codes,
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
            UserInput::VirtualDPad(dpad) => [&dpad.up, &dpad.down, &dpad.left, &dpad.right]
                .into_iter()
                .any(|button| self.button_pressed(*button)),
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
            InputKind::DualAxis(axis) => self.extract_dual_axis_data(&axis).is_some(),
            InputKind::SingleAxis(axis) => {
                let value = self.input_value(&button.into(), false);

                value < axis.negative_low || value > axis.positive_low
            }
            InputKind::GamepadButton(button_type) => self
                .associated_gamepad
                .into_iter()
                .chain(self.gamepads.iter())
                .any(|gamepad| {
                    self.gamepad_buttons.pressed(GamepadButton {
                        gamepad,
                        button_type,
                    })
                }),
            InputKind::Keyboard(keycode) => {
                matches!(self.keycodes, Some(keycodes) if keycodes.pressed(keycode))
            }
            InputKind::KeyLocation(scan_code) => {
                matches!(self.scan_codes, Some(scan_codes) if scan_codes.pressed(scan_code))
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

                // The compiler will compile this into a direct f64 accumulation when opt-level >= 1.
                //
                // PERF: this summing is computed for every individual input
                // This should probably be computed once, and then cached / read
                // Fix upstream!
                let Vec2 { x, y } = mouse_wheel
                    .iter()
                    .map(|wheel| Vec2::new(wheel.x, wheel.y))
                    .sum();
                match mouse_wheel_direction {
                    MouseWheelDirection::Up => y > 0.0,
                    MouseWheelDirection::Down => y < 0.0,
                    MouseWheelDirection::Left => x < 0.0,
                    MouseWheelDirection::Right => x > 0.0,
                }
            }
            InputKind::MouseMotion(mouse_motion_direction) => {
                // The compiler will compile this into a direct f64 accumulation when opt-level >= 1.
                let Vec2 { x, y } = self.mouse_motion.iter().map(|motion| motion.delta).sum();
                match mouse_motion_direction {
                    MouseMotionDirection::Up => y > 0.0,
                    MouseMotionDirection::Down => y < 0.0,
                    MouseMotionDirection::Left => x < 0.0,
                    MouseMotionDirection::Right => x > 0.0,
                }
            }
        }
    }

    /// Are all of the `buttons` pressed?
    #[must_use]
    pub fn all_buttons_pressed(&self, buttons: &[InputKind]) -> bool {
        buttons.iter().all(|button| self.button_pressed(*button))
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
        let use_button_value = || -> f32 { f32::from(self.input_pressed(input)) };

        // Helper that takes the value returned by an axis and returns 0.0 if it is not within the
        // triggering range.
        let value_in_axis_range = |axis: &SingleAxis, mut value: f32| -> f32 {
            if include_deadzone {
                if value >= axis.negative_low && value <= axis.positive_low {
                    return 0.0;
                }

                let deadzone = if value.is_sign_positive() {
                    axis.positive_low.abs()
                } else {
                    axis.negative_low.abs()
                };
                value = deadzone_axis_value(value, deadzone);
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
                        let get_gamepad_value = |gamepad: Gamepad| -> f32 {
                            self.gamepad_axes
                                .get(GamepadAxis { gamepad, axis_type })
                                .unwrap_or_default()
                        };
                        if let Some(gamepad) = self.associated_gamepad {
                            let value = get_gamepad_value(gamepad);
                            value_in_axis_range(single_axis, value)
                        } else {
                            self.gamepads
                                .iter()
                                .map(get_gamepad_value)
                                .find(|value| *value != 0.0)
                                .map_or(0.0, |value| value_in_axis_range(single_axis, value))
                        }
                    }
                    AxisType::MouseWheel(axis_type) => {
                        let Some(mouse_wheel) = &self.mouse_wheel else {
                            return 0.0;
                        };

                        // The compiler will compile this into a direct f64 accumulation when opt-level >= 1.
                        let Vec2 { x, y } = mouse_wheel
                            .iter()
                            .map(|wheel| Vec2::new(wheel.x, wheel.y))
                            .sum();
                        let movement = match axis_type {
                            MouseWheelAxisType::X => x,
                            MouseWheelAxisType::Y => y,
                        };
                        value_in_axis_range(single_axis, movement)
                    }
                    AxisType::MouseMotion(axis_type) => {
                        // The compiler will compile this into a direct f64 accumulation when opt-level >= 1.
                        let Vec2 { x, y } = self.mouse_motion.iter().map(|e| e.delta).sum();
                        let movement = match axis_type {
                            MouseMotionAxisType::X => x,
                            MouseMotionAxisType::Y => y,
                        };
                        value_in_axis_range(single_axis, movement)
                    }
                }
            }
            UserInput::VirtualAxis(axis) => {
                self.extract_single_axis_data(&axis.positive, &axis.negative)
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
                            self.input_value(&InputKind::SingleAxis(*axis).into(), true)
                        }
                        InputKind::MouseWheel(axis) => {
                            has_axis = true;
                            self.input_value(&InputKind::MouseWheel(*axis).into(), true)
                        }
                        InputKind::MouseMotion(axis) => {
                            has_axis = true;
                            self.input_value(&InputKind::MouseMotion(*axis).into(), true)
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
                let get_gamepad_value = |gamepad: Gamepad| -> f32 {
                    self.gamepad_button_axes
                        .get(GamepadButton {
                            gamepad,
                            button_type: *button_type,
                        })
                        .unwrap_or_else(use_button_value)
                };
                if let Some(gamepad) = self.associated_gamepad {
                    get_gamepad_value(gamepad)
                } else {
                    self.gamepads
                        .iter()
                        .map(get_gamepad_value)
                        .find(|value| *value != 0.0)
                        .unwrap_or_default()
                }
            }
            _ => use_button_value(),
        }
    }

    /// Get the axis pair associated to the user input.
    ///
    /// If `input` is a chord, returns result of the first dual axis in the chord.
    /// If `input` is not a [`DualAxis`] or [`VirtualDPad`](crate::axislike::VirtualDPad), returns [`None`].
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
            UserInput::VirtualDPad(dpad) => {
                let x = self.extract_single_axis_data(&dpad.right, &dpad.left);
                let y = self.extract_single_axis_data(&dpad.up, &dpad.down);
                Some(DualAxisData::new(x, y))
            }
            _ => None,
        }
    }

    fn extract_single_axis_data(&self, positive: &InputKind, negative: &InputKind) -> f32 {
        let positive = self.input_value(&UserInput::Single(*positive), true);
        let negative = self.input_value(&UserInput::Single(*negative), true);

        positive.abs() - negative.abs()
    }

    fn extract_dual_axis_data(&self, dual_axis: &DualAxis) -> Option<DualAxisData> {
        let x = self.input_value(&dual_axis.x.into(), false);
        let y = self.input_value(&dual_axis.y.into(), false);

        dual_axis.deadzone.deadzone_input_value(x, y)
    }
}

// Clones and collects the received events into a `Vec`.
#[inline]
fn collect_events_cloned<T: Event + Clone>(events: &Events<T>) -> Vec<T> {
    events.get_reader().read(events).cloned().collect()
}

/// A mutable collection of [`Input`] structs, which can be used for mocking user inputs.
///
/// These are typically collected via a system from the [`World`] as resources.
// WARNING: If you update the fields of this type, you must also remember to update `InputMocking::reset_inputs`.
#[derive(Debug)]
pub struct MutableInputStreams<'a> {
    /// A [`GamepadButton`] [`Input`] stream
    pub gamepad_buttons: &'a mut Input<GamepadButton>,
    /// A [`GamepadButton`] [`Axis`] stream
    pub gamepad_button_axes: &'a mut Axis<GamepadButton>,
    /// A [`GamepadAxis`] [`Axis`] stream
    pub gamepad_axes: &'a mut Axis<GamepadAxis>,
    /// A list of registered [`Gamepads`]
    pub gamepads: &'a mut Gamepads,
    /// Events used for mocking gamepad-related inputs
    pub gamepad_events: &'a mut Events<GamepadEvent>,

    /// A [`KeyCode`] [`Input`] stream
    pub keycodes: &'a mut Input<KeyCode>,
    /// A [`ScanCode`] [`Input`] stream
    pub scan_codes: &'a mut Input<ScanCode>,
    /// Events used for mocking keyboard-related inputs
    pub keyboard_events: &'a mut Events<KeyboardInput>,

    /// A [`MouseButton`] [`Input`] stream
    pub mouse_buttons: &'a mut Input<MouseButton>,
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
            ResMut<Input<GamepadButton>>,
            ResMut<Axis<GamepadButton>>,
            ResMut<Axis<GamepadAxis>>,
            ResMut<Gamepads>,
            ResMut<Events<GamepadEvent>>,
            ResMut<Input<KeyCode>>,
            ResMut<Input<ScanCode>>,
            ResMut<Events<KeyboardInput>>,
            ResMut<Input<MouseButton>>,
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
            scan_codes,
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
            scan_codes: scan_codes.into_inner(),
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
            scan_codes: Some(mutable_streams.scan_codes),
            mouse_buttons: Some(mutable_streams.mouse_buttons),
            mouse_wheel: Some(collect_events_cloned(mutable_streams.mouse_wheel)),
            mouse_motion: collect_events_cloned(mutable_streams.mouse_motion),
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
            scan_codes: Some(mutable_streams.scan_codes),
            mouse_buttons: Some(mutable_streams.mouse_buttons),
            mouse_wheel: Some(collect_events_cloned(mutable_streams.mouse_wheel)),
            mouse_motion: collect_events_cloned(mutable_streams.mouse_motion),
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
