use crate::input_like::{ButtonLike, DualAxisLike, InputLike, InputLikeObject, SingleAxisLike};
use bevy::input::Input;
use bevy::prelude::{GamepadButton, GamepadButtonType, Gamepads, Reflect, World};
use erased_serde::Serialize;

impl ButtonLike for GamepadButtonType {
    /// Returns true if the button is pressed for any gamepad.
    ///
    /// To specify a specific gamepad, use [`GamepadButton`] instead, or call
    /// [`InputMap::set_gamepad`] to convert all the [`GamepadButtonType`]s to
    /// [`GamepadButtonType`]s to [`GamepadButton`]s.
    fn input_pressed(&self, world: &World) -> bool {
        let Some(gamepads) = world.get_resource::<Gamepads>() else {
            return false;
        };
        let Some(gamepad_buttons) = world.get_resource::<Input<GamepadButton>>() else {
            return false;
        };
        let buttons_for_all_gamepads = gamepads
            .iter()
            .map(|gamepad| GamepadButton::new(gamepad, *self));
        gamepad_buttons.any_pressed(buttons_for_all_gamepads)
    }

    fn clone_button(&self) -> Box<dyn ButtonLike> {
        Box::new(*self)
    }
}

impl SingleAxisLike for GamepadButtonType {
    fn clone_axis(&self) -> Box<dyn SingleAxisLike> {
        Box::new(*self)
    }
}

impl InputLikeObject for GamepadButtonType {
    fn as_button(&self) -> Option<&dyn ButtonLike> {
        Some(self)
    }

    fn as_axis(&self) -> Option<&dyn SingleAxisLike> {
        Some(self)
    }

    fn as_dual_axis(&self) -> Option<&dyn DualAxisLike> {
        None
    }

    fn clone_dyn(&self) -> Box<dyn InputLikeObject> {
        Box::new(*self)
    }

    fn as_serialize(&self) -> &dyn Serialize {
        self
    }

    fn as_reflect(&self) -> &dyn Reflect {
        self
    }
}

impl<'a> InputLike<'a> for GamepadButtonType {}
