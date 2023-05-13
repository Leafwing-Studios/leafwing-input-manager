use crate::axislike::DualAxisData;
use crate::prelude::DualAxis;
use crate::user_input::{InputLikeObject, ReflectInputLike};
use bevy::prelude::World;
use bevy::utils::HashMap;
use std::any::{Any, TypeId};

/// Todo: rename this to `InputStreams` once [`crate::input_streams::InputStreams`] struct is removed
pub trait InputStreamsTrait {
    fn input_pressed(&self, world: &World, input: &dyn InputLikeObject) -> bool;
    fn input_value(&self, world: &World, input: &dyn InputLikeObject) -> f32;
    fn input_axis_pair(&self, world: &World, input: &dyn InputLikeObject) -> Option<DualAxisData>;
}

/// Routes input queries to the correct [`InputStreamsTrait`] based on the underlying type of the input.
pub struct InputStreamsRouter<'a> {
    input_streams: HashMap<TypeId, Box<dyn InputStreamsTrait>>,
    world: &'a World,
}

impl<'a> InputStreamsRouter<'a> {
    pub fn collect(input_likes: &[ReflectInputLike], world: &'a World) -> Self {
        let mut input_streams = HashMap::default();
        for input_like in input_likes.into_iter() {
            let x: Box<dyn InputStreamsTrait> = (input_like.input_streams)(world);
            input_streams.insert((*x).type_id(), x);
        }

        Self {
            input_streams,
            world,
        }
    }

    pub fn input_pressed(&self, input: &dyn InputLikeObject) -> bool {
        self.input_streams
            .get(&input.type_id())
            .map(|x| x.input_pressed(self.world, input))
            .unwrap_or_default()
    }
    pub fn input_value(&self, input: &dyn InputLikeObject) -> f32 {
        self.input_streams
            .get(&input.type_id())
            .map(|x| x.input_value(self.world, input))
            .unwrap_or_default()
    }
    pub fn input_axis_pair(&self, input: &dyn InputLikeObject) -> Option<DualAxisData> {
        self.input_streams
            .get(&input.type_id())
            .map(|x| x.input_axis_pair(self.world, input))
            .unwrap_or_default()
    }
}
