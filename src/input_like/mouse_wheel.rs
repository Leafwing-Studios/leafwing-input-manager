use crate::input_like::{ButtonLike, DualAxisLike, InputLike, InputLikeObject, SingleAxisLike};
use bevy::app::App;
use bevy::input::mouse::MouseWheel;
use bevy::input::{Input, InputSystem};
use bevy::prelude::{
    DetectChangesMut, EventReader, IntoSystemConfig, Plugin, Reflect, ResMut, World,
};
use serde::{Deserialize, Serialize};

/// A buttonlike-input triggered by [`MouseWheel`](bevy::input::mouse::MouseWheel) events
///
/// These will be considered pressed if non-zero net movement in the correct direction is detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum MouseWheelDirection {
    /// Corresponds to `+y`
    Up,
    /// Corresponds to `-y`
    Down,
    /// Corresponds to `+x`
    Right,
    /// Corresponds to `-x`
    Left,
}

pub struct MouseWheelDirectionPlugin;
impl Plugin for MouseWheelDirectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Input<MouseWheelDirection>>();
        app.add_system(mouse_wheel_direction_system.in_set(InputSystem));
    }
}

pub fn mouse_wheel_direction_system(
    mut mouse_wheel_direction_input: ResMut<Input<MouseWheelDirection>>,
    mut event_reader: EventReader<MouseWheel>,
) {
    let mut total_x_movement = 0.0;
    let mut total_y_movement = 0.0;
    for mouse_wheel_event in event_reader.iter() {
        total_x_movement += mouse_wheel_event.x;
        total_y_movement += mouse_wheel_event.y;
    }

    for (value, pos, neg) in [
        (
            total_x_movement,
            MouseWheelDirection::Right,
            MouseWheelDirection::Left,
        ),
        (
            total_y_movement,
            MouseWheelDirection::Up,
            MouseWheelDirection::Down,
        ),
    ] {
        if value > 0.0 {
            mouse_wheel_direction_input.press(pos);
            mouse_wheel_direction_input.release(neg);
        } else if value < 0.0 {
            mouse_wheel_direction_input.press(neg);
            mouse_wheel_direction_input.release(pos);
        } else {
            mouse_wheel_direction_input.release(pos);
            mouse_wheel_direction_input.release(neg);
        }
    }
}

impl ButtonLike for MouseWheelDirection {
    fn input_pressed(&self, world: &World) -> bool {
        let Some(input) = world.get_resource::<Input<MouseWheelDirection>>() else {
            return false;
        };

        input.pressed(*self)
    }

    fn clone_dyn(&self) -> Box<dyn ButtonLike> {
        Box::new(*self)
    }
}

impl SingleAxisLike for MouseWheelDirection {}

impl InputLikeObject for MouseWheelDirection {
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

impl<'a> InputLike<'a> for MouseWheelDirection {}
