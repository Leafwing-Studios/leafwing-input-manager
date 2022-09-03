//! Contains main plugin exported by this crate.

use crate::clashing_inputs::ClashStrategy;
use crate::Actionlike;
use core::hash::Hash;
use core::marker::PhantomData;
use std::fmt::Debug;

use bevy::app::{App, CoreStage, Plugin};
use bevy::ecs::prelude::*;
use bevy::input::InputSystem;
#[cfg(feature = "ui")]
use bevy::ui::UiSystem;

/// A [`Plugin`] that collects [`Input`](bevy::input::Input) from disparate sources, producing an [`ActionState`](crate::action_state::ActionState) that can be conveniently checked
///
/// This plugin needs to be passed in an [`Actionlike`] enum type that you've created for your game.
/// Each variant represents a "virtual button" whose state is stored in an [`ActionState`](crate::action_state::ActionState) struct.
///
/// Each [`InputManagerBundle`](crate::InputManagerBundle) contains:
///  - an [`InputMap`](crate::input_map::InputMap) component, which stores an entity-specific mapping between the assorted input streams and an internal repesentation of "actions"
///  - an [`ActionState`](crate::action_state::ActionState) component, which stores the current input state for that entity in an source-agnostic fashion
///
/// If you have more than one distinct type of action (e.g. menu actions, camera actions and player actions), consider creating multiple `Actionlike` enums
/// and adding a copy of this plugin for each `Actionlike` type.
///  
/// ## Systems
///
/// All systems added by this plugin can be dynamically enabled and disabled by setting the value of the [`ToggleActions<A>`] resource is set.
/// This can be useful when working with states to pause the game, navigate menus or so on.
///
/// **WARNING:** Theses systems run during [`CoreStage::PreUpdate`].
/// If you have systems that care about inputs and actions that also run during this stage,
/// you must define an ordering between your systems or behavior will be very erratic.
/// The stable labels for these systems are available under [`InputManagerSystem`] enum.
///
/// Complete list:
///
/// - [`tick_action_state`](crate::systems::tick_action_state), which resets the `pressed` and `just_pressed` fields of the [`ActionState`](crate::action_state::ActionState) each frame
///     - labeled [`InputManagerSystem::Reset`]
/// - [`update_action_state`](crate::systems::update_action_state), which collects [`Input`](bevy::input::Input) resources to update the [`ActionState`](crate::action_state::ActionState)
///     - labeled [`InputManagerSystem::Update`]
/// - [`update_action_state_from_interaction`](crate::systems::update_action_state_from_interaction), for triggering actions from buttons
///    - powers the [`ActionStateDriver`](crate::action_state::ActionStateDriver) component baseod on an [`Interaction`](bevy::ui::Interaction) component
///    - labeled [`InputManagerSystem::Update`]
/// - [`release_on_disable`](crate::systems::release_on_disable), which resets action states when [`ToggleActions`] is flipped, to avoid persistent presses.
pub struct InputManagerPlugin<A: Actionlike> {
    _phantom: PhantomData<A>,
    machine: Machine,
}

// Deriving default induces an undesired bound on the generic
impl<A: Actionlike> Default for InputManagerPlugin<A> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData::default(),
            machine: Machine::Client,
        }
    }
}

impl<A: Actionlike> InputManagerPlugin<A> {
    /// Creates a version of the plugin intended to run on the server
    ///
    /// Inputs will not be processed; instead, [`ActionState`](crate::action_state::ActionState)
    /// should be copied directly from the state provided by the client,
    /// or constructed from [`ActionDiff`](crate::action_state::ActionDiff) event streams.
    #[must_use]
    pub fn server() -> Self {
        Self {
            _phantom: PhantomData::default(),
            machine: Machine::Server,
        }
    }
}

/// Which machine is this plugin running on?
enum Machine {
    Server,
    Client,
}

impl<A: Actionlike> Plugin for InputManagerPlugin<A> {
    fn build(&self, app: &mut App) {
        use crate::systems::*;

        match self.machine {
            Machine::Client => {
                app.add_system_to_stage(
                    CoreStage::PreUpdate,
                    tick_action_state::<A>
                        .with_run_criteria(run_if_enabled::<A>)
                        .label(InputManagerSystem::Tick)
                        .before(InputManagerSystem::Update),
                )
                .add_system_to_stage(
                    CoreStage::PreUpdate,
                    update_action_state::<A>
                        .with_run_criteria(run_if_enabled::<A>)
                        .label(InputManagerSystem::Update)
                        .after(InputSystem),
                )
                .add_system_to_stage(
                    CoreStage::PreUpdate,
                    release_on_disable::<A>
                        .label(InputManagerSystem::ReleaseOnDisable)
                        .after(InputManagerSystem::Update),
                )
                .add_system_to_stage(CoreStage::PostUpdate, release_on_input_map_removed::<A>);

                #[cfg(feature = "ui")]
                app.add_system_to_stage(
                    CoreStage::PreUpdate,
                    update_action_state_from_interaction::<A>
                        .with_run_criteria(run_if_enabled::<A>)
                        .label(InputManagerSystem::ManualControl)
                        .before(InputManagerSystem::ReleaseOnDisable)
                        .after(InputManagerSystem::Tick)
                        // Must run after the system is updated from inputs, or it will be forcibly released due to the inputs
                        // not being pressed
                        .after(InputManagerSystem::Update)
                        .after(UiSystem::Focus)
                        .after(InputSystem),
                );
            }
            Machine::Server => {
                app.add_system_to_stage(
                    CoreStage::PreUpdate,
                    tick_action_state::<A>
                        .with_run_criteria(run_if_enabled::<A>)
                        .label(InputManagerSystem::Tick)
                        .before(InputManagerSystem::Update),
                );
            }
        };

        // Resources
        app.init_resource::<ToggleActions<A>>()
            .init_resource::<ClashStrategy>();
    }
}

/// Controls whether or not the [`ActionState`](crate::action_state::ActionState) / [`InputMap`](crate::input_map::InputMap) pairs of type `A` are active
///
/// If this resource does not exist, actions work normally, as if `ToggleActions::enabled == true`.
pub struct ToggleActions<A: Actionlike> {
    /// When this is false, [`ActionState`](crate::action_state::ActionState)'s corresponding to `A` will ignore user inputs
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

/// [`SystemLabel`]s for the [`crate::systems`] used by this crate
///
/// `Reset` must occur before `Update`
#[derive(SystemLabel, Clone, Hash, Debug, PartialEq, Eq)]
pub enum InputManagerSystem {
    /// Advances actions timers to clean up the state of the input manager and clear `just_pressed` and just_released`
    Tick,
    /// Collects input data to update the [`ActionState`](crate::action_state::ActionState)
    Update,
    /// Release all actions in all [`ActionState`](crate::action_state::ActionState)s if [`ToggleActions`](crate::plugin::ToggleActions) was added
    ReleaseOnDisable,
    /// Manually control the [`ActionState`](crate::action_state::ActionState)
    ///
    /// Must run after [`InputManagerSystem::Update`] or the action state will be overriden
    ManualControl,
}
