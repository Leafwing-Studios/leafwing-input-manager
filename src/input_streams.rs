//! Unified input streams for working with [`bevy::input`] data.

use bevy::ecs::prelude::{Events, ResMut, World};
use bevy::ecs::system::SystemState;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::input::{
    gamepad::{Gamepad, GamepadAxis, GamepadButton, GamepadEvent, Gamepads},
    keyboard::{KeyCode, KeyboardInput},
    mouse::{MouseButton, MouseButtonInput},
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
    /// The [`AccumulatedMouseScroll`] for the frame
    pub mouse_scroll: &'a mut AccumulatedMouseScroll,
    /// The [`AccumulatedMouseMovement`] for the frame
    pub mouse_motion: &'a mut AccumulatedMouseMovement,
    /// Mouse scroll events used for mocking mouse scroll inputs
    pub mouse_scroll_events: &'a mut Events<MouseWheel>,
    /// Mouse motion events used for mocking mouse motion inputs
    pub mouse_motion_events: &'a mut Events<MouseMotion>,

    /// The [`Gamepad`] that this struct will detect inputs from
    pub associated_gamepad: Option<Gamepad>,
}

impl<'a> MutableInputStreams<'a> {
    /// Construct a [`MutableInputStreams`] from the [`World`]
    pub fn from_world(world: &'a mut World, gamepad: Option<Gamepad>) -> Self {
        // Initialize accumulated mouse movement and scroll resources if they don't exist
        // These are special-cased because they are not initialized by the InputPlugin
        if !world.contains_resource::<AccumulatedMouseMovement>() {
            world.init_resource::<AccumulatedMouseMovement>();
        }

        if !world.contains_resource::<AccumulatedMouseScroll>() {
            world.init_resource::<AccumulatedMouseScroll>();
        }

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
            ResMut<AccumulatedMouseScroll>,
            ResMut<AccumulatedMouseMovement>,
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
            mouse_scroll,
            mouse_motion,
            mouse_scroll_events,
            mouse_motion_events,
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
            mouse_scroll: mouse_scroll.into_inner(),
            mouse_motion: mouse_motion.into_inner(),
            mouse_scroll_events: mouse_scroll_events.into_inner(),
            mouse_motion_events: mouse_motion_events.into_inner(),
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
            mouse_scroll: mutable_streams.mouse_scroll,
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
            mouse_buttons: Some(mutable_streams.mouse_buttons),
            mouse_scroll: mutable_streams.mouse_scroll,
            mouse_motion: mutable_streams.mouse_motion,
            associated_gamepad: mutable_streams.associated_gamepad,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{InputStreams, MutableInputStreams};
    use crate::prelude::{MockInput, QueryInput};
    use crate::user_input::ModifierKey;
    use bevy::input::InputPlugin;
    use bevy::prelude::*;

    #[test]
    fn modifier_key_triggered_by_either_input() {
        let mut app = App::new();
        app.add_plugins(InputPlugin);

        let mut input_streams = MutableInputStreams::from_world(app.world_mut(), None);
        assert!(!InputStreams::from(&input_streams).pressed(ModifierKey::Control));

        input_streams.press_input(KeyCode::ControlLeft);
        app.update();

        let mut input_streams = MutableInputStreams::from_world(app.world_mut(), None);
        assert!(InputStreams::from(&input_streams).pressed(ModifierKey::Control));

        input_streams.reset_inputs();
        app.update();

        let mut input_streams = MutableInputStreams::from_world(app.world_mut(), None);
        assert!(!InputStreams::from(&input_streams).pressed(ModifierKey::Control));

        input_streams.press_input(KeyCode::ControlRight);
        app.update();

        let input_streams = MutableInputStreams::from_world(app.world_mut(), None);
        assert!(InputStreams::from(&input_streams).pressed(ModifierKey::Control));
    }
}
