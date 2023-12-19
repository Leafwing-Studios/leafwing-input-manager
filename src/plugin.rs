//! Contains main plugin exported by this crate.

use crate::action_state::{ActionData, ActionState, Timing};
use crate::axislike::{
    AxisType, DeadZoneShape, DualAxis, DualAxisData, MouseMotionAxisType, MouseWheelAxisType,
    SingleAxis, VirtualAxis, VirtualDPad,
};
use crate::buttonlike::{MouseMotionDirection, MouseWheelDirection};
use crate::clashing_inputs::ClashStrategy;
use crate::dynamic_action::DynAction;
use crate::input_map::InputMap;
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

/// A [`Plugin`] that collects [`Input`](bevy::input::Input) from disparate sources, producing an [`ActionState`] that can be conveniently checked
///
/// This plugin needs to be passed in an [`Actionlike`] enum type that you've created for your game.
/// Each variant represents a "virtual button" whose state is stored in an [`ActionState`] struct.
///
/// Each [`InputManagerBundle`](crate::InputManagerBundle) contains:
///  - an [`InputMap`](crate::input_map::InputMap) component, which stores an entity-specific mapping between the assorted input streams and an internal representation of "actions"
///  - an [`ActionState`] component, which stores the current input state for that entity in an source-agnostic fashion
///
/// If you have more than one distinct type of action (e.g. menu actions, camera actions and player actions), consider creating multiple `Actionlike` enums
/// and adding a copy of this plugin for each `Actionlike` type.
///
/// ## Systems
///
/// All systems added by this plugin can be dynamically enabled and disabled by setting the value of the [`ToggleActions<A>`] resource is set.
/// This can be useful when working with states to pause the game, navigate menus or so on.
///
/// **WARNING:** These systems run during [`PreUpdate`].
/// If you have systems that care about inputs and actions that also run during this stage,
/// you must define an ordering between your systems or behavior will be very erratic.
/// The stable system sets for these systems are available under [`InputManagerSystem`] enum.
///
/// Complete list:
///
/// - [`tick_action_state`](crate::systems::tick_action_state), which resets the `pressed` and `just_pressed` fields of the [`ActionState`] each frame
/// - [`update_action_state`](crate::systems::update_action_state), which collects [`Input`](bevy::input::Input) resources to update the [`ActionState`]
/// - [`update_action_state_from_interaction`](crate::systems::update_action_state_from_interaction), for triggering actions from buttons
///    - powers the [`ActionStateDriver`](crate::action_state::ActionStateDriver) component based on an [`Interaction`](bevy::ui::Interaction) component
/// - [`release_on_disable`](crate::systems::release_on_disable), which resets action states when [`ToggleActions`] is flipped, to avoid persistent presses.
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
    /// or constructed from [`ActionDiff`](crate::action_state::ActionDiff) event streams.
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
                        .run_if(run_if_enabled::<A>)
                        .in_set(InputManagerSystem::Tick)
                        .before(InputManagerSystem::Update),
                )
                .add_systems(
                    PreUpdate,
                    release_on_disable::<A>
                        .in_set(InputManagerSystem::ReleaseOnDisable)
                        .after(InputManagerSystem::Update),
                )
                .add_systems(PostUpdate, release_on_input_map_removed::<A>);

                app.add_systems(
                    PreUpdate,
                    update_action_state::<A>
                        .run_if(run_if_enabled::<A>)
                        .in_set(InputManagerSystem::Update),
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
                        .before(InputManagerSystem::ReleaseOnDisable)
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
                        .run_if(run_if_enabled::<A>)
                        .in_set(InputManagerSystem::ManualControl),
                );
            }
            Machine::Server => {
                app.add_systems(
                    PreUpdate,
                    tick_action_state::<A>
                        .run_if(run_if_enabled::<A>)
                        .in_set(InputManagerSystem::Tick),
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
            .register_type::<Timing>()
            .register_type::<VirtualDPad>()
            .register_type::<VirtualAxis>()
            .register_type::<SingleAxis>()
            .register_type::<DualAxis>()
            .register_type::<AxisType>()
            .register_type::<MouseWheelAxisType>()
            .register_type::<MouseMotionAxisType>()
            .register_type::<DualAxisData>()
            .register_type::<DeadZoneShape>()
            .register_type::<ButtonState>()
            .register_type::<MouseWheelDirection>()
            .register_type::<MouseMotionDirection>()
            .register_type::<DynAction>()
            // Resources
            .init_resource::<ToggleActions<A>>()
            .init_resource::<ClashStrategy>();
    }
}

/// Controls whether or not the [`ActionState`] / [`InputMap`](crate::input_map::InputMap) pairs of type `A` are active
///
/// If this resource does not exist, actions work normally, as if `ToggleActions::enabled == true`.
#[derive(Resource)]
pub struct ToggleActions<A: Actionlike> {
    /// When this is false, [`ActionState`]'s corresponding to `A` will ignore user inputs
    ///
    /// When this is set to false, all corresponding [`ActionState`]s are released
    pub enabled: bool,
    /// Marker that stores the type of action to toggle
    pub phantom: PhantomData<A>,
}

impl<A: Actionlike> ToggleActions<A> {
    /// A [`ToggleActions`] in enabled state.
    pub const ENABLED: ToggleActions<A> = ToggleActions::<A> {
        enabled: true,
        phantom: PhantomData::<A>,
    };
    /// A [`ToggleActions`] in disabled state.
    pub const DISABLED: ToggleActions<A> = ToggleActions::<A> {
        enabled: false,
        phantom: PhantomData::<A>,
    };
}

// Implement manually to not require [`Default`] for `A`
impl<A: Actionlike> Default for ToggleActions<A> {
    fn default() -> Self {
        Self {
            enabled: true,
            phantom: PhantomData::<A>,
        }
    }
}

/// [`SystemSet`]s for the [`crate::systems`] used by this crate
///
/// `Reset` must occur before `Update`
#[derive(SystemSet, Clone, Hash, Debug, PartialEq, Eq)]
pub enum InputManagerSystem {
    /// Advances action timers.
    ///
    /// Cleans up the state of the input manager, clearing `just_pressed` and just_released`
    Tick,
    /// Collects input data to update the [`ActionState`]
    Update,
    /// Release all actions in all [`ActionState`]s if [`ToggleActions`] was added
    ReleaseOnDisable,
    /// Manually control the [`ActionState`]
    ///
    /// Must run after [`InputManagerSystem::Update`] or the action state will be overridden
    ManualControl,
}
