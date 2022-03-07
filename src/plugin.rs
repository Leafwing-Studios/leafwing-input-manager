//! Contains main plugin exported by this crate.

use crate::Actionlike;
use core::any::TypeId;
use core::hash::Hash;
use core::marker::PhantomData;
use std::fmt::Debug;

use bevy::app::{App, CoreStage, Plugin};
use bevy::ecs::prelude::*;
use bevy::ecs::schedule::ShouldRun;
use bevy::ecs::system::Resource;
use bevy::input::InputSystem;
use bevy::ui::UiSystem;

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
pub struct InputManagerPlugin<A: Actionlike, UserState: Resource + PartialEq + Clone = ()> {
    _phantom: PhantomData<A>,
    state_variant: UserState,
    machine: Machine,
}

// Deriving default induces an undesired bound on the generic
impl<A: Actionlike> Default for InputManagerPlugin<A> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData::default(),
            state_variant: (),
            machine: Machine::Client,
        }
    }
}

impl<A: Actionlike, UserState: Resource + PartialEq + Clone> InputManagerPlugin<A, UserState> {
    /// Creates a version of this plugin that will only run in the specified `state_variant`
    ///
    /// # Example
    /// ```rust
    /// use bevy::prelude::*;
    /// use bevy::input::InputPlugin;
    /// use leafwing_input_manager::prelude::*;

    ///
    /// #[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
    /// enum PlayerAction {
    ///    // Movement
    ///    Up,
    ///    Down,
    ///    Left,
    ///    Right,
    /// }
    ///
    /// #[derive(Debug, Clone, Eq, PartialEq, Hash)]
    /// enum GameState {
    ///     Playing,
    ///     Paused,
    ///     Menu,
    /// }
    ///
    /// App::new()
    /// .add_plugins(MinimalPlugins)
    /// .add_plugin(InputPlugin)
    /// .add_state(GameState::Playing)
    /// .add_plugin(InputManagerPlugin::<PlayerAction, GameState>::run_in_state(GameState::Playing))
    /// .update();
    /// ```
    #[must_use]
    pub fn run_in_state(state_variant: UserState) -> Self {
        Self {
            _phantom: PhantomData::default(),
            state_variant,
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
            state_variant: (),
            machine: Machine::Server,
        }
    }
}

/// Which machine is this plugin running on?
enum Machine {
    Server,
    Client,
}

impl<A: Actionlike, UserState: Resource + Eq + Debug + Clone + Hash> Plugin
    for InputManagerPlugin<A, UserState>
{
    fn build(&self, app: &mut App) {
        use crate::systems::*;

        let input_manager_systems = match self.machine {
            Machine::Client => SystemSet::new()
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
                    update_action_state_from_interaction::<A>
                        .label(InputManagerSystem::ManualControl)
                        .after(InputManagerSystem::Reset)
                        // Must run after the system is updated from inputs, or it will be forcibly released due to the inputs
                        // not being pressed
                        .after(InputManagerSystem::Update)
                        .after(UiSystem::Focus)
                        .after(InputSystem),
                ),
            Machine::Server => SystemSet::new().with_system(
                tick_action_state::<A>
                    .label(InputManagerSystem::Reset)
                    .before(InputManagerSystem::Update),
            ),
        };

        // If a state has been provided
        // Only run this plugin's systems in the state variant provided
        // Note that this does not perform the standard looping behavior
        // as otherwise we would be limited to the stage that state was added in T_T
        if TypeId::of::<UserState>() != TypeId::of::<()>() {
            // https://github.com/bevyengine/rfcs/pull/45 will make special-casing state support unnecessary

            // Captured the state variant we want our systems to run in in a run-criteria closure
            let desired_state_variant = self.state_variant.clone();

            // The `SystemSet` methods take self by ownership, so we must store a new system set
            let input_manager_systems = input_manager_systems.with_run_criteria(
                move |current_state: Res<State<UserState>>| {
                    if *current_state.current() == desired_state_variant {
                        ShouldRun::Yes
                    } else {
                        ShouldRun::No
                    }
                },
            );

            // Add the systems to our app
            app.add_system_set_to_stage(CoreStage::PreUpdate, input_manager_systems);
        } else {
            // Add the systems to our app
            // Must be split, as the original `input_manager_systems` is consumed in the state branch
            app.add_system_set_to_stage(CoreStage::PreUpdate, input_manager_systems);
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
    /// Manually control the [`ActionState`](crate::action_state::ActionState)
    ///
    /// Must run after [`InputManagerSystem::Update`] or the action state will be overriden
    ManualControl,
}
