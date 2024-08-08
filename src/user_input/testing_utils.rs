//! Utilities for testing user input.

use bevy::{
    app::App,
    math::Vec2,
    prelude::{Gamepad, Gamepads, World},
};

use super::{updating::CentralInputStore, Axislike, Buttonlike, DualAxislike};

#[cfg(feature = "gamepad")]
use crate::user_input::gamepad::find_gamepad;

#[cfg(not(feature = "gamepad"))]
fn find_gamepad(_gamepads: &Gamepads) -> Gamepad {
    Gamepad::new(0)
}

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

impl FetchUserInput for World {
    fn read_pressed(&mut self, input: impl Buttonlike) -> bool {
        let input_store = self.resource::<CentralInputStore>();
        let gamepad = match self.get_resource::<Gamepads>() {
            Some(gamepads) => find_gamepad(gamepads),
            None => Gamepad::new(0),
        };

        input.pressed(input_store, gamepad)
    }

    fn read_axis_value(&mut self, input: impl Axislike) -> f32 {
        let input_store = self.resource::<CentralInputStore>();
        let gamepad = match self.get_resource::<Gamepads>() {
            Some(gamepads) => find_gamepad(gamepads),
            None => Gamepad::new(0),
        };

        input.value(input_store, gamepad)
    }

    fn read_dual_axis_values(&mut self, input: impl DualAxislike) -> Vec2 {
        let input_store = self.resource::<CentralInputStore>();
        let gamepad = match self.get_resource::<Gamepads>() {
            Some(gamepads) => find_gamepad(gamepads),
            None => Gamepad::new(0),
        };

        input.axis_pair(input_store, gamepad)
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
