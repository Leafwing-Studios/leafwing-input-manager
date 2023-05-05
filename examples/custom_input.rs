use bevy::input::mouse::MouseMotion;
use bevy::input::InputSystem;
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionData;
use leafwing_input_manager::buttonlike::ButtonState;
use leafwing_input_manager::plugin::InputManagerSystem;
use leafwing_input_manager::prelude::*;
use leafwing_input_manager::user_input::{AxisLike, ButtonLike, InputLike, InputLikeMethods};
use serde::{Deserialize, Serialize};
use std::any::{Any, TypeId};

#[derive(Debug, Clone, Copy, PartialEq, Reflect, FromReflect, Serialize, Deserialize)]
#[reflect(Debug, PartialEq, Serialize, Deserialize)]
pub struct WindowMotion {
    /// The change in the position of the Window since the last event was sent.
    pub delta: Vec2,
}

#[derive(Actionlike, Debug, Clone, Copy)]
enum ChangeColorAction {
    Red,
    Green,
    Blue,
    Yellow,
}

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq, Copy, Reflect)]
enum WindowMotionDirection {
    Left = 0,
    Right,
    Up,
    Down,
}

impl ButtonLike for WindowMotionDirection {}
impl AxisLike for WindowMotionDirection {}

impl InputLikeMethods for WindowMotionDirection {
    fn clashes(&self, other: &dyn InputLikeMethods) -> bool {
        self.eq_dyn(other)
    }

    fn as_button(&self) -> Option<Box<dyn ButtonLike>> {
        Some(Box::new(*self))
    }

    fn as_axis(&self) -> Option<Box<dyn AxisLike>> {
        None
    }

    fn len(&self) -> usize {
        1
    }

    fn raw_inputs(&self) -> Vec<Box<(dyn InputLikeMethods)>> {
        vec![Box::new(*self)]
    }

    fn clone_dyn(&self) -> Box<dyn InputLikeMethods> {
        Box::new(*self)
    }

    fn as_serialize(&self) -> &dyn erased_serde::Serialize {
        self
    }

    fn input_variant_id(&self) -> usize {
        *self as usize
    }
}

pub struct WindowMotionInputStream<'a> {
    window_motion: &'a Events<WindowMotion>,
}

impl<'de> InputLike<'de> for WindowMotionDirection {
    fn which_pressed(world: &mut World) -> Vec<ActionData> {
        todo!()
    }
}

impl TryFrom<&Box<dyn InputLikeMethods>> for WindowMotionDirection {
    type Error = ();

    fn try_from(value: &Box<dyn InputLikeMethods>) -> Result<Self, Self::Error> {
        if value.as_ref().type_id() != TypeId::of::<WindowMotionDirection>() {
            return Err(());
        }

        match value.input_variant_id() {
            0 => Ok(WindowMotionDirection::Left),
            1 => Ok(WindowMotionDirection::Right),
            2 => Ok(WindowMotionDirection::Up),
            3 => Ok(WindowMotionDirection::Down),
            _ => Err(()),
        }
    }
}

impl From<&MouseMotion> for WindowMotionDirection {
    fn from(value: &MouseMotion) -> Self {
        if value.delta.x.abs() > value.delta.y.abs() {
            if value.delta.x.is_sign_positive() {
                Self::Right
            } else {
                Self::Left
            }
        } else if value.delta.y.is_sign_positive() {
            Self::Down
        } else {
            Self::Up
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(InputManagerPlugin::<ChangeColorAction>::default())
        .init_resource::<ActionState<ChangeColorAction>>()
        .register_type::<WindowMotionDirection>()
        .add_startup_system(setup)
        .add_systems((
            update_action_state_from_window_motion
                .in_base_set(CoreSet::PreUpdate)
                .in_set(InputManagerSystem::Update)
                .after(InputSystem),
            update_color,
        ))
        .run()
}

fn setup(mut commands: Commands) {
    commands.insert_resource(InputMap::<ChangeColorAction>::new([
        (WindowMotionDirection::Left, ChangeColorAction::Red),
        (WindowMotionDirection::Right, ChangeColorAction::Green),
        (WindowMotionDirection::Up, ChangeColorAction::Blue),
        (WindowMotionDirection::Down, ChangeColorAction::Yellow),
    ]));
    commands.spawn(Camera2dBundle::default());
}

fn update_color(
    action_state: Res<ActionState<ChangeColorAction>>,
    mut clear_color: ResMut<ClearColor>,
) {
    if action_state.just_pressed(ChangeColorAction::Blue) {
        clear_color.0 = Color::BLUE;
    }
    if action_state.just_pressed(ChangeColorAction::Green) {
        clear_color.0 = Color::GREEN;
    }
    if action_state.just_pressed(ChangeColorAction::Yellow) {
        clear_color.0 = Color::YELLOW;
    }
    if action_state.just_pressed(ChangeColorAction::Red) {
        clear_color.0 = Color::RED;
    }
}

fn update_action_state_from_window_motion(
    input_map: Res<InputMap<ChangeColorAction>>,
    mut action_state: ResMut<ActionState<ChangeColorAction>>,
    mut movement_events: EventReader<MouseMotion>,
    mut directions_last_frame: Local<Vec<WindowMotionDirection>>,
) {
    let mut directions_this_frame: Vec<WindowMotionDirection> = movement_events
        .iter()
        .map(WindowMotionDirection::from)
        .collect();

    for (input, action) in input_map.iter() {
        for input in input.iter() {
            if let Ok(mouse_movement) = WindowMotionDirection::try_from(input) {
                if !directions_this_frame.is_empty() {
                    println!(
                        "{mouse_movement:?} {directions_this_frame:?} {}",
                        directions_this_frame.contains(&mouse_movement)
                    );
                }
                if directions_this_frame.contains(&mouse_movement) {
                    action_state.set_action_data(
                        action,
                        ActionData {
                            state: if directions_last_frame.contains(&mouse_movement) {
                                ButtonState::Pressed
                            } else {
                                ButtonState::JustPressed
                            },
                            value: 1.0,
                            axis_pair: None,
                            timing: Default::default(),
                            consumed: false,
                        },
                    )
                } else {
                    // TODO: this doesn't work correctly because there can be multiple inputs
                    //       bound to an action, so setting the action data to released overrides
                    //       the other sources.
                    //       Need to use something like input_map::which_pressed
                    action_state.set_action_data(
                        action,
                        ActionData {
                            state: if directions_last_frame.contains(&mouse_movement) {
                                ButtonState::JustReleased
                            } else {
                                ButtonState::Released
                            },
                            value: 0.0,
                            axis_pair: None,
                            timing: Default::default(),
                            consumed: false,
                        },
                    )
                }
            }
            std::mem::swap(directions_last_frame.as_mut(), &mut directions_this_frame);
        }
    }
}
