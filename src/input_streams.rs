//! Unified input streams for working with [`bevy::input`] data.

use bevy::input::{
    gamepad::{Gamepad, GamepadAxis, GamepadButton, GamepadEventRaw, Gamepads},
    keyboard::{KeyCode, KeyboardInput},
    mouse::{MouseButton, MouseButtonInput, MouseMotion, MouseWheel},
    Axis, Input,
};
use petitset::PetitSet;

use bevy::ecs::prelude::{Events, ResMut, World};
use bevy::ecs::system::SystemState;

use crate::axislike::{
    AxisType, DualAxisData, MouseMotionAxisType, MouseWheelAxisType, SingleAxis, VirtualDPad,
};
use crate::buttonlike::{MouseMotionDirection, MouseWheelDirection};
use crate::user_input::{InputKind, UserInput};

/// A collection of [`Input`] structs, which can be used to update an [`InputMap`](crate::input_map::InputMap).
///
/// These are typically collected via a system from the [`World`](bevy::prelude::World) as resources.
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
    pub keycode: &'a Input<KeyCode>,
    /// A [`MouseButton`] [`Input`] stream
    pub mouse_button: &'a Input<MouseButton>,
    /// A [`MouseWheel`] event stream
    pub mouse_wheel: &'a Events<MouseWheel>,
    /// A [`MouseMotion`] event stream
    pub mouse_motion: &'a Events<MouseMotion>,
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
        let keyboard = world.resource::<Input<KeyCode>>();
        let mouse = world.resource::<Input<MouseButton>>();
        let mouse_wheel = world.resource::<Events<MouseWheel>>();
        let mouse_motion = world.resource::<Events<MouseMotion>>();

        InputStreams {
            gamepad_buttons,
            gamepad_button_axes,
            gamepad_axes,
            gamepads,
            keycode: keyboard,
            mouse_button: mouse,
            mouse_wheel,
            mouse_motion,
            associated_gamepad: gamepad,
        }
    }
}

