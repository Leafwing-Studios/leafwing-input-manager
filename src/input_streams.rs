//! Unified input streams for working with [`bevy::input`] data.

use bevy::ecs::prelude::{Event, Events, ResMut, World};
use bevy::ecs::system::SystemState;
use bevy::input::{
    gamepad::{Gamepad, GamepadAxis, GamepadButton, GamepadEvent, Gamepads},
    keyboard::{KeyCode, KeyboardInput},
    mouse::{MouseButton, MouseButtonInput, MouseMotion, MouseWheel},
    Axis, ButtonInput,
};

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

        let mouse_wheel: Vec<MouseWheel> = collect_events_cloned(mouse_wheel);
        let mouse_motion: Vec<MouseMotion> = collect_events_cloned(mouse_motion);

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

// Clones and collects the received events into a `Vec`.
#[inline]
fn collect_events_cloned<T: Event + Clone>(events: &Events<T>) -> Vec<T> {
    events.get_reader().read(events).cloned().collect()
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
    /// Otherwise, use the first registered gamepad, if any.
    pub fn guess_gamepad(&self) -> Option<Gamepad> {
        self.associated_gamepad
            .or_else(|| self.gamepads.iter().next())
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
            mouse_buttons: Some(mutable_streams.mouse_buttons),
            mouse_wheel: Some(collect_events_cloned(mutable_streams.mouse_wheel)),
            mouse_motion: collect_events_cloned(mutable_streams.mouse_motion),
            associated_gamepad: mutable_streams.associated_gamepad,
        }
    }
}
