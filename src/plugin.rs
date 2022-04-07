//! Contains main plugin exported by this crate.

use crate::clashing_inputs::ClashStrategy;
use crate::Actionlike;
use core::hash::Hash;
use core::marker::PhantomData;
use std::fmt::Debug;

use bevy_app::{App, CoreStage, Plugin};
use bevy_ecs::prelude::*;
use bevy_input::InputSystem;
#[cfg(feature = "ui")]
use bevy_ui::UiSystem;

/// A [`Plugin`] that collects [`Input`](bevy::input::Input) from disparate sources, producing an [`ActionState`](crate::action_state::ActionState) to consume in game logic
///
/// This plugin needs to be passed in an [`Actionlike`] enum type that you've created for your game,
/// which acts as a "virtual button" that can be comfortably consumed
///
/// Each [`InputManagerBundle`](crate::InputManagerBundle) contains:
///  - an [`InputMap`](crate::input_map::InputMap) component, which stores an entity-specific mapping between the assorted input streams and an internal repesentation of "actions"
///  - an [`ActionState`](crate::action_state::ActionState) component, which stores the current input state for that entity in an source-agnostic fashion
///
/// ## Systems
/// - [`tick_action_state`](crate::systems::tick_action_state), which resets the `pressed` and `just_pressed` fields of the [`ActionState`](crate::action_state::ActionState) each frame
///     - labeled [`InputManagerSystem::Reset`]
/// - [`update_action_state`](crate::systems::update_action_state), which collects [`Input`](bevy::input::Input) resources to update the [`ActionState`](crate::action_state::ActionState)
///     - labeled [`InputManagerSystem::Update`]
/// - [`update_action_state_from_interaction`](crate::systems::update_action_state_from_interaction), for triggering actions from buttons
///    - powers the [`ActionStateDriver`](crate::action_state::ActionStateDriver) component baseod on an [`Interaction`](bevy::ui::Interaction) component
///    - labeled [`InputManagerSystem::Update`]
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

        let input_manager_systems = match self.machine {
            Machine::Client => {
                let system_set = SystemSet::new()
                    .with_system(
                        tick_action_state::<A>
                            .label(InputManagerSystem::Reset)
                            .before(InputManagerSystem::Update),
                    )
                    .with_system(
                        update_action_state::<A>
                            .label(InputManagerSystem::Update)
                            .after(InputSystem),
                    )
                    .with_system(
                        release_on_disable::<A>
                            .label(InputManagerSystem::ReleaseOnDisable)
                            .after(InputManagerSystem::Update),
                    );
                #[cfg(feature = "ui")]
                {
                    system_set.with_system(
                        update_action_state_from_interaction::<A>
                            .label(InputManagerSystem::ManualControl)
                            .before(InputManagerSystem::ReleaseOnDisable)
                            .after(InputManagerSystem::Reset)
                            // Must run after the system is updated from inputs, or it will be forcibly released due to the inputs
                            // not being pressed
                            .after(InputManagerSystem::Update)
                            .after(UiSystem::Focus)
                            .after(InputSystem),
                    )
                }
                #[cfg(not(feature = "ui"))]
                {
                    system_set
                }
            }
            Machine::Server => SystemSet::new().with_system(
                tick_action_state::<A>
                    .label(InputManagerSystem::Reset)
                    .before(InputManagerSystem::Update),
            ),
        };

        // Add the systems to our app
        app.add_system_set_to_stage(CoreStage::PreUpdate, input_manager_systems);

        // Resources
        app.init_resource::<ClashStrategy>();
    }
}

/// A resource which disables all input for the specified [`Actionlike`] type `A` if present in world
pub struct DisableInput<A: Actionlike> {
    _phantom: PhantomData<A>,
}

// Implement manually to not require [`Default`] for `A`
impl<A: Actionlike> Default for DisableInput<A> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData::<A>,
        }
    }
}

/// [`SystemLabel`]s for the [`crate::systems`] used by this crate
///
/// `Reset` must occur before `Update`
#[derive(SystemLabel, Clone, Hash, Debug, PartialEq, Eq)]
pub enum InputManagerSystem {
    /// Cleans up the state of the input manager, clearing `just_pressed` and just_released`
    Reset,
    /// Collects input data to update the [`ActionState`](crate::action_state::ActionState)
    Update,
    /// Release all actions in all [`ActionState`]s if [`DisableInput`] was added
    ReleaseOnDisable,
    /// Manually control the [`ActionState`](crate::action_state::ActionState)
    ///
    /// Must run after [`InputManagerSystem::Update`] or the action state will be overriden
    ManualControl,
}
