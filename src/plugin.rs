//! Contains the main plugin exported by this crate.

use core::hash::Hash;
use core::marker::PhantomData;
use std::fmt::Debug;

use bevy::app::{App, FixedPostUpdate, Plugin, RunFixedMainLoop};
use bevy::input::InputSystem;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::time::run_fixed_main_schedule;
#[cfg(feature = "ui")]
use bevy::ui::UiSystem;
use updating::CentralInputStore;

use crate::action_state::{ActionState, ButtonData};
use crate::clashing_inputs::ClashStrategy;
use crate::input_map::InputMap;
use crate::input_processing::*;
#[cfg(feature = "mouse")]
use crate::systems::{accumulate_mouse_movement, accumulate_mouse_scroll};
#[cfg(feature = "timing")]
use crate::timing::Timing;
use crate::user_input::*;
use crate::Actionlike;

/// A [`Plugin`] that collects [`ButtonInput`] from disparate sources,
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
/// - [`update_action_state`](crate::systems::update_action_state), which collects [`ButtonInput`] resources to update the [`ActionState`]
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

impl<A: Actionlike + TypePath + bevy::reflect::GetTypeRegistration> Plugin
    for InputManagerPlugin<A>
{
    fn build(&self, app: &mut App) {
        use crate::systems::*;

        match self.machine {
            Machine::Client => {
                // TODO: this should be part of `bevy_input`
                if !app.is_plugin_added::<AccumulatorPlugin>() {
                    app.add_plugins((AccumulatorPlugin, CentralInputStorePlugin));
                }

                if !app.is_plugin_added::<CentralInputStorePlugin>() {
                    app.add_plugins(CentralInputStorePlugin);
                }

                // Main schedule
                app.add_systems(
                    PreUpdate,
                    (
                        tick_action_state::<A>.in_set(TickActionStateSystem::<A>::new()),
                        clear_central_input_store,
                    )
                        .in_set(InputManagerSystem::Tick)
                        .before(InputManagerSystem::Update),
                )
                .add_systems(PostUpdate, release_on_input_map_removed::<A>);

                app.add_systems(
                    PreUpdate,
                    update_action_state::<A>.in_set(InputManagerSystem::Update),
                );

                app.configure_sets(
                    PreUpdate,
                    InputManagerSystem::ManualControl.after(InputManagerSystem::Update),
                );

                app.configure_sets(
                    PreUpdate,
                    InputManagerSystem::Unify.after(InputManagerSystem::Filter),
                );

                app.configure_sets(
                    PreUpdate,
                    InputManagerSystem::Update
                        .after(InputSystem)
                        .after(InputManagerSystem::Unify),
                );

                #[cfg(any(feature = "egui", feature = "ui"))]
                app.add_systems(
                    PreUpdate,
                    filter_captured_input
                        .before(update_action_state::<A>)
                        .in_set(InputManagerSystem::Filter),
                );

                #[cfg(feature = "egui")]
                app.configure_sets(
                    PreUpdate,
                    InputManagerSystem::Filter.after(bevy_egui::EguiSet::ProcessInput),
                );

                #[cfg(feature = "ui")]
                app.configure_sets(PreUpdate, InputManagerSystem::Filter.after(UiSystem::Focus));

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

                // FixedMain schedule
                app.add_systems(
                    RunFixedMainLoop,
                    (
                        swap_to_fixed_update::<A>,
                        // we want to update the ActionState only once, even if the FixedMain schedule runs multiple times
                        update_action_state::<A>,
                    )
                        .chain()
                        .before(run_fixed_main_schedule),
                );

                app.add_systems(FixedPostUpdate, release_on_input_map_removed::<A>);
                app.add_systems(
                    FixedPostUpdate,
                    tick_action_state::<A>
                        .in_set(TickActionStateSystem::<A>::new())
                        .in_set(InputManagerSystem::Tick)
                        .before(InputManagerSystem::Update),
                );
                app.add_systems(
                    RunFixedMainLoop,
                    swap_to_update::<A>.after(run_fixed_main_schedule),
                );
            }
            Machine::Server => {
                app.add_systems(
                    PreUpdate,
                    tick_action_state::<A>
                        .in_set(TickActionStateSystem::<A>::new())
                        .in_set(InputManagerSystem::Tick),
                );
            }
        };

        #[cfg(feature = "mouse")]
        app.register_type::<AccumulatedMouseMovement>()
            .register_type::<AccumulatedMouseScroll>()
            .register_buttonlike_input::<MouseMoveDirection>()
            .register_axislike_input::<MouseMoveAxis>()
            .register_dual_axislike_input::<MouseMove>()
            .register_buttonlike_input::<MouseScrollDirection>()
            .register_axislike_input::<MouseScrollAxis>()
            .register_dual_axislike_input::<MouseScroll>();

        #[cfg(feature = "keyboard")]
        app.register_buttonlike_input::<KeyCode>()
            .register_buttonlike_input::<ModifierKey>()
            .register_axislike_input::<KeyboardVirtualAxis>()
            .register_dual_axislike_input::<KeyboardVirtualDPad>()
            .register_triple_axislike_input::<KeyboardVirtualDPad3D>();

        #[cfg(feature = "gamepad")]
        app.register_buttonlike_input::<GamepadControlDirection>()
            .register_axislike_input::<GamepadControlAxis>()
            .register_dual_axislike_input::<GamepadStick>()
            .register_buttonlike_input::<GamepadButtonType>()
            .register_axislike_input::<GamepadVirtualAxis>()
            .register_dual_axislike_input::<GamepadVirtualDPad>();

        // Chords
        app.register_buttonlike_input::<ButtonlikeChord>()
            .register_axislike_input::<AxislikeChord>()
            .register_dual_axislike_input::<DualAxislikeChord>()
            .register_triple_axislike_input::<TripleAxislikeChord>();

        // General-purpose reflection
        app.register_type::<ActionState<A>>()
            .register_type::<InputMap<A>>()
            .register_type::<ButtonData>()
            .register_type::<ActionState<A>>()
            .register_type::<CentralInputStore>();

        // Processors
        app.register_type::<AxisProcessor>()
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
            .register_type::<CircleDeadZone>();

        // Resources
        app.init_resource::<ClashStrategy>();

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
    /// Accumulates various input event streams into a total delta for the frame.
    Accumulate,
    /// Filters out inputs that are captured by UI elements.
    Filter,
    /// Gathers all of the input data into the [`CentralInputStore`] resource.
    Unify,
    /// Collects input data to update the [`ActionState`].
    ///
    /// See [`UpdateableUserInput`](crate::user_input::updating) for more information.
    Update,
    /// Manually control the [`ActionState`]
    ///
    /// Must run after [`InputManagerSystem::Update`] or the action state will be overridden
    ManualControl,
}

#[derive(SystemSet, Clone, Hash, Debug, PartialEq, Eq)]
/// [`SystemSet`] for the [`tick_action_state`](crate::systems::tick_action_state) system, is a child set of [`InputManagerSystem::Tick`].
pub struct TickActionStateSystem<A: Actionlike> {
    phantom_data: PhantomData<A>,
}

impl<A: Actionlike> TickActionStateSystem<A> {
    /// Creates a [`TickActionStateSystem`] set instance.
    pub fn new() -> Self {
        Self {
            phantom_data: PhantomData,
        }
    }
}

impl<A: Actionlike> Default for TickActionStateSystem<A> {
    fn default() -> Self {
        Self::new()
    }
}

/// A plugin to handle accumulating mouse movement and scroll events.
///
/// This is a clearer, more reliable and more efficient approach to computing the total mouse movement and scroll for the frame.
///
/// This plugin is public to allow it to be used in tests: users should always have this plugin implicitly added by [`InputManagerPlugin`].
/// Ultimately, this should be included as part of [`InputPlugin`](bevy::input::InputPlugin): see [bevy#13915](https://github.com/bevyengine/bevy/issues/13915).
pub struct AccumulatorPlugin;

impl Plugin for AccumulatorPlugin {
    #[allow(unused_variables)]
    fn build(&self, app: &mut App) {
        #[cfg(feature = "mouse")]
        {
            app.init_resource::<AccumulatedMouseMovement>();
            app.init_resource::<AccumulatedMouseScroll>();

            // TODO: these should be part of bevy_input
            app.add_systems(
                PreUpdate,
                (accumulate_mouse_movement, accumulate_mouse_scroll)
                    .in_set(InputManagerSystem::Accumulate),
            );

            app.configure_sets(
                PreUpdate,
                InputManagerSystem::Accumulate
                    .after(InputSystem)
                    .before(InputManagerSystem::Unify),
            );
        }
    }
}

/// A plugin that keeps track of all inputs in a central store.
///
/// This plugin is added by default by [`InputManagerPlugin`],
/// and will register all of the standard [`UserInput`]s.
///
/// To add more inputs, call [`CentralInputStore::register_input_kind`] during [`App`] setup.
pub struct CentralInputStorePlugin;

impl Plugin for CentralInputStorePlugin {
    fn build(&self, app: &mut App) {
        let mut central_input_store = CentralInputStore::default();
        central_input_store.register_standard_input_kinds(app);

        app.insert_resource(central_input_store);
    }
}
