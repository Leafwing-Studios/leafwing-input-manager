use crate::axislike::DualAxisData;
use crate::prelude::DualAxis;
use crate::user_input::{InputLikeObject, ReflectInputLike};
use bevy::app::AppTypeRegistry;
use bevy::prelude::World;
use bevy::utils::HashMap;
use std::any::TypeId;

pub trait InputStreams {
    fn input_pressed(&self, world: &World, input: &dyn InputLikeObject) -> bool;
    fn input_value(&self, world: &World, input: &dyn InputLikeObject) -> f32;
    fn input_axis_pair(&self, world: &World, input: &dyn InputLikeObject) -> Option<DualAxisData>;
}

/// Routes input queries to the correct [`InputStreams`] based on the underlying type of the input.
pub struct InputStreamsRouter<'a> {
    pub input_streams: HashMap<TypeId, Box<dyn InputStreams>>,
    world: &'a World,
}

impl<'a> InputStreamsRouter<'a> {
    pub fn collect(world: &'a World) -> Self {
        let input_likes = {
            let type_registry = world.resource::<AppTypeRegistry>().read();

            type_registry
                .iter()
                .filter_map(|type_registration| {
                    type_registry
                        .get_type_data::<ReflectInputLike>(type_registration.type_id())
                        .map(|x| (type_registration.type_id(), x.clone()))
                })
                .collect::<Vec<_>>()
        };
        let mut input_streams = HashMap::default();
        for (type_id, input_like) in input_likes.iter() {
            let x: Box<dyn InputStreams> = (input_like.input_streams)(world);
            input_streams.insert(*type_id, x);
        }

        Self {
            input_streams,
            world,
        }
    }

    pub fn input_pressed(&self, input: &dyn InputLikeObject) -> bool {
        self.input_streams
            .get(&input.as_reflect().type_id())
            .map(|x| x.input_pressed(self.world, input))
            .unwrap_or_default()
    }
    pub fn input_value(&self, input: &dyn InputLikeObject) -> f32 {
        self.input_streams
            .get(&input.as_reflect().type_id())
            .map(|x| x.input_value(self.world, input))
            .unwrap_or_default()
    }
    pub fn input_axis_pair(&self, input: &dyn InputLikeObject) -> Option<DualAxisData> {
        self.input_streams
            .get(&input.as_reflect().type_id())
            .map(|x| x.input_axis_pair(self.world, input))
            .unwrap_or_default()
    }
}
