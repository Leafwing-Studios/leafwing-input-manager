//! Utilities for testing user input.

use bevy::{app::App, math::Vec2, prelude::World};

use super::{updating::CentralInputStore, Axislike, Buttonlike, DualAxislike};

/// A trait used to quickly fetch the value of a given [`UserInput`](crate::user_input::UserInput).
///
/// This can be useful for testing purposes.
pub trait FetchUserInput {
    /// Returns `true` if the given [`Buttonlike`] input is currently pressed.
    fn read_pressed(&mut self, input: impl Buttonlike) -> bool;

    /// Returns the value of the given [`Axislike`] input.
    fn read_axis_value(&mut self, input: impl Axislike) -> f32;

    /// Returns the value of the given [`DualAxislike`] input.
    fn read_dual_axis_values(&mut self, input: impl DualAxislike) -> Vec2;
}

impl FetchUserInput for CentralInputStore {
    fn read_pressed(&mut self, input: impl Buttonlike) -> bool {
        input.pressed(self)
    }

    fn read_axis_value(&mut self, input: impl Axislike) -> f32 {
        input.value(self)
    }

    fn read_dual_axis_values(&mut self, input: impl DualAxislike) -> Vec2 {
        input.axis_pair(self)
    }
}

impl FetchUserInput for World {
    fn read_pressed(&mut self, input: impl Buttonlike) -> bool {
        let mut input_store = CentralInputStore::from_world(self);
        input_store.read_pressed(input)
    }

    fn read_axis_value(&mut self, input: impl Axislike) -> f32 {
        let mut input_store = CentralInputStore::from_world(self);
        input_store.read_axis_value(input)
    }

    fn read_dual_axis_values(&mut self, input: impl DualAxislike) -> Vec2 {
        let mut input_store = CentralInputStore::from_world(self);
        input_store.read_dual_axis_values(input)
    }
}

impl FetchUserInput for App {
    fn read_pressed(&mut self, input: impl Buttonlike) -> bool {
        self.world_mut().read_pressed(input)
    }

    fn read_axis_value(&mut self, input: impl Axislike) -> f32 {
        self.world_mut().read_axis_value(input)
    }

    fn read_dual_axis_values(&mut self, input: impl DualAxislike) -> Vec2 {
        self.world_mut().read_dual_axis_values(input)
    }
}
