use crate::input_like::mouse_motion::{mouse_motion_direction_system, MouseMotionDirection};
use crate::input_like::{ButtonLike, DualAxisLike, InputLike, InputLikeObject, SingleAxisLike};
use crate::plugin::InputManagerSystem;
use bevy::app::App;
use bevy::input::mouse::MouseWheel;
use bevy::input::{Axis, Input, InputSystem};
use bevy::math::Vec2;
use bevy::prelude::{EventReader, IntoSystemConfigs, Plugin, PreUpdate, Reflect, ResMut, World};
use serde::{Deserialize, Serialize};

// TODO: Move this plugin of the mouse_wheel module. Not sure where we want it yet.
/// Adds Bevy [`Input`] resources for [`MouseWheelDirection`](MouseWheelDirection)
/// and [`MouseMotionDirection`](MouseMotionDirection), and [`Axis`] resource for
/// [`MouseWheelAxis`](MouseWheelAxis) with systems to update them.
pub struct ExtraMouseInputsPlugin;

impl Plugin for ExtraMouseInputsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Input<MouseWheelDirection>>();
        app.init_resource::<Axis<MouseWheelAxis>>();
        app.init_resource::<Input<MouseMotionDirection>>();
        app.add_systems(
            PreUpdate,
            (
                mouse_wheel_direction_system,
                mouse_wheel_axis_system,
                mouse_motion_direction_system,
            )
                .in_set(InputManagerSystem::Update)
                .after(InputSystem),
        );
    }
}

/// A buttonlike-input triggered by [`MouseWheel`](MouseWheel) events
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

impl From<MouseWheelDirection> for Vec2 {
    fn from(value: MouseWheelDirection) -> Self {
        match value {
            MouseWheelDirection::Up => Vec2::new(0.0, 1.0),
            MouseWheelDirection::Down => Vec2::new(0.0, -1.0),
            MouseWheelDirection::Right => Vec2::new(1.0, 0.0),
            MouseWheelDirection::Left => Vec2::new(-1.0, 0.0),
        }
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

    fn clone_button(&self) -> Box<dyn ButtonLike> {
        Box::new(*self)
    }
}

impl SingleAxisLike for MouseWheelDirection {
    fn clone_axis(&self) -> Box<dyn SingleAxisLike> {
        Box::new(*self)
    }
}

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

/// The direction of motion of the mouse wheel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum MouseWheelAxis {
    /// Horizontal movement.
    ///
    /// This is much less common than the `Y` variant, and is only supported on some devices.
    X,
    /// Vertical movement.
    ///
    /// This is the standard behavior for a mouse wheel, used to scroll up and down pages.
    Y,
}

pub fn mouse_wheel_axis_system(
    mut mouse_wheel_axis_input: ResMut<Axis<MouseWheelAxis>>,
    mut event_reader: EventReader<MouseWheel>,
) {
    let mut total_x_movement = 0.0;
    let mut total_y_movement = 0.0;
    for mouse_wheel_event in event_reader.iter() {
        total_x_movement += mouse_wheel_event.x;
        total_y_movement += mouse_wheel_event.y;
    }

    mouse_wheel_axis_input.set(MouseWheelAxis::X, total_x_movement);
    mouse_wheel_axis_input.set(MouseWheelAxis::Y, total_y_movement);
}

impl ButtonLike for MouseWheelAxis {
    fn input_pressed(&self, world: &World) -> bool {
        let Some(axis) = world.get_resource::<Axis<MouseWheelAxis>>() else {
            return false;
        };

        axis.get(*self).unwrap_or_default() != 0.0
    }

    fn clone_button(&self) -> Box<dyn ButtonLike> {
        Box::new(*self)
    }
}

impl SingleAxisLike for MouseWheelAxis {
    fn input_value(&self, world: &World) -> f32 {
        let Some(axis) = world.get_resource::<Axis<MouseWheelAxis>>() else {
            return 0.0;
        };

        axis.get_unclamped(*self).unwrap_or_default()
    }

    fn clone_axis(&self) -> Box<dyn SingleAxisLike> {
        Box::new(*self)
    }
}

impl InputLikeObject for MouseWheelAxis {
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

impl<'a> InputLike<'a> for MouseWheelAxis {}
