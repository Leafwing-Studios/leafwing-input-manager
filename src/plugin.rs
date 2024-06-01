//! Contains the main plugin exported by this crate.

use crate::action_state::{ActionData, ActionState};
use crate::axislike::{
    AxisType, DualAxis, DualAxisData, MouseMotionAxisType, MouseWheelAxisType, SingleAxis,
    VirtualAxis, VirtualDPad,
};
use crate::buttonlike::{MouseMotionDirection, MouseWheelDirection};
use crate::clashing_inputs::ClashStrategy;
use crate::input_map::InputMap;
use crate::input_processing::*;
#[cfg(feature = "timing")]
use crate::timing::Timing;
use crate::user_input::{InputKind, Modifier, UserInput};
use crate::Actionlike;
use core::hash::Hash;
use core::marker::PhantomData;
use std::fmt::Debug;

use bevy::app::{App, Plugin};
use bevy::ecs::prelude::*;
use bevy::input::{ButtonState, InputSystem};
use bevy::prelude::{PostUpdate, PreUpdate};
use bevy::reflect::TypePath;
#[cfg(feature = "ui")]
use bevy::ui::UiSystem;

/// A [`Plugin`] that collects [`ButtonInput`](bevy::input::ButtonInput) from disparate sources,
/// producing an [`ActionState`] that can be conveniently checked
///
/// This plugin needs to be passed in an [`Actionlike`] enum type that you've created for your game.
/// Each variant represents a "virtual button" whose state is stored in an [`ActionState`] struct.
///
/// Each [`InputManagerBundle`](crate::InputManagerBundle) contains:
///  - an [`InputMap`] component, which stores an entity-specific mapping between the assorted input streams and an internal representation of "actions"
///  - an [`ActionState`] component, which stores the current input state for that entity in a source-agnostic fashion
///
/// If you have more than one distinct type of action (e.g., menu actions, camera actions, and player actions),
/// consider creating multiple `Actionlike` enums
/// and adding a copy of this plugin for each `Actionlike` type.
///
/// All actions can be dynamically enabled or disabled by calling the relevant methods on
/// `ActionState<A>`. This can be useful when working with states to pause the game, navigate
/// menus, and so on.
///
/// ## Systems
///
/// **WARNING:** These systems run during [`PreUpdate`].
/// If you have systems that care about inputs and actions that also run during this stage,
/// you must define an ordering between your systems or behavior will be very erratic.
/// The stable system sets for these systems are available under [`InputManagerSystem`] enum.
///
/// Complete list:
///
/// - [`tick_action_state`](crate::systems::tick_action_state), which resets the `pressed` and `just_pressed` fields of the [`ActionState`] each frame
/// - [`update_action_state`](crate::systems::update_action_state), which collects [`ButtonInput`](bevy::input::ButtonInput) resources to update the [`ActionState`]
/// - [`update_action_state_from_interaction`](crate::systems::update_action_state_from_interaction), for triggering actions from buttons
///    - powers the [`ActionStateDriver`](crate::action_driver::ActionStateDriver) component based on an [`Interaction`](bevy::ui::Interaction) component
pub struct InputManagerPlugin<A: Actionlike> {
    _phantom: PhantomData<A>,
    machine: Machine,
}

// Deriving default induces an undesired bound on the generic
impl<A: Actionlike> Default for InputManagerPlugin<A> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
            machine: Machine::Client,
        }
    }
}

impl<A: Actionlike> InputManagerPlugin<A> {
    /// Creates a version of the plugin intended to run on the server
    ///
    /// Inputs will not be processed; instead, [`ActionState`]
    /// should be copied directly from the state provided by the client,
    /// or constructed from [`ActionDiff`](crate::action_diff::ActionDiff) event streams.
    #[must_use]
    pub fn server() -> Self {
        Self {
            _phantom: PhantomData,
            machine: Machine::Server,
        }
    }
}

/// Which machine is this plugin running on?
enum Machine {
    Server,
    Client,
}

