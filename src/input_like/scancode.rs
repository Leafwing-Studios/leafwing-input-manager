use crate::input_like::{ButtonLike, DualAxisLike, InputLike, InputLikeObject, SingleAxisLike};
use crate::prelude::QwertyScanCode;
use bevy::input::Input;
use bevy::prelude::{Reflect, ScanCode, World};
use erased_serde::Serialize;

impl ButtonLike for ScanCode {
    fn input_pressed(&self, world: &World) -> bool {
        world.resource::<Input<ScanCode>>().pressed(*self)
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
    fn clashes(&self, other: &dyn InputLikeObject) -> bool {
        other
            .as_reflect()
            .downcast_ref::<ScanCode>()
            .map(|other| self == other)
            .unwrap_or_default()
    }

    fn as_button(&self) -> Option<Box<dyn ButtonLike>> {
        Some(Box::new(*self))
    }

    fn as_axis(&self) -> Option<Box<dyn SingleAxisLike>> {
        None
    }

    fn as_dual_axis(&self) -> Option<Box<dyn DualAxisLike>> {
        None
    }

    fn len(&self) -> usize {
        1
    }

    fn raw_inputs(&self) -> Vec<Box<dyn InputLikeObject>> {
        vec![Box::new(*self)]
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
    fn clashes(&self, other: &dyn InputLikeObject) -> bool {
        other
            .as_reflect()
            .downcast_ref::<QwertyScanCode>()
            .map(|other| self == other)
            .unwrap_or_default()
            || other
                .as_reflect()
                .downcast_ref::<ScanCode>()
                .map(|other| ScanCode::from(*self) == *other)
                .unwrap_or_default()
    }

    fn as_button(&self) -> Option<Box<dyn ButtonLike>> {
        Some(Box::new(*self))
    }

    fn as_axis(&self) -> Option<Box<dyn SingleAxisLike>> {
        None
    }

    fn as_dual_axis(&self) -> Option<Box<dyn DualAxisLike>> {
        None
    }

    fn len(&self) -> usize {
        1
    }

    fn raw_inputs(&self) -> Vec<Box<dyn InputLikeObject>> {
        vec![Box::new(*self)]
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
