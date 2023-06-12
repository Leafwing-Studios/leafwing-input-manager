use crate::input_like::{ButtonLike, DualAxisLike, InputLike, InputLikeObject, SingleAxisLike};
use bevy::input::Input;
use bevy::prelude::{MouseButton, Reflect, World};
use erased_serde::Serialize;
impl ButtonLike for MouseButton {
    fn input_pressed(&self, world: &World) -> bool {
        if let Some(input) = world.get_resource::<Input<MouseButton>>() {
            input.pressed(*self)
        } else {
            false
        }
    }

    fn clone_button(&self) -> Box<dyn ButtonLike> {
        Box::new(*self)
    }
}
impl InputLikeObject for MouseButton {
    fn as_button(&self) -> Option<&dyn ButtonLike> {
        Some(self)
    }

    fn as_axis(&self) -> Option<&dyn SingleAxisLike> {
        None
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

impl<'a> InputLike<'a> for MouseButton {}
