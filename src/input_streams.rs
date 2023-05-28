use crate::axislike::DualAxisData;
use crate::input_like::InputLikeObject;
use bevy::prelude::World;

pub struct InputStreams<'a> {
    world: &'a World,
}

impl<'a> InputStreams<'a> {
    pub fn from_world(world: &'a World) -> Self {
        Self { world }
    }

    pub fn input_pressed(&self, input: &dyn InputLikeObject) -> bool {
        input
            .as_button()
            .map_or(false, |x| x.input_pressed(self.world))
    }
    pub fn input_value(&self, input: &dyn InputLikeObject) -> f32 {
        input.as_axis().map_or(0.0, |x| x.input_value(self.world))
    }
    pub fn input_axis_pair(&self, input: &dyn InputLikeObject) -> Option<DualAxisData> {
        input
            .as_dual_axis()
            .and_then(|x| x.input_axis_pair(self.world))
    }
}
