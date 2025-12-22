//! Utilities for testing user input.

use bevy::{
    app::App,
    ecs::system::SystemState,
    input::gamepad::Gamepad,
    math::Vec2,
    prelude::{Entity, Query, With, World},
};

use super::{Axislike, Buttonlike, DualAxislike, updating::CentralInputStore};

#[cfg(feature = "gamepad")]
use crate::user_input::gamepad::find_gamepad;

#[cfg(not(feature = "gamepad"))]
fn find_gamepad(_: Option<Query<Entity, With<Gamepad>>>) -> Entity {
    Entity::PLACEHOLDER
}

/// A trait used to quickly fetch the value of a given [`UserInput`](crate::user_input::UserInput).
///
/// This can be useful for testing purposes.
pub trait FetchUserInput {
    /// Returns `true` if the given [`Buttonlike`] input is currently pressed.
    fn read_pressed(&mut self, input: impl Buttonlike) -> bool;

    /// Returns the value of the given [`Buttonlike`] input.
    fn read_button_value(&mut self, input: impl Buttonlike) -> f32;

    /// Returns the value of the given [`Axislike`] input.
    fn read_axis_value(&mut self, input: impl Axislike) -> f32;

    /// Returns the value of the given [`DualAxislike`] input.
    fn read_dual_axis_values(&mut self, input: impl DualAxislike) -> Vec2;
}

impl FetchUserInput for World {
    fn read_pressed(&mut self, input: impl Buttonlike) -> bool {
        let mut query_state = SystemState::<Query<Entity, With<Gamepad>>>::new(self);
        let query = query_state.get(self);
        let gamepad = find_gamepad(Some(query));
        let input_store = self.resource::<CentralInputStore>();

        input.pressed(input_store, gamepad)
    }

    fn read_button_value(&mut self, input: impl Buttonlike) -> f32 {
        let mut query_state = SystemState::<Query<Entity, With<Gamepad>>>::new(self);
        let query = query_state.get(self);
        let gamepad = find_gamepad(Some(query));
        let input_store = self.resource::<CentralInputStore>();

        input.value(input_store, gamepad)
    }

    fn read_axis_value(&mut self, input: impl Axislike) -> f32 {
        let mut query_state = SystemState::<Query<Entity, With<Gamepad>>>::new(self);
        let query = query_state.get(self);
        let gamepad = find_gamepad(Some(query));
        let input_store = self.resource::<CentralInputStore>();

        input.value(input_store, gamepad)
    }

    fn read_dual_axis_values(&mut self, input: impl DualAxislike) -> Vec2 {
        let mut query_state = SystemState::<Query<Entity, With<Gamepad>>>::new(self);
        let query = query_state.get(self);
        let gamepad = find_gamepad(Some(query));
        let input_store = self.resource::<CentralInputStore>();

        input.axis_pair(input_store, gamepad)
    }
}

impl FetchUserInput for App {
    fn read_pressed(&mut self, input: impl Buttonlike) -> bool {
        self.world_mut().read_pressed(input)
    }

    fn read_button_value(&mut self, input: impl Buttonlike) -> f32 {
        self.world_mut().read_button_value(input)
    }

    fn read_axis_value(&mut self, input: impl Axislike) -> f32 {
        self.world_mut().read_axis_value(input)
    }

    fn read_dual_axis_values(&mut self, input: impl DualAxislike) -> Vec2 {
        self.world_mut().read_dual_axis_values(input)
    }
}
