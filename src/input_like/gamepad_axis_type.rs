use crate::input_like::{ButtonLike, DualAxisLike, InputLike, InputLikeObject, SingleAxisLike};
use bevy::input::Axis;
use bevy::prelude::{GamepadAxis, GamepadAxisType, Gamepads, Reflect, World};
use erased_serde::Serialize;

impl ButtonLike for GamepadAxisType {
    /// Returns true if the axis is pressed for any gamepad.
    ///
    /// To specify a specific gamepad, use [`GamepadButton`] instead, or call
    /// [`InputMap::set_gamepad`] to convert all the [`GamepadButtonType`]s to
    /// [`GamepadButtonType`]s to [`GamepadButton`]s.
    fn input_pressed(&self, world: &World) -> bool {
        let Some(gamepads) = world.get_resource::<Gamepads>() else {
            return false;
        };

        let Some(gamepad_axis) = world.get_resource::<Axis<GamepadAxis>>() else {
            return false;
        };
        gamepads.iter().any(|gamepad| {
            gamepad_axis
                .get(GamepadAxis {
                    gamepad,
                    axis_type: *self,
                })
                .map(|axis_value| axis_value != 0.0)
                .unwrap_or_default()
        })
    }

    fn clone_button(&self) -> Box<dyn ButtonLike> {
        Box::new(*self)
    }
}

impl SingleAxisLike for GamepadAxisType {
    fn input_value(&self, world: &World) -> f32 {
        let Some(gamepads) = world.get_resource::<Gamepads>() else {
            return 0.0;
        };

        let Some(gamepad_axis) = world.get_resource::<Axis<GamepadAxis>>() else {
            return 0.0;
        };
        gamepads
            .iter()
            .map(|gamepad| {
                gamepad_axis
                    .get(GamepadAxis {
                        gamepad,
                        axis_type: *self,
                    })
                    .unwrap_or_default()
            })
            .sum()
    }

    fn clone_axis(&self) -> Box<dyn SingleAxisLike> {
        Box::new(*self)
    }
}

impl InputLikeObject for GamepadAxisType {
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

impl<'a> InputLike<'a> for GamepadAxisType {}
