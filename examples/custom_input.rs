use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use itertools::Itertools;
use leafwing_input_manager::axislike::DualAxisData;
use leafwing_input_manager::prelude::*;
use leafwing_input_manager::press_scheduler::PressScheduler;
use leafwing_input_manager::user_input::{
    AxisLike, ButtonLike, InputLike, InputLikeObject, InputStreamsTrait, ReflectInputLike,
};
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
#[reflect(InputLike, Debug, PartialEq, Serialize, Deserialize)]
enum WindowMotionDirection {
    Left = 0,
    Right,
    Up,
    Down,
}

impl ButtonLike for WindowMotionDirection {}
impl AxisLike for WindowMotionDirection {}

impl InputLikeObject for WindowMotionDirection {
    fn clashes(&self, other: &dyn InputLikeObject) -> bool {
        if let Some(other) = other.as_reflect().downcast_ref::<WindowMotionDirection>() {
            return self == other;
        }
        false
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

    fn raw_inputs(&self) -> Vec<Box<(dyn InputLikeObject)>> {
        vec![Box::new(*self)]
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

pub struct WindowMotionInputStream<'a> {
    window_motion: &'a Events<WindowMotion>,
}

impl<'a> InputStreamsTrait<'a> for WindowMotionInputStream<'a> {
    fn input_pressed(&self, world: &World, input: &dyn InputLikeObject) -> bool {
        let Some(input) = input.as_reflect().downcast_ref::<WindowMotionDirection>() else {
            return false;
        };
        // FIXME: verify that this works and doesn't double count events
        let mut event_reader = self.window_motion.get_reader();
        event_reader
            .iter(self.window_motion)
            .map(WindowMotionDirection::from)
            .contains(input)
    }

    fn input_value(&self, world: &World, input: &dyn InputLikeObject) -> f32 {
        let Some(input) = input.as_reflect().downcast_ref::<WindowMotionDirection>() else {
            return 0.0;
        };
        let mut event_reader = self.window_motion.get_reader();
        event_reader
            .iter(self.window_motion)
            .find(|i| WindowMotionDirection::from(*i) == *input)
            .map(|x| x.delta.x.abs().max(x.delta.y.abs()))
            .unwrap_or_default()
    }

    fn input_axis_pair(&self, world: &World, input: &dyn InputLikeObject) -> Option<DualAxisData> {
        let Some(input) = input.as_reflect().downcast_ref::<WindowMotionDirection>() else {
            return None;
        };
        let mut event_reader = self.window_motion.get_reader();
        Some(
            event_reader
                .iter(self.window_motion)
                .find(|i| WindowMotionDirection::from(*i) == *input)
                .map(|x| DualAxisData::from_xy(x.delta))
                .unwrap_or_default(),
        )
    }
}

impl<'de> InputLike<'de> for WindowMotionDirection {
    fn input_streams(world: &mut World) -> Box<dyn InputStreamsTrait<'_> + '_> {
        Box::new(WindowMotionInputStream {
            window_motion: world.resource::<Events<WindowMotion>>(),
        })
    }
}

impl TryFrom<&Box<dyn InputLikeObject>> for WindowMotionDirection {
    type Error = ();

    fn try_from(value: &Box<dyn InputLikeObject>) -> Result<Self, Self::Error> {
        if value.as_ref().type_id() != TypeId::of::<WindowMotionDirection>() {
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
        .add_plugins(DefaultPlugins)
        .add_plugin(InputManagerPlugin::<ChangeColorAction>::default())
        .init_resource::<ActionState<ChangeColorAction>>()
        .add_event::<WindowMotion>()
        .register_type::<WindowMotionDirection>()
        .add_startup_system(setup)
        .add_system(update_action_state::<ChangeColorAction>)
        .add_systems((update_window_motion, apply_system_buffers, update_color).chain())
        .run()
}

// TODO: This should be done in the library instead of in this example
fn update_action_state<A: Actionlike>(world: &mut World) {
    let input_likes = {
        let type_registry = world.resource::<AppTypeRegistry>().read();

        type_registry
            .iter()
            .filter_map(|type_registration| {
                type_registry.get_type_data::<ReflectInputLike>(type_registration.type_id())
            })
            .cloned()
            .collect::<Vec<_>>()
    };

    let mut state = SystemState::<(
        Res<ClashStrategy>,
        Option<ResMut<ActionState<A>>>,
        Option<Res<InputMap<A>>>,
        Option<ResMut<PressScheduler<A>>>,
        Query<(
            &mut ActionState<A>,
            &InputMap<A>,
            Option<&mut PressScheduler<A>>,
        )>,
    )>::new(world);
    let (clash_strategy, mut action_state, input_map, mut press_scheduler, mut query) =
        state.get_mut(world);
    let resources = input_map
        .zip(action_state)
        .map(|(input_map, action_state)| {
            (
                Mut::from(action_state),
                input_map.into_inner(),
                press_scheduler.map(Mut::from),
            )
        });

    let input_streams: Vec<_> = input_likes
        .iter()
        .map(|input_like| (input_like.input_streams)(world))
        .collect();
    for (mut action_state, input_map, press_scheduler) in query.iter_mut().chain(resources) {
        action_state.update(input_map.which_pressed(input_streams, clash_strategy));
        if let Some(mut press_scheduler) = press_scheduler {
            press_scheduler.apply(&mut action_state);
        }
    }
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