impl<A: Actionlike + TypePath> Plugin for InputManagerPlugin<A> {
    fn build(&self, app: &mut App) {
        use crate::systems::*;

        match self.machine {
            Machine::Client => {
                app.add_systems(
                    PreUpdate,
                    tick_action_state::<A>
                        .in_set(InputManagerSystem::Tick)
                        .before(InputManagerSystem::Update),
                )
                .add_systems(PostUpdate, release_on_input_map_removed::<A>);

                app.add_systems(
                    PreUpdate,
                    update_action_state::<A>.in_set(InputManagerSystem::Update),
                );

                app.configure_sets(PreUpdate, InputManagerSystem::Update.after(InputSystem));

                #[cfg(feature = "egui")]
                app.configure_sets(
                    PreUpdate,
                    InputManagerSystem::Update.after(bevy_egui::EguiSet::ProcessInput),
                );

                #[cfg(feature = "ui")]
                app.configure_sets(PreUpdate, InputManagerSystem::Update.after(UiSystem::Focus));

                #[cfg(feature = "ui")]
                app.configure_sets(
                    PreUpdate,
                    InputManagerSystem::ManualControl
                        .after(InputManagerSystem::Tick)
                        // Must run after the system is updated from inputs, or it will be forcibly released due to the inputs
                        // not being pressed
                        .after(InputManagerSystem::Update)
                        .after(UiSystem::Focus)
                        .after(InputSystem),
                );

                #[cfg(feature = "ui")]
                app.add_systems(
                    PreUpdate,
                    update_action_state_from_interaction::<A>
                        .in_set(InputManagerSystem::ManualControl),
                );
            }
            Machine::Server => {
                app.add_systems(
                    PreUpdate,
                    tick_action_state::<A>.in_set(InputManagerSystem::Tick),
                );
            }
        };

        app.register_type::<ActionState<A>>()
            .register_type::<InputMap<A>>()
            .register_type::<UserInput>()
            .register_type::<InputKind>()
            .register_type::<ActionData>()
            .register_type::<Modifier>()
            .register_type::<ActionState<A>>()
            .register_type::<VirtualDPad>()
            .register_type::<VirtualAxis>()
            .register_type::<SingleAxis>()
            .register_type::<DualAxis>()
            .register_type::<AxisType>()
            .register_type::<MouseWheelAxisType>()
            .register_type::<MouseMotionAxisType>()
            .register_type::<DualAxisData>()
            .register_type::<ButtonState>()
            .register_type::<MouseWheelDirection>()
            .register_type::<MouseMotionDirection>()
            // Processors
            .register_type::<AxisProcessor>()
            .register_type::<AxisBounds>()
            .register_type::<AxisExclusion>()
            .register_type::<AxisDeadZone>()
            .register_type::<DualAxisProcessor>()
            .register_type::<DualAxisInverted>()
            .register_type::<DualAxisSensitivity>()
            .register_type::<DualAxisBounds>()
            .register_type::<DualAxisExclusion>()
            .register_type::<DualAxisDeadZone>()
            .register_type::<CircleBounds>()
            .register_type::<CircleExclusion>()
            .register_type::<CircleDeadZone>()
            // Resources
            .init_resource::<ClashStrategy>();

        #[cfg(feature = "timing")]
        app.register_type::<Timing>();
    }
}

/// [`SystemSet`]s for the [`crate::systems`] used by this crate
///
/// `Reset` must occur before `Update`
#[derive(SystemSet, Clone, Hash, Debug, PartialEq, Eq)]
pub enum InputManagerSystem {
    /// Advances action timers.
    ///
    /// Cleans up the state of the input manager, clearing `just_pressed` and `just_released`
    Tick,
    /// Collects input data to update the [`ActionState`]
    Update,
    /// Manually control the [`ActionState`]
    ///
    /// Must run after [`InputManagerSystem::Update`] or the action state will be overridden
    ManualControl,
}
