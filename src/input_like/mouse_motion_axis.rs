use crate::input_like::{ButtonLike, DualAxisLike, InputLike, InputLikeObject, SingleAxisLike};
use bevy::input::mouse::MouseMotion;
use bevy::prelude::{Events, Reflect, World};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum MouseMotionAxis {
    /// Horizontal movement.
    X,
    /// Vertical movement.
    Y,
}

impl ButtonLike for MouseMotionAxis {
    fn input_pressed(&self, world: &World) -> bool {
        let Some(events) = world.get_resource::<Events<MouseMotion>>() else {
            return false;
        };

        let mut event_reader = events.get_reader();
        event_reader
            .iter(events)
            .map(|i| match self {
                MouseMotionAxis::X => i.delta.x,
                MouseMotionAxis::Y => i.delta.y,
            })
            .any(|i| i != 0.0)
    }

    fn clone_button(&self) -> Box<dyn ButtonLike> {
        Box::new(*self)
    }
}

impl SingleAxisLike for MouseMotionAxis {
    fn input_value(&self, world: &World) -> f32 {
        let Some(events) = world.get_resource::<Events<MouseMotion>>() else {
            return 0.0;
        };

        let mut event_reader = events.get_reader();
        event_reader
            .iter(events)
            .map(|i| match self {
                MouseMotionAxis::X => i.delta.x,
                MouseMotionAxis::Y => i.delta.y,
            })
            .sum()
    }

    fn clone_axis(&self) -> Box<dyn SingleAxisLike> {
        Box::new(*self)
    }
}

impl InputLikeObject for MouseMotionAxis {
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

    fn as_serialize(&self) -> &dyn erased_serde::Serialize {
        self
    }

    fn as_reflect(&self) -> &dyn Reflect {
        self
    }
}

impl<'a> InputLike<'a> for MouseMotionAxis {}
