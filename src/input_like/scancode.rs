use crate::axislike::DualAxisData;
use crate::input_like::{AxisLike, ButtonLike, InputLike, InputLikeObject};
use crate::input_streams::InputStreams;
use crate::prelude::QwertyScanCode;
use bevy::input::Input;
use bevy::prelude::{Reflect, ScanCode, World};
use erased_serde::Serialize;

impl ButtonLike for ScanCode {}

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

    fn as_axis(&self) -> Option<Box<dyn AxisLike>> {
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

struct ScanCodeInputStreams;

impl InputStreams for ScanCodeInputStreams {
    fn input_pressed(&self, world: &World, input: &dyn InputLikeObject) -> bool {
        input
            .as_reflect()
            .downcast_ref()
            .map(|input| world.resource::<Input<ScanCode>>().pressed(*input))
            .unwrap_or_default()
    }

    fn input_value(&self, world: &World, input: &dyn InputLikeObject) -> f32 {
        if self.input_pressed(world, input) {
            1.0
        } else {
            0.0
        }
    }

    fn input_axis_pair(
        &self,
        _world: &World,
        _input: &dyn InputLikeObject,
    ) -> Option<DualAxisData> {
        None
    }
}

impl<'a> InputLike<'a> for ScanCode {
    fn input_streams(_world: &World) -> Box<dyn InputStreams> {
        Box::new(ScanCodeInputStreams)
    }
}

impl ButtonLike for QwertyScanCode {}

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

    fn as_axis(&self) -> Option<Box<dyn AxisLike>> {
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

struct QwertyScanCodeInputStreams;

impl InputStreams for QwertyScanCodeInputStreams {
    fn input_pressed(&self, world: &World, input: &dyn InputLikeObject) -> bool {
        input
            .as_reflect()
            .downcast_ref::<QwertyScanCode>()
            .map(|input| {
                world
                    .resource::<Input<ScanCode>>()
                    .pressed(ScanCode::from(*input))
            })
            .unwrap_or_default()
    }

    fn input_value(&self, world: &World, input: &dyn InputLikeObject) -> f32 {
        if self.input_pressed(world, input) {
            1.0
        } else {
            0.0
        }
    }

    fn input_axis_pair(
        &self,
        _world: &World,
        _input: &dyn InputLikeObject,
    ) -> Option<DualAxisData> {
        None
    }
}

impl<'a> InputLike<'a> for QwertyScanCode {
    fn input_streams(_world: &World) -> Box<dyn InputStreams> {
        Box::new(QwertyScanCodeInputStreams)
    }
}
