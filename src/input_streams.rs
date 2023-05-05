//! Unified input streams for working with [`bevy::input`] data.

use bevy::input::{
    gamepad::{Gamepad, GamepadAxis, GamepadButton, GamepadEvent, Gamepads},
    keyboard::{KeyCode, KeyboardInput, ScanCode},
    mouse::{MouseButton, MouseButtonInput, MouseMotion, MouseWheel},
    Axis, Input,
};
use petitset::PetitSet;

use bevy::ecs::prelude::{Events, ResMut, World};
use bevy::ecs::system::SystemState;

use crate::axislike::DualAxisData;
use crate::prelude::DualAxis;
use crate::user_input::{InputKind, InputLikeMethods, UserInput};

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
    pub keycodes: Option<&'a Input<KeyCode>>,
    /// A [`ScanCode`] [`Input`] stream
    pub scan_codes: Option<&'a Input<ScanCode>>,
    /// A [`MouseButton`] [`Input`] stream
    pub mouse_buttons: Option<&'a Input<MouseButton>>,
    /// A [`MouseWheel`] event stream
    pub mouse_wheel: Option<&'a Events<MouseWheel>>,
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
        let keycodes = world.get_resource::<Input<KeyCode>>();
        let scan_codes = world.get_resource::<Input<ScanCode>>();
        let mouse_buttons = world.get_resource::<Input<MouseButton>>();
        let mouse_wheel = world.get_resource::<Events<MouseWheel>>();
        let mouse_motion = world.resource::<Events<MouseMotion>>();

        InputStreams {
            gamepad_buttons,
            gamepad_button_axes,
            gamepad_axes,
            gamepads,
            keycodes,
            scan_codes,
            mouse_buttons,
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
            None => self.gamepads.iter().next(),
        }
    }

    /// Is the `input` matched by the [`InputStreams`]?
    pub fn input_pressed(&self, input: &dyn InputLikeMethods) -> bool {
        // todo!()
        false
    }

    /// Is at least one of the `inputs` pressed?
    #[must_use]
    pub fn any_pressed(&self, inputs: &PetitSet<UserInput, 16>) -> bool {
        todo!()
    }

    /// Is the `button` pressed?
    #[must_use]
    pub fn button_pressed(&self, button: InputKind) -> bool {
        todo!();
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
    /// be sure to clamp the returned data.
    pub fn input_value(&self, input: &impl InputLikeMethods) -> f32 {
        todo!();
    }

    /// Get the axis pair associated to the user input.
    ///
    /// If `input` is a chord, returns result of the first dual axis in the chord.

    /// If `input` is not a [`DualAxis`](crate::axislike::DualAxis) or [`VirtualDPad`], returns [`None`].
    ///
    /// # Warning
    ///
    /// If you need to ensure that this value is always in the range `[-1., 1.]`,
    /// be sure to clamp the returned data.
    pub fn input_axis_pair(&self, input: &impl InputLikeMethods) -> Option<DualAxisData> {
        // TODO:
        None
    }

    fn extract_dual_axis_data(&self, dual_axis: &DualAxis) -> DualAxisData {
        let x = self.input_value(&UserInput::Single(InputKind::SingleAxis(dual_axis.x)));
        let y = self.input_value(&UserInput::Single(InputKind::SingleAxis(dual_axis.y)));

        if x > dual_axis.x.positive_low
            || x < dual_axis.x.negative_low
            || y > dual_axis.y.positive_low
            || y < dual_axis.y.negative_low
        {
            DualAxisData::new(x, y)
        } else {
            DualAxisData::new(0.0, 0.0)
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
            mouse_wheel: Some(mutable_streams.mouse_wheel),
            mouse_motion: mutable_streams.mouse_motion,
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
            mouse_wheel: Some(mutable_streams.mouse_wheel),
            mouse_motion: mutable_streams.mouse_motion,
            associated_gamepad: mutable_streams.associated_gamepad,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MutableInputStreams;
    use crate::prelude::MockInput;
    use bevy::input::InputPlugin;
    use bevy::prelude::*;

    #[test]
    fn modifier_key_triggered_by_either_input() {
        use crate::user_input::Modifier;
        let mut app = App::new();
        app.add_plugin(InputPlugin);

        let mut input_streams = MutableInputStreams::from_world(&mut app.world, None);
        assert!(!input_streams.pressed(Modifier::Control));

        input_streams.send_input(KeyCode::LControl);
        app.update();

        let mut input_streams = MutableInputStreams::from_world(&mut app.world, None);
        assert!(input_streams.pressed(Modifier::Control));

        input_streams.reset_inputs();
        app.update();

        let mut input_streams = MutableInputStreams::from_world(&mut app.world, None);
        assert!(!input_streams.pressed(Modifier::Control));

        input_streams.send_input(KeyCode::RControl);
        app.update();

        let input_streams = MutableInputStreams::from_world(&mut app.world, None);
        assert!(input_streams.pressed(Modifier::Control));
    }
}
