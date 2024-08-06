//! Unified input streams for working with [`bevy::input`] data.

use bevy::ecs::prelude::World;
use bevy::input::{
    gamepad::{Gamepad, GamepadAxis, GamepadButton, Gamepads},
    keyboard::KeyCode,
    mouse::MouseButton,
    Axis, ButtonInput,
};

use crate::user_input::{AccumulatedMouseMovement, AccumulatedMouseScroll};

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
    /// The [`AccumulatedMouseScroll`] for the frame
    pub mouse_scroll: &'a AccumulatedMouseScroll,
    /// The [`AccumulatedMouseMovement`] for the frame
    pub mouse_motion: &'a AccumulatedMouseMovement,
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
        let mouse_wheel = world.resource::<AccumulatedMouseScroll>();
        let mouse_motion = world.resource::<AccumulatedMouseMovement>();

        InputStreams {
            gamepad_buttons,
            gamepad_button_axes,
            gamepad_axes,
            gamepads,
            keycodes,
            mouse_buttons,
            mouse_scroll: mouse_wheel,
            mouse_motion,
            associated_gamepad: gamepad,
        }
    }
}
