//! Utilities for testing user input.

use bevy::{app::App, math::Vec2, prelude::World};

use crate::input_streams::InputStreams;

use super::{Axislike, Buttonlike, DualAxislike};

/// A trait used to quickly fetch the value of a given [`UserInput`].
///
/// This can be useful for testing purposes.
pub trait FetchUserInput {
    /// Returns `true` if the given [`Buttonlike`] input is currently pressed.
    fn pressed(&self, input: impl Buttonlike) -> bool;

    /// Returns the value of the given [`Axislike`] input.
    fn read_axis_value(&self, input: impl Axislike) -> f32;

    /// Returns the value of the given [`DualAxislike`] input.
    fn read_dual_axis_values(&self, input: impl DualAxislike) -> Vec2;
}

impl<'a> FetchUserInput for InputStreams<'a> {
    fn pressed(&self, input: impl Buttonlike) -> bool {
        input.pressed(self)
    }

    fn read_axis_value(&self, input: impl Axislike) -> f32 {
        input.value(self)
    }

    fn read_dual_axis_values(&self, input: impl DualAxislike) -> Vec2 {
        input.axis_pair(self)
    }
}

impl FetchUserInput for World {
    fn pressed(&self, input: impl Buttonlike) -> bool {
        let input_streams = InputStreams::from_world(self, None);
        input_streams.pressed(input)
    }

    fn read_axis_value(&self, input: impl Axislike) -> f32 {
        let input_streams = InputStreams::from_world(self, None);
        input_streams.read_axis_value(input)
    }

    fn read_dual_axis_values(&self, input: impl DualAxislike) -> Vec2 {
        let input_streams = InputStreams::from_world(self, None);
        input_streams.read_dual_axis_values(input)
    }
}

impl FetchUserInput for App {
    fn pressed(&self, input: impl Buttonlike) -> bool {
        self.world().pressed(input)
    }

    fn read_axis_value(&self, input: impl Axislike) -> f32 {
        self.world().read_axis_value(input)
    }

    fn read_dual_axis_values(&self, input: impl DualAxislike) -> Vec2 {
        self.world().read_dual_axis_values(input)
    }
}
