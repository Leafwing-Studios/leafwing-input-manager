//! Shows how you might integrate your own library with leafwing_input_manager to provide custom
//! inputs.
//! Creates a custom input type from [`WindowMotion`] events, and binds it to changing the background
//! color.
use bevy::prelude::*;
use bevy::window::WindowResolution;
use itertools::Itertools;
use leafwing_input_manager::input_like::{
    ButtonLike, DualAxisLike, InputLike, InputLikeObject, SingleAxisLike,
};
use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};
use std::any::TypeId;

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

impl ButtonLike for WindowMotionDirection {
    fn input_pressed(&self, world: &World) -> bool {
        // FIXME: verify that this works and doesn't double count events
        let Some(window_motion) = world.get_resource::<Events<WindowMotion>>() else {
            return false;
        };
        let mut event_reader = window_motion.get_reader();
        event_reader
            .iter(window_motion)
            .map(WindowMotionDirection::from)
            .contains(self)
    }

    fn clone_dyn(&self) -> Box<dyn ButtonLike> {
        Box::new(*self)
    }
}
impl SingleAxisLike for WindowMotionDirection {
    fn input_value(&self, world: &World) -> f32 {
        let Some(window_motion) = world.get_resource::<Events<WindowMotion>>() else {
            return 0.0;
        };
        let mut event_reader = window_motion.get_reader();
        event_reader
            .iter(window_motion)
            .find(|i| WindowMotionDirection::from(*i) == *self)
            .map(|x| x.delta.x.abs().max(x.delta.y.abs()))
            .unwrap_or_default()
    }
}

impl InputLikeObject for WindowMotionDirection {
    fn as_button(&self) -> Option<&dyn ButtonLike> {
        Some(self)
    }

    fn as_axis(&self) -> Option<&dyn SingleAxisLike> {
        None
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

impl<'de> InputLike<'de> for WindowMotionDirection {}

impl TryFrom<&Box<dyn InputLikeObject>> for WindowMotionDirection {
    type Error = ();

    fn try_from(value: &Box<dyn InputLikeObject>) -> Result<Self, Self::Error> {
        if value.as_ref().as_reflect().type_id() != TypeId::of::<WindowMotionDirection>() {
            return Err(());
        }

        value
            .as_reflect()
            .downcast_ref::<WindowMotionDirection>()
            .cloned()
            .ok_or(())
    }
}

impl From<&WindowMotion> for WindowMotionDirection {
    fn from(value: &WindowMotion) -> Self {
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
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(480.0, 360.0),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugin(InputManagerPlugin::<ChangeColorAction>::default())
        .init_resource::<ActionState<ChangeColorAction>>()
        .add_event::<WindowMotion>()
        .register_type::<WindowMotionDirection>()
        .add_startup_system(setup)
        .add_systems((update_window_motion, apply_system_buffers, update_color).chain())
        .run()
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(InputMap::<ChangeColorAction>::new([
        (WindowMotionDirection::Left, ChangeColorAction::Red),
        (WindowMotionDirection::Right, ChangeColorAction::Green),
        (WindowMotionDirection::Up, ChangeColorAction::Blue),
        (WindowMotionDirection::Down, ChangeColorAction::Yellow),
    ]));
    commands.spawn(Camera2dBundle::default());
    commands.spawn((TextBundle {
        text: Text::from_section(
            "Drag the window around to change the color!",
            TextStyle {
                font: asset_server.load("Montserrat/Montserrat-VariableFont_wght.ttf"),
                font_size: 40.0,
                color: Color::WHITE,
            },
        )
        .with_alignment(TextAlignment::Center),
        style: Style {
            position_type: PositionType::Absolute,
            position: UiRect::all(Val::Px(10.0)),
            max_size: Size::new(Val::Px(460.0), Val::Undefined),
            ..Default::default()
        },
        ..Default::default()
    },));
}

fn update_color(
    action_state: Res<ActionState<ChangeColorAction>>,
    mut clear_color: ResMut<ClearColor>,
    time: Res<Time>,
) {
    let delta = time.delta_seconds() * 2.0;
    if action_state.pressed(ChangeColorAction::Blue) {
        clear_color.0 += Color::BLUE * delta;
        clear_color.0 += Color::rgb(1.0, 1.0, 0.0) * -delta;
    }
    if action_state.pressed(ChangeColorAction::Green) {
        clear_color.0 += Color::GREEN * delta;
        clear_color.0 += Color::rgb(1.0, 0.0, 1.0) * -delta;
    }
    if action_state.pressed(ChangeColorAction::Yellow) {
        clear_color.0 += Color::YELLOW * delta;
        clear_color.0 += Color::rgb(0.0, 0.0, 1.0) * -delta;
    }
    if action_state.pressed(ChangeColorAction::Red) {
        clear_color.0 += Color::RED * delta;
        clear_color.0 += Color::rgb(0.0, 1.0, 1.0) * -delta;
    }
    let old_color = clear_color.0;
    clear_color.0.set_r(old_color.r().clamp(0.0, 1.0));
    clear_color.0.set_g(old_color.g().clamp(0.0, 1.0));
    clear_color.0.set_b(old_color.b().clamp(0.0, 1.0));
}

fn update_window_motion(
    mut events: EventWriter<WindowMotion>,
    mut window_moved_events: EventReader<WindowMoved>,
    mut last_window_pos: Local<IVec2>,
) {
    for event in window_moved_events.iter() {
        let delta = Vec2::new(
            event.position.x as f32 - last_window_pos.x as f32,
            event.position.y as f32 - last_window_pos.y as f32,
        );
        events.send(WindowMotion { delta });
        *last_window_pos = event.position;
    }
}
