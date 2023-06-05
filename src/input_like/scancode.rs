use crate::input_like::{ButtonLike, DualAxisLike, InputLike, InputLikeObject, SingleAxisLike};
use crate::prelude::QwertyScanCode;
use bevy::input::Input;
use bevy::prelude::{Reflect, ScanCode, World};
use erased_serde::Serialize;

impl ButtonLike for ScanCode {
    fn input_pressed(&self, world: &World) -> bool {
        world.resource::<Input<ScanCode>>().pressed(*self)
    }

    fn clone_dyn(&self) -> Box<dyn ButtonLike> {
        Box::new(*self)
    }
}

impl SingleAxisLike for ScanCode {
    fn input_value(&self, world: &World) -> f32 {
        if self.input_pressed(world) {
            1.0
        } else {
            0.0
        }
    }
}

impl InputLikeObject for ScanCode {
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

impl<'a> InputLike<'a> for ScanCode {}

impl ButtonLike for QwertyScanCode {
    fn input_pressed(&self, world: &World) -> bool {
        world
            .resource::<Input<ScanCode>>()
            .pressed(ScanCode::from(*self))
    }

    fn clone_dyn(&self) -> Box<dyn ButtonLike> {
        Box::new(*self)
    }
}

impl SingleAxisLike for QwertyScanCode {
    fn input_value(&self, world: &World) -> f32 {
        if self.input_pressed(world) {
            1.0
        } else {
            0.0
        }
    }
}

impl InputLikeObject for QwertyScanCode {
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

impl<'a> InputLike<'a> for QwertyScanCode {}