// Input checking
impl<'a> InputStreams<'a> {
    /// Guess which registered [`Gamepad`] should be used.
    ///
    /// If an associated gamepad is set, use that.
    /// Otherwise use the first registered gamepad, if any.
    pub fn guess_gamepad(&self) -> Option<Gamepad> {
        match self.associated_gamepad {
            Some(gamepad) => Some(gamepad),
            None => self.gamepads.iter().next().copied(),
        }
    }

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
            InputKind::SingleAxis(_) => {
                let value = self.input_value(&UserInput::Single(button));

                value != 0.0
            }
            InputKind::GamepadButton(gamepad_button) => {
                if let Some(gamepad) = self.guess_gamepad() {
                    self.gamepad_buttons.pressed(GamepadButton {
                        gamepad,
                        button_type: gamepad_button,
                    })
                } else {
                    false
                }
            }
            InputKind::Keyboard(keycode) => self.keycode.pressed(keycode),
            InputKind::Mouse(mouse_button) => self.mouse_button.pressed(mouse_button),
            InputKind::MouseWheel(mouse_wheel_direction) => {
                let mut total_mouse_wheel_movement = 0.0;

                // FIXME: verify that this works and doesn't double count events
                let mut event_reader = self.mouse_wheel.get_reader();

                // PERF: this summing is computed for every individual input
                // This should probably be computed once, and then cached / read
                // Fix upstream!
                for mouse_wheel_event in event_reader.iter(self.mouse_wheel) {
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

                // FIXME: verify that this works and doesn't double count events
                let mut event_reader = self.mouse_motion.get_reader();

                for mouse_motion_event in event_reader.iter(self.mouse_motion) {
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

        // Helper that takes the value returned by an axis and returns 0.0 if it is not within the
        // triggering range.
        let value_in_axis_range = |axis: &SingleAxis, value: f32| -> f32 {
            if value >= axis.negative_low && value <= axis.positive_low {
                0.0
            } else {
                value
            }
        };

        match input {
            UserInput::Single(InputKind::SingleAxis(single_axis)) => {
                match single_axis.axis_type {
                    AxisType::Gamepad(axis_type) => {
                        if let Some(gamepad) = self.guess_gamepad() {
                            let value = self
                                .gamepad_axes
                                .get(GamepadAxis { gamepad, axis_type })
                                .unwrap_or_default();

                            value_in_axis_range(single_axis, value)
                        } else {
                            0.0
                        }
                    }
                    AxisType::MouseWheel(axis_type) => {
                        let mut total_mouse_wheel_movement = 0.0;
                        // FIXME: verify that this works and doesn't double count events
                        let mut event_reader = self.mouse_wheel.get_reader();

                        for mouse_wheel_event in event_reader.iter(self.mouse_wheel) {
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
                        // FIXME: verify that this works and doesn't double count events
                        let mut event_reader = self.mouse_motion.get_reader();

                        for mouse_wheel_event in event_reader.iter(self.mouse_motion) {
                            total_mouse_motion_movement += match axis_type {
                                MouseMotionAxisType::X => mouse_wheel_event.delta.x,
                                MouseMotionAxisType::Y => mouse_wheel_event.delta.y,
                            }
                        }
                        value_in_axis_range(single_axis, total_mouse_motion_movement)
                    }
                }
            }
            UserInput::Single(InputKind::DualAxis(_)) => {
                self.input_axis_pair(input).unwrap_or_default().length()
            }
            UserInput::VirtualDPad { .. } => {
                self.input_axis_pair(input).unwrap_or_default().length()
            }
            // This is required because upstream bevy::input still waffles about whether triggers are buttons or axes
            UserInput::Single(InputKind::GamepadButton(button_type)) => {
                if let Some(gamepad) = self.guess_gamepad() {
                    // Get the value from the registered gamepad
                    self.gamepad_button_axes
                        .get(GamepadButton {
                            gamepad,
                            button_type: *button_type,
                        })
                        .unwrap_or_else(use_button_value)
                } else {
                    0.0
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
                let x = self.input_value(&UserInput::Single(*right)).abs()
                    - self.input_value(&UserInput::Single(*left)).abs();
                let y = self.input_value(&UserInput::Single(*up)).abs()
                    - self.input_value(&UserInput::Single(*down)).abs();
                Some(DualAxisData::new(x, y))
            }
            _ => None,
        }
    }
}

/// A mutable collection of [`Input`] structs, which can be used for mocking user inputs.
///
/// These are typically collected via a system from the [`World`](bevy::prelude::World) as resources.
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
    pub gamepad_events: &'a mut Events<GamepadEventRaw>,

    /// A [`KeyCode`] [`Input`] stream
    pub keycode: &'a mut Input<KeyCode>,
    /// Events used for mocking keyboard-related inputs
    pub keyboard_events: &'a mut Events<KeyboardInput>,

    /// A [`MouseButton`] [`Input`] stream
    pub mouse_button: &'a mut Input<MouseButton>,
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
            ResMut<Events<GamepadEventRaw>>,
            ResMut<Input<KeyCode>>,
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
            keyboard,
            keyboard_events,
            mouse,
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
            keycode: keyboard.into_inner(),
            keyboard_events: keyboard_events.into_inner(),
            mouse_button: mouse.into_inner(),
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
            None => self.gamepads.iter().next().copied(),
        }
    }
}

impl<'a> From<MutableInputStreams<'a>> for InputStreams<'a> {
    fn from(mutable_streams: MutableInputStreams<'a>) -> Self {
        InputStreams {
            // This absurd-looking &*(foo) pattern convinces the compiler
            // that we want a reference to the underlying data with the correct lifetime
            gamepad_buttons: &*(mutable_streams.gamepad_buttons),
            gamepad_button_axes: &*(mutable_streams.gamepad_button_axes),
            gamepad_axes: &*(mutable_streams.gamepad_axes),
            gamepads: &*(mutable_streams.gamepads),
            keycode: &*(mutable_streams.keycode),
            mouse_button: &*(mutable_streams.mouse_button),
            mouse_wheel: &*(mutable_streams.mouse_wheel),
            mouse_motion: &*(mutable_streams.mouse_motion),
            associated_gamepad: mutable_streams.associated_gamepad,
        }
    }
}

impl<'a> From<&'a MutableInputStreams<'a>> for InputStreams<'a> {
    fn from(mutable_streams: &'a MutableInputStreams<'a>) -> Self {
        InputStreams {
            // This absurd-looking &*(foo) pattern convinces the compiler
            // that we want a reference to the underlying data with the correct lifetime
            gamepad_buttons: &*(mutable_streams.gamepad_buttons),
            gamepad_button_axes: &*(mutable_streams.gamepad_button_axes),
            gamepad_axes: &*(mutable_streams.gamepad_axes),
            gamepads: &*(mutable_streams.gamepads),
            keycode: &*(mutable_streams.keycode),
            mouse_button: &*(mutable_streams.mouse_button),
            mouse_wheel: &*(mutable_streams.mouse_wheel),
            mouse_motion: &*(mutable_streams.mouse_motion),
            associated_gamepad: mutable_streams.associated_gamepad,
        }
    }
}
