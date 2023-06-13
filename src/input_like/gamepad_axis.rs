use crate::input_like::{ButtonLike, DualAxisLike, InputLike, InputLikeObject, SingleAxisLike};
use bevy::input::{Axis, Input};
use bevy::prelude::{GamepadAxis, Reflect, World};
use erased_serde::Serialize;

impl ButtonLike for GamepadAxis {
    fn input_pressed(&self, world: &World) -> bool {
        let Some(gamepad_axis) = world.get_resource::<Input<GamepadAxis>>() else {
            return false;
        };

        gamepad_axis.pressed(*self)
    }

    fn clone_button(&self) -> Box<dyn ButtonLike> {
        Box::new(*self)
    }
}

impl SingleAxisLike for GamepadAxis {
    fn input_value(&self, world: &World) -> f32 {
        let Some(gamepad_axis) = world.get_resource::<Axis<GamepadAxis>>() else {
            return 0.0;
        };

        gamepad_axis.get(*self).unwrap_or_default()
    }

    fn clone_axis(&self) -> Box<dyn SingleAxisLike> {
        Box::new(*self)
    }
}

impl InputLikeObject for GamepadAxis {
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

impl<'a> InputLike<'a> for GamepadAxis {}
